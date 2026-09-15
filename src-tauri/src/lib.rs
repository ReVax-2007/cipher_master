use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum CipherType {
    CustomFuthark,
    Caesar,
    Atbash,
    Substitution,
    Vigenere,
    RailFence,
    Affine,
    A1Z26,
    FillInTheBlank,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Difficulty {
    Easy,   
    Medium, 
    Hard,   
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[tauri::command(rename_all = "snake_case")]
fn generate_cipher_challenge(
    cipher_type: String,
    difficulty: String,
    length_choice: usize,
) -> Result<String, String> {
    let cipher = match cipher_type.as_str() {
        "CustomFuthark" => CipherType::CustomFuthark,
        "Caesar" => CipherType::Caesar,
        "Atbash" => CipherType::Atbash,
        "Substitution" => CipherType::Substitution,
        "Vigenere" => CipherType::Vigenere,
        "RailFence" => CipherType::RailFence,
        "Affine" => CipherType::Affine,
        "FillInTheBlank" => CipherType::FillInTheBlank,
        _ => CipherType::A1Z26,
    };

    let diff = match difficulty.as_str() {
        "Easy" => Difficulty::Easy,
        "Medium" => Difficulty::Medium,
        _ => Difficulty::Hard,
    };

    let raw_plaintext = get_ai_quote(length_choice);
    let formatted_plaintext = format_text_for_difficulty(&raw_plaintext, diff);
    let (ciphertext, secret_key, content_hint) = encrypt(&formatted_plaintext, cipher, diff);

    let desktop_path = get_desktop_path();
    let challenge_file = desktop_path.join("cipher_challenge.txt");
    let hint_file = desktop_path.join("cipher_hint.txt");
    let solution_file = desktop_path.join("cipher_solution.txt");

    let mut challenge_out = File::create(&challenge_file).map_err(|e| e.to_string())?;
    write!(
        challenge_out,
        "=== CIPHER CHALLENGE ===\nCipher: {:?}\nDifficulty: {:?}\n\n{}\n",
        cipher, diff, ciphertext
    ).map_err(|e| e.to_string())?;

    let mut hint_out = File::create(&hint_file).map_err(|e| e.to_string())?;
    write!(
        hint_out,
        "=== CIPHER SOLUTION HINTS & RIDDLE ===\nCipher Type: {:?}\n\nANSWER HINTS:\n{}\n",
        cipher, content_hint
    ).map_err(|e| e.to_string())?;

    let mut solution_out = File::create(&solution_file).map_err(|e| e.to_string())?;
    write!(
        solution_out,
        "=== CIPHER SOLUTION & KEY ===\nCipher Type: {:?}\nKey/Parameters: {}\n\nORIGINAL PLAINTEXT:\n{}\n",
        cipher, secret_key, raw_plaintext
    ).map_err(|e| e.to_string())?;

    Ok(format!(
        "SUCCESS!\nChallenge text:\n{}\n\nFiles saved to Desktop:\n- {}\n- {}\n- {}",
        ciphertext,
        challenge_file.display(),
        hint_file.display(),
        solution_file.display()
    ))
}

fn get_ai_quote(length_choice: usize) -> String {
    let word_count = match length_choice {
        1 => "5 to 8",
        2 => "12 to 18",
        _ => "25 to 35",
    };

    let prompt = format!(
        "Generate a single inspiring or philosophical quote between {} words. Return ONLY the quote text without quotes, explanation, or attribution.",
        word_count
    );

    let req = OllamaRequest {
        model: "llama3.2:1b",
        prompt: &prompt,
        stream: false,
    };

    match ureq::post("http://localhost:11434/api/generate")
        .timeout(std::time::Duration::from_secs(5))
        .send_json(&req)
    {
        Ok(res) => {
            if let Ok(ollama_res) = res.into_json::<OllamaResponse>() {
                let cleaned = ollama_res
                    .response
                    .trim()
                    .replace('"', "")
                    .replace('\n', " ")
                    .to_uppercase();
                if !cleaned.is_empty() {
                    return cleaned;
                }
            }
            fallback_quote(length_choice)
        }
        Err(_) => fallback_quote(length_choice),
    }
}

fn fallback_quote(length_choice: usize) -> String {
    let topics = ["VIKINGS", "KNOWLEDGE", "FORTUNE", "STRATEGY", "FREEDOM"];
    let actions = ["SAILED ACROSS THE UNKNOWN", "DISCOVERED HIDDEN TRUTHS", "FAVORED THE BRAVE MINDS", "PROTECTED LIBERTY WITH SECRETS"];
    let mut rng = rand::thread_rng();

    let base = format!("{} {}", topics.choose(&mut rng).unwrap(), actions.choose(&mut rng).unwrap());
    if length_choice > 1 {
        format!("{} AND UNLOCKED THE SECRETS OF THE ANCIENT RUNES", base)
    } else {
        base
    }
}

fn get_desktop_path() -> PathBuf {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .map(|p| p.join("Desktop"))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn format_text_for_difficulty(text: &str, diff: Difficulty) -> String {
    match diff {
        Difficulty::Easy => text.to_string(),
        Difficulty::Medium => text
            .to_uppercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace())
            .collect(),
        Difficulty::Hard => {
            let cleaned: String = text
                .to_uppercase()
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect();
            cleaned
                .as_bytes()
                .chunks(5)
                .map(|chunk| std::str::from_utf8(chunk).unwrap())
                .collect::<Vec<&str>>()
                .join(" ")
        }
    }
}

fn get_custom_futhark_map() -> HashMap<char, char> {
    let mut map = HashMap::new();
    map.insert('A', 'ᚨ'); map.insert('B', 'ᛒ'); map.insert('C', 'ᚲ');
    map.insert('D', 'ᛞ'); map.insert('E', 'ᛖ'); map.insert('F', 'ᚠ');
    map.insert('G', 'ᚷ'); map.insert('H', 'ᚺ'); map.insert('I', 'ᛁ');
    map.insert('J', 'ᛃ'); map.insert('K', 'ᚲ'); map.insert('L', 'ᛚ');
    map.insert('M', 'ᛗ'); map.insert('N', 'ᚾ'); map.insert('O', 'ᛟ');
    map.insert('P', 'ᛉ'); map.insert('Q', 'ᚲ'); map.insert('R', 'ᚱ');
    map.insert('S', 'ᛋ'); map.insert('T', 'ᛏ'); map.insert('U', 'ᚢ');
    map.insert('V', 'ᚹ'); map.insert('W', 'ᚹ'); map.insert('X', 'ᚦ');
    map.insert('Y', 'ᛇ'); map.insert('Z', 'ᛉ');
    map
}

fn encrypt(text: &str, cipher: CipherType, diff: Difficulty) -> (String, String, String) {
    let mut rng = rand::thread_rng();

    let (ciphertext, key_info, custom_hint) = match cipher {
        CipherType::CustomFuthark => {
            let map = get_custom_futhark_map();
            let encrypted = text
                .to_uppercase()
                .chars()
                .map(|c| *map.get(&c).unwrap_or(&c))
                .collect();
            (encrypted, "Elder Futhark Key Map".to_string(), None)
        }
        CipherType::Caesar => {
            let shift = rng.gen_range(1..25) as u8;
            let encrypted = text
                .chars()
                .map(|c| {
                    if c.is_ascii_alphabetic() {
                        let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                        (((c as u8 - base + shift) % 26) + base) as char
                    } else { c }
                })
                .collect();
            (encrypted, format!("Caesar Shift = {}", shift), None)
        }
        CipherType::Atbash => {
            let encrypted = text
                .chars()
                .map(|c| {
                    if c.is_ascii_alphabetic() {
                        let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                        ((25 - (c as u8 - base)) + base) as char
                    } else { c }
                })
                .collect();
            (encrypted, "Atbash (Reversed Alphabet)".to_string(), None)
        }
        CipherType::Substitution => {
            let mut alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
            let mut shuffled = alphabet.clone();
            shuffled.shuffle(&mut rng);
            let map: HashMap<char, char> = alphabet.drain(..).zip(shuffled.into_iter()).collect();
            let encrypted = text
                .chars()
                .map(|c| {
                    if c.is_ascii_alphabetic() {
                        let upper = c.to_ascii_uppercase();
                        let mapped = map[&upper];
                        if c.is_ascii_lowercase() { mapped.to_ascii_lowercase() } else { mapped }
                    } else { c }
                })
                .collect();
            let key_str = map.iter().map(|(k, v)| format!("{}->{}", k, v)).collect::<Vec<String>>().join(", ");
            (encrypted, format!("Substitution Table: [{}]", key_str), None)
        }
        CipherType::Vigenere => {
            let keys = vec!["ODIN", "THOR", "VALHALLA", "RUNE", "RUST", "CRYPT"];
            let key = *keys.choose(&mut rng).unwrap();
            let key_bytes = key.as_bytes();
            let mut key_idx = 0;

            let encrypted = text
                .chars()
                .map(|c| {
                    if c.is_ascii_alphabetic() {
                        let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                        let k = (key_bytes[key_idx % key_bytes.len()] - b'A') as u8;
                        key_idx += 1;
                        (((c as u8 - base + k) % 26) + base) as char
                    } else { c }
                })
                .collect();
            (encrypted, format!("Vigenere Passphrase: {}", key), None)
        }
        CipherType::RailFence => {
            let rails = rng.gen_range(2..=4);
            let mut fence: Vec<Vec<char>> = vec![Vec::new(); rails];
            let mut rail = 0;
            let mut direction = 1;

            for c in text.chars() {
                fence[rail].push(c);
                if rail == 0 { direction = 1; }
                else if rail == rails - 1 { direction = -1; }
                rail = (rail as i32 + direction) as usize;
            }

            let encrypted = fence.into_iter().flatten().collect();
            (encrypted, format!("Rail Fence Depth = {} Rails", rails), None)
        }
        CipherType::Affine => {
            let a_options = vec![3, 5, 7, 9, 11, 15, 17, 19, 21, 23];
            let a = *a_options.choose(&mut rng).unwrap();
            let b = rng.gen_range(1..20);

            let encrypted = text
                .chars()
                .map(|c| {
                    if c.is_ascii_alphabetic() {
                        let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                        let x = (c as u8 - base) as usize;
                        let enc = ((a * x + b) % 26) as u8;
                        (base + enc) as char
                    } else { c }
                })
                .collect();
            (encrypted, format!("Affine Cipher: E(x) = ({}x + {}) mod 26", a, b), None)
        }
        CipherType::A1Z26 => {
            let encrypted = text
                .chars()
                .filter_map(|c| {
                    if c.is_ascii_alphabetic() {
                        let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                        Some(format!("-{}", c as u8 - base + 1))
                    } else if c.is_whitespace() {
                        Some("   ".to_string())
                    } else { None }
                })
                .collect::<String>()
                .trim_start_matches('-')
                .to_string();
            (encrypted, "A1Z26 Position Mapping".to_string(), None)
        }
        CipherType::FillInTheBlank => {
            let (challenge, key, hint) = generate_fill_in_the_blank(text, diff, &mut rng);
            (challenge, key, Some(hint))
        }
    };

    let content_hint = custom_hint.unwrap_or_else(|| generate_content_hints(text, diff));
    (ciphertext, key_info, content_hint)
}

fn generate_fill_in_the_blank(
    text: &str,
    diff: Difficulty,
    rng: &mut impl Rng,
) -> (String, String, String) {
    let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
    let mut numbers: Vec<u8> = (1..=26).collect();
    numbers.shuffle(rng);
    let mapping: HashMap<char, u8> = alphabet.iter().copied().zip(numbers).collect();

    let challenge = text
        .chars()
        .map(|character| {
            if character.is_ascii_alphabetic() {
                format!("{:02} ", mapping[&character.to_ascii_uppercase()])
            } else if character.is_whitespace() {
                "   ".to_string()
            } else {
                character.to_string()
            }
        })
        .collect::<String>()
        .trim_end()
        .to_string();

    let used_letters: Vec<char> = alphabet
        .iter()
        .copied()
        .filter(|letter| text.to_ascii_uppercase().contains(*letter))
        .collect();
    let clue_count = match diff {
        Difficulty::Easy => 4,
        Difficulty::Medium => 3,
        Difficulty::Hard => 2,
    }
    .min(used_letters.len());
    let clues = used_letters
        .iter()
        .take(clue_count)
        .map(|letter| format!("{} = {:02}", letter, mapping[letter]))
        .collect::<Vec<String>>();

    let key = alphabet
        .iter()
        .map(|letter| format!("{}={:02}", letter, mapping[letter]))
        .collect::<Vec<String>>()
        .join(", ");
    let hint = format!(
        "FILL-IN-THE-BLANK RULES:\n- Each number represents one letter.\n- Every letter uses a unique number from 01 to 26.\n- Repeated numbers represent repeated letters.\n\nKNOWN LETTER MAPPINGS:\n{}",
        clues.join(", ")
    );

    (challenge, format!("Fill-in-the-Blank Number Key: {}", key), hint)
}

fn generate_content_hints(plaintext: &str, diff: Difficulty) -> String {
    let cleaned: Vec<char> = plaintext
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    let total_letters = cleaned.len();
    if total_letters == 0 {
        return "No alphabetic letters found in text.".to_string();
    }

    let mut counts: HashMap<char, usize> = HashMap::new();
    for &c in &cleaned {
        *counts.entry(c).or_insert(0) += 1;
    }

    let mut sorted_counts: Vec<(char, usize)> = counts.into_iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(&a.1));

    let top_letter = sorted_counts[0].0;
    let top_count = sorted_counts[0].1;

    let words: Vec<&str> = plaintext.split_whitespace().collect();
    let word_count = words.len();

    let riddle = format!(
        "RIDDLE HINT:\n\"Look for an idea involving the key word '{}' hidden beneath.\"",
        words.get(words.len() / 2).unwrap_or(&"TRUTH")
    );

    match diff {
        Difficulty::Easy => {
            let vowels: HashSet<char> = ['A', 'E', 'I', 'O', 'U'].iter().cloned().collect();
            let vowel_count: usize = cleaned.iter().filter(|c| vowels.contains(c)).count();
            format!(
                "{}\n\nSTRUCTURAL HINTS:\n\
                 - Word Count: {}\n\
                 - Most Common Letter in Solution: '{}' (Appears {} times)\n\
                 - Total Vowels in Solution: {}\n\
                 - First Word Length: {} letters | Last Word Length: {} letters",
                riddle,
                word_count,
                top_letter,
                top_count,
                vowel_count,
                words.first().unwrap_or(&"").len(),
                words.last().unwrap_or(&"").len(),
            )
        }
        Difficulty::Medium => {
            format!(
                "{}\n\nSTRUCTURAL HINTS:\n\
                 - Total Characters: {}\n\
                 - Word Count: {}\n\
                 - Most Common Character in Solution: '{}' ({}% of all characters)",
                riddle,
                total_letters,
                word_count,
                top_letter,
                (top_count * 100) / total_letters
            )
        }
        Difficulty::Hard => {
            format!(
                "{}\n\nSTRUCTURAL HINTS:\n\
                 - Total Characters: {}\n\
                 - Top Character Frequency Ratio: {}/{}",
                riddle, total_letters, top_count, total_letters
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_in_the_blank_uses_unique_stable_numbers_and_clues() {
        let (challenge, key, hint) = generate_fill_in_the_blank(
            "ABBA CAB",
            Difficulty::Easy,
            &mut rand::thread_rng(),
        );

        let numbers: Vec<&str> = challenge
            .split_whitespace()
            .filter(|token| token.chars().all(|character| character.is_ascii_digit()))
            .collect();
        assert_eq!(numbers.len(), 7);
        assert_eq!(numbers[1], numbers[2]);
        assert_eq!(numbers[0], numbers[3]);
        assert!(key.contains("A="));
        assert!(key.contains("Z="));
        assert!(hint.contains("KNOWN LETTER MAPPINGS:"));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![generate_cipher_challenge])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}