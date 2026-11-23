import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { CIPHERS, createChallenge, describeKey, hintFor } from "../ciphers.js";
import { ChallengeStore } from "../store.js";

const SAMPLE = "KNOWLEDGE IS THE KEY, MY FRIEND";
const letters = (s) => s.replace(/[^A-Z]/g, "");

test("every bot cipher round-trips", () => {
  for (const [type, engine] of Object.entries(CIPHERS)) {
    for (let i = 0; i < 25; i++) {
      const c = createChallenge(SAMPLE, type);
      assert.equal(letters(engine.decrypt(c.text, c.key)), letters(SAMPLE), type);
      assert.ok(describeKey(c).length > 0 && hintFor(c).length > 0);
    }
  }
});

test("known answers match the desktop app", () => {
  assert.equal(CIPHERS.a1z26.encrypt("HELLO WORLD"), "8-5-12-12-15   23-15-18-12-4");
  assert.equal(CIPHERS.baconian.encrypt("AB Z"), "AAAAA AAAAB / BBAAB");
  assert.equal(CIPHERS.caesar.encrypt("HELLO", { shift: 3 }), "KHOOR");
});

test("store persists challenges across restarts", () => {
  const path = join(mkdtempSync(join(tmpdir(), "cm-bot-")), "challenges.json");
  new ChallengeStore(path).set("g:u", createChallenge(SAMPLE, "atbash"));
  assert.equal(new ChallengeStore(path).get("g:u").type, "atbash");
});
