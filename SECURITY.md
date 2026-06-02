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
