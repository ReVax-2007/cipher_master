//! User settings, stored as JSON in the app's config folder.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// Custom folder for the challenge files. `None` = Desktop (with fallbacks).
    pub output_dir: Option<String>,
    /// Write the three .txt files on every generation.
    pub save_files: bool,

    pub ollama_enabled: bool,
    pub ollama_url: String,
    pub ollama_model: String,

    /// "none", "openai" or "anthropic". Used when Ollama is off or unavailable.
    pub cloud_provider: String,
    pub cloud_api_key: String,
    pub cloud_model: String,

    pub github_token: String,
    pub gist_public: bool,
    pub discord_webhook_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            output_dir: None,
            save_files: true,
            ollama_enabled: true,
            ollama_url: "http://127.0.0.1:11434".into(),
            ollama_model: "llama3.2:1b".into(),
            cloud_provider: "none".into(),
            cloud_api_key: String::new(),
            cloud_model: String::new(),
            github_token: String::new(),
            gist_public: false,
            discord_webhook_url: String::new(),
        }
    }
}

impl Settings {
    pub fn default_cloud_model(provider: &str) -> &'static str {
        match provider {
            "openai" => "gpt-4o-mini",
            "anthropic" => "claude-haiku-4-5",
            _ => "",
        }
    }

    pub fn cloud_model_or_default(&self) -> String {
        if self.cloud_model.trim().is_empty() {
            Self::default_cloud_model(&self.cloud_provider).to_string()
        } else {
            self.cloud_model.trim().to_string()
        }
    }

    /// A copy safe to send to the UI: secrets are replaced with a marker telling whether one is set.
    pub fn redacted(&self) -> Settings {
        let mask = |s: &str| if s.is_empty() { String::new() } else { SECRET_MARKER.to_string() };
        Settings {
            cloud_api_key: mask(&self.cloud_api_key),
            github_token: mask(&self.github_token),
            ..self.clone()
        }
    }

    /// Applies an update from the UI, keeping existing secrets when the UI sends the marker back.
    pub fn merged_from_ui(&self, mut incoming: Settings) -> Settings {
        if incoming.cloud_api_key == SECRET_MARKER {
            incoming.cloud_api_key = self.cloud_api_key.clone();
        }
        if incoming.github_token == SECRET_MARKER {
            incoming.github_token = self.github_token.clone();
        }
        incoming.cloud_api_key = incoming.cloud_api_key.trim().to_string();
        incoming.github_token = incoming.github_token.trim().to_string();
        incoming.discord_webhook_url = incoming.discord_webhook_url.trim().to_string();
        incoming.ollama_url = incoming.ollama_url.trim().trim_end_matches('/').to_string();
        if incoming.ollama_url.is_empty() {
            incoming.ollama_url = Settings::default().ollama_url;
        }
        incoming.output_dir = incoming.output_dir.filter(|d| !d.trim().is_empty());
        incoming
    }
}

pub const SECRET_MARKER: &str = "••••••••";

pub fn settings_path(config_dir: &Path) -> PathBuf {
    config_dir.join("settings.json")
}

pub fn load(config_dir: &Path) -> Settings {
    std::fs::read_to_string(settings_path(config_dir))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub async fn save(config_dir: &Path, settings: &Settings) -> Result<(), String> {
    tokio::fs::create_dir_all(config_dir).await.map_err(|e| e.to_string())?;
    let path = settings_path(config_dir);
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, json).await.map_err(|e| format!("Could not save settings: {e}"))?;
    // The file can hold API keys: keep it private to the user on Unix-like systems.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_survive_a_redacted_round_trip() {
        let stored = Settings { cloud_api_key: "sk-123".into(), github_token: "ghp_1".into(), ..Default::default() };
        let from_ui = stored.redacted();
        assert_eq!(from_ui.cloud_api_key, SECRET_MARKER);
        let merged = stored.merged_from_ui(from_ui);
        assert_eq!(merged.cloud_api_key, "sk-123");
        assert_eq!(merged.github_token, "ghp_1");
    }

    #[test]
    fn missing_fields_use_defaults() {
        let s: Settings = serde_json::from_str(r#"{"ollama_model":"phi3"}"#).unwrap();
        assert_eq!(s.ollama_model, "phi3");
        assert!(s.save_files);
    }
}
