use clap::Parser;
use std::fs;

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

    for line in contents.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("apt install") {
            let package = trimmed.replace("apt install", "").trim().to_string();
            println!("📦 [apt]  Package: {}", package);
        } else if trimmed.starts_with("pip install") {
            let package = trimmed.replace("pip install", "").trim().to_string();
            println!("📦 [pip]  Package: {}", package);
            check_pypi(&package).await;
        } else if trimmed.starts_with("npm install") {
            let package = trimmed.replace("npm install", "").trim().to_string();
            println!("📦 [npm]  Package: {}", package);
        }
    }
}

async fn check_pypi(package: &str) {
    let url = format!("https://pypi.org/pypi/{}/json", package);
    let response = reqwest::get(&url).await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("   ✅ Found on PyPI");
            } else {
                println!("   🚨 NOT found on PyPI - possible hallucination!");
            }
        }
        Err(_) => {
            println!("   ⚠️  Could not reach PyPI - check your connection");
        }
    }
}