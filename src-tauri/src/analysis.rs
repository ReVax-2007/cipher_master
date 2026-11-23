//! Cryptanalysis metrics shared by the hint generator and the in-app decryption workspace.

use serde::Serialize;
use std::collections::HashMap;

/// Relative English letter frequencies (percent), A..Z.
pub const ENGLISH_FREQ: [f64; 26] = [
    8.2, 1.5, 2.8, 4.3, 12.7, 2.2, 2.0, 6.1, 7.0, 0.15, 0.77, 4.0, 2.4, 6.7, 7.5, 1.9, 0.095, 6.0,
    6.3, 9.1, 2.8, 0.98, 2.4, 0.15, 2.0, 0.074,
];

#[derive(Debug, Clone, Serialize)]
pub struct Analysis {
    /// Number of A–Z letters.
    pub letters: usize,
    /// Count of each letter A..Z.
    pub counts: [usize; 26],
    /// Index of coincidence over the letters (or over all symbols when there are no letters).
    pub ic: f64,
    /// Plain-language reading of the IC value.
    pub ic_reading: String,
    pub top_bigrams: Vec<(String, usize)>,
    pub top_trigrams: Vec<(String, usize)>,
    /// Most common non-space symbols (useful for runes, numbers and A/B groups).
    pub top_symbols: Vec<(String, usize)>,
    pub english_freq: [f64; 26],
}

fn top_n(map: HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = map.into_iter().filter(|(_, c)| *c > 1).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

fn index_of_coincidence(counts: impl Iterator<Item = usize>) -> f64 {
    let (mut total, mut sum) = (0usize, 0usize);
    for c in counts {
        total += c;
        sum += c * c.saturating_sub(1);
    }
    if total < 2 { 0.0 } else { sum as f64 / (total * (total - 1)) as f64 }
}

pub fn analyze(text: &str) -> Analysis {
    let letters: Vec<char> = text.chars().filter(|c| c.is_ascii_alphabetic()).map(|c| c.to_ascii_uppercase()).collect();
    let mut counts = [0usize; 26];
    for c in &letters {
        counts[(*c as u8 - b'A') as usize] += 1;
    }

    // N-grams are counted inside words so word gaps don't create fake pairs.
    let mut bigrams = HashMap::new();
    let mut trigrams = HashMap::new();
    for word in text.split_whitespace() {
        let w: Vec<char> = word.chars().filter(|c| c.is_ascii_alphabetic()).map(|c| c.to_ascii_uppercase()).collect();
        for win in w.windows(2) {
            *bigrams.entry(win.iter().collect::<String>()).or_insert(0) += 1;
        }
        for win in w.windows(3) {
            *trigrams.entry(win.iter().collect::<String>()).or_insert(0) += 1;
        }
    }

    let mut symbols: HashMap<String, usize> = HashMap::new();
    let tokenized_numbers = text.chars().any(|c| c.is_ascii_digit()) && letters.is_empty();
    if tokenized_numbers {
        for tok in text.split(|c: char| !c.is_ascii_digit()).filter(|t| !t.is_empty()) {
            *symbols.entry(tok.to_string()).or_insert(0) += 1;
        }
    } else {
        for c in text.chars().filter(|c| !c.is_whitespace()) {
            *symbols.entry(c.to_string()).or_insert(0) += 1;
        }
    }

    let ic = if letters.len() >= 2 {
        index_of_coincidence(counts.iter().copied())
    } else {
        index_of_coincidence(symbols.values().copied())
    };
    let ic_reading = if ic == 0.0 {
        "Not enough text to measure.".to_string()
    } else if ic > 0.058 {
        "Close to English (~0.067): likely a transposition or a single-alphabet substitution.".to_string()
    } else if ic > 0.045 {
        "Between English and random: possibly a short-key polyalphabetic cipher.".to_string()
    } else {
        "Close to random (~0.038): likely a polyalphabetic cipher (Vigenere, Enigma) or a chain.".to_string()
    };

    Analysis {
        letters: letters.len(),
        counts,
        ic,
        ic_reading,
        top_bigrams: top_n(bigrams, 6),
        top_trigrams: top_n(trigrams, 4),
        top_symbols: top_n(symbols, 8),
        english_freq: ENGLISH_FREQ,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ic_of_english_is_higher_than_random() {
        let english = analyze("THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG AND THEN THE DOG SLEEPS UNDER THE TREE IN THE SUN");
        let uniform = analyze("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        assert!(english.ic > uniform.ic);
        assert_eq!(uniform.ic, 0.0);
        assert!(english.top_bigrams.iter().take(2).any(|(b, _)| b == "TH"));
    }

    #[test]
    fn numeric_text_uses_number_tokens() {
        let a = analyze("08 05 12 12 15   23 15");
        assert_eq!(a.letters, 0);
        assert!(a.top_symbols.iter().any(|(s, c)| s == "12" && *c == 2));
    }
}
