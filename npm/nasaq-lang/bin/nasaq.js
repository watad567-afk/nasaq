#!/usr/bin/env node
/**
 * @nasaq/lang — CLI wrapper
 * Prefers `nasaq` on PATH (from `cargo install`) or NASAQ_BIN env.
 */
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const candidates = [
  process.env.NASAQ_BIN,
  join(here, "..", "..", "target", "release", process.platform === "win32" ? "nasaq.exe" : "nasaq"),
  "nasaq",
].filter(Boolean);

let bin = null;
for (const c of candidates) {
  if (c === "nasaq" || existsSync(c)) {
    bin = c;
    break;
  }
}

if (!bin) {
  console.error(`Nasaq compiler not found.
Install from source:
  git clone https://github.com/nasaq-lang/nasaq
  cd nasaq && cargo build --release
  cargo install --path crates/nasaq_cli

Or set NASAQ_BIN to your nasaq executable.`);
  process.exit(1);
}

const result = spawnSync(bin, process.argv.slice(2), { stdio: "inherit", shell: bin === "nasaq" });
process.exit(result.status ?? 1);
