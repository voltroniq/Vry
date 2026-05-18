# Vry ⚡
### Zero-trust AI script execution for Linux developers

> AI writes the script. Vry checks it before it touches your system.

---

## The Problem

AI coding assistants hallucinate package names. Attackers register those fake names as malware — a technique called **slopsquatting**. You run the AI-generated script, the malicious package installs silently, and your system is compromised.

Even when packages are real, a legitimate-looking name registered 2 days ago by an unknown account is a red flag. Nothing currently catches this automatically.

**ShellCheck** checks syntax. **Docker** is too heavy. There's nothing in between.

Vry fills that gap.

---

## What Vry Does

Vry sits between your terminal and your OS. Before any AI-generated script runs, it:

1. **Audits** every package install command against PyPI and npm registries
2. **Flags** packages that don't exist (hallucinations) or are suspiciously new (< 30 days old)
3. **Translates** package manager commands to match your actual Linux distro *(coming in v0.2)*
4. **Ghost runs** the script in a sandbox and shows you a diff before anything touches your real system *(coming in v0.3)*

---

## Demo

```
$ vry test.sh

Vry - The Semantic Script Wrapper
Scanning: test.sh

📦 [pip]  Package: requests
   ✅ Found on PyPI (first published 5572 days ago)
📦 [npm]  Package: lodash
   ✅ Found on npm (first published 5138 days ago)
📦 [npm]  Package: totally-fake-pkg-xyz
   🚨 NOT found on npm - possible hallucination!
📦 [npm]  Package: new-suspicious-pkg
   🚨 RISKY: Found on npm but only 3 days old!

────────────────────────────────────────
Vry Scan Complete
────────────────────────────────────────
Packages scanned:  4
✅ Safe:           2
🚨 Risky:          2

⚠️  Review risky packages before running this script.
```

---

## Installation

### From source (requires Rust)

```bash
# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/voltroniq/Vry
cd Vry
cargo build --release

# Run
./target/release/vry your_script.sh
```

Pre-built binaries coming with v0.2.

---

## Usage

```bash
# Scan a script
vry script.sh

# Scan an AI-generated script before running it
vry ai_output.sh
```

### Supported package managers

| Command | Detected |
|---|---|
| `apt install` | ✅ |
| `apt-get install` | ✅ |
| `pip install` | ✅ |
| `pip3 install` | ✅ |
| `npm install` | ✅ |
| `npm i` | ✅ |
| `yarn add` | ✅ |

---

## Roadmap

### v0.1 — Registry Auditor ✅ (current)
- Detect package install commands in shell scripts
- Verify packages exist on PyPI and npm
- Flag packages under 30 days old
- Summary report with safe/risky counts

### v0.2 — Cross-distro Translator *(in progress)*
- Detect your Linux distro automatically
- Translate `apt install X` → `pacman -S X` / `dnf install X`
- Hardcoded lookup table of 50 common packages

### v0.3 — Ghost Run Sandbox
- Execute script inside a Linux namespace
- Show diff of what *would* change on your real system
- Require explicit confirmation before real execution

### v0.4 — Enterprise Layer
- eBPF kernel-level syscall interception via Aya
- Web dashboard with audit logs per developer
- GitHub Actions integration: `vry audit` in CI/CD pipelines

---

## Why Rust?

Vry is written in Rust because a security tool must itself be secure. Rust's memory safety guarantees mean Vry cannot be exploited via buffer overflows or use-after-free bugs. It compiles to a single fast binary with no runtime, no daemon, and no Docker required.

---

## The Slopsquatting Threat

Slopsquatting is documented and growing. As AI-assisted development becomes standard, hallucinated package names become an attack surface. Attackers monitor AI outputs, register the hallucinated names, and wait.

Vry is built specifically to close this window.

---

## Contributing

This project is in early development. Issues, feedback, and PRs are welcome.

If you work in DevSecOps or supply chain security and want to talk: open an issue.

---

## License

Apache 2.0

---

*Built in public. Follow the build journey on [Linkedin](#).*