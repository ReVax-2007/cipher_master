// GUI tests for public/index.html. The Tauri backend is replaced by an in-page mock so the tests
// exercise the real frontend and the exact IPC contract (command names + argument shapes).
// Run: cd tests/gui && npm install && npx playwright install chromium && npm test
import { test, before, after } from "node:test";
import assert from "node:assert/strict";
import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, resolve } from "node:path";
import { chromium } from "playwright";

const page_url = pathToFileURL(resolve(dirname(fileURLToPath(import.meta.url)), "../../public/index.html")).href;
let browser;

function mockBackend() {
  const calls = [];
  window.__calls = calls;
  const ciphers = [
    ["CustomFuthark", "Elder Futhark Runes", true], ["Caesar", "Caesar Shift", false], ["Atbash", "Atbash", false],
    ["Vigenere", "Vigenere", false], ["Playfair", "Playfair", false], ["Enigma", "Enigma I", false], ["A1Z26", "A1Z26 Numeric", true],
  ].map(([id, label, terminal]) => ({ id, label, terminal }));
  const settings = { output_dir: null, save_files: true, ollama_enabled: true, ollama_url: "http://127.0.0.1:11434", ollama_model: "llama3.2:1b", cloud_provider: "none", cloud_api_key: "", cloud_model: "", github_token: "", gist_public: false, discord_webhook_url: "" };
  const hints = ["STRUCTURE\n- Word count: 2", "RIDDLE\nkey word 'WORLD'", "KEY (Caesar Shift)\nodd"];
  window.__TAURI__ = { core: { invoke: async (cmd, args) => {
    calls.push([cmd, args ?? null]);
    switch (cmd) {
      case "list_ciphers": return ciphers;
      case "futhark_map": return [["A", "ᚨ"], ["B", "ᛒ"], ["E", "ᛖ"], ["T", "ᛏ"]];
      case "get_settings": return settings;
      case "save_settings": return args.settings;
      case "list_ollama_models": return ["llama3.2:1b", "phi3"];
      case "generate_challenge": return { ciphertext: "KHOOR ZRUOG", cipher_label: "Caesar Shift", ciphers: args.request.ciphers, difficulty: args.request.difficulty, source: "Bundled quote library", hint_count: hints.length, files: ["/tmp/cipher_challenge.txt"], output_dir: "/tmp", cyberchef_supported: true, created_unix: 1 };
      case "analyze_text": return { letters: 10, counts: Array(26).fill(0).map((_, i) => (i === 14 ? 3 : 0)), ic: 0.0667, ic_reading: "Close to English", top_bigrams: [["OO", 1]], top_trigrams: [], top_symbols: [], english_freq: [] };
      case "get_hint": return hints[args.index];
      case "check_answer": return /^hello world$/i.test(args.guess.trim()) ? { correct: true, match_percent: 100 } : { correct: false, match_percent: 40 };
      case "reveal_solution": return { plaintext: "HELLO WORLD", keys: ["Caesar shift = 3"], flag: "CTF{HELLO_WORLD}" };
      case "publish_gist": return "https://gist.github.com/x/1";
      case "export_ctf": return ["/tmp/ctf/x/challenge.yml"];
      case "open_cyberchef": return { url: "https://gchq.github.io/CyberChef/#input=x", supported: true };
      default: return null;
    }
  } } };
}

async function openApp() {
  const page = await browser.newPage();
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.addInitScript(mockBackend);
  await page.goto(page_url);
  await page.waitForSelector("#chain select");
  return { page, errors, calls: () => page.evaluate(() => window.__calls) };
}

before(async () => { browser = await chromium.launch(); });
after(async () => { await browser?.close(); });

test("generates a challenge with the selected options", async () => {
  const { page, errors, calls } = await openApp();
  await page.selectOption("#chain select", "Caesar");
  await page.selectOption("#difficulty", "Hard");
  await page.selectOption("#length", "3");
  await page.click("#generate");
  await page.waitForFunction(() => document.getElementById("output").textContent === "KHOOR ZRUOG");
  const gen = (await calls()).find(([c]) => c === "generate_challenge");
  assert.deepEqual(gen[1], { request: { ciphers: ["Caesar"], difficulty: "Hard", length_choice: 3, custom_text: null } });
  assert.match(await page.textContent("#output-meta"), /cipher_challenge\.txt/);
  assert.deepEqual(errors, []);
});

test("chain builder sends stages in order and blocks invalid chains", async () => {
  const { page, calls } = await openApp();
  await page.click("#add-stage");
  // The default first stage is runes (a symbol cipher), so the new stage is inserted before it.
  assert.deepEqual(await page.$$eval("#chain select", (s) => s.map((x) => x.value)), ["Caesar", "CustomFuthark"]);
  assert.equal(await page.isDisabled("#generate"), false);
  await page.selectOption("#chain .chain-stage:nth-child(1) select", "Vigenere");
  await page.selectOption("#chain .chain-stage:nth-child(2) select", "A1Z26");
  await page.fill("#custom-text", "  my own text  ");
  await page.click("#generate");
  await page.waitForFunction(() => document.getElementById("status").textContent === "Challenge ready");
  const gen = (await calls()).find(([c]) => c === "generate_challenge");
  assert.deepEqual(gen[1].request.ciphers, ["Vigenere", "A1Z26"]);
  assert.equal(gen[1].request.custom_text, "my own text");
  await page.selectOption("#chain .chain-stage:nth-child(1) select", "A1Z26");
  assert.equal(await page.isDisabled("#generate"), true);
});

test("timed mode: hints, wrong and right answers, best score", async () => {
  const { page } = await openApp();
  await page.click("#generate");
  await page.waitForFunction(() => document.getElementById("status").textContent === "Challenge ready");
  await page.click("nav button[data-tab=solve]");
  assert.equal(await page.inputValue("#work-text"), "KHOOR ZRUOG");
  await page.click("#timer-start");
  await page.click("#hint");
  await page.waitForSelector("#hints li");
  assert.match(await page.textContent("#hint"), /2 left/);
  await page.fill("#answer", "nope");
  await page.click("#check");
  await page.waitForFunction(() => /40%/.test(document.getElementById("answer-result").textContent));
  await page.fill("#answer", "Hello World");
  await page.click("#check");
  await page.waitForFunction(() => /Correct!/.test(document.getElementById("answer-result").textContent));
  assert.match(await page.textContent("#answer-result"), /points/);
  assert.match(await page.textContent("#best-scores"), /Easy: \d+ pts/);
});

test("workspace tools: Caesar explorer and substitution scratchpad", async () => {
  const { page, calls } = await openApp();
  await page.click("nav button[data-tab=solve]");
  await page.fill("#work-text", "KHOOR");
  await page.locator("#shift").fill("3");
  assert.equal(await page.textContent("#shift-preview"), "HELLO");
  await page.fill('#submap input[data-letter="K"]', "h");
  assert.equal(await page.textContent("#sub-preview"), "Hhoor");
  await page.waitForFunction(() => window.__calls.some(([c]) => c === "analyze_text"));
  assert.ok((await calls()).some(([c, a]) => c === "analyze_text" && a.text === "KHOOR"));
  await page.waitForFunction(() => /Index of Coincidence/.test(document.getElementById("stats").textContent));
});

test("rune map translates both ways and buttons append runes", async () => {
  const { page } = await openApp();
  await page.click("nav button[data-tab=runes]");
  await page.fill("#latin-in", "beat");
  assert.equal(await page.inputValue("#rune-in"), "ᛒᛖᚨᛏ");
  await page.click("#rune-clear");
  await page.click('#rune-grid button[title="ᛏ = T"]');
  assert.equal(await page.inputValue("#latin-in"), "T");
});

test("settings save, model list and share actions use the IPC contract", async () => {
  const { page, calls } = await openApp();
  await page.click("nav button[data-tab=settings]");
  await page.click("#refresh-models");
  await page.waitForFunction(() => document.querySelectorAll("#set-ollama-model option").length === 2);
  await page.selectOption("#set-ollama-model", "phi3");
  await page.fill("#set-discord", "https://discord.com/api/webhooks/1/abc");
  await page.click("#save-settings");
  await page.waitForFunction(() => document.getElementById("settings-toast").textContent === "Settings saved.");
  const saved = (await calls()).filter(([c]) => c === "save_settings").pop()[1].settings;
  assert.equal(saved.ollama_model, "phi3");
  assert.equal(saved.discord_webhook_url, "https://discord.com/api/webhooks/1/abc");

  await page.click("nav button[data-tab=create]");
  assert.equal(await page.isDisabled("#gist"), true);
  await page.click("#generate");
  await page.waitForFunction(() => document.getElementById("status").textContent === "Challenge ready");
  await page.check("#gist-solution");
  await page.click("#gist");
  await page.waitForFunction(() => /Gist published/.test(document.getElementById("share-toast").textContent));
  const gist = (await calls()).find(([c]) => c === "publish_gist");
  assert.deepEqual(gist[1], { include_hints: false, include_solution: true });
});
