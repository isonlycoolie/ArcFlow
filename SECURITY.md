# Security Policy

## Reporting a Vulnerability

If you believe you have found a security issue in ArcFlow, do not open a public GitHub issue with exploit details, credentials, customer data, or production URLs.

Report privately through GitHub Security Advisories on this repository, or contact the repository owner directly with:

- A description of the issue and affected components
- Steps to reproduce, with a minimal proof of concept if possible
- Impact assessment if known
- Suggested mitigations or fixes, if any

We aim to acknowledge reports within 3 business days and provide a remediation timeline within 14 days for high-severity issues. Critical vulnerabilities that could cause data loss, credential exposure, or remote code execution are prioritized immediately.

## Safe Contributions

- Never commit API keys, PyPI tokens, `.env` files, local scan reports, or `scripts/.pypi-token`
- Do not paste full webhook payloads, trace exports with sensitive content, or production URLs with embedded secrets in issues or pull requests
- Use environment variables for provider keys (`api_key_env` in workflow definitions, not inline secrets)
- Rotate any credential that may have appeared in Git history, logs, CI output, or support bundles

## Supported Versions

Security fixes are applied to the active development branch and released on tagged versions of `arcflow-server`, SDK packages, and related binaries. Self-hosted operators should run supported releases and follow [documentation/deployment/production-checklist.md](documentation/deployment/production-checklist.md).
