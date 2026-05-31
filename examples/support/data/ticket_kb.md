# Acme SaaS — Tier-1 support knowledge (synthetic)

## Authentication & SSO

### Ticket #1001 — Password reset (standard)

**Symptoms:** User forgot password, no SSO involved.  
**Resolution:** Settings → Security → Send reset link. Link expires in 15 minutes. If email not received, check spam and verify primary email on account.

### Ticket #1002 — SSO failures after IdP certificate rotation

**Symptoms:** All SSO users fail login; error `SAML signature validation failed`.  
**Resolution:**

1. Confirm new IdP x509 certificate uploaded in Acme Admin → SSO → Identity Provider
2. Download Acme SP metadata and re-upload to IdP if endpoints changed
3. Test with a single pilot user before broad announcement
4. Rollback: re-upload previous cert if rotation window < 24h

### Ticket #1003 — Seat provisioning delay

**Symptoms:** New hire cannot access workspace 24h after invite.  
**Resolution:** Check Admin → Users → Pending invites. Resend invite. Verify SCIM sync if HRIS integration enabled.

## Billing (escalate to L2)

Do not promise refunds in L1. Escalate with account ID and invoice number.
