/** Execute a compiled `.nq` module via Node (no `.js` extension required). */
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const target = process.argv[2];
if (!target) {
  console.error("usage: node nq-run.nqr <module.nq>");
  process.exit(1);
}

const source = readFileSync(target, "utf8");
const url = `data:text/javascript;charset=utf-8,${encodeURIComponent(source)}`;
await import(url);
