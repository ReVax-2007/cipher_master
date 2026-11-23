# ᛏ Cipher Master Engine — Technical & Operational Context Document

This document serves as the single source of truth for **Cipher Master Engine**. It is designed to provide complete context to human developers, maintainers, and LLMs/AI assistants. It details the product requirements, system architecture, file structure, technology stack, cipher implementations, prompt engineering strategy, and software release procedures.

---

## 1. Executive Summary & Product Specification

* **Project Name**: Cipher Master Engine
* **Repository**: `https://github.com/ReVax-2007/cipher_master.git`
* **Target Platforms**: macOS (x86_64 / ARM64 `.dmg`) & Windows (x86_64 `.exe` / `.msi`)
* **Primary Objective**: Provide an offline-first desktop application that generates cryptanalytic challenges, structured hints, and solution keys. Outputs are automatically rendered and written to plain-text files on the user's Desktop.
* **License**: **CC BY-NC 4.0** (Creative Commons Attribution-NonCommercial 4.0 International). Commercial usage, reselling, or bundling into paid software is strictly prohibited.

---

## 2. Technology Stack & Dependencies

### Desktop Framework & Engine
* **Tauri v2 (`@tauri-apps/cli` v2)**: Native OS bindings, windowing, and IPC bridge. Chosen for its small memory footprint, security boundaries, and lightweight native builds compared to Electron.
* **Rust (2021 Edition / 1.75+)**: Backend logic, cipher encryption algorithms, file system IO, and HTTP client integration.

### Frontend
* **HTML5 / CSS3 / JavaScript (ES6+)**: Plain, lightweight web interface located in the decoupled `./public` asset directory. Communicates with Rust strictly via Tauri IPC (`window.__TAURI__.core.invoke`).

### Cargo Dependencies (`src-tauri/Cargo.toml`)
* `tauri`: Core framework features with `protocol-asset` and window controls.
* `serde` / `serde_json`: High-performance JSON serialization/deserialization for Rust-to-JS IPC payloads.
* `rand`: Key generation uses `rand::rngs::OsRng` (cryptographically secure).
* `reqwest` (rustls) + `tokio`: Async HTTP client for Ollama / OpenAI / Anthropic / GitHub / Discord, and async file IO.
* `tauri-plugin-dialog`, `tauri-plugin-opener`: Native folder picker; opening URLs and folders.

### External Integrations
* **Ollama (Optional)**: Connects via local HTTP REST API at `http://127.0.0.1:11434/api/generate` requesting the `llama3.2:1b` model.
* **Built-in Fallback**: Internal algorithmic phrase generator (`fallback_quote`) that operates when Ollama is unavailable, offline, or uninstalled.

---

## 3. Directory & File Architecture

```text
cipher_master/
├── .github/
│   └── workflows/
│       └── release.yml          # GitHub Actions CI/CD pipeline (macOS + Windows)
├── public/
│   └── index.html               # Isolated frontend web assets (Tauri frontendDist)
├── src-tauri/
│   ├── Cargo.toml               # Rust dependencies and binary metadata
│   ├── tauri.conf.json          # Tauri app configuration, bundle ID, window specs
│   ├── icons/                   # Multi-resolution platform icons (.icns, .ico, PNGs)
│   └── src/
│       ├── lib.rs               # Core cipher logic, file IO, IPC commands & AI client
│       └── main.rs              # Execution entry point
├── app-icon.png                 # Master 1024x1024 high-resolution source icon
├── SUGGESTED_FEATURES.md        # Feature roadmap
├── SUGGESTED_INTEGRATIONS.md    # Third-party service integration roadmap
├── SUGGESTED_UPGRADES.md        # Infrastructure and architectural upgrade roadmap
├── LICENSE                      # CC BY-NC 4.0 Legal terms
└── README.md                    # Primary repository documentation
```

---

## 4. Key Configurations (`tauri.conf.json`)

To compile valid release installers, the bundle identifier must avoid default placeholders (`com.tauri.dev`).

```json
{
  "$schema": "[https://schema.tauri.app/config/2](https://schema.tauri.app/config/2)",
  "productName": "Cipher Master",
  "version": "1.2.0",
  "identifier": "com.xvoet.ciphermaster",
  "build": {
    "frontendDist": "../public"
  },
  "app": {
    "windows": [
      {
        "title": "Cipher Master",
        "width": 800,
        "height": 600,
        "resizable": true,
        "fullscreen": false
      }
    ]
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

---

## 5. Core Engine Implementations & Methodologies

### A. Supported Cipher Algorithms (`src-tauri/src/ciphers.rs`)

Each cipher is a `CipherKey` variant with `encrypt` and `decrypt`; keys are generated with `OsRng`.

1. **Elder Futhark Runes** — one-to-one map A..Z → ᚨᛒᚲᛞᛖᚠᚷᚺᛁᛃᚴᛚᛗᚾᛟᛈᛩᚱᛋᛏᚢᚡᚹᛪᛇᛉ.
2. **Caesar** — $E(x) = (x + k) \bmod 26$, $k \in [1, 25]$.
3. **Atbash** — $E(x) = 25 - x$.
4. **Monoalphabetic Substitution** — random permutation of A..Z.
5. **Vigenère** — keyword shift per letter.
6. **Rail Fence** — zig-zag over 2–5 rails.
7. **Affine** — $E(x) = (ax + b) \bmod 26$, $\gcd(a, 26) = 1$.
8. **A1Z26** — letters → 1..26 joined by `-`, words separated by three spaces.
9. **Fill in the Blank** — random bijection A..Z → 01..26.
10. **Playfair** — 5x5 keyword square, I=J, X (or Q) filler; must be the first stage of a chain.
11. **Enigma I** — rotors I–V, reflector B/C, rings, plugboard (2/6/10 pairs by difficulty), correct double stepping. Reciprocal.
12. **Baconian** — 26-letter A/B 5-bit groups, words separated by ` / `.
13. **Columnar Transposition** — keyword order, irregular rows; Hard = two passes.

**Chaining** (`encrypt_chain` / `decrypt_chain_steps`): up to 4 stages. In a chain every stage works on compact A–Z/0–9 text so stages compose exactly. Symbol ciphers (Runes, A1Z26, Fill in the Blank, Baconian) must be last; Playfair must be first.

**Cryptanalysis** (`analysis.rs`): letter counts, Index of Coincidence (English ≈ 0.067, random ≈ 0.038), top bigrams/trigrams, top symbols. Used for hints and by the in-app workspace (`analyze_text` command).

### B. Difficulty Formatting Pipeline

* **Easy**: Maintains original spaces, lower/upper case, and punctuation. Appends detailed vowel count and frequency metrics in `cipher_hint.txt`.
* **Medium**: Converts string to uppercase and strips all spaces and non-alphanumeric characters.
* **Hard**: Uppercases, strips non-alphanumeric characters, and formats the output into uniform 5-character blocks (e.g., `HELL O WORL D` $\rightarrow$ `HELLO WORLD` $\rightarrow$ `HELLO WORLD`).

### C. Desktop Output File Protocol

Upon execution of the `generate_challenge` IPC command (when "save files" is on), the engine uses the custom output folder from Settings if one is set, otherwise resolves the user's Desktop through Tauri's path API (`app.path().desktop_dir()`, which follows OneDrive/redirected Desktops on Windows), falling back to `Documents/Cipher Master`, `~/Cipher Master`, then the app data folder, and writes three distinct files (UTF-8 with BOM and CRLF line endings on Windows):

1. `cipher_challenge.txt`: Contains difficulty level, cipher type name, and formatted ciphertext.
2. `cipher_hint.txt`: Contains character length, structural analysis, letter distribution metrics, and riddle clues.
3. `cipher_solution.txt`: Contains original plaintext, every stage's key, step-by-step decoding, and a `CTF{...}` flag.

Other IPC commands: `get_hint`, `check_answer`, `reveal_solution`, `analyze_text`, `list_ciphers`, `futhark_map`, `get_settings`, `save_settings`, `pick_output_folder`, `list_ollama_models`, `open_output_folder`, `open_url`, `publish_gist`, `send_to_discord`, `open_cyberchef`, `export_ctf`, `print_challenge`.

---

## 6. AI Prompting Strategy & Local Fallback

### Ollama Prompt Strategy

When an AI quote is requested, `ureq::post` issues a JSON payload to `http://127.0.0.1:11434/api/generate`:

```json
{
  "model": "llama3.2:1b",
  "prompt": "Generate a single inspirational, philosophical, or historical quote suitable for a cryptographic puzzle. Output ONLY the quote text itself. Do not include quotes, commentary, or attribution.",
  "stream": false
}
```

### Algorithmic Fallback Engine

If Ollama fails to respond within the socket timeout or is offline, `fallback_quote(length_choice)` executes deterministically:

* Uses randomized array selection across pre-defined arrays (`TOPICS`, `ACTIONS`, `QUALIFIERS`).
* Concatenates items into structured uppercase sentences (e.g., `"KNOWLEDGE FAVORED THE BRAVE MINDS AND PROTECTED LIBERTY WITH SECRETS"`).

---

## 7. Development, Replicating & Debugging

### Running Dev Mode

To run and inspect IPC events locally on macOS:

```bash
cargo tauri dev
```

### Manual Quote Modifications

If you want to alter or add static quotes without Ollama, update `fallback_quote` in `src-tauri/src/lib.rs`:

```rust
fn fallback_quote(_length_choice: usize) -> String {
    "CUSTOM PASSTHROUGH TEXT GOES HERE".to_string()
}
```

### Isolating IPC Mismatches

When calling Rust commands from JS, parameter names must match verbatim in camelCase or snake_case as declared:

* **Rust**: `pub fn generate_cipher_files(cipher_type: String, difficulty: String)`
* **JS**: `invoke('generate_cipher_files', { cipherType: 'caesar', difficulty: 'easy' })`

---

## 8. CI/CD & Maintenance Rules

### Branching Policy

Direct pushes to `master` are strictly disabled in project guidelines. All modifications must follow:

1. Feature branch creation (`git checkout -b feature/name`).
2. Local validation (`cargo tauri dev`).
3. PR submission and merge into `master`.

### Triggering Production Releases (Maintainers Only)

Releases are built via GitHub Actions (`.github/workflows/release.yml`) targeting `macos-latest` (universal Apple Silicon + Intel build), `windows-latest` and `ubuntu-22.04`, plus an Android job that attaches APK/AAB files to the same draft release.

```bash
# Tag master commit
git tag v1.0.x
git push origin v1.0.x
```

GitHub Actions compiles `.dmg`, `.exe`, and `.msi` installers and attaches them directly as release binaries to the corresponding release draft.

## 9. Foundation & Construction Prompts

This section documents the foundational prompts and guidelines used to construct the Cipher Master Engine architecture, backend algorithms, and release workflows.

### A. Initial Architectural Prompt (Foundation & Tech Stack Setup)
> **Goal:** Establish a lightweight, cross-platform Tauri v2 desktop application with Rust backend logic and isolated frontend assets.
>
> **Prompt:**
> "Act as a Senior Rust & Desktop Engineer. I want to build a desktop app named 'Cipher Master' using Tauri v2 and Rust. 
> Key Requirements:
> 1. Separate backend logic into Rust (`src-tauri/src/lib.rs`) and keep web assets isolated in a `./public` folder.
> 2. Implement IPC communication between JavaScript (`window.__TAURI__.core.invoke`) and Rust commands.
> 3. Implement 8 cipher engines: Elder Futhark Runes, Caesar Shift, Atbash, Monoalphabetic Substitution, Vigenère, Rail Fence, Affine Cipher, and A1Z26.
> 4. Support three difficulty levels (Easy, Medium, Hard) that modify punctuation, casing, and 5-character block grouping.
> 5. Output three plain-text files directly to the user's Desktop upon execution: `cipher_challenge.txt`, `cipher_hint.txt`, and `cipher_solution.txt`."

### B. Local AI Integration & Fallback Prompt
> **Goal:** Connect to local LLM instances while maintaining robust offline fallback functionality.
>
> **Prompt:**
> "Integrate a local Ollama API client using the synchronous `ureq` crate in Rust.
> Requirements:
> 1. Query `http://127.0.0.1:11434/api/generate` with model `llama3.2:1b` to pull a single short philosophical/inspirational quote.
> 2. Implement a non-blocking timeout strategy. If Ollama is offline or takes too long, seamlessly fall back to an internal algorithmic quote generator function (`fallback_quote`) without crashing or showing an error in the GUI."

### C. Bundle Identifier & Asset Isolation Debugging Prompts
> **Goal:** Resolve Tauri production bundler errors regarding reserved identifiers and asset directory overlapping.
>
> **Prompt:**
> "Fix the Tauri build errors:
> 1. `Error You must change the bundle identifier`: Update `tauri.conf.json` with a unique reverse-DNS bundle identifier (`com.xvoet.ciphermaster`).
> 2. `Error The configured frontendDist includes the src-tauri folder`: Separate `index.html` into a dedicated `public/` directory and update `frontendDist` in `tauri.conf.json` to point to `../public`."

### D. CI/CD Release Pipeline Prompt
> **Goal:** Automate cross-platform compilation for macOS and Windows using GitHub Actions.
>
> **Prompt:**
> "Create a GitHub Actions release workflow `.github/workflows/release.yml` triggered on version tags (`v*`).
> Requirements:
> 1. Use a matrix build targeting `macos-latest` and `windows-latest`.
> 2. Use `tauri-apps/tauri-action@v0` to build `.dmg`, `.exe`, and `.msi` installers.
> 3. Ensure source code tarballs/zips are not uploaded as standalone release binaries, attaching only compiled production installers to the GitHub Release draft."