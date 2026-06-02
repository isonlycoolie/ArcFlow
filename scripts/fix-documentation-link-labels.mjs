#!/usr/bin/env node
// Rewrite markdown link labels that expose filenames into reader-facing titles.

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const repoRoot = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const docDir = path.join(repoRoot, "documentation");

const ACRONYMS = new Map([
  ["http", "HTTP"],
  ["https", "HTTPS"],
  ["api", "API"],
  ["sdk", "SDK"],
  ["cli", "CLI"],
  ["sql", "SQL"],
  ["sse", "SSE"],
  ["hitl", "HITL"],
  ["rag", "RAG"],
  ["otel", "OTel"],
  ["otlp", "OTLP"],
  ["postgres", "Postgres"],
  ["npm", "npm"],
  ["tsx", "tsx"],
  ["byo", "BYO"],
  ["rcs", "RCS"],
  ["wasm", "WASM"],
  ["ui", "UI"],
  ["id", "ID"],
  ["json", "JSON"],
  ["yaml", "YAML"],
  ["docker", "Docker"],
  ["vs", "VS"],
]);

const SLUG_OVERRIDES = new Map([
  ["http-api-reference", "HTTP API reference"],
  ["run-state-machine", "Run state machine"],
  ["maturity-and-known-gaps", "Maturity and known gaps"],
  ["sec-1-compliance", "Trace data policy compliance"],
  ["sec-1-rules", "Trace data policy rules"],
  ["sec-1-and-data-safety", "Trace data policy and data safety"],
  ["trace-events-normative", "Trace events (normative)"],
  ["trace-event-reference", "Trace event reference"],
  ["execution-traces", "Execution traces"],
  ["quickstart-python", "Python quickstart"],
  ["quickstart-typescript", "TypeScript quickstart"],
  ["quickstart-server-api", "Server API quickstart"],
  ["install-and-build", "Install and build"],
  ["admin-api-reference", "Admin API reference"],
  ["admin-api-contract", "Admin API contract"],
  ["03-admin-api-contract", "Admin API contract"],
  ["05-security-model", "Security model"],
  ["origin-and-rate-limiting", "Origin and rate limiting"],
  ["request-path", "Request path"],
  ["byo-relay-deployment", "BYO Relay deployment"],
  ["relay-byo-deployment", "Relay BYO deployment"],
  ["static-chat-widget", "Static chat widget"],
  ["knowledge-and-publish", "Knowledge and publish"],
  ["browser-sdk-api", "Browser SDK API"],
  ["security-model", "Security model"],
  ["site-lifecycle", "Site lifecycle"],
  ["workflow-registry", "Workflow registry"],
  ["graph-workflows", "Graph workflows"],
  ["validation-and-testing", "Validation and testing"],
  ["webhook-security", "Webhook security"],
  ["recovery-and-resume", "Recovery and resume"],
  ["hitl-overview", "Human-in-the-loop overview"],
  ["streaming-in-the-browser", "Streaming in the browser"],
  ["knowledge-ingestion", "Knowledge ingestion"],
  ["observability-otel", "OpenTelemetry observability"],
  ["postgres-schema", "Postgres schema"],
  ["the-rcs-contract", "The workflow specification contract"],
  ["rcs-schema", "RCS schema guide"],
  ["what-is-arcflow", "What is ArcFlow?"],
  ["architecture-overview", "Architecture overview"],
  ["first-linear-workflow", "First linear workflow"],
  ["hitl-approval-flow", "HITL approval flow"],
  ["external-webhook", "External webhook"],
  ["rag-chatbot", "RAG chatbot"],
  ["catalog", "Examples catalog"],
  ["chat-rag", "Chat RAG example"],
  ["online-application-chatbot", "Online application chatbot"],
  ["byo-docker", "BYO Docker relay"],
  ["arcflow-server", "Server API quickstart"],
  ["static-product", "Static product overview"],
  ["reliability", "Reliability guides"],
  ["workflow", "Workflow guides"],
  ["tutorials", "Tutorial tracks"],
  ["graph-view", "Graph view"],
  ["trace-timeline", "Trace timeline"],
  ["edge-alpha", "Edge WASM (alpha)"],
  ["provider-configuration", "Provider configuration"],
]);

function walkMarkdownFiles(dir) {
  const files = [];
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

function slugFromHref(href) {
  const withoutAnchor = href.split("#")[0];
  const base = path.basename(withoutAnchor).replace(/\.md$/i, "");
  return base.replace(/^\d+-/, "");
}

function titleFromSlug(slug) {
  if (SLUG_OVERRIDES.has(slug)) {
    return SLUG_OVERRIDES.get(slug);
  }

  const words = slug.split("-").map((word) => {
    const lower = word.toLowerCase();
    if (ACRONYMS.has(lower)) {
      return ACRONYMS.get(lower);
    }
    if (/^\d+$/.test(word)) {
      return word;
    }
    return word.charAt(0).toUpperCase() + word.slice(1).toLowerCase();
  });

  const title = words.join(" ");
  if (slug === "overview") {
    return "Overview";
  }
  if (slug === "idempotency") {
    return "Idempotency";
  }
  if (slug === "authentication") {
    return "Authentication";
  }
  if (slug === "validate") {
    return "Validate command";
  }
  if (slug === "migrate") {
    return "Migrate command";
  }
  if (slug === "trace") {
    return "Trace command";
  }
  if (slug === "run") {
    return "Run command";
  }
  if (slug === "init") {
    return "Init command";
  }
  return title;
}

function titleFromHref(href) {
  const slug = slugFromHref(href);
  const parts = href.split("#")[0].split("/").filter(Boolean);
  const baseTitle = titleFromSlug(slug);

  if (parts.length >= 2 && slug === "overview") {
    const section = parts[parts.length - 2];
    if (section === "relay") return "Relay overview";
    if (section === "server") return "Server overview";
    if (section === "static-product") return "Static product overview";
    if (section === "cli") return "CLI overview";
    if (section === "vscode") return "VS Code extension overview";
    if (section === "home") return "Documentation home";
  }

  if (parts.length >= 3 && parts[0] === ".." && parts[1] === "guides") {
    return baseTitle;
  }

  return baseTitle;
}

function looksLikeFilename(label) {
  if (/\.md$/i.test(label)) return true;
  if (/^[\w./-]+\/[\w.-]+$/.test(label) && label.includes("/")) return true;
  if (/^\d{2}-[\w-]+$/.test(label)) return true;
  if (/^[a-z0-9]+(?:-[a-z0-9]+)+\.md$/i.test(label)) return true;
  return false;
}

function labelMatchesHrefBasename(label, href) {
  const slug = slugFromHref(href);
  const normalized = label.replace(/\.md$/i, "").replace(/^\d+-/, "");
  return normalized.toLowerCase() === slug.toLowerCase();
}

function looksLikeSlugLabel(label) {
  if (label.includes(" ") || label.includes("/")) return false;
  if (/^[a-z][a-z0-9-]*$/.test(label) && label.includes("-")) return true;
  if (/^(reliability|workflow|tutorials|arcflow-server)$/.test(label)) return true;
  return false;
}

function isExternalHref(href) {
  return /^https?:\/\//i.test(href);
}

function shouldRewrite(label, href) {
  if (!href || isExternalHref(href)) return false;
  if (looksLikeFilename(label)) return true;
  if (looksLikeSlugLabel(label)) return true;
  if (labelMatchesHrefBasename(label, href)) return true;
  return false;
}

function rewriteLinks(content) {
  return content.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (match, label, href) => {
    if (!shouldRewrite(label, href)) {
      return match;
    }
    const newLabel = titleFromHref(href);
    if (newLabel === label) {
      return match;
    }
    return `[${newLabel}](${href})`;
  });
}

function main() {
  let fileCount = 0;
  let linkCount = 0;

  for (const filePath of walkMarkdownFiles(docDir)) {
    const original = fs.readFileSync(filePath, "utf8");
    let replacements = 0;
    const updated = original.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (match, label, href) => {
      if (!shouldRewrite(label, href)) {
        return match;
      }
      const newLabel = titleFromHref(href);
      if (newLabel === label) {
        return match;
      }
      replacements += 1;
      return `[${newLabel}](${href})`;
    });

    if (updated !== original) {
      fs.writeFileSync(filePath, updated, "utf8");
      fileCount += 1;
      linkCount += replacements;
      console.log(`${path.relative(repoRoot, filePath)}: ${replacements} link(s)`);
    }
  }

  console.log(`Done: ${linkCount} link label(s) in ${fileCount} file(s).`);
}

main();
