#!/usr/bin/env node
/** Rewrite internal codenames in documentation/ to enterprise-facing prose. */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { enterpriseProseRulesPart1 } from "./enterprise-prose-rules-part1.mjs";
import { enterpriseProseRulesPart2 } from "./enterprise-prose-rules-part2.mjs";

const repoRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const targetDir = path.join(repoRoot, "documentation");

const rules = [...enterpriseProseRulesPart1, ...enterpriseProseRulesPart2];

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

/** @param {string} content */
function applyEnterpriseProse(content) {
  let text = content;

  for (const [pattern, replacement] of rules) {
    text = text.replace(pattern, replacement);
  }

  text = text.replace(/  +/g, " ");
  text = text.replace(/ +\./g, ".");
  text = text.replace(/ +\|/g, " |");
  text = text.replace(/ \(\)/g, "");
  text = text.replace(/,\s*,/g, ",");

  return text;
}

function main() {
  if (!fs.existsSync(targetDir)) {
    console.error(`normalize-documentation-prose: missing ${targetDir}`);
    process.exit(1);
  }

  let count = 0;
  for (const filePath of walkMarkdownFiles(targetDir)) {
    const original = fs.readFileSync(filePath, "utf8");
    const updated = applyEnterpriseProse(original);
    if (updated !== original) {
      fs.writeFileSync(filePath, updated, "utf8");
      count += 1;
    }
  }

  console.log(`normalize-documentation-prose: updated ${count} files`);
}

main();
