# Security Review Checklist: Dependency Policy Gates

**Purpose**: Reviewer-owned security review of the specification and planned implementation.
**Feature**: [spec.md](../spec.md)

- [ ] The policy checker version and acquisition path are independently reviewed and exact-pinned.
- [ ] Pull-request execution has read-only permissions, no secrets, and no privileged event trigger.
- [ ] Fork-controlled code cannot execute on a persistent self-hosted runner with sensitive files or credentials.
- [ ] Advisory refresh failure or stale data cannot yield a successful fresh-scan status.
- [ ] Root and fuzz lockfiles are both checked without auto-fix or mutation.
- [ ] Unknown source, license, advisory, banned crate, wildcard, and duplicate defaults fail closed.
- [ ] No exception can permit a TLS backend that conflicts with ADR-0005.
- [ ] Each exception is exact, reviewed, auditable, and time-bounded; expired or orphaned exceptions fail.
- [ ] Exception validation cannot be bypassed by aliases, malformed fields, path traversal, or duplicate IDs.
- [ ] CI output contains package-policy evidence only and no credential or secret material.
- [ ] License metadata limitations and lack of legal completeness claims are documented.
- [ ] Scheduled release checkout identifies the exact branch and commit that was scanned.
- [ ] No rule implies GitHub branch-protection enforcement without administrator-side evidence.
