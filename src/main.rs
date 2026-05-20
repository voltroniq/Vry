use clap::Parser;
use std::fs;
use chrono::{NaiveDate, Utc};

#[derive(Parser)]
#[command(name = "vry")]
#[command(about = "Zero-trust AI script execution")]
struct Cli {
    /// Path to the shell script to verify
    script: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let contents = fs::read_to_string(&args.script).unwrap_or_else(|err| {
        eprintln!("Error: could not read '{}': {}", args.script, err);
        std::process::exit(1);
    });

    println!("Vry - The Semantic Script Wrapper");
    println!("Scanning: {}\n", args.script);

    let distro = detect_distro();
    let package_manager = match distro.as_str() {
        "ubuntu" | "debian" | "linuxmint" => "apt",
        "arch" | "manjaro" | "endeavouros" => "pacman",
        "fedora" | "rhel" | "centos" => "dnf",
        _ => "unknown",
    };

    println!("Detected distro:   {}", distro);
    println!("Package manager:   {}\n", package_manager);

    let mut total = 0;
    let mut risky = 0;

    for line in contents.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("apt install")
            || trimmed.starts_with("apt-get install")
        {
            let package = trimmed
                .replace("apt-get install", "")
                .replace("apt install", "")
                .trim()
                .to_string();
            println!("📦 [apt]  Package: {}", package);
            total += 1;

        } else if trimmed.starts_with("pip install")
            || trimmed.starts_with("pip3 install")
        {
            let package = trimmed
                .replace("pip3 install", "")
                .replace("pip install", "")
                .trim()
                .to_string();
            println!("📦 [pip]  Package: {}", package);
            total += 1;
            let was_risky = check_pypi(&package).await;
            if was_risky { risky += 1; }

        } else if trimmed.starts_with("npm install")
            || trimmed.starts_with("npm i ")
            || trimmed.starts_with("yarn add")
        {
            let package = trimmed
                .replace("npm install", "")
                .replace("npm i ", "")
                .replace("yarn add", "")
                .trim()
                .to_string();
            println!("📦 [npm]  Package: {}", package);
            total += 1;
            let was_risky = check_npm(&package).await;
            if was_risky { risky += 1; }
        }
    }

    let safe = total - risky;

    println!("\n{}", "─".repeat(40));
    println!("Vry Scan Complete");
    println!("{}", "─".repeat(40));
    println!("Packages scanned:  {}", total);
    println!("✅ Safe:           {}", safe);
    println!("🚨 Risky:          {}", risky);

    if risky > 0 {
        println!("\n⚠️  Review risky packages before running this script.");
    } else {
        println!("\n✅ All packages look safe.");
    }
}

async fn check_pypi(package: &str) -> bool {
    let url = format!("https://pypi.org/pypi/{}/json", package);
    let response = reqwest::get(&url).await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let json: serde_json::Value = resp.json().await.unwrap_or_default();
                let earliest = find_earliest_pypi_date(&json);

                match earliest {
                    None => {
                        println!("   ✅ Found on PyPI (creation date unknown)");
                        false
                    }
                    Some(date_str) => {
                        let age_days = days_since(&date_str);
                        if age_days < 30 {
                            println!("   🚨 RISKY: Found on PyPI but only {} days old!", age_days);
                            true
                        } else {
                            println!("   ✅ Found on PyPI (first published {} days ago)", age_days);
                            false
                        }
                    }
                }
            } else {
                println!("   🚨 NOT found on PyPI - possible hallucination!");
                true
            }
        }
        Err(_) => {
            println!("   ⚠️  Could not reach PyPI - check your connection");
            false
        }
    }
}

fn find_earliest_pypi_date(json: &serde_json::Value) -> Option<String> {
    // "releases" is a dict: { "1.0.0": [{upload_time: "..."}, ...], "1.0.1": [...] }
    let releases = json["releases"].as_object()?;
    let mut earliest: Option<String> = None;

    for (_version, files) in releases {
        if let Some(files_array) = files.as_array() {
            for file in files_array {
                if let Some(upload_time) = file["upload_time"].as_str() {
                    match &earliest {
                        None => earliest = Some(upload_time.to_string()),
                        Some(current) => {
                            // ISO date strings sort correctly as plain strings
                            if upload_time < current.as_str() {
                                earliest = Some(upload_time.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    earliest
}

fn days_since(date_str: &str) -> i64 {
    // Take just the date part: "2024-01-15" from "2024-01-15T10:30:00"
    let date_part = &date_str[..10];

    match NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
        Ok(date) => {
            let today = Utc::now().date_naive();
            (today - date).num_days()
        }
        Err(_) => 999 // can't parse = assume old = safe
    }
}

async fn check_npm(package: &str) -> bool {
    let url = format!("https://registry.npmjs.org/{}", package);
    let response = reqwest::get(&url).await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let json: serde_json::Value = resp.json().await.unwrap_or_default();
                let created = json["time"]["created"].as_str().unwrap_or("");

                if created.is_empty() {
                    println!("   ✅ Found on npm (creation date unknown)");
                    return false;
                }

                let age_days = days_since(created);
                if age_days < 30 {
                    println!("   🚨 RISKY: Found on npm but only {} days old!", age_days);
                    true
                } else {
                    println!("   ✅ Found on npm (first published {} days ago)", age_days);
                    false
                }
            } else {
                println!("   🚨 NOT found on npm - possible hallucination!");
                true
            }
        }
        Err(_) => {
            println!("   ⚠️  Could not reach npm - check your connection");
            false
        }
    }
}

// distro detection
fn detect_distro() -> String {
    let contents = fs::read_to_string("/etc/os-release").unwrap_or_default();

    for line in contents.lines() {
        if line.starts_with("ID=") {
            let id = line
                .replace("ID=", "")
                .replace('"', "")
                .trim()
                .to_lowercase();
            return id;
        }
    }
    "unknown".to_string()
}