# ⚡ Suggested Upgrades

This document covers architectural, performance, security, and infrastructure enhancements for **Cipher Master**.

---

## 1. Backend & Performance Enhancements
* **Async IO File Generation**: Migrate file writing operations in `src-tauri/src/lib.rs` to asynchronous `tokio::fs` tasks to keep the UI fully responsive during disk operations.
* **Non-Blocking AI Requests**: Implement async client connections for Ollama calls with configured timeout thresholds to prevent UI hangs on slow model initializations.

## 2. Cryptographic & Generator Quality
* **Secure Randomness (`rand::rngs::OsRng`)**: Upgrade standard PRNG key selection to cryptographically secure random number generators for key generation.
* **Index & Vowels Cryptanalysis Metrics**: Expand the `cipher_hint.txt` generator to include standard Index of Coincidence ($IC$) metrics and bigram frequency counts.

## 3. Quality Assurance & CI/CD
* **Rust Unit Testing Suite**: Expand automated test coverage in `src-tauri` for each cipher engine module (verifying round-trip `encode` and `decode` results).
* **Automated GUI Testing**: Introduce Playwright / WebDriver testing for Tauri frontend IPC commands in GitHub Actions.
