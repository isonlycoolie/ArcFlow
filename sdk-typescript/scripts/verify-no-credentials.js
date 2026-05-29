#!/usr/bin/env node
"use strict";

const { readFileSync, readdirSync, statSync } = require("node:fs");
const { join } = require("node:path");

const ROOT = join(__dirname, "..");
const PATTERNS = [/sk-[A-Za-z0-9]{20,}/, /AKIA[0-9A-Z]{16}/];

function walk(dir, files = []) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      if (entry === "node_modules" || entry === "target") continue;
      walk(full, files);
    } else if (/\.(ts|js|rs)$/.test(entry)) {
      files.push(full);
    }
  }
  return files;
}

const violations = [];
for (const file of walk(join(ROOT, "arcflow"))) {
  const text = readFileSync(file, "utf8");
  for (const pattern of PATTERNS) {
    if (pattern.test(text)) violations.push(`${file}: ${pattern}`);
  }
}

if (violations.length) {
  console.error("Credential-like patterns found:");
  violations.forEach((v) => console.error(`  ${v}`));
  process.exit(1);
}

console.log("OK: no credential patterns in TypeScript SDK sources");
