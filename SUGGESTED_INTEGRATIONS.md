# 🔗 Suggested Integrations

This document outlines proposed integrations with external services, tools, and platforms to extend the functionality and ecosystem of **Cipher Master**.

---

## 1. Cloud & Synchronization Services
* **GitHub Gist Sync**: Allow users to publish generated challenges or solution sheets directly to a private/public GitHub Gist with one click.
* **Discord Webhook Notifications**: Option to dispatch newly generated cipher challenges directly into a dedicated Discord channel for community puzzle solving.

## 2. Expanded AI & Model Providers
* **Ollama Model Selector**: Allow users to select alternate local LLM models (e.g., `mistral`, `phi3`, `gemma`) directly from the GUI settings panel.
* **OpenAI / Anthropic API Option**: Optional fallback API key input for cloud-hosted LLM inference when local resources (Ollama) are insufficient or unavailable.

## 3. Educational & Cryptanalysis Tools
* **CyberChef Deep Link / Export**: Export cipher matrices directly to CyberChef recipes for step-by-step cryptanalysis visualization.
* **CTF (Capture The Flag) Exporters**: Format output files into standard CTF platform formats (e.g., CTFd, HackTheBox puzzle structures).

## 4. Hardware & Peripherals
* **Custom Printed Output (Thermal Printers)**: Direct USB/Bluetooth printer integration to print cryptographic escape room physical receipts/slips.

## 5. Other OSes
* **Android Packaging**: The release workflow provisions Java 17, Android API 35, build tools, the Android NDK, and Rust Android targets, initializes Tauri Android, and builds both APK and AAB packages. Local builds use `cargo tauri android init` followed by `cargo tauri android build --apk --aab`.
* **Android release artifacts**: GitHub Actions uploads the APK/AAB files to the workflow run and attaches them to tagged GitHub releases. Store-ready distribution still requires Android signing secrets and a signing configuration.

## 6. Discord Slash Commands
* **Discord bot starter**: `integrations/discord-bot` provides `/cipher`, `/hint`, and `/solution`. It uses the bundled quote library, keeps challenge state per user, and makes hints and solutions ephemeral.
* **Future bot engines**: Add the remaining Cipher Master algorithms behind the same command contract, then move challenge persistence to a small database if the bot is deployed across multiple instances.

### Discord Setup Quick Start
1. Create an application in the [Discord Developer Portal](https://discord.com/developers/applications).
2. Copy the Application ID from **General Information** and reset/copy the bot token from **Bot**.
3. In **OAuth2 > URL Generator**, select `bot` and `applications.commands`, plus the `Send Messages` permission, then invite the bot to a test server.
4. Run from the repository root:
	```bash
	cd integrations/discord-bot
	npm install
	export DISCORD_TOKEN="your-bot-token"
	export DISCORD_CLIENT_ID="your-application-id"
	npm run register
	npm start
	```
5. Test `/cipher`, then use `/hint` or `/solution`. Never commit `DISCORD_TOKEN`; global command registration can take up to an hour to appear.

## 7. Integration Principles
* Keep integrations opt-in and credentials in environment variables.
* Keep solutions private by default.
* Reuse the bundled quote fallback whenever an external AI provider is unavailable.