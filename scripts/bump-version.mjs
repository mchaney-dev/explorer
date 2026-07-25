#!/usr/bin/env node
// Bump the project version everywhere it lives, in one shot, so the git tag,
// package.json, Cargo.toml, tauri.conf.json, and Cargo.lock never drift apart.
//
// Usage:
//   node scripts/bump-version.mjs 0.2.0            # update the files only
//   node scripts/bump-version.mjs 0.2.0 --release  # also git commit + tag v0.2.0
//   node scripts/bump-version.mjs 0.2.0 --dry       # preview changes, write nothing
//
// After a --release run, push the commit and tag together with:
//   git push --follow-tags

import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

const args = process.argv.slice(2);
const flags = new Set(args.filter((a) => a.startsWith("--")));
const positional = args.filter((a) => !a.startsWith("--"));

const raw = positional[0];
if (!raw) {
  console.error("Error: version required, e.g. `node scripts/bump-version.mjs 0.2.0`");
  process.exit(1);
}

// Accept either "v0.2.0" or "0.2.0"; files store it without the leading v.
const version = raw.replace(/^v/, "");
if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(`Error: "${raw}" is not a valid semver version (expected X.Y.Z).`);
  process.exit(1);
}

const dry = flags.has("--dry");
const release = flags.has("--release");
const changes = [];

// >= 1.0.0 with no pre-release suffix is a stable release; 0.x, or any version
// carrying a "-suffix" (e.g. 1.0.0-rc1), is flagged as a GitHub pre-release.
const major = parseInt(version.split(".")[0], 10);
const isPreRelease = major < 1 || version.includes("-");

function updateJson(relPath) {
  const abs = join(root, relPath);
  const obj = JSON.parse(readFileSync(abs, "utf8"));
  changes.push({ relPath, abs, old: obj.version, next: (() => {
    obj.version = version;
    return JSON.stringify(obj, null, 2) + "\n";
  })() });
}

function updateCargoToml(relPath) {
  const abs = join(root, relPath);
  const lines = readFileSync(abs, "utf8").split("\n");
  let inPackage = false, done = false, old = null;
  for (let i = 0; i < lines.length; i++) {
    const t = lines[i].trim();
    if (t.startsWith("[")) {
      inPackage = t === "[package]";
    } else if (inPackage && !done) {
      const m = lines[i].match(/^version\s*=\s*"([^"]*)"/);
      if (m) {
        old = m[1];
        lines[i] = lines[i].replace(/"[^"]*"/, `"${version}"`);
        done = true;
      }
    }
  }
  if (!done) throw new Error(`Could not find [package] version in ${relPath}`);
  changes.push({ relPath, abs, old, next: lines.join("\n") });
}

function updateCargoLock(relPath) {
  const abs = join(root, relPath);
  if (!existsSync(abs)) {
    console.warn(`(skipping ${relPath}: not found)`);
    return;
  }
  const content = readFileSync(abs, "utf8");
  // Only the local `explorer` package entry, not any dependency.
  const re = /(name = "explorer"\r?\nversion = ")([^"]*)(")/;
  const m = content.match(re);
  if (!m) {
    console.warn(`(skipping ${relPath}: explorer package entry not found — run \`cargo check\` to sync it)`);
    return;
  }
  changes.push({ relPath, abs, old: m[2], next: content.replace(re, `$1${version}$3`) });
}

function updatePipelinePreRelease(relPath, isPre) {
  const abs = join(root, relPath);
  if (!existsSync(abs)) {
    console.warn(`(skipping ${relPath}: not found)`);
    return;
  }
  const content = readFileSync(abs, "utf8");
  const re = /^(\s*isPreRelease:\s*)(true|false)\s*$/m;
  const m = content.match(re);
  if (!m) {
    console.warn(`(skipping ${relPath}: isPreRelease flag not found)`);
    return;
  }
  const nextVal = String(isPre);
  if (m[2] === nextVal) {
    console.log(`  ${relPath.padEnd(28)} isPreRelease already ${nextVal}`);
    return;
  }
  changes.push({
    relPath,
    abs,
    old: `isPreRelease ${m[2]}`,
    to: `isPreRelease ${nextVal}`,
    next: content.replace(re, `$1${nextVal}`),
  });
}

updateJson("package.json");
updateJson("src-tauri/tauri.conf.json");
updateCargoToml("src-tauri/Cargo.toml");
updateCargoLock("src-tauri/Cargo.lock");
updatePipelinePreRelease("azure-pipelines.yml", isPreRelease);

console.log(`\nBumping to ${version}:`);
for (const c of changes) {
  console.log(`  ${c.relPath.padEnd(28)} ${c.old ?? "?"} -> ${c.to ?? version}`);
}

if (dry) {
  console.log("\n--dry: no files written.");
  process.exit(0);
}

for (const c of changes) writeFileSync(c.abs, c.next);
console.log("\nFiles updated.");

if (release) {
  const files = changes.map((c) => c.relPath);
  execFileSync("git", ["add", ...files], { cwd: root, stdio: "inherit" });
  execFileSync("git", ["commit", "-m", `Release v${version}`], { cwd: root, stdio: "inherit" });
  execFileSync("git", ["tag", `v${version}`], { cwd: root, stdio: "inherit" });
  console.log(`\nCommitted and tagged v${version}. Push with:\n  git push --follow-tags`);
} else {
  const files = changes.map((c) => c.relPath).join(" ");
  console.log(`\nNext:\n  git add ${files}\n  git commit -m "Release v${version}"\n  git tag v${version}\n  git push --follow-tags`);
}
