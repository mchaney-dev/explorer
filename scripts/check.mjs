#!/usr/bin/env node
// Runs the same three gates as the CI Rust job, in order, stopping at the
// first failure. Mirrors .github/workflows/ci.yml, so a green run here means a
// green run in CI. Invoked by `npm run check` and the VS Code "Test" task.

import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const srcTauri = join(dirname(fileURLToPath(import.meta.url)), "..", "src-tauri");

const steps = [
  ["fmt", ["fmt", "--all", "--", "--check"]],
  ["clippy", ["clippy", "--all-targets", "--", "-D", "warnings"]],
  ["test", ["test"]],
];

for (const [name, args] of steps) {
  console.log(`\n=== ${name}: cargo ${args.join(" ")} ===`);
  try {
    execFileSync("cargo", args, { cwd: srcTauri, stdio: "inherit" });
  } catch {
    console.error(`\nFAILED: ${name} — fix it before pushing.`);
    process.exit(1);
  }
}
console.log("\nAll checks passed. Safe to push.");
