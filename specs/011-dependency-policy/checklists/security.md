# Security Review Checklist: Dependency Policy Gates

**Purpose**: Reviewer-owned security review of the specification and planned implementation.
**Feature**: [spec.md](../spec.md)

- [x] The policy checker version and acquisition path are independently reviewed and exact-pinned.
- [x] Pull-request execution has read-only permissions, no secrets, and no privileged event trigger.
- [x] Fork-controlled code cannot execute on a persistent self-hosted runner with sensitive files or credentials.
- [x] Advisory refresh failure or stale data cannot yield a successful fresh-scan status.
- [x] Root and fuzz lockfiles are both checked without auto-fix or mutation.
- [x] The FR-013 manifest correction is limited to first-party unpublished fuzz metadata and preserves resolved packages and both lockfiles.
- [x] Unknown source, license, advisory, banned crate, wildcard, and duplicate defaults fail closed.
- [x] No exception can permit a TLS backend that conflicts with ADR-0005.
- [x] Each exception is exact, reviewed, auditable, and time-bounded; expired or orphaned exceptions fail.
- [x] Exception validation cannot be bypassed by aliases, malformed fields, path traversal, or duplicate IDs.
- [x] CI output contains package-policy evidence only and no credential or secret material.
- [x] License metadata limitations and lack of legal completeness claims are documented.
- [x] Scheduled release checkout identifies the exact branch and commit that was scanned.
- [x] No rule implies GitHub branch-protection enforcement without administrator-side evidence.
