//! Quote sources: local Ollama, optional cloud providers (OpenAI / Anthropic), then the bundled library.
//! All network calls are async with short connect timeouts so the UI never hangs.

use rand::seq::{IteratorRandom, SliceRandom};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

use crate::settings::Settings;

pub struct Quote {
    pub text: String,
    pub source: String,
}

fn client(timeout_secs: u64) -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_millis(1500))
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(concat!("CipherMaster/", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap_or_default()
}

fn prompt(length_choice: usize) -> String {
    let word_count = match length_choice {
        1 => "5 to 8",
        2 => "12 to 18",
        _ => "25 to 35",
    };
    format!(
        "Generate a single inspiring or philosophical quote between {word_count} words. \
         Return ONLY the quote text without quotes, explanation, or attribution."
    )
}

pub async fn get_quote(settings: &Settings, length_choice: usize) -> Quote {
    let prompt = prompt(length_choice);

    if settings.ollama_enabled {
        if let Ok(text) = ollama_generate(settings, &prompt).await {
            return Quote { text, source: format!("Ollama ({})", settings.ollama_model) };
        }
    }

    if !settings.cloud_api_key.is_empty() {
        let model = settings.cloud_model_or_default();
        let result = match settings.cloud_provider.as_str() {
            "openai" => openai_generate(&settings.cloud_api_key, &model, &prompt).await,
            "anthropic" => anthropic_generate(&settings.cloud_api_key, &model, &prompt).await,
            _ => Err(String::new()),
        };
        if let Ok(text) = result {
            return Quote { text, source: format!("{} ({model})", settings.cloud_provider) };
        }
    }

    Quote { text: fallback_quote(length_choice), source: "Bundled quote library".into() }
}

fn accept(raw: &str) -> Result<String, String> {
    let cleaned = clean_ai_quote(raw);
    if cleaned.split_whitespace().count() >= 3 && cleaned.chars().any(|c| c.is_ascii_alphabetic()) {
        Ok(cleaned)
    } else {
        Err("The model returned an unusable quote.".into())
    }
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

async fn ollama_generate(settings: &Settings, prompt: &str) -> Result<String, String> {
    let res: OllamaResponse = client(30)
        .post(format!("{}/api/generate", settings.ollama_url))
        .json(&json!({ "model": settings.ollama_model, "prompt": prompt, "stream": false }))
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    accept(&res.response)
}

#[derive(Deserialize)]
struct OllamaTags {
    models: Vec<OllamaModel>,
}
#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

/// Lists the models installed in the local Ollama instance.
pub async fn list_ollama_models(base_url: &str) -> Result<Vec<String>, String> {
    let tags: OllamaTags = client(5)
        .get(format!("{}/api/tags", base_url.trim_end_matches('/')))
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|_| "Ollama is not running (start it with `ollama serve`).".to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let mut names: Vec<String> = tags.models.into_iter().map(|m| m.name).collect();
    names.sort();
    Ok(names)
}

async fn openai_generate(api_key: &str, model: &str, prompt: &str) -> Result<String, String> {
    let res: serde_json::Value = client(30)
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({ "model": model, "max_tokens": 120, "messages": [{ "role": "user", "content": prompt }] }))
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    accept(res["choices"][0]["message"]["content"].as_str().unwrap_or_default())
}

async fn anthropic_generate(api_key: &str, model: &str, prompt: &str) -> Result<String, String> {
    let res: serde_json::Value = client(30)
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({ "model": model, "max_tokens": 120, "messages": [{ "role": "user", "content": prompt }] }))
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    accept(res["content"][0]["text"].as_str().unwrap_or_default())
}

pub fn clean_ai_quote(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(c, '"' | '\u{201C}' | '\u{201D}' | '*'))
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' => '\'',
            '\u{2013}' | '\u{2014}' => '-',
            c if c.is_whitespace() => ' ',
            c => c,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .to_uppercase()
}

pub fn bundled_quotes() -> Vec<String> {
    // Never panic at runtime if quotes.json is edited into invalid JSON; the unit tests catch that instead.
    serde_json::from_str::<Vec<String>>(include_str!("../data/quotes.json"))
        .ok()
        .filter(|quotes| !quotes.is_empty())
        .unwrap_or_else(|| vec!["KNOWLEDGE IS THE KEY THAT OPENS EVERY LOCKED DOOR".to_string()])
}

pub fn fallback_quote(length_choice: usize) -> String {
    let quotes = bundled_quotes();
    let target_range = match length_choice {
        1 => 5..=10,
        2 => 11..=20,
        _ => 21..=40,
    };
    let mut rng = rand::rngs::OsRng;
    quotes
        .iter()
        .filter(|quote| target_range.contains(&quote.split_whitespace().count()))
        .choose(&mut rng)
        .or_else(|| quotes.choose(&mut rng))
        .cloned()
        .unwrap_or_default()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_quotes_file_is_valid_json() {
        let quotes: Vec<String> = serde_json::from_str(include_str!("../data/quotes.json"))
            .expect("src-tauri/data/quotes.json must be a valid JSON array of strings (check for trailing commas)");
        assert!(quotes.len() > 10);
    }

    #[test]
    fn ai_quote_cleanup_strips_smart_quotes() {
        assert_eq!(clean_ai_quote("\u{201C}Be  brave,\n friend\u{201D}"), "BE BRAVE, FRIEND");
        assert!(accept("ok").is_err());
    }

    #[tokio::test]
    async fn offline_settings_use_bundled_library() {
        let settings = Settings { ollama_enabled: false, ..Default::default() };
        let q = get_quote(&settings, 2).await;
        assert_eq!(q.source, "Bundled quote library");
        assert!(!q.text.is_empty());
    }
}
