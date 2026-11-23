//! Opt-in integrations: GitHub Gist, Discord webhooks, CyberChef deep links and CTF exports.

use serde_json::json;
use std::time::Duration;

use crate::Challenge;

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .user_agent(concat!("CipherMaster/", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap_or_default()
}

/// Publishes the challenge (and optionally hints/solution) to a GitHub Gist. Returns the Gist URL.
pub async fn publish_gist(
    token: &str,
    public: bool,
    challenge: &Challenge,
    include_hints: bool,
    include_solution: bool,
) -> Result<String, String> {
    if token.is_empty() {
        return Err("Add a GitHub token (with the \"gist\" scope) in Settings first.".into());
    }
    let mut files = serde_json::Map::new();
    files.insert("cipher_challenge.txt".into(), json!({ "content": challenge.challenge_text() }));
    if include_hints {
        files.insert("cipher_hint.txt".into(), json!({ "content": challenge.hint_text() }));
    }
    if include_solution {
        files.insert("cipher_solution.txt".into(), json!({ "content": challenge.solution_text() }));
    }
    let res = client()
        .post("https://api.github.com/gists")
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&json!({
            "description": format!("Cipher Master challenge - {}", challenge.cipher_label()),
            "public": public,
            "files": files,
        }))
        .send()
        .await
        .map_err(|e| format!("Could not reach GitHub: {e}"))?;
    let status = res.status();
    let body: serde_json::Value = res.json().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!(
            "GitHub rejected the Gist ({status}): {}",
            body["message"].as_str().unwrap_or("check that the token has the gist scope")
        ));
    }
    body["html_url"].as_str().map(str::to_string).ok_or_else(|| "GitHub did not return a Gist URL.".into())
}

pub fn is_discord_webhook(url: &str) -> bool {
    ["https://discord.com/api/webhooks/", "https://discordapp.com/api/webhooks/", "https://ptb.discord.com/api/webhooks/", "https://canary.discord.com/api/webhooks/"]
        .iter()
        .any(|p| url.starts_with(p))
}

/// Posts the challenge to a Discord channel through an incoming webhook.
pub async fn send_discord(webhook_url: &str, challenge: &Challenge) -> Result<(), String> {
    if !is_discord_webhook(webhook_url) {
        return Err("Add a Discord webhook URL (https://discord.com/api/webhooks/...) in Settings first.".into());
    }
    let mut body = challenge.ciphertext.clone();
    if body.chars().count() > 1700 {
        body = body.chars().take(1700).collect::<String>() + "...";
    }
    let content = format!(
        "**New Cipher Master challenge**\nCipher: **{}** | Difficulty: **{:?}**\n```\n{}\n```\nReply with your decryption!",
        challenge.cipher_label(),
        challenge.difficulty,
        body.replace("```", "'''")
    );
    let res = client()
        .post(webhook_url)
        .json(&json!({ "username": "Cipher Master", "content": content, "allowed_mentions": { "parse": [] } }))
        .send()
        .await
        .map_err(|e| format!("Could not reach Discord: {e}"))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Discord rejected the message ({}).", res.status()))
    }
}

fn base64(input: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        for i in 0..=chunk.len() {
            out.push(T[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

fn percent_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'(' | b')' | b',' | b'\'' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Builds a CyberChef link that loads the ciphertext and, where CyberChef has matching operations,
/// a recipe that undoes the cipher chain. Returns (url, fully_supported).
pub fn cyberchef_url(challenge: &Challenge) -> (String, bool) {
    let ops: Vec<Option<String>> = challenge.keys.iter().rev().map(|k| k.cyberchef_decode_op()).collect();
    let supported = ops.iter().all(Option::is_some);
    let recipe: String = if supported { ops.into_iter().flatten().collect() } else { String::new() };
    let mut url = String::from("https://gchq.github.io/CyberChef/#");
    if !recipe.is_empty() {
        url.push_str("recipe=");
        url.push_str(&percent_encode(&recipe));
        url.push('&');
    }
    url.push_str("input=");
    url.push_str(&base64(challenge.ciphertext.as_bytes()));
    (url, supported)
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn yaml_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"))
}

/// Files for a CTFd challenge (ctfcli `challenge.yml` format) plus a platform-neutral JSON.
/// Returns (relative path, contents) pairs.
pub fn ctf_export(challenge: &Challenge) -> Vec<(String, String)> {
    let name = format!("{} ({:?})", challenge.cipher_label(), challenge.difficulty);
    let folder = format!("ctf/{}-{}", slug(&name), challenge.created_unix);
    let points = match challenge.difficulty {
        crate::Difficulty::Easy => 100,
        crate::Difficulty::Medium => 250,
        crate::Difficulty::Hard => 500,
    } * challenge.keys.len() as u32;
    let flag = challenge.flag();
    let hints_yaml: String = challenge
        .hints
        .iter()
        .enumerate()
        .map(|(i, h)| format!("  - content: {}\n    cost: {}\n", yaml_str(h), 10 * (i as u32 + 1)))
        .collect();
    let description = format!(
        "Decrypt the attached ciphertext. Submit the plaintext as CTF{{WORDS_SEPARATED_BY_UNDERSCORES}}.\n\nCiphertext:\n{}",
        challenge.ciphertext
    );
    let yaml = format!(
        "# CTFd challenge generated by Cipher Master. Import with: ctf challenge install\nname: {}\nauthor: Cipher Master\ncategory: Crypto\ndescription: {}\nvalue: {points}\ntype: standard\nflags:\n  - type: static\n    content: {}\n    data: case_insensitive\nhints:\n{}files:\n  - dist/challenge.txt\ntags:\n  - crypto\n  - {}\nstate: visible\nversion: \"0.1\"\n",
        yaml_str(&name),
        yaml_str(&description),
        yaml_str(&flag),
        hints_yaml,
        slug(challenge.keys[0].cipher_type().id()),
    );
    let generic = serde_json::to_string_pretty(&json!({
        "name": name,
        "category": "Crypto",
        "points": points,
        "difficulty": format!("{:?}", challenge.difficulty),
        "ciphers": challenge.keys.iter().map(|k| k.cipher_type().label()).collect::<Vec<_>>(),
        "ciphertext": challenge.ciphertext,
        "flag": flag,
        "hints": challenge.hints,
        "solution": { "plaintext": challenge.plaintext, "keys": challenge.keys.iter().map(|k| k.describe()).collect::<Vec<_>>() },
    }))
    .unwrap_or_default();
    vec![
        (format!("{folder}/challenge.yml"), yaml),
        (format!("{folder}/dist/challenge.txt"), challenge.challenge_text()),
        (format!("{folder}/challenge.json"), generic),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_standard() {
        assert_eq!(base64(b"Hello"), "SGVsbG8");
        assert_eq!(base64(b"Ma"), "TWE");
        assert_eq!(base64(b"Man"), "TWFu");
    }

    #[test]
    fn discord_urls_are_validated() {
        assert!(is_discord_webhook("https://discord.com/api/webhooks/1/abc"));
        assert!(!is_discord_webhook("https://evil.example/api/webhooks/1"));
    }
}
