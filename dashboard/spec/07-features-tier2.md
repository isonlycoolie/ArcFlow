# Tier 2 features (future)

Tier 2 extends operator control after R1 stabilizes. Spec now so Tier 1 UI does not paint into a corner.

## T2-1 — Multi-site management

- `GET /v1/admin/sites` list with pagination
- Search by display name
- Site archive / soft delete (API TBD)

## T2-2 — Usage dashboard

- Daily run count chart per site
- Data source: `arcflow_site_usage_daily` meter table
- Export CSV for billing integration

## T2-3 — BYO LLM keys

- Vault-backed provider keys per site or org
- Server routes already support env keys; dashboard adds rotate + test connection
- Never display full key after save; show last four characters only

## T2-4 — Inline workflows

- Enable `allow_inline` via PATCH with tier gate
- JSON editor with schema validation against RCS
- Publish arbitrary workflow name + version

## T2-5 — Visual workflow builder

- Graph canvas for steps and edges
- Export to RCS YAML; publish via registry API
- Out of scope until FP-5 debugger UX matures in VS Code extension

## T2-6 — Account and billing

- Stripe customer portal embed
- Plan limits enforced server-side (site count, rpm cap)

## Feature gating UI

Show locked Tier 2 nav items with upgrade tooltip rather than hiding routes entirely, so operators understand the roadmap.
