// Cipher engines for the Discord bot. They mirror the desktop app (src-tauri/src/ciphers.rs) for the
// ciphers that read well in a chat message.
import { randomInt } from "node:crypto";

const A = 65;
const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const KEYWORDS = ["ODIN", "THOR", "VALHALLA", "RUNE", "RUST", "CRYPT", "FREYA", "LOKI", "MJOLNIR", "YGGDRASIL", "BIFROST", "SKALD"];
export const FUTHARK = [..."ᚨᛒᚲᛞᛖᚠᚷᚺᛁᛃᚴᛚᛗᚾᛟᛈᛩᚱᛋᛏᚢᚡᚹᛪᛇᛉ"];

const pick = (list) => list[randomInt(list.length)];
const mapLetters = (text, f) => text.replace(/[A-Z]/g, (c) => String.fromCharCode(A + (((f(c.charCodeAt(0) - A) % 26) + 26) % 26)));
const modInverse = (a) => { for (let x = 1; x < 26; x++) if ((a * x) % 26 === 1) return x; return 1; };
function shuffled(items) { const a = [...items]; for (let i = a.length - 1; i > 0; i--) { const j = randomInt(i + 1); [a[i], a[j]] = [a[j], a[i]]; } return a; }
function railPattern(len, rails) { const cycle = 2 * (rails - 1); return Array.from({ length: len }, (_, i) => { const p = i % cycle; return p < rails ? p : cycle - p; }); }

export const CIPHERS = {
  caesar: {
    label: "Caesar",
    key: () => ({ shift: randomInt(1, 26) }),
    encrypt: (t, k) => mapLetters(t, (x) => x + k.shift),
    decrypt: (t, k) => mapLetters(t, (x) => x - k.shift),
    describe: (k) => `Caesar shift ${k.shift}`,
    hint: (k) => `The shift is ${k.shift % 2 ? "odd" : "even"} and ${k.shift < 13 ? "smaller" : "not smaller"} than 13.`,
  },
  atbash: {
    label: "Atbash",
    key: () => ({}),
    encrypt: (t) => mapLetters(t, (x) => 25 - x),
    decrypt: (t) => mapLetters(t, (x) => 25 - x),
    describe: () => "Atbash: reversed alphabet",
    hint: () => "Map each letter to its mirror in the alphabet (A<->Z).",
  },
  vigenere: {
    label: "Vigenere",
    key: () => ({ keyword: pick(KEYWORDS) }),
    encrypt: (t, k) => { let i = 0; return mapLetters(t, (x) => x + k.keyword.charCodeAt(i++ % k.keyword.length) - A); },
    decrypt: (t, k) => { let i = 0; return mapLetters(t, (x) => x - (k.keyword.charCodeAt(i++ % k.keyword.length) - A)); },
    describe: (k) => `Vigenere keyword ${k.keyword}`,
    hint: (k) => `The keyword has ${k.keyword.length} letters, starts with '${k.keyword[0]}' and has a Norse theme.`,
  },
  affine: {
    label: "Affine",
    key: () => ({ a: pick([3, 5, 7, 9, 11, 15, 17, 19, 21, 23, 25]), b: randomInt(1, 26) }),
    encrypt: (t, k) => mapLetters(t, (x) => k.a * x + k.b),
    decrypt: (t, k) => mapLetters(t, (x) => modInverse(k.a) * (x - k.b)),
    describe: (k) => `Affine E(x) = (${k.a}x + ${k.b}) mod 26`,
    hint: (k) => `The multiplier a = ${k.a}.`,
  },
  substitution: {
    label: "Substitution",
    key: () => ({ alphabet: shuffled(ALPHABET).join("") }),
    encrypt: (t, k) => mapLetters(t, (x) => k.alphabet.charCodeAt(x) - A),
    decrypt: (t, k) => mapLetters(t, (x) => k.alphabet.indexOf(String.fromCharCode(A + x))),
    describe: (k) => `Substitution: ${[...ALPHABET].map((p, i) => `${p}->${k.alphabet[i]}`).join(" ")}`,
    hint: (k) => `E->${k.alphabet[4]}, T->${k.alphabet[19]}, A->${k.alphabet[0]}.`,
  },
  railfence: {
    label: "Rail Fence",
    key: () => ({ rails: randomInt(2, 5) }),
    encrypt: (t, k) => { const chars = [...t]; const p = railPattern(chars.length, k.rails); let out = ""; for (let r = 0; r < k.rails; r++) chars.forEach((c, i) => { if (p[i] === r) out += c; }); return out; },
    decrypt: (t, k) => { const chars = [...t]; const p = railPattern(chars.length, k.rails); const order = chars.map((_, i) => i).sort((a, b) => p[a] - p[b] || a - b); const out = []; order.forEach((dst, src) => { out[dst] = chars[src]; }); return out.join(""); },
    describe: (k) => `Rail Fence with ${k.rails} rails`,
    hint: (k) => `The fence uses an ${k.rails % 2 ? "odd" : "even"} number of rails.`,
  },
  a1z26: {
    label: "A1Z26",
    key: () => ({}),
    encrypt: (t) => t.split(/\s+/).map((w) => [...w.replace(/[^A-Z]/g, "")].map((c) => c.charCodeAt(0) - A + 1).join("-")).filter(Boolean).join("   "),
    decrypt: (t) => t.split("   ").map((w) => w.split("-").map((n) => String.fromCharCode(A + Number(n) - 1)).join("")).join(" "),
    describe: () => "A1Z26: A=1 ... Z=26",
    hint: () => "Numbers are alphabet positions.",
  },
  baconian: {
    label: "Baconian",
    key: () => ({}),
    encrypt: (t) => t.split(/\s+/).map((w) => [...w.replace(/[^A-Z]/g, "")].map((c) => (c.charCodeAt(0) - A).toString(2).padStart(5, "0").replace(/0/g, "A").replace(/1/g, "B")).join(" ")).filter(Boolean).join(" / "),
    decrypt: (t) => t.split(" / ").map((w) => w.split(" ").map((g) => String.fromCharCode(A + parseInt(g.replace(/A/g, "0").replace(/B/g, "1"), 2))).join("")).join(" "),
    describe: () => "Baconian 26-letter variant (A=AAAAA)",
    hint: () => "Each 5-letter A/B group is a letter in binary (A=0, B=1).",
  },
  runes: {
    label: "Elder Futhark Runes",
    key: () => ({}),
    encrypt: (t) => t.replace(/[A-Z]/g, (c) => FUTHARK[c.charCodeAt(0) - A]),
    decrypt: (t) => [...t].map((c) => { const i = FUTHARK.indexOf(c); return i < 0 ? c : ALPHABET[i]; }).join(""),
    describe: () => `Runes: ${[...ALPHABET].map((l, i) => `${l}=${FUTHARK[i]}`).join(" ")}`,
    hint: () => "ᚨ=A, ᛖ=E, ᛏ=T.",
  },
};

export function createChallenge(plaintext, type) {
  const engine = CIPHERS[type];
  if (!engine) throw new Error(`Unknown cipher type: ${type}`);
  const text = plaintext.toUpperCase();
  const key = engine.key();
  return { type, label: engine.label, plaintext: text, key, text: engine.encrypt(text, key), createdAt: Date.now() };
}

export const describeKey = (c) => CIPHERS[c.type].describe(c.key);
export const hintFor = (c) => CIPHERS[c.type].hint(c.key);
