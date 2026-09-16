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
* `rand`: Pseudo-random number generation for cipher key generation, rotational offsets, and matrix shuffling.
* `ureq`: Synchronous, lightweight HTTP client used to query local LLM REST endpoints without pulling in heavy asynchronous runtimes.

### External Integrations
* **Ollama (Optional)**: Connects via local HTTP REST API at `http://localhost:11434/api/generate` requesting the `llama3.2:1b` model.
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
  "version": "1.0.0",
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

### A. Supported Cipher Algorithms (`src-tauri/src/lib.rs`)

1. **Elder Futhark Runes**: Maps standard Latin characters ($A..Z$) to Unicode Elder Futhark runic characters ($\text{ᚠ, ᚢ, ᚦ, ᚨ, ᚱ, ᚲ, \dots}$).
2. **Caesar Shift**: Rotates alphabetic index by a pseudo-random integer $k \in [1, 25]$:

$$E(x) = (x + k) \pmod{26}$$


3. **Atbash Cipher**: Reverses the alphabet index:

$$E(x) = (25 - x) \pmod{26}$$


4. **Monoalphabetic Substitution**: Generates a shuffled 26-character alphabet array via `rand::seq::SliceRandom` and maps $A..Z$ directly to the permutation.
5. **Vigenère Cipher**: Accepts a multi-character key phrase $K$; shifts character $i$ using offset $K[i \pmod{|K|}]$.
6. **Rail Fence (Zig-Zag)**: Constructs a $W \times H$ rail matrix based on $N$ rails, routes plaintext in a zig-zag movement, and reads row-by-row.
7. **Affine Cipher**: Multiplies and shifts characters using key pair $(a, b)$ where $\gcd(a, 26) = 1$:

$$E(x) = (a \cdot x + b) \pmod{26}$$


8. **A1Z26 Numeric**: Converts each letter to its 1-based numerical index separated by hyphens (e.g., $A=1, B=2, Z=26$).
9. **Fill in the Blank**: Builds a randomized one-to-one mapping from every letter in $A..Z$ to a unique number in $01..26$. The challenge displays those numbers while preserving word boundaries, `cipher_hint.txt` reveals a small set of known letter mappings, and `cipher_solution.txt` stores the complete mapping.

### B. Difficulty Formatting Pipeline

* **Easy**: Maintains original spaces, lower/upper case, and punctuation. Appends detailed vowel count and frequency metrics in `cipher_hint.txt`.
* **Medium**: Converts string to uppercase and strips all spaces and non-alphanumeric characters.
* **Hard**: Uppercases, strips non-alphanumeric characters, and formats the output into uniform 5-character blocks (e.g., `HELL O WORL D` $\rightarrow$ `HELLO WORLD` $\rightarrow$ `HELLO WORLD`).

### C. Desktop Output File Protocol

Upon execution of the `generate_cipher_files` IPC command, the engine resolves the user's local Desktop path (`dirs::desktop_dir` or `dirs-next`) and writes three distinct files:

1. `cipher_challenge.txt`: Contains difficulty level, cipher type name, and formatted ciphertext.
2. `cipher_hint.txt`: Contains character length, structural analysis, letter distribution metrics, and riddle clues.
3. `cipher_solution.txt`: Contains original plaintext, decryption parameters/keys, and step-by-step decoding notes.

---

## 6. AI Prompting Strategy & Local Fallback

### Ollama Prompt Strategy

When an AI quote is requested, `ureq::post` issues a JSON payload to `http://localhost:11434/api/generate`:

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

Releases are built via GitHub Actions (`.github/workflows/release.yml`) targeting `macos-latest`, `windows-latest` and `ubuntu-latest`.

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
> 1. Query `http://localhost:11434/api/generate` with model `llama3.2:1b` to pull a single short philosophical/inspirational quote.
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