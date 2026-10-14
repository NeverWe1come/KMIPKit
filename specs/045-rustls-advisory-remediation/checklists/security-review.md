# Security Review Checklist: Rustls Advisory Remediation

**Purpose**: Independently review the dependency remediation and its security claims
**Created**: 2026-10-14
**Feature**: [spec.md](../spec.md)
**Review Ownership**: Independent security reviewer; only the reviewer marks items complete.

## Review criteria

- [ ] Each RustSec advisory is pinned to its exact ID, affected range, and patched version
- [ ] Dependency update removes the findings without adding a waiver, ignore, or broader policy exception
- [ ] The exact pin and lockfile diff are minimal and reproducible
- [ ] Existing TLS 1.3/mTLS policy, Rustls backend, and secret-redaction behavior remain intact
- [ ] Current online policy results cover both root and fuzz workspaces
- [ ] No raw advisory scanner output, secrets, or credentials are added to logs or documentation
- [ ] Review conclusion and any residual risk are recorded in the PR evidence
