# ᛏ Cipher Master Engine

An AI-powered desktop application built with **Rust** and **Tauri v2** that generates randomized cryptographic challenges, structural analysis hints, and solution sheets directly to your Desktop.


---

## ⚠️ Contribution & Branching Rules

> **Important**: Changes must **never** be pushed directly to the `master` branch. 
> All new features, bug fixes, or documentation updates should be developed on separate feature branches and submitted via Pull Requests to maintain pipeline integrity.

---

## 🌟 Key Features

* **13 Cipher Engines** (every one has an encoder *and* a decoder, verified by round-trip tests):
  * **Elder Futhark Runes**: One-to-one runic map (A=ᚨ … Z=ᛉ), shown in the in-app Rune Map.
  * **Caesar Shift**: Rotational shift (1–25).
  * **Atbash Cipher**: Reversed alphabet substitution.
  * **Monoalphabetic Substitution**: Randomized key mapping.
  * **Vigenère Cipher**: Polyalphabetic cipher using periodic keywords.
  * **Rail Fence (Zig-Zag)**: Multi-rail transposition.
  * **Affine Cipher**: $E(x) = (ax + b) \pmod{26}$.
  * **A1Z26 Numeric**: Positional index substitution.
  * **Fill in the Blank**: Every letter gets a unique number `01`–`26`; a few mappings are given as clues.
  * **Playfair**: Digraph substitution with a 5x5 keyword square (I=J, X filler).
  * **Enigma I**: Three-rotor emulator (rotors I–V, reflector B/C, ring settings, plugboard, double stepping).
  * **Baconian**: Binary A/B substitution (26-letter variant).
  * **Columnar Transposition**: Keyword column transposition; Hard mode uses a double (two-pass) transposition.
* **Chained encryption**: Stack up to four ciphers (e.g. Vigenère → Rail Fence → Baconian). The solution file lists every key and each decoding step.
* **Quote sources**: Local [Ollama](https://ollama.com/) with a model picker (any installed model, e.g. `llama3.2:1b`, `mistral`, `phi3`, `gemma`), an optional OpenAI or Anthropic API key as a cloud fallback, and the bundled `src-tauri/data/quotes.json` library when nothing else is available. You can also type your own text.
* **Difficulty Scaling**:
  * **Easy**: Keeps spaces and punctuation; detailed hints.
  * **Medium**: Uppercase, punctuation removed.
  * **Hard**: Five-character blocks, stronger keys, fewer hints.
* **Progressive hints & cryptanalysis**: Hints go from structure, to Index of Coincidence and bigram counts, to a riddle, to partial keys.
* **Decryption workspace**: Letter-frequency chart against English, IC, bigrams/trigrams, a Caesar shift explorer and a live substitution scratchpad.
* **Timed challenge mode**: Countdown-style timer, answer checking, scoring by speed, difficulty, chain length and hints used, plus best scores.
* **Output files**: Three plain-text files on your Desktop (the real Desktop reported by the OS, so OneDrive-redirected Windows Desktops work) or in any folder you choose in Settings:
  * `cipher_challenge.txt`: Formatted ciphertext ready to solve.
  * `cipher_hint.txt`: Numbered progressive hints.
  * `cipher_solution.txt`: Plaintext, keys, step-by-step decoding and a CTF flag.
* **Share & export**: Publish to a GitHub Gist, post to a Discord channel via webhook, open in CyberChef with a ready-made decoding recipe (where CyberChef supports the ciphers), export a CTFd challenge (`challenge.yml` for `ctfcli`, plus a platform-neutral JSON), and print a receipt-width slip on any printer, including USB/Bluetooth thermal printers.
* **Cross-Platform Support**: Windows, macOS (Apple Silicon + Intel universal build), Linux and Android.

### Settings and privacy

Settings live in the app's config folder (`settings.json`). API keys and the GitHub token are stored there on your computer only and are never shown back in the UI. All integrations are opt-in, and solutions are only shared when you tick "Gist includes solution".

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

* `/cipher` creates a challenge. Use the optional `type` argument to pick Caesar, Atbash, Vigenère, Affine, Substitution, Rail Fence, A1Z26, Baconian or Runes.
* `/hint` returns a private hint for your current challenge.
* `/solution` returns the plaintext and key privately.

The bot uses the bundled `src-tauri/data/quotes.json` library. Challenges are saved to `integrations/discord-bot/data/challenges.json` (override with `CHALLENGE_STORE`) so they survive restarts, and expire after 7 days. For several bot instances, swap `store.js` for a shared database. Run the bot's tests with `npm test`.

---

## 📁 Project Architecture

```text
cipher_master/
├── .github/workflows/
│   ├── ci.yml                   # Tests on every push/PR (Linux + Windows, GUI, Discord bot)
│   └── release.yml              # Cross-platform release builds on version tags
├── public/
│   └── index.html               # App UI: Create, Solve & Timed Mode, Rune Map, Settings
├── src-tauri/
│   ├── Cargo.toml               # Rust dependencies
│   ├── tauri.conf.json          # Tauri app and bundle settings
│   ├── data/quotes.json         # Bundled quote library
│   └── src/
│       ├── lib.rs               # Tauri commands, challenge building, hints, file output
│       ├── ciphers.rs           # 13 cipher engines (encrypt + decrypt) and chaining
│       ├── analysis.rs          # Frequency, Index of Coincidence, n-grams
│       ├── ai.rs                # Ollama / OpenAI / Anthropic quote sources + bundled fallback
│       ├── integrations.rs      # Gist, Discord webhook, CyberChef links, CTFd export
│       ├── settings.rs          # Persisted user settings
│       └── main.rs              # Entry point
├── tests/gui/                   # Playwright GUI tests (mocked Tauri backend)
├── integrations/discord-bot/    # Optional Discord slash-command bot
└── Cargo.toml                   # Cargo workspace (member: src-tauri)
```

---

## 🛠️ Tech Stack & Prerequisites

### Core Tech Stack

* **Language**: Rust
* **Desktop Application Framework**: Tauri v2
* **Frontend**: HTML5 / CSS3 / JavaScript (ES6+)
* **Dependencies**:
* `rand` (`OsRng`): Cryptographically secure key generation and shuffling.
* `reqwest` + `tokio`: Async HTTP (Ollama, OpenAI, Anthropic, GitHub, Discord) and async file IO, so the UI never blocks.
* `tauri-plugin-dialog` / `tauri-plugin-opener`: Folder picker and opening links/folders.
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
target/release/bundle/dmg/Cipher Master_<version>_aarch64.dmg
```

The repository root is a Cargo workspace, so build output goes to the root `target/` folder. On Windows, the same command produces `target/release/bundle/nsis/*-setup.exe` and `target/release/bundle/msi/*.msi`.

### Running the tests

```bash
cargo test -p app                                  # cipher, hint, settings and export tests
cd tests/gui && npm install && npx playwright install chromium && npm test   # GUI tests
cd integrations/discord-bot && npm install && npm test                      # bot tests
```

All three suites also run in GitHub Actions on every push and pull request (`.github/workflows/ci.yml`), including a Windows build.

---

## 🌐 Automated Cross-Platform Releases

Cross-platform builds (macOS `.dmg` and Windows `.exe`/`.msi`) are compiled automatically via **GitHub Actions** when a version tag is pushed to the repository.

---

## 🌐 Release Pipeline

Release binaries for macOS (`.dmg`) and Windows (`.exe` / `.msi`) are automatically generated by GitHub Actions when a release tag is pushed by a repository maintainer.

### For Maintainers: Creating a Release

1. Ensure all feature PRs are reviewed, tested, and merged into `master`.
2. Bump `version` in `src-tauri/tauri.conf.json` so it matches the tag (installer file names use it).
3. Create and push a version tag on the latest `master` commit:

   ```bash
   git tag v1.2.0
   git push origin v1.2.0
   ```
4. GitHub Actions executes the build pipeline, generating compiled assets and drafting a new release on the GitHub Releases page.

---
