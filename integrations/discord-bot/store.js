// Tiny JSON-file persistence so challenges survive a bot restart.
// For several bot instances, replace this with a shared database (e.g. Redis or Postgres).
import { readFileSync, writeFileSync, mkdirSync, renameSync } from "node:fs";
import { dirname } from "node:path";

export class ChallengeStore {
  constructor(path, maxAgeMs = 7 * 24 * 60 * 60 * 1000) {
    this.path = path;
    this.maxAgeMs = maxAgeMs;
    this.map = new Map();
    try { this.map = new Map(Object.entries(JSON.parse(readFileSync(path, "utf8")))); } catch { /* first run */ }
    this.prune();
  }
  get(id) { return this.map.get(id); }
  set(id, value) { this.map.set(id, value); this.save(); }
  prune() { const cutoff = Date.now() - this.maxAgeMs; for (const [id, c] of this.map) if ((c.createdAt ?? 0) < cutoff) this.map.delete(id); }
  save() {
    this.prune();
    mkdirSync(dirname(this.path), { recursive: true });
    const tmp = this.path + ".tmp";
    writeFileSync(tmp, JSON.stringify(Object.fromEntries(this.map)));
    renameSync(tmp, this.path);
  }
}
