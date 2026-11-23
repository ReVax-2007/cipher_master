//! Cipher engines. Every cipher has an `encrypt` and a `decrypt` so challenges can be verified,
//! chained, and solved inside the app.

use rand::seq::SliceRandom;
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};

use crate::Difficulty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CipherType {
    CustomFuthark,
    Caesar,
    Atbash,
    Substitution,
    Vigenere,
    RailFence,
    Affine,
    A1Z26,
    FillInTheBlank,
    Playfair,
    Enigma,
    Baconian,
    Columnar,
}

impl CipherType {
    pub const ALL: [CipherType; 13] = [
        CipherType::CustomFuthark,
        CipherType::Caesar,
        CipherType::Atbash,
        CipherType::Substitution,
        CipherType::Vigenere,
        CipherType::RailFence,
        CipherType::Affine,
        CipherType::A1Z26,
        CipherType::FillInTheBlank,
        CipherType::Playfair,
        CipherType::Enigma,
        CipherType::Baconian,
        CipherType::Columnar,
    ];

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.id() == id)
    }

    pub fn id(self) -> &'static str {
        match self {
            CipherType::CustomFuthark => "CustomFuthark",
            CipherType::Caesar => "Caesar",
            CipherType::Atbash => "Atbash",
            CipherType::Substitution => "Substitution",
            CipherType::Vigenere => "Vigenere",
            CipherType::RailFence => "RailFence",
            CipherType::Affine => "Affine",
            CipherType::A1Z26 => "A1Z26",
            CipherType::FillInTheBlank => "FillInTheBlank",
            CipherType::Playfair => "Playfair",
            CipherType::Enigma => "Enigma",
            CipherType::Baconian => "Baconian",
            CipherType::Columnar => "Columnar",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CipherType::CustomFuthark => "Elder Futhark Runes",
            CipherType::Caesar => "Caesar Shift",
            CipherType::Atbash => "Atbash",
            CipherType::Substitution => "Monoalphabetic Substitution",
            CipherType::Vigenere => "Vigenere",
            CipherType::RailFence => "Rail Fence",
            CipherType::Affine => "Affine",
            CipherType::A1Z26 => "A1Z26 Numeric",
            CipherType::FillInTheBlank => "Fill in the Blank",
            CipherType::Playfair => "Playfair",
            CipherType::Enigma => "Enigma I",
            CipherType::Baconian => "Baconian",
            CipherType::Columnar => "Columnar Transposition",
        }
    }

    /// Ciphers whose output is no longer A–Z text (runes, numbers, A/B groups).
    /// They can only be the last stage of a chain.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            CipherType::CustomFuthark
                | CipherType::A1Z26
                | CipherType::FillInTheBlank
                | CipherType::Baconian
        )
    }
}

pub const MAX_CHAIN: usize = 4;

pub fn validate_chain(chain: &[CipherType]) -> Result<(), String> {
    if chain.is_empty() {
        return Err("Pick at least one cipher.".into());
    }
    if chain.len() > MAX_CHAIN {
        return Err(format!("A chain can have at most {MAX_CHAIN} ciphers."));
    }
    if let Some(bad) = chain[..chain.len() - 1].iter().find(|c| c.is_terminal()) {
        return Err(format!(
            "{} turns the text into symbols, so it can only be the last cipher in a chain.",
            bad.label()
        ));
    }
    if chain.iter().skip(1).any(|c| *c == CipherType::Playfair) {
        return Err("Playfair adds filler letters, so it must be the first cipher in a chain.".into());
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnigmaSettings {
    /// Rotor numbers 1–5, left to right.
    pub rotors: [u8; 3],
    /// Start positions 0–25 (A–Z), left to right.
    pub positions: [u8; 3],
    /// Ring settings 0–25 (A–Z), left to right.
    pub rings: [u8; 3],
    /// "B" or "C".
    pub reflector: String,
    /// Plugboard pairs such as "AQ".
    pub plugboard: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "cipher")]
pub enum CipherKey {
    CustomFuthark,
    Caesar { shift: u8 },
    Atbash,
    /// Cipher letter for A..Z, in order.
    Substitution { alphabet: String },
    Vigenere { keyword: String },
    RailFence { rails: usize },
    Affine { a: u32, b: u32 },
    A1Z26,
    /// Number (1–26) for A..Z, in order.
    FillInTheBlank { numbers: Vec<u8> },
    Playfair { keyword: String },
    Enigma(EnigmaSettings),
    Baconian,
    Columnar { keyword: String, passes: u8 },
}

const KEYWORDS: [&str; 12] = [
    "ODIN", "THOR", "VALHALLA", "RUNE", "RUST", "CRYPT", "FREYA", "LOKI", "MJOLNIR", "YGGDRASIL",
    "BIFROST", "SKALD",
];

const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub const FUTHARK: [char; 26] = [
    'ᚨ', 'ᛒ', 'ᚲ', 'ᛞ', 'ᛖ', 'ᚠ', 'ᚷ', 'ᚺ', 'ᛁ', 'ᛃ', 'ᚴ', 'ᛚ', 'ᛗ', 'ᚾ', 'ᛟ', 'ᛈ', 'ᛩ', 'ᚱ', 'ᛋ',
    'ᛏ', 'ᚢ', 'ᚡ', 'ᚹ', 'ᛪ', 'ᛇ', 'ᛉ',
];

impl CipherKey {
    pub fn random<R: Rng + CryptoRng>(cipher: CipherType, diff: Difficulty, rng: &mut R) -> Self {
        let keyword = || -> String { (*KEYWORDS.choose(&mut rand::rngs::OsRng).unwrap()).to_string() };
        match cipher {
            CipherType::CustomFuthark => CipherKey::CustomFuthark,
            CipherType::Caesar => CipherKey::Caesar { shift: rng.gen_range(1..=25) },
            CipherType::Atbash => CipherKey::Atbash,
            CipherType::Substitution => {
                let mut letters: Vec<char> = ALPHABET.chars().collect();
                letters.shuffle(rng);
                CipherKey::Substitution { alphabet: letters.into_iter().collect() }
            }
            CipherType::Vigenere => CipherKey::Vigenere { keyword: keyword() },
            CipherType::RailFence => CipherKey::RailFence {
                rails: match diff {
                    Difficulty::Hard => rng.gen_range(3..=5),
                    _ => rng.gen_range(2..=4),
                },
            },
            CipherType::Affine => CipherKey::Affine {
                a: *[3, 5, 7, 9, 11, 15, 17, 19, 21, 23, 25].choose(rng).unwrap(),
                b: rng.gen_range(1..26),
            },
            CipherType::A1Z26 => CipherKey::A1Z26,
            CipherType::FillInTheBlank => {
                let mut numbers: Vec<u8> = (1..=26).collect();
                numbers.shuffle(rng);
                CipherKey::FillInTheBlank { numbers }
            }
            CipherType::Playfair => CipherKey::Playfair { keyword: keyword() },
            CipherType::Enigma => {
                let mut rotors = [1u8, 2, 3, 4, 5];
                rotors.shuffle(rng);
                let pair_count = match diff {
                    Difficulty::Easy => 2,
                    Difficulty::Medium => 6,
                    Difficulty::Hard => 10,
                };
                let mut letters: Vec<char> = ALPHABET.chars().collect();
                letters.shuffle(rng);
                let plugboard = letters
                    .chunks(2)
                    .take(pair_count)
                    .map(|p| p.iter().collect())
                    .collect();
                CipherKey::Enigma(EnigmaSettings {
                    rotors: [rotors[0], rotors[1], rotors[2]],
                    positions: [rng.gen_range(0..26), rng.gen_range(0..26), rng.gen_range(0..26)],
                    rings: [rng.gen_range(0..26), rng.gen_range(0..26), rng.gen_range(0..26)],
                    reflector: if rng.gen_bool(0.5) { "B" } else { "C" }.to_string(),
                    plugboard,
                })
            }
            CipherType::Baconian => CipherKey::Baconian,
            CipherType::Columnar => CipherKey::Columnar {
                keyword: keyword(),
                passes: if matches!(diff, Difficulty::Hard) { 2 } else { 1 },
            },
        }
    }

    pub fn cipher_type(&self) -> CipherType {
        match self {
            CipherKey::CustomFuthark => CipherType::CustomFuthark,
            CipherKey::Caesar { .. } => CipherType::Caesar,
            CipherKey::Atbash => CipherType::Atbash,
            CipherKey::Substitution { .. } => CipherType::Substitution,
            CipherKey::Vigenere { .. } => CipherType::Vigenere,
            CipherKey::RailFence { .. } => CipherType::RailFence,
            CipherKey::Affine { .. } => CipherType::Affine,
            CipherKey::A1Z26 => CipherType::A1Z26,
            CipherKey::FillInTheBlank { .. } => CipherType::FillInTheBlank,
            CipherKey::Playfair { .. } => CipherType::Playfair,
            CipherKey::Enigma(_) => CipherType::Enigma,
            CipherKey::Baconian => CipherType::Baconian,
            CipherKey::Columnar { .. } => CipherType::Columnar,
        }
    }

    /// Full key, written to the solution file.
    pub fn describe(&self) -> String {
        match self {
            CipherKey::CustomFuthark => format!(
                "Elder Futhark rune map: {}",
                ALPHABET.chars().zip(FUTHARK).map(|(l, r)| format!("{l}={r}")).collect::<Vec<_>>().join(" ")
            ),
            CipherKey::Caesar { shift } => format!("Caesar shift = {shift} (decrypt by shifting back {shift})"),
            CipherKey::Atbash => "Atbash (A<->Z, B<->Y, ...)".into(),
            CipherKey::Substitution { alphabet } => format!(
                "Substitution table: {}",
                ALPHABET.chars().zip(alphabet.chars()).map(|(p, c)| format!("{p}->{c}")).collect::<Vec<_>>().join(", ")
            ),
            CipherKey::Vigenere { keyword } => format!("Vigenere keyword: {keyword}"),
            CipherKey::RailFence { rails } => format!("Rail Fence depth = {rails} rails"),
            CipherKey::Affine { a, b } => format!(
                "Affine E(x) = ({a}x + {b}) mod 26; D(x) = {}(x - {b}) mod 26",
                mod_inverse(*a).unwrap_or(0)
            ),
            CipherKey::A1Z26 => "A1Z26: A=1 ... Z=26, letters joined by '-', words separated by three spaces".into(),
            CipherKey::FillInTheBlank { numbers } => format!(
                "Fill-in-the-Blank number key: {}",
                ALPHABET.chars().zip(numbers).map(|(l, n)| format!("{l}={n:02}")).collect::<Vec<_>>().join(", ")
            ),
            CipherKey::Playfair { keyword } => format!(
                "Playfair keyword: {keyword} (I=J, X used as filler). Square:\n{}",
                playfair_square(keyword).chunks(5).map(|r| r.iter().collect::<String>()).collect::<Vec<_>>().join("\n")
            ),
            CipherKey::Enigma(s) => format!(
                "Enigma I - rotors (L-M-R): {} | start positions: {} | rings: {} | reflector {} | plugboard: {}",
                s.rotors.iter().map(|r| roman(*r)).collect::<Vec<_>>().join("-"),
                s.positions.iter().map(|p| (b'A' + p) as char).collect::<String>(),
                s.rings.iter().map(|p| (b'A' + p) as char).collect::<String>(),
                s.reflector,
                if s.plugboard.is_empty() { "none".into() } else { s.plugboard.join(" ") }
            ),
            CipherKey::Baconian => "Baconian (26-letter variant): A=AAAAA, B=AAAAB, ... Z=BBAAB; words separated by ' / '".into(),
            CipherKey::Columnar { keyword, passes } => format!(
                "Columnar transposition keyword: {keyword}, {passes} pass(es); read columns in alphabetical keyword order"
            ),
        }
    }

    /// A partial key clue for the progressive hint system.
    pub fn partial_hint(&self) -> String {
        match self {
            CipherKey::CustomFuthark => "Runes: ᚨ=A, ᛖ=E, ᛏ=T (the Rune Map tab has the rest).".into(),
            CipherKey::Caesar { shift } => format!(
                "The Caesar shift is {} and {} than 13.",
                if shift % 2 == 0 { "even" } else { "odd" },
                if *shift < 13 { "smaller" } else { "not smaller" }
            ),
            CipherKey::Atbash => "Each letter maps to its mirror: A<->Z, M<->N.".into(),
            CipherKey::Substitution { alphabet } => {
                let a: Vec<char> = alphabet.chars().collect();
                format!("Three substitution pairs: E->{}, T->{}, A->{}.", a[4], a[19], a[0])
            }
            CipherKey::Vigenere { keyword } => format!(
                "The Vigenere keyword has {} letters, starts with '{}', and has a Norse theme.",
                keyword.len(),
                &keyword[..1]
            ),
            CipherKey::RailFence { rails } => format!("The fence uses {} rails.", if rails % 2 == 0 { "an even number of" } else { "an odd number of" }),
            CipherKey::Affine { a, .. } => format!("The Affine multiplier a = {a}."),
            CipherKey::A1Z26 => "Numbers are alphabet positions: 1=A, 26=Z.".into(),
            CipherKey::FillInTheBlank { .. } => "Repeated numbers are repeated letters; look for short common words.".into(),
            CipherKey::Playfair { keyword } => format!(
                "The Playfair keyword has {} letters and starts with '{}'.",
                keyword.len(),
                &keyword[..1]
            ),
            CipherKey::Enigma(s) => format!(
                "Enigma rotor order is {} with reflector {}.",
                s.rotors.iter().map(|r| roman(*r)).collect::<Vec<_>>().join("-"),
                s.reflector
            ),
            CipherKey::Baconian => "Each 5-character A/B group is a letter in binary: A=0, B=1 (AAAAA=A).".into(),
            CipherKey::Columnar { keyword, passes } => format!(
                "The transposition keyword has {} letters ({} pass{}).",
                keyword.len(),
                passes,
                if *passes == 1 { "" } else { "es" }
            ),
        }
    }

    /// CyberChef operation(s) that undo this cipher, if CyberChef has a matching operation.
    pub fn cyberchef_decode_op(&self) -> Option<String> {
        match self {
            CipherKey::Caesar { shift } => Some(format!("ROT13(true,true,false,{})", 26 - *shift as u32)),
            CipherKey::Atbash => Some("Atbash_Cipher()".into()),
            CipherKey::Vigenere { keyword } => Some(format!("Vigenère_Decode('{keyword}')")),
            CipherKey::Affine { a, b } => Some(format!("Affine_Cipher_Decode({a},{b})")),
            CipherKey::RailFence { rails } => Some(format!("Rail_Fence_Cipher_Decode({rails},0)")),
            CipherKey::Substitution { alphabet } => Some(format!("Substitute('{alphabet}','{ALPHABET}')")),
            _ => None,
        }
    }
}

fn roman(n: u8) -> &'static str {
    ["?", "I", "II", "III", "IV", "V"].get(n as usize).copied().unwrap_or("?")
}

fn mod_inverse(a: u32) -> Option<u32> {
    (1..26).find(|x| (a * x) % 26 == 1)
}

/// Applies `f` to the 0–25 index of every ASCII letter, preserving case and leaving other characters alone.
fn map_letters(text: &str, mut f: impl FnMut(u8) -> u8) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_alphabetic() {
                let base = if c.is_ascii_uppercase() { b'A' } else { b'a' };
                (base + f(c as u8 - base) % 26) as char
            } else {
                c
            }
        })
        .collect()
}

fn letters_only(text: &str) -> Vec<u8> {
    text.chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase() as u8 - b'A')
        .collect()
}

fn group(text: &str, size: usize) -> String {
    text.chars()
        .collect::<Vec<char>>()
        .chunks(size)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

fn compact(text: &str) -> String {
    text.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect()
}

/// Applies the ciphers left to right. A single cipher keeps the text's formatting. In a chain every
/// stage works on compact A–Z/0–9 text (no spaces or punctuation), so transpositions and symbol
/// ciphers can be stacked and still be undone exactly.
pub fn encrypt_chain(text: &str, keys: &[CipherKey]) -> String {
    if keys.len() == 1 {
        return encrypt(text, &keys[0]);
    }
    let out = keys.iter().fold(text.to_string(), |acc, key| encrypt(&compact(&acc), key));
    match keys.last() {
        Some(last) if !last.cipher_type().is_terminal() && !out.contains(' ') => group(&out, 5),
        _ => out,
    }
}

/// Undoes the chain right to left, returning the text after each undone stage (last = plaintext).
pub fn decrypt_chain_steps(text: &str, keys: &[CipherKey]) -> Vec<String> {
    if keys.len() == 1 {
        return vec![decrypt(text, &keys[0])];
    }
    let mut steps = Vec::new();
    let mut current = text.to_string();
    for key in keys.iter().rev() {
        let input = if key.cipher_type().is_terminal() { current.clone() } else { compact(&current) };
        current = compact(&decrypt(&input, key));
        steps.push(current.clone());
    }
    steps
}

#[cfg(test)]
pub fn decrypt_chain(text: &str, keys: &[CipherKey]) -> String {
    decrypt_chain_steps(text, keys).pop().unwrap_or_default()
}

pub fn encrypt(text: &str, key: &CipherKey) -> String {
    match key {
        CipherKey::CustomFuthark => text
            .chars()
            .map(|c| {
                if c.is_ascii_alphabetic() {
                    FUTHARK[(c.to_ascii_uppercase() as u8 - b'A') as usize]
                } else {
                    c
                }
            })
            .collect(),
        CipherKey::Caesar { shift } => map_letters(text, |x| x + shift),
        CipherKey::Atbash => map_letters(text, |x| 25 - x),
        CipherKey::Substitution { alphabet } => {
            let table: Vec<u8> = alphabet.bytes().map(|b| b - b'A').collect();
            map_letters(text, |x| table[x as usize])
        }
        CipherKey::Vigenere { keyword } => {
            let k: Vec<u8> = keyword.bytes().map(|b| b - b'A').collect();
            let mut i = 0;
            map_letters(text, |x| {
                let out = x + k[i % k.len()];
                i += 1;
                out
            })
        }
        CipherKey::RailFence { rails } => {
            let chars: Vec<char> = text.chars().collect();
            let pattern = rail_pattern(chars.len(), *rails);
            (0..*rails)
                .flat_map(|r| chars.iter().zip(&pattern).filter(move |(_, p)| **p == r).map(|(c, _)| *c))
                .collect()
        }
        CipherKey::Affine { a, b } => map_letters(text, |x| ((a * x as u32 + b) % 26) as u8),
        CipherKey::A1Z26 => text
            .split_whitespace()
            .map(|word| {
                letters_only(word).iter().map(|x| (x + 1).to_string()).collect::<Vec<_>>().join("-")
            })
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join("   "),
        CipherKey::FillInTheBlank { numbers } => text
            .chars()
            .map(|c| {
                if c.is_ascii_alphabetic() {
                    format!("{:02} ", numbers[(c.to_ascii_uppercase() as u8 - b'A') as usize])
                } else if c.is_whitespace() {
                    "   ".to_string()
                } else {
                    c.to_string()
                }
            })
            .collect::<String>()
            .trim_end()
            .to_string(),
        CipherKey::Playfair { keyword } => {
            let square = playfair_square(keyword);
            let pairs = playfair_pairs(text);
            let out: String = pairs.iter().flat_map(|&(a, b)| playfair_step(&square, a, b, 1)).collect();
            group(&out, 2)
        }
        CipherKey::Enigma(settings) => Enigma::new(settings).run(text),
        CipherKey::Baconian => text
            .split_whitespace()
            .map(|word| {
                letters_only(word)
                    .iter()
                    .map(|x| (0..5).rev().map(|bit| if x >> bit & 1 == 1 { 'B' } else { 'A' }).collect::<String>())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" / "),
        CipherKey::Columnar { keyword, passes } => {
            let mut chars: Vec<char> = text.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect();
            for _ in 0..*passes {
                chars = columnar_encrypt(&chars, keyword);
            }
            group(&chars.into_iter().collect::<String>(), 5)
        }
    }
}

pub fn decrypt(text: &str, key: &CipherKey) -> String {
    match key {
        CipherKey::CustomFuthark => text
            .chars()
            .map(|c| FUTHARK.iter().position(|r| *r == c).map_or(c, |i| (b'A' + i as u8) as char))
            .collect(),
        CipherKey::Caesar { shift } => map_letters(text, |x| x + 26 - shift),
        CipherKey::Atbash => map_letters(text, |x| 25 - x),
        CipherKey::Substitution { alphabet } => {
            let mut inverse = [0u8; 26];
            for (plain, c) in alphabet.bytes().enumerate() {
                inverse[(c - b'A') as usize] = plain as u8;
            }
            map_letters(text, |x| inverse[x as usize])
        }
        CipherKey::Vigenere { keyword } => {
            let k: Vec<u8> = keyword.bytes().map(|b| b - b'A').collect();
            let mut i = 0;
            map_letters(text, |x| {
                let out = x + 26 - k[i % k.len()];
                i += 1;
                out
            })
        }
        CipherKey::RailFence { rails } => {
            let chars: Vec<char> = text.chars().collect();
            let pattern = rail_pattern(chars.len(), *rails);
            let mut order: Vec<usize> = (0..chars.len()).collect();
            order.sort_by_key(|&i| (pattern[i], i));
            let mut out = vec![' '; chars.len()];
            for (src, &dst) in order.iter().enumerate() {
                out[dst] = chars[src];
            }
            out.into_iter().collect()
        }
        CipherKey::Affine { a, b } => {
            let inv = mod_inverse(*a).unwrap_or(1);
            map_letters(text, |x| ((inv * (x as u32 + 26 - b % 26)) % 26) as u8)
        }
        CipherKey::A1Z26 => text
            .split("   ")
            .map(|word| {
                word.split('-')
                    .filter_map(|n| n.trim().parse::<u8>().ok())
                    .filter(|n| (1..=26).contains(n))
                    .map(|n| (b'A' + n - 1) as char)
                    .collect::<String>()
            })
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        CipherKey::FillInTheBlank { numbers } => {
            let chars: Vec<char> = text.chars().collect();
            let mut out = String::new();
            let mut i = 0;
            while i < chars.len() {
                let c = chars[i];
                if c.is_ascii_digit() && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                    let n = (c as u8 - b'0') * 10 + (chars[i + 1] as u8 - b'0');
                    let letter = numbers.iter().position(|x| *x == n).map_or('?', |p| (b'A' + p as u8) as char);
                    out.push(letter);
                    i += 2;
                    if chars.get(i) == Some(&' ') {
                        i += 1;
                    }
                } else if c == ' ' {
                    let start = i;
                    while i < chars.len() && chars[i] == ' ' {
                        i += 1;
                    }
                    for _ in 0..((i - start) / 3).max(1) {
                        out.push(' ');
                    }
                } else {
                    out.push(c);
                    i += 1;
                }
            }
            out
        }
        CipherKey::Playfair { keyword } => {
            let square = playfair_square(keyword);
            let letters: Vec<u8> = letters_only(text);
            letters
                .chunks(2)
                .filter(|p| p.len() == 2)
                .flat_map(|p| {
                    playfair_step(&square, (b'A' + p[0]) as char, (b'A' + p[1]) as char, 4)
                })
                .collect()
        }
        CipherKey::Enigma(settings) => Enigma::new(settings).run(text),
        CipherKey::Baconian => text
            .split(" / ")
            .map(|word| {
                word.split_whitespace()
                    .filter(|g| g.len() == 5)
                    .map(|g| {
                        let v = g.chars().fold(0u8, |acc, c| acc << 1 | u8::from(c == 'B' || c == 'b'));
                        if v < 26 { (b'A' + v) as char } else { '?' }
                    })
                    .collect::<String>()
            })
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        CipherKey::Columnar { keyword, passes } => {
            let mut chars: Vec<char> = text.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
            for _ in 0..*passes {
                chars = columnar_decrypt(&chars, keyword);
            }
            chars.into_iter().collect()
        }
    }
}

fn rail_pattern(len: usize, rails: usize) -> Vec<usize> {
    let rails = rails.max(2);
    let cycle = 2 * (rails - 1);
    (0..len)
        .map(|i| {
            let p = i % cycle;
            if p < rails { p } else { cycle - p }
        })
        .collect()
}

fn column_order(keyword: &str) -> Vec<usize> {
    let k: Vec<char> = keyword.chars().collect();
    let mut order: Vec<usize> = (0..k.len()).collect();
    order.sort_by_key(|&i| (k[i], i));
    order
}

fn columnar_encrypt(chars: &[char], keyword: &str) -> Vec<char> {
    let cols = keyword.len().max(1);
    column_order(keyword)
        .into_iter()
        .flat_map(|col| chars.iter().skip(col).step_by(cols).copied())
        .collect()
}

fn columnar_decrypt(chars: &[char], keyword: &str) -> Vec<char> {
    let cols = keyword.len().max(1);
    let n = chars.len();
    let mut out = vec![' '; n];
    let mut pos = 0;
    for col in column_order(keyword) {
        let mut idx = col;
        while idx < n {
            out[idx] = chars[pos];
            pos += 1;
            idx += cols;
        }
    }
    out
}

pub fn playfair_square(keyword: &str) -> Vec<char> {
    let mut square = Vec::with_capacity(25);
    for c in keyword.chars().chain(ALPHABET.chars()) {
        let c = match c.to_ascii_uppercase() {
            'J' => 'I',
            c => c,
        };
        if c.is_ascii_uppercase() && !square.contains(&c) {
            square.push(c);
        }
    }
    square
}

fn playfair_pairs(text: &str) -> Vec<(char, char)> {
    let letters: Vec<char> = text
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| match c.to_ascii_uppercase() {
            'J' => 'I',
            c => c,
        })
        .collect();
    let filler = |c: char| if c == 'X' { 'Q' } else { 'X' };
    let mut pairs = Vec::new();
    let mut i = 0;
    while i < letters.len() {
        let a = letters[i];
        match letters.get(i + 1) {
            Some(&b) if b != a => {
                pairs.push((a, b));
                i += 2;
            }
            _ => {
                pairs.push((a, filler(a)));
                i += 1;
            }
        }
    }
    pairs
}

/// `step` = 1 to encrypt (move right/down), 4 to decrypt (move left/up, i.e. +4 mod 5).
fn playfair_step(square: &[char], a: char, b: char, step: usize) -> [char; 2] {
    let fix = |c: char| if c == 'J' { 'I' } else { c };
    let pa = square.iter().position(|&c| c == fix(a)).unwrap_or(0);
    let pb = square.iter().position(|&c| c == fix(b)).unwrap_or(0);
    let (ra, ca, rb, cb) = (pa / 5, pa % 5, pb / 5, pb % 5);
    if ra == rb {
        [square[ra * 5 + (ca + step) % 5], square[rb * 5 + (cb + step) % 5]]
    } else if ca == cb {
        [square[((ra + step) % 5) * 5 + ca], square[((rb + step) % 5) * 5 + cb]]
    } else {
        [square[ra * 5 + cb], square[rb * 5 + ca]]
    }
}

const ROTORS: [(&str, u8); 5] = [
    ("EKMFLGDQVZNTOWYHXUSPAIBRCJ", b'Q'),
    ("AJDKSIRUXBLHWTMCQGZNPYFVOE", b'E'),
    ("BDFHJLCPRTXVZNYEIWGAKMUSQO", b'V'),
    ("ESOVPZJAYQUIRHXLNFTGKDCMWB", b'J'),
    ("VZBRGITYUPSDNHLXAWMJQOFECK", b'Z'),
];

struct Enigma {
    forward: [[u8; 26]; 3],
    backward: [[u8; 26]; 3],
    notches: [u8; 3],
    positions: [u8; 3],
    rings: [u8; 3],
    reflector: [u8; 26],
    plugboard: [u8; 26],
}

impl Enigma {
    fn new(s: &EnigmaSettings) -> Self {
        let mut forward = [[0u8; 26]; 3];
        let mut backward = [[0u8; 26]; 3];
        let mut notches = [0u8; 3];
        for slot in 0..3 {
            let (wiring, notch) = ROTORS[(s.rotors[slot].clamp(1, 5) - 1) as usize];
            for (i, b) in wiring.bytes().enumerate() {
                forward[slot][i] = b - b'A';
                backward[slot][(b - b'A') as usize] = i as u8;
            }
            notches[slot] = notch - b'A';
        }
        let refl = if s.reflector == "C" { "FVPJIAOYEDRZXWGCTKUQSBNMHL" } else { "YRUHQSLDPXNGOKMIEBFZCWVJAT" };
        let mut reflector = [0u8; 26];
        for (i, b) in refl.bytes().enumerate() {
            reflector[i] = b - b'A';
        }
        let mut plugboard: [u8; 26] = core::array::from_fn(|i| i as u8);
        for pair in &s.plugboard {
            let p: Vec<u8> = pair.bytes().filter(|b| b.is_ascii_uppercase()).map(|b| b - b'A').collect();
            if p.len() == 2 {
                plugboard[p[0] as usize] = p[1];
                plugboard[p[1] as usize] = p[0];
            }
        }
        Enigma { forward, backward, notches, positions: s.positions, rings: s.rings, reflector, plugboard }
    }

    fn step(&mut self) {
        let p = &mut self.positions;
        if p[1] == self.notches[1] {
            p[0] = (p[0] + 1) % 26;
            p[1] = (p[1] + 1) % 26;
        } else if p[2] == self.notches[2] {
            p[1] = (p[1] + 1) % 26;
        }
        p[2] = (p[2] + 1) % 26;
    }

    fn pass(&self, slot: usize, c: u8, table: &[u8; 26]) -> u8 {
        let offset = (26 + self.positions[slot] - self.rings[slot]) % 26;
        (table[((c + offset) % 26) as usize] + 26 - offset) % 26
    }

    fn press(&mut self, c: u8) -> u8 {
        self.step();
        let mut x = self.plugboard[c as usize];
        for slot in (0..3).rev() {
            x = self.pass(slot, x, &self.forward[slot]);
        }
        x = self.reflector[x as usize];
        for slot in 0..3 {
            x = self.pass(slot, x, &self.backward[slot]);
        }
        self.plugboard[x as usize]
    }

    /// Enigma is reciprocal, so the same call encrypts and decrypts. Non-letters pass through.
    fn run(mut self, text: &str) -> String {
        text.chars()
            .map(|c| {
                if c.is_ascii_alphabetic() {
                    (b'A' + self.press(c.to_ascii_uppercase() as u8 - b'A')) as char
                } else {
                    c
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    fn norm(s: &str) -> String {
        s.chars().filter(|c| c.is_ascii_alphabetic()).map(|c| c.to_ascii_uppercase()).collect()
    }

    const SAMPLE: &str = "Knowledge is the key that opens every locked door, my friend";

    #[test]
    fn every_cipher_round_trips() {
        for cipher in CipherType::ALL {
            for diff in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
                for _ in 0..20 {
                    let key = CipherKey::random(cipher, diff, &mut OsRng);
                    let ct = encrypt(SAMPLE, &key);
                    let pt = decrypt(&ct, &key);
                    if cipher == CipherType::Playfair {
                        let strip = |s: &str| norm(s).replace('J', "I").replace(['X', 'Q'], "");
                        assert_eq!(strip(&pt), strip(SAMPLE), "{key:?}");
                    } else {
                        assert_eq!(norm(&pt), norm(SAMPLE), "{cipher:?} {key:?} -> {ct}");
                    }
                }
            }
        }
    }

    #[test]
    fn case_preserving_ciphers_are_exact() {
        for cipher in [CipherType::Caesar, CipherType::Atbash, CipherType::Substitution, CipherType::Vigenere, CipherType::RailFence, CipherType::Affine] {
            let key = CipherKey::random(cipher, Difficulty::Easy, &mut OsRng);
            assert_eq!(decrypt(&encrypt(SAMPLE, &key), &key), SAMPLE, "{cipher:?}");
        }
    }

    #[test]
    fn enigma_known_answer() {
        let key = CipherKey::Enigma(EnigmaSettings {
            rotors: [1, 2, 3],
            positions: [0, 0, 0],
            rings: [0, 0, 0],
            reflector: "B".into(),
            plugboard: vec![],
        });
        assert_eq!(encrypt("AAAAA", &key), "BDZGO");
    }

    #[test]
    fn enigma_double_step_known_answer() {
        // Rotors I-II-III, positions A-D-U, rings A: classic double-stepping example (ADU -> ADV -> AEW -> BFX).
        let key = CipherKey::Enigma(EnigmaSettings {
            rotors: [1, 2, 3],
            positions: [0, 3, 20],
            rings: [0, 0, 0],
            reflector: "B".into(),
            plugboard: vec![],
        });
        let ct = encrypt("AAAA", &key);
        assert_eq!(decrypt(&ct, &key), "AAAA");
        let mut e = Enigma::new(match &key { CipherKey::Enigma(s) => s, _ => unreachable!() });
        let mut seen = vec![];
        for _ in 0..3 {
            e.step();
            seen.push(e.positions.iter().map(|p| (b'A' + p) as char).collect::<String>());
        }
        assert_eq!(seen, ["ADV", "AEW", "BFX"]);
    }

    #[test]
    fn playfair_known_answer() {
        let key = CipherKey::Playfair { keyword: "PLAYFAIREXAMPLE".into() };
        assert_eq!(encrypt("Hide the gold in the tree stump", &key), "BM OD ZB XD NA BE KU DM UI XM MO UV IF");
    }

    #[test]
    fn columnar_known_answer() {
        let key = CipherKey::Columnar { keyword: "ZEBRAS".into(), passes: 1 };
        // Wikipedia example (irregular case, no padding).
        assert_eq!(norm(&encrypt("WE ARE DISCOVERED FLEE AT ONCE", &key)), "EVLNACDTESEAROFODEECWIREE");
    }

    #[test]
    fn baconian_and_a1z26_formats() {
        assert_eq!(encrypt("ab z", &CipherKey::Baconian), "AAAAA AAAAB / BBAAB");
        assert_eq!(encrypt("HELLO WORLD", &CipherKey::A1Z26), "8-5-12-12-15   23-15-18-12-4");
    }

    #[test]
    fn futhark_is_one_to_one() {
        let mut runes = FUTHARK.to_vec();
        runes.sort();
        runes.dedup();
        assert_eq!(runes.len(), 26);
    }

    #[test]
    fn random_chains_round_trip() {
        use rand::Rng;
        let mut rng = OsRng;
        for _ in 0..300 {
            let len = rng.gen_range(2..=MAX_CHAIN);
            let mut chain: Vec<CipherType> = (0..len - 1)
                .map(|_| *CipherType::ALL.iter().filter(|c| !c.is_terminal() && **c != CipherType::Playfair).collect::<Vec<_>>().choose(&mut rng).unwrap()).copied()
                .collect();
            chain.push(**CipherType::ALL.iter().filter(|c| **c != CipherType::Playfair).collect::<Vec<_>>().choose(&mut rng).unwrap());
            validate_chain(&chain).unwrap();
            let keys: Vec<CipherKey> = chain.iter().map(|c| CipherKey::random(*c, Difficulty::Hard, &mut rng)).collect();
            let ct = encrypt_chain(SAMPLE, &keys);
            assert_eq!(norm(&decrypt_chain(&ct, &keys)), norm(SAMPLE), "{chain:?}");
        }
    }

    #[test]
    fn chains_round_trip_and_validate() {
        let chain = [CipherType::Caesar, CipherType::RailFence, CipherType::Enigma, CipherType::Baconian];
        validate_chain(&chain).unwrap();
        let keys: Vec<CipherKey> = chain.iter().map(|c| CipherKey::random(*c, Difficulty::Medium, &mut OsRng)).collect();
        let ct = encrypt_chain(SAMPLE, &keys);
        assert_eq!(norm(&decrypt_chain(&ct, &keys)), norm(SAMPLE));
        assert!(validate_chain(&[CipherType::A1Z26, CipherType::Caesar]).is_err());
        assert!(validate_chain(&[]).is_err());
        assert!(validate_chain(&[CipherType::Caesar, CipherType::Playfair]).is_err());
        let chain = [CipherType::Playfair, CipherType::RailFence, CipherType::FillInTheBlank];
        validate_chain(&chain).unwrap();
        let keys: Vec<CipherKey> = chain.iter().map(|c| CipherKey::random(*c, Difficulty::Easy, &mut OsRng)).collect();
        let strip = |s: &str| norm(s).replace('J', "I").replace(['X', 'Q'], "");
        assert_eq!(strip(&decrypt_chain(&encrypt_chain(SAMPLE, &keys), &keys)), strip(SAMPLE));
    }
}
