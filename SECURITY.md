# Security policy

Responsible disclosure

If you discover a security vulnerability in ArcFlow, please disclose it privately to the maintainers so we can triage and remediate before public disclosure. Contact: security@arcflow.dev (placeholder) or open a private support ticket.

We aim to acknowledge reports within 3 business days and provide a remediation timeline within 14 days for high-severity issues. For critical vulnerabilities that could cause data loss or remote code execution, we will prioritize fixes immediately and coordinate disclosure and rotation steps.

What to include

- A clear description of the vulnerability and the components affected
- Steps to reproduce (PoC), including minimal code or requests where applicable
- Any suggested mitigations or fixes

If you need to provide sensitive proof-of-concept data, please use encrypted attachments (PGP) and share keys as needed.

Public policy

We will publish a security advisory for resolved vulnerabilities that materially affect users, subject to coordinated disclosure timelines.
# Security

## Reporting a vulnerability

If you believe you have found a security issue in ArcFlow, **do not open a public GitHub issue** with exploit details, credentials, or customer data.

Report privately to the maintainers through GitHub Security Advisories on this repository, or contact the repository owner directly with:

- A description of the issue and affected components
- Steps to reproduce (minimal proof of concept is enough)
- Impact assessment if known

We will acknowledge receipt and work on a fix before any public disclosure when appropriate.

## Safe contributions

- Never commit API keys, PyPI tokens, `.env` files, or `scripts/.pypi-token`
- Do not paste full webhook payloads, trace exports with sensitive content, or production URLs with embedded secrets in issues or PRs
- Use environment variables for provider keys (`api_key_env` in workflow definitions, not inline secrets)

## Supported versions

Security fixes are applied to the active development branch and released on tagged versions of `arcflow-server`, SDK packages, and related binaries. Self-hosted operators should run supported releases and follow [documentation/deployment/production-checklist.md](documentation/deployment/production-checklist.md).
