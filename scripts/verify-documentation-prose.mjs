#!/usr/bin/env node
// Fail if documentation markdown files contain internal planning language.

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const repoRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const docDir = path.join(repoRoot, "documentation");

const forbidden = [
  { name: "Sprint N", pattern: /\bSprint \d+\b/g },
  { name: "feat/fp-", pattern: /feat\/fp-/gi },
  { name: "FINAL-PRODUCTION", pattern: /FINAL-PRODUCTION/gi },
  { name: "ADR-N", pattern: /\bADR-\d+\b/g },
];

function walkMarkdownFiles(dir) {
  const files = [];
  if (!fs.existsSync(dir)) return files;

  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...walkMarkdownFiles(full));
    } else if (entry.name.endsWith(".md")) {
      files.push(full);
    }
  }

  return files;
}

function main() {
  if (!fs.existsSync(docDir)) {
    console.error(`verify-documentation-prose: missing ${docDir}`);
    process.exit(1);
  }

  const violations = [];

  for (const filePath of walkMarkdownFiles(docDir)) {
    const rel = path.relative(repoRoot, filePath).replace(/\\/g, "/");
    const content = fs.readFileSync(filePath, "utf8");
    const lines = content.split("\n");

    for (const rule of forbidden) {
      rule.pattern.lastIndex = 0;
      for (let i = 0; i < lines.length; i += 1) {
        const line = lines[i];
        rule.pattern.lastIndex = 0;
        if (rule.pattern.test(line)) {
          violations.push({ file: rel, line: i + 1, rule: rule.name, text: line.trim() });
        }
      }
    }
  }

  if (violations.length === 0) {
    console.log("OK: documentation prose check passed");
    return;
  }

  console.error("ERROR: internal planning language found in documentation/:");
  for (const v of violations) {
    console.error(`  ${v.file}:${v.line} [${v.rule}] ${v.text}`);
  }
  process.exit(1);
}

main();
