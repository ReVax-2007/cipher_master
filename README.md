# ᛏ Cipher Master Engine

An AI-powered desktop application built with **Rust** and **Tauri v2** that generates randomized cryptographic challenges, structural analysis hints, and solution sheets directly to your Desktop.

---

## ⚠️ Contribution & Branching Rules

> **Important**: Changes must **never** be pushed directly to the `master` branch. 
> All new features, bug fixes, or documentation updates should be developed on separate feature branches and submitted via Pull Requests to maintain pipeline integrity.

---

## 🌟 Key Features

* **9 Cipher Engines**:
  * **Elder Futhark Runes**: Custom handwritten runic character map.
  * **Caesar Shift**: Dynamic rotational shift ($1..25$).
  * **Atbash Cipher**: Reversed alphabet substitution.
  * **Monoalphabetic Substitution**: Randomized key mapping.
  * **Vigenère Cipher**: Polyalphabetic cipher using periodic key phrases.
  * **Rail Fence (Zig-Zag)**: Multi-rail transposition matrix.
  * **Affine Cipher**: Mathematical linear function cipher ($E(x) = (ax + b) \pmod{26}$).
  * **A1Z26 Numeric**: Positional index substitution.
  * **Fill in the Blank**: Assigns every letter a unique number from `01` to `26`, reveals a few mappings as clues, and preserves repeated-letter patterns.
* **Local AI Integration & Fallback Support**: Connects to a local [Ollama](https://ollama.com/) instance (`llama3.2:1b`) to generate dynamic quotes. If Ollama is not installed or running, the engine selects a quote from `src-tauri/data/quotes.json`.
* **Difficulty Scaling**:
  * **Easy**: Preserves original word boundaries, spaces, and punctuation. Generates detailed letter frequency and vowel count hints.
  * **Medium**: Converts text to uppercase and strips all punctuation.
  * **Hard**: Groups output into uniform 5-character blocks for classic cryptanalysis difficulty.
* **Automated Desktop Outputs**: Generates three plain-text files on the user's Desktop upon execution:
  * `cipher_challenge.txt`: Formatted ciphertext ready to solve.
  * `cipher_hint.txt`: Cryptanalytic structural metrics and riddle clues.
  * `cipher_solution.txt`: Full original plaintext and decryption parameters/keys.

### Fill in the Blank Mode

This mode creates a numeric substitution puzzle. Each letter is assigned a unique number from `01` to `26`; the same letter always produces the same number, while different letters never share a number. Word boundaries remain visible in the challenge, and `cipher_hint.txt` reveals a small number of starter mappings based on the selected difficulty. The complete alphabet mapping is available only in `cipher_solution.txt`.
* **Cross-Platform Support**: Native desktop GUI built for macOS and Windows.

---

## ✏️ How to Manually Change Text / Quotes (Without Ollama)

If you do not want to run Ollama locally or prefer using your own custom paragraphs, quotes, or text passages:

1. Edit `src-tauri/data/quotes.json` and add plain JSON strings to the array.
2. Keep the file as valid JSON. Quotes are selected by word count when possible, then a random entry is used as a final fallback.
3. Rebuild or run `cargo tauri dev` to apply the updated text pool.

## Android Packaging

Tauri v2 supports Android from the same project. Install Android Studio, the Android SDK, and the Rust Android targets, then run:

```bash
cargo tauri android init
cargo tauri android dev
cargo tauri android build
```

The generated Android project lives under `src-tauri/gen/android` and should be reviewed before committing. The responsive frontend and bundled quote library work without a desktop-only dependency; Ollama remains optional and is normally unavailable on a phone.

## Discord Slash Commands Setup

The opt-in starter bot lives in `integrations/discord-bot`. It supports `/cipher`, `/hint`, and `/solution`. Challenges are stored in memory per guild and user; hints and solutions are sent as ephemeral replies.

### 1. Create the Discord application

1. Open the [Discord Developer Portal](https://discord.com/developers/applications) and select **New Application**.
2. Open **General Information** and copy the **Application ID**. This is `DISCORD_CLIENT_ID`.
3. Open **Bot**, select **Reset Token**, and copy the token once. This is `DISCORD_TOKEN`. Treat it like a password and never commit it.
4. Under **OAuth2 > URL Generator**, select the `bot` and `applications.commands` scopes.
5. Select the `Send Messages` bot permission, open the generated URL, and invite the bot to a test server.

### 2. Install and configure the bot

From the repository root:

```bash
cd integrations/discord-bot
npm install
```

Set the credentials in the same terminal session. macOS/Linux:

```bash
export DISCORD_TOKEN="paste-your-bot-token-here"
export DISCORD_CLIENT_ID="paste-your-application-id-here"
```

PowerShell:

```powershell
$env:DISCORD_TOKEN = "paste-your-bot-token-here"
$env:DISCORD_CLIENT_ID = "paste-your-application-id-here"
```

### 3. Register and run the commands

```bash
npm run register
npm start
```

The registration command installs the commands globally for the application. Discord can take up to an hour to propagate global commands. Keep `npm start` running while testing, or run it with a process manager for a hosted deployment.

Available commands:

* `/cipher` creates a Caesar or Atbash challenge. Use the optional `type` argument to choose the cipher.
* `/hint` returns a private hint for your current challenge.
* `/solution` returns the plaintext and key privately.

The bot uses the bundled `src-tauri/data/quotes.json` library. It does not call Ollama or persist challenges after restart. Add a database before running multiple bot instances or requiring durable challenge history.

---

## 📁 Project Architecture

```text
cipher_game/
├── .github/
│   └── workflows/
│       └── release.yml          # GitHub Actions CI/CD for cross-platform releases
├── public/
│   └── index.html               # Desktop application UI (Tauri frontend)
├── src-tauri/
│   ├── Cargo.toml               # Backend Rust dependencies and crate metadata
│   ├── tauri.conf.json          # Tauri application and window settings
│   ├── icons/                   # Multi-resolution application icons
│   └── src/
│       ├── lib.rs               # Encryption algorithms, AI client, & Tauri commands
│       └── main.rs              # Application entry point
├── app-icon.png                 # Master 1024x1024 source icon
└── Cargo.toml                   # Workspace configuration
```

---

## 🛠️ Tech Stack & Prerequisites

### Core Tech Stack

* **Language**: Rust
* **Desktop Application Framework**: Tauri v2
* **Frontend**: HTML5 / CSS3 / JavaScript (ES6+)
* **Dependencies**:
* `rand`: Pseudo-random key generation and shuffling.
* `ureq`: Lightweight HTTP client for querying the local Ollama API.
* `serde` / `serde_json`: JSON serialization for Tauri IPC and API communication.



### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) (1.75 or newer)
* `cargo-tauri` CLI (Install via `cargo install tauri-cli --version "^2.0"`)
* *(Optional)* [Ollama](https://ollama.com/) running locally on port `11434` with `llama3.2:1b` installed (`ollama pull llama3.2:1b`).
* *(Optional, Discord bot only)* [Node.js](https://nodejs.org/) 20 or newer and npm.

---

## 🚀 Local Development

### 1. Clone the Repository

```bash
git clone [https://github.com/ReVax-2007/cipher_game.git](https://github.com/ReVax-2007/cipher_game.git)
cd cipher_game
```

### 2. Create a Feature Branch

```bash
git checkout -b feature/my-new-feature
```

### 3. Run in Dev Mode

Run the Tauri development environment:

```bash
cargo tauri dev
```

This compiles the Rust backend, serves the local frontend, and opens the interactive desktop application window.

---

## 📦 Building Production Installers

### Local Mac Build (.dmg)

To build a release installer locally on macOS:

```bash
cargo tauri build
```

The output installer will be saved at:

```text
src-tauri/target/release/bundle/dmg/Cipher Master_1.0.0_x64.dmg
```

---

## 🌐 Automated Cross-Platform Releases

Cross-platform builds (macOS `.dmg` and Windows `.exe`/`.msi`) are compiled automatically via **GitHub Actions** when a version tag is pushed to the repository.

Having automated release documentation in a repository's `README.md` is standard practice, but phrasing it as an absolute guarantee can be confusing if the build pipeline requires manual PR reviews, branch protection checks, or maintainer approval before tag creation.

In collaborative or open-source projects, contributors usually do not have permission to push tags to trigger releases themselves. Writing it from the perspective of **how maintainers trigger releases** clarifies the process for everyone.

---

## 🌐 Release Pipeline

Release binaries for macOS (`.dmg`) and Windows (`.exe` / `.msi`) are automatically generated by GitHub Actions when a release tag is pushed by a repository maintainer.

### For Maintainers: Creating a Release

1. Ensure all feature PRs are reviewed, tested, and merged into `master`.
2. Create and push a version tag on the latest `master` commit:

   ```bash
   git tag v1.0.x
   git push origin v1.0.x
   ```
3. GitHub Actions executes the build pipeline, generating compiled assets and drafting a new release on the GitHub Releases page.

---

### Why this structure works better:

* **Clear Roles:** Specifies that triggering releases is an action taken by maintainers after verification, rather than an automatic step for any contributor.
* **Review-First:** Keeps the emphasis on merging verified code via PRs into `master` before tagging.
* **Agnostic Context:** Works cleanly whether you are working on the project alone or with outside collaborators.


---

## 📜 License

Distributed under the CC BY-NC 4.0 License (Non-Commercial Use Only). See `LICENSE` for details.
