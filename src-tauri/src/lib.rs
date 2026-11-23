mod ai;
mod analysis;
mod ciphers;
mod integrations;
mod settings;

use ciphers::{CipherKey, CipherType};
use serde::{Deserialize, Serialize};
use settings::Settings;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

const OUTPUT_FOLDER_NAME: &str = "Cipher Master";
const MAX_CUSTOM_TEXT: usize = 2000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "Easy" => Ok(Difficulty::Easy),
            "Medium" => Ok(Difficulty::Medium),
            "Hard" => Ok(Difficulty::Hard),
            other => Err(format!("Unknown difficulty: {other}")),
        }
    }
}

/// The challenge currently loaded in the app (used by hints, answer checking and integrations).
#[derive(Debug, Clone, Serialize)]
pub struct Challenge {
    pub plaintext: String,
    pub formatted: String,
    pub ciphertext: String,
    pub keys: Vec<CipherKey>,
    pub difficulty: Difficulty,
    pub hints: Vec<String>,
    pub source: String,
    pub created_unix: u64,
}

impl Challenge {
    pub fn cipher_label(&self) -> String {
        self.keys.iter().map(|k| k.cipher_type().label()).collect::<Vec<_>>().join(" -> ")
    }

    pub fn challenge_text(&self) -> String {
        format!(
            "=== CIPHER CHALLENGE ===\nCipher: {}\nDifficulty: {:?}\n{}\n{}\n",
            self.cipher_label(),
            self.difficulty,
            if self.keys.len() > 1 { "Chained: the ciphers were applied left to right; undo them right to left.\n" } else { "" },
            self.ciphertext
        )
    }

    pub fn hint_text(&self) -> String {
        let numbered: Vec<String> = self.hints.iter().enumerate().map(|(i, h)| format!("HINT {}:\n{}", i + 1, h)).collect();
        format!(
            "=== CIPHER SOLUTION HINTS & RIDDLE ===\nCipher Type: {}\n\nHints get stronger as you go - read one at a time.\n\n{}\n",
            self.cipher_label(),
            numbered.join("\n\n")
        )
    }

    pub fn solution_text(&self) -> String {
        let keys: Vec<String> = self.keys.iter().enumerate().map(|(i, k)| format!("Stage {}: {}", i + 1, k.describe())).collect();
        let steps: Vec<String> = ciphers::decrypt_chain_steps(&self.ciphertext, &self.keys)
            .into_iter()
            .zip(self.keys.iter().enumerate().rev())
            .map(|(text, (i, key))| format!("Undo stage {} ({}):\n{}", i + 1, key.cipher_type().label(), text))
            .collect();
        format!(
            "=== CIPHER SOLUTION & KEY ===\nCipher Type: {}\nDifficulty: {:?}\nQuote source: {}\n\nKEYS:\n{}\n\nDECODING STEPS:\n{}\n\nORIGINAL PLAINTEXT:\n{}\n\nCTF FLAG:\n{}\n",
            self.cipher_label(),
            self.difficulty,
            self.source,
            keys.join("\n"),
            steps.join("\n\n"),
            self.plaintext,
            self.flag()
        )
    }

    pub fn flag(&self) -> String {
        let words: Vec<String> = self
            .plaintext
            .split_whitespace()
            .map(|w| w.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_uppercase())
            .filter(|w| !w.is_empty())
            .collect();
        format!("CTF{{{}}}", words.join("_"))
    }
}

pub struct AppState {
    challenge: Mutex<Option<Challenge>>,
    settings: Mutex<Settings>,
    config_dir: PathBuf,
}

fn current_settings(state: &AppState) -> Settings {
    state.settings.lock().map(|s| s.clone()).unwrap_or_default()
}

fn current_challenge(state: &AppState) -> Result<Challenge, String> {
    state
        .challenge
        .lock()
        .ok()
        .and_then(|c| c.clone())
        .ok_or_else(|| "Generate a challenge first.".to_string())
}

fn format_text_for_difficulty(text: &str, diff: Difficulty) -> String {
    match diff {
        Difficulty::Easy => text.to_string(),
        Difficulty::Medium => text
            .to_uppercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        Difficulty::Hard => {
            let cleaned: Vec<char> = text.to_uppercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
            cleaned.chunks(5).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>().join(" ")
        }
    }
}

fn plaintext_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_uppercase())
        .filter(|w| !w.is_empty())
        .collect()
}

/// Progressive hints, from gentle (structure) to strong (key fragments, first word).
fn build_hints(raw: &str, formatted: &str, ciphertext: &str, keys: &[CipherKey], diff: Difficulty) -> Vec<String> {
    let words = plaintext_words(raw);
    let plain = analysis::analyze(formatted);
    let mut hints = Vec::new();

    let mut structure = vec![format!("- Word count: {}", words.len()), format!("- Letters: {}", plain.letters)];
    if diff != Difficulty::Hard {
        if let Some((i, n)) = plain.counts.iter().enumerate().max_by_key(|(i, n)| (**n, std::cmp::Reverse(*i))) {
            structure.push(format!(
                "- Most common letter in the solution: '{}' ({}% of letters)",
                (b'A' + i as u8) as char,
                n * 100 / plain.letters.max(1)
            ));
        }
    }
    if diff == Difficulty::Easy {
        let vowels: usize = [0, 4, 8, 14, 20].iter().map(|i| plain.counts[*i]).sum();
        structure.push(format!("- Vowels in the solution: {vowels}"));
        structure.push(format!(
            "- First word: {} letters | Last word: {} letters",
            words.first().map_or(0, |w| w.chars().count()),
            words.last().map_or(0, |w| w.chars().count())
        ));
    }
    hints.push(format!("STRUCTURE\n{}", structure.join("\n")));

    let cipher = analysis::analyze(ciphertext);
    let mut crypt = vec![format!("- Index of Coincidence of the ciphertext: {:.4} ({})", cipher.ic, cipher.ic_reading)];
    if !cipher.top_bigrams.is_empty() {
        crypt.push(format!(
            "- Most common ciphertext bigrams: {}",
            cipher.top_bigrams.iter().map(|(b, n)| format!("{b} x{n}")).collect::<Vec<_>>().join(", ")
        ));
    } else if !cipher.top_symbols.is_empty() {
        crypt.push(format!(
            "- Most common ciphertext symbols: {}",
            cipher.top_symbols.iter().take(5).map(|(b, n)| format!("{b} x{n}")).collect::<Vec<_>>().join(", ")
        ));
    }
    crypt.push("- In English the most common bigrams are TH, HE, IN, ER, AN.".into());
    hints.push(format!("CRYPTANALYSIS\n{}", crypt.join("\n")));

    hints.push(format!(
        "RIDDLE\n\"Look for an idea involving the key word '{}' hidden beneath.\"",
        words.get(words.len() / 2).map(String::as_str).unwrap_or("TRUTH")
    ));

    for key in keys.iter().rev() {
        let mut hint = format!("KEY ({})\n{}", key.cipher_type().label(), key.partial_hint());
        if let CipherKey::FillInTheBlank { numbers } = key {
            let clue_count = match diff {
                Difficulty::Easy => 4,
                Difficulty::Medium => 3,
                Difficulty::Hard => 2,
            };
            let clues: Vec<String> = (0..26u8)
                .filter(|i| plain.counts[*i as usize] > 0)
                .take(clue_count)
                .map(|i| format!("{} = {:02}", (b'A' + i) as char, numbers[i as usize]))
                .collect();
            hint.push_str(&format!(
                "\nRules: each number is one letter; every letter has a unique number 01-26.\nKnown letter mappings: {}",
                clues.join(", ")
            ));
        }
        hints.push(hint);
    }

    if diff != Difficulty::Hard {
        if let Some(first) = words.first() {
            hints.push(format!("GIVEAWAY\nThe first word is '{first}'."));
        }
    }
    hints
}

fn create_challenge(
    plaintext: String,
    source: String,
    chain: &[CipherType],
    diff: Difficulty,
) -> Result<Challenge, String> {
    ciphers::validate_chain(chain)?;
    let mut rng = rand::rngs::OsRng;
    let keys: Vec<CipherKey> = chain.iter().map(|c| CipherKey::random(*c, diff, &mut rng)).collect();
    let formatted = format_text_for_difficulty(&plaintext, diff);
    if !formatted.chars().any(|c| c.is_ascii_alphabetic()) {
        return Err("The text needs at least one letter A-Z.".into());
    }
    let ciphertext = ciphers::encrypt_chain(&formatted, &keys);
    let hints = build_hints(&plaintext, &formatted, &ciphertext, &keys, diff);
    Ok(Challenge {
        plaintext,
        formatted,
        ciphertext,
        keys,
        difficulty: diff,
        hints,
        source,
        created_unix: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
    })
}

/// Writes a UTF-8 text file. On Windows, uses CRLF line endings and a UTF-8 BOM so that
/// Notepad and other editors display the runic / special characters correctly.
async fn write_text_file(path: &Path, contents: &str) -> Result<(), String> {
    #[cfg(windows)]
    let contents = format!("\u{FEFF}{}", contents.replace("\r\n", "\n").replace('\n', "\r\n"));
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| format!("Could not create {}: {e}", parent.display()))?;
    }
    tokio::fs::write(path, contents.as_bytes())
        .await
        .map_err(|e| format!("Could not write {}: {}", path.display(), e))
}

/// Picks where files go: the custom folder from Settings, else the user's real Desktop as reported by
/// the OS (follows OneDrive / redirected Desktops on Windows), then Documents, Home, and finally the
/// app's own data folder (used on mobile).
async fn resolve_output_dir<R: tauri::Runtime>(app: &tauri::AppHandle<R>, settings: &Settings) -> Result<PathBuf, String> {
    let paths = app.path();
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(custom) = &settings.output_dir {
        candidates.push(PathBuf::from(custom));
    }
    candidates.extend(
        [
            paths.desktop_dir().ok(),
            paths.document_dir().ok().map(|p| p.join(OUTPUT_FOLDER_NAME)),
            paths.home_dir().ok().map(|p| p.join(OUTPUT_FOLDER_NAME)),
            paths.app_data_dir().ok(),
        ]
        .into_iter()
        .flatten(),
    );

    let mut last_error = String::from("no candidate folders were available");
    for dir in candidates {
        match tokio::fs::create_dir_all(&dir).await {
            Ok(()) if dir.is_dir() => return Ok(dir),
            Ok(()) => last_error = format!("{} is not a folder", dir.display()),
            Err(e) => last_error = format!("{}: {}", dir.display(), e),
        }
    }
    Err(format!("Could not find a writable folder for the challenge files ({last_error})"))
}

async fn write_challenge_files(dir: &Path, challenge: &Challenge) -> Result<Vec<String>, String> {
    let files = [
        (dir.join("cipher_challenge.txt"), challenge.challenge_text()),
        (dir.join("cipher_hint.txt"), challenge.hint_text()),
        (dir.join("cipher_solution.txt"), challenge.solution_text()),
    ];
    let mut written = Vec::new();
    for (path, contents) in files {
        write_text_file(&path, &contents).await?;
        written.push(path.display().to_string());
    }
    Ok(written)
}

// ---------------------------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    /// Cipher ids, applied in order (one id = a single cipher).
    pub ciphers: Vec<String>,
    pub difficulty: String,
    pub length_choice: usize,
    /// Optional custom plaintext; when set, no quote is generated.
    pub custom_text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChallengeView {
    pub ciphertext: String,
    pub cipher_label: String,
    pub ciphers: Vec<String>,
    pub difficulty: Difficulty,
    pub source: String,
    pub hint_count: usize,
    pub files: Vec<String>,
    pub output_dir: Option<String>,
    pub cyberchef_supported: bool,
    pub created_unix: u64,
}

#[tauri::command]
async fn generate_challenge<R: tauri::Runtime>(app: tauri::AppHandle<R>, state: State<'_, AppState>, request: GenerateRequest) -> Result<ChallengeView, String> {
    let chain: Vec<CipherType> = request
        .ciphers
        .iter()
        .map(|id| CipherType::from_id(id).ok_or_else(|| format!("Unknown cipher type: {id}")))
        .collect::<Result<_, _>>()?;
    ciphers::validate_chain(&chain)?;
    let diff = Difficulty::parse(&request.difficulty)?;
    let settings = current_settings(&state);

    let custom = request.custom_text.as_deref().map(|t| t.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|t| !t.is_empty());
    let (plaintext, source) = match custom {
        Some(text) if text.chars().count() > MAX_CUSTOM_TEXT => {
            return Err(format!("Custom text is limited to {MAX_CUSTOM_TEXT} characters."))
        }
        Some(text) => (text, "Custom text".to_string()),
        None => {
            let quote = ai::get_quote(&settings, request.length_choice).await;
            (quote.text, quote.source)
        }
    };

    let challenge = create_challenge(plaintext, source, &chain, diff)?;

    let (files, output_dir) = if settings.save_files {
        let dir = resolve_output_dir(&app, &settings).await?;
        (write_challenge_files(&dir, &challenge).await?, Some(dir.display().to_string()))
    } else {
        (Vec::new(), None)
    };

    let view = ChallengeView {
        ciphertext: challenge.ciphertext.clone(),
        cipher_label: challenge.cipher_label(),
        ciphers: chain.iter().map(|c| c.id().to_string()).collect(),
        difficulty: diff,
        source: challenge.source.clone(),
        hint_count: challenge.hints.len(),
        files,
        output_dir,
        cyberchef_supported: integrations::cyberchef_url(&challenge).1,
        created_unix: challenge.created_unix,
    };
    if let Ok(mut slot) = state.challenge.lock() {
        *slot = Some(challenge);
    }
    Ok(view)
}

#[tauri::command]
fn get_hint(state: State<'_, AppState>, index: usize) -> Result<String, String> {
    let challenge = current_challenge(&state)?;
    challenge.hints.get(index).cloned().ok_or_else(|| "No more hints.".to_string())
}

#[derive(Serialize)]
struct AnswerResult {
    correct: bool,
    /// Percentage of letters in the right position.
    match_percent: u32,
}

fn score_answer(plaintext: &str, guess: &str) -> AnswerResult {
    let norm = |s: &str| s.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect::<Vec<_>>();
    let (truth, guess) = (norm(plaintext), norm(guess));
    let matches = truth.iter().zip(&guess).filter(|(a, b)| a == b).count();
    AnswerResult {
        correct: !truth.is_empty() && truth == guess,
        match_percent: (matches * 100 / truth.len().max(guess.len()).max(1)) as u32,
    }
}

#[tauri::command]
fn check_answer(state: State<'_, AppState>, guess: String) -> Result<AnswerResult, String> {
    Ok(score_answer(&current_challenge(&state)?.plaintext, &guess))
}

#[derive(Serialize)]
struct SolutionView {
    plaintext: String,
    keys: Vec<String>,
    flag: String,
}

#[tauri::command]
fn reveal_solution(state: State<'_, AppState>) -> Result<SolutionView, String> {
    let c = current_challenge(&state)?;
    Ok(SolutionView { plaintext: c.plaintext.clone(), keys: c.keys.iter().map(|k| k.describe()).collect(), flag: c.flag() })
}

#[tauri::command]
fn analyze_text(text: String) -> analysis::Analysis {
    analysis::analyze(&text)
}

#[derive(Serialize)]
struct CipherInfo {
    id: &'static str,
    label: &'static str,
    terminal: bool,
}

#[tauri::command]
fn list_ciphers() -> Vec<CipherInfo> {
    CipherType::ALL.iter().map(|c| CipherInfo { id: c.id(), label: c.label(), terminal: c.is_terminal() }).collect()
}

#[tauri::command]
fn futhark_map() -> Vec<(char, char)> {
    ('A'..='Z').zip(ciphers::FUTHARK).collect()
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    current_settings(&state).redacted()
}

#[tauri::command]
async fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<Settings, String> {
    let merged = current_settings(&state).merged_from_ui(settings);
    if !merged.discord_webhook_url.is_empty() && !integrations::is_discord_webhook(&merged.discord_webhook_url) {
        return Err("The Discord webhook URL must start with https://discord.com/api/webhooks/".into());
    }
    settings::save(&state.config_dir, &merged).await?;
    if let Ok(mut slot) = state.settings.lock() {
        *slot = merged.clone();
    }
    Ok(merged.redacted())
}

#[tauri::command]
async fn pick_output_folder<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<Option<String>, String> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog().file().set_title("Choose where Cipher Master saves files").blocking_pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(picked.and_then(|p| p.into_path().ok()).map(|p| p.display().to_string()))
}

#[tauri::command]
async fn list_ollama_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let url = current_settings(&state).ollama_url;
    ai::list_ollama_models(&url).await
}

#[tauri::command]
async fn open_output_folder<R: tauri::Runtime>(app: tauri::AppHandle<R>, state: State<'_, AppState>) -> Result<(), String> {
    let dir = resolve_output_dir(&app, &current_settings(&state)).await?;
    app.opener().open_path(dir.display().to_string(), None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_url<R: tauri::Runtime>(app: tauri::AppHandle<R>, url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("Only https links can be opened.".into());
    }
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
async fn publish_gist(state: State<'_, AppState>, include_hints: bool, include_solution: bool) -> Result<String, String> {
    let challenge = current_challenge(&state)?;
    let settings = current_settings(&state);
    integrations::publish_gist(&settings.github_token, settings.gist_public, &challenge, include_hints, include_solution).await
}

#[tauri::command]
async fn send_to_discord(state: State<'_, AppState>) -> Result<(), String> {
    let challenge = current_challenge(&state)?;
    integrations::send_discord(&current_settings(&state).discord_webhook_url, &challenge).await
}

#[derive(Serialize)]
struct CyberChefLink {
    url: String,
    supported: bool,
}

#[tauri::command]
fn open_cyberchef<R: tauri::Runtime>(app: tauri::AppHandle<R>, state: State<'_, AppState>) -> Result<CyberChefLink, String> {
    let (url, supported) = integrations::cyberchef_url(&current_challenge(&state)?);
    app.opener().open_url(url.clone(), None::<&str>).map_err(|e| e.to_string())?;
    Ok(CyberChefLink { url, supported })
}

#[tauri::command]
async fn export_ctf<R: tauri::Runtime>(app: tauri::AppHandle<R>, state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let challenge = current_challenge(&state)?;
    let dir = resolve_output_dir(&app, &current_settings(&state)).await?;
    let mut written = Vec::new();
    for (rel, contents) in integrations::ctf_export(&challenge) {
        let path = dir.join(rel);
        write_text_file(&path, &contents).await?;
        written.push(path.display().to_string());
    }
    Ok(written)
}

/// Opens the system print dialog. The page's print stylesheet lays the challenge out as a narrow
/// receipt slip, so any installed printer works - including USB/Bluetooth thermal receipt printers.
#[tauri::command]
fn print_challenge<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or_else(|| "Main window not found.".to_string())?
        .print()
        .map_err(|e| e.to_string())
}

fn with_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
            generate_challenge,
            get_hint,
            check_answer,
            reveal_solution,
            analyze_text,
            list_ciphers,
            futhark_map,
            get_settings,
            save_settings,
            pick_output_folder,
            list_ollama_models,
            open_output_folder,
            open_url,
            publish_gist,
            send_to_discord,
            open_cyberchef,
            export_ctf,
            print_challenge,
        ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    with_commands(tauri::Builder::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir().unwrap_or_else(|_| PathBuf::from("."));
            let settings = settings::load(&config_dir);
            app.manage(AppState { challenge: Mutex::new(None), settings: Mutex::new(settings), config_dir });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(chain: &[CipherType], diff: Difficulty) -> Challenge {
        create_challenge("The quick brown fox jumps over the lazy dog".into(), "test".into(), chain, diff).unwrap()
    }

    #[test]
    fn hard_mode_groups_and_hints_use_real_words() {
        assert_eq!(format_text_for_difficulty("THE QUICK BROWN FOX", Difficulty::Hard), "THEQU ICKBR OWNFO X");
        let c = sample(&[CipherType::Caesar], Difficulty::Hard);
        assert!(c.hints.iter().any(|h| h.contains("'JUMPS'")), "{:?}", c.hints);
        assert!(!c.hints.iter().any(|h| h.starts_with("GIVEAWAY")));
    }

    #[test]
    fn every_cipher_and_difficulty_generates_and_solves() {
        for cipher in CipherType::ALL {
            for diff in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
                let c = sample(&[cipher], diff);
                assert!(!c.ciphertext.trim().is_empty());
                assert!(c.solution_text().contains("ORIGINAL PLAINTEXT"));
                assert!(c.hints.len() >= 4);
            }
        }
    }

    #[test]
    fn fill_in_the_blank_hint_has_known_mappings() {
        let c = sample(&[CipherType::FillInTheBlank], Difficulty::Easy);
        assert!(c.hint_text().contains("Known letter mappings:"));
    }

    #[test]
    fn chained_challenge_solution_shows_each_stage() {
        let c = sample(&[CipherType::Vigenere, CipherType::Columnar, CipherType::A1Z26], Difficulty::Medium);
        let s = c.solution_text();
        assert!(s.contains("Undo stage 3") && s.contains("Undo stage 1"));
        assert_eq!(c.cipher_label(), "Vigenere -> Columnar Transposition -> A1Z26 Numeric");
    }

    #[test]
    fn answers_are_scored_ignoring_case_and_punctuation() {
        assert!(score_answer("Hello, world!", "hello world").correct);
        let partial = score_answer("HELLO WORLD", "HELLO WORXX");
        assert!(!partial.correct && partial.match_percent == 80);
        assert_eq!(sample(&[CipherType::Atbash], Difficulty::Easy).flag(), "CTF{THE_QUICK_BROWN_FOX_JUMPS_OVER_THE_LAZY_DOG}");
    }

    #[test]
    fn ctf_export_and_cyberchef_link() {
        let c = sample(&[CipherType::Caesar, CipherType::RailFence], Difficulty::Medium);
        let files = integrations::ctf_export(&c);
        assert_eq!(files.len(), 3);
        assert!(files[0].1.contains("type: standard") && files[0].1.contains("CTF{"));
        let (url, supported) = integrations::cyberchef_url(&c);
        assert!(supported && url.contains("Rail_Fence_Cipher_Decode") && url.contains("ROT13"));
        let enigma = sample(&[CipherType::Enigma], Difficulty::Easy);
        assert!(!integrations::cyberchef_url(&enigma).1);
    }

    /// Drives the real commands through Tauri's IPC layer with the exact argument names the UI sends.
    #[test]
    fn ipc_contract_matches_the_frontend() {
        use tauri::ipc::{CallbackFn, InvokeBody};
        use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
        use tauri::webview::InvokeRequest;

        let dir = std::env::temp_dir().join(format!("cipher_master_ipc_{}", std::process::id()));
        let app = with_commands(mock_builder())
            .manage(AppState {
                challenge: Mutex::new(None),
                settings: Mutex::new(Settings { ollama_enabled: false, save_files: false, ..Default::default() }),
                config_dir: dir.clone(),
            })
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
        // The app's own origin; anything else is treated as a remote URL and denied by the ACL.
        let origin = if cfg!(windows) { "http://tauri.localhost" } else { "tauri://localhost" };
        let call = |cmd: &str, body: serde_json::Value| {
            get_ipc_response(
                &webview,
                InvokeRequest {
                    cmd: cmd.into(),
                    callback: CallbackFn(0),
                    error: CallbackFn(1),
                    url: origin.parse().unwrap(),
                    body: InvokeBody::Json(body),
                    headers: Default::default(),
                    invoke_key: INVOKE_KEY.to_string(),
                },
            )
            .map(|r| r.deserialize::<serde_json::Value>().unwrap())
        };

        let view = call(
            "generate_challenge",
            serde_json::json!({ "request": { "ciphers": ["Caesar", "Baconian"], "difficulty": "Hard", "length_choice": 3, "custom_text": "hello world" } }),
        )
        .unwrap();
        assert_eq!(view["source"], "Custom text");
        assert_eq!(view["cipher_label"], "Caesar Shift -> Baconian");
        assert!(call("get_hint", serde_json::json!({ "index": 0 })).unwrap().as_str().unwrap().contains("STRUCTURE"));
        assert_eq!(call("check_answer", serde_json::json!({ "guess": "Hello, World" })).unwrap()["correct"], true);
        assert_eq!(call("reveal_solution", serde_json::json!({})).unwrap()["flag"], "CTF{HELLO_WORLD}");
        assert!(call("analyze_text", serde_json::json!({ "text": "ABC" })).unwrap()["letters"] == 3);
        assert_eq!(call("list_ciphers", serde_json::json!({})).unwrap().as_array().unwrap().len(), 13);
        assert_eq!(call("futhark_map", serde_json::json!({})).unwrap()[0], serde_json::json!(["A", "ᚨ"]));
        let mut s = call("get_settings", serde_json::json!({})).unwrap();
        s["ollama_model"] = "phi3".into();
        assert_eq!(call("save_settings", serde_json::json!({ "settings": s })).unwrap()["ollama_model"], "phi3");
        assert!(dir.join("settings.json").exists());
        // Wrong argument names must fail loudly (this is what broke the original app).
        assert!(call("get_hint", serde_json::json!({ "idx": 0 })).is_err());
        // publish_gist accepts the snake_case names the UI sends, and reports the missing token.
        let err = call("publish_gist", serde_json::json!({ "include_hints": true, "include_solution": false })).unwrap_err();
        assert!(err.to_string().contains("GitHub token"), "{err}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn writes_all_three_files() {
        let dir = std::env::temp_dir().join(format!("cipher_master_test_{}", std::process::id()));
        let c = sample(&[CipherType::CustomFuthark], Difficulty::Easy);
        let files = write_challenge_files(&dir, &c).await.unwrap();
        assert_eq!(files.len(), 3);
        let text = std::fs::read_to_string(dir.join("cipher_challenge.txt")).unwrap();
        assert!(text.contains("ᛏᚺᛖ"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
