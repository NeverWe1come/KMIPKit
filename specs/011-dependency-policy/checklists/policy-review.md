# Dependency Policy Requirements Review

**Purpose**: Reviewer-owned checklist for evaluating the policy requirements.
**Feature**: [spec.md](../spec.md)

- [ ] Do both root and fuzz graphs include all features, development licenses and duplicate checks, and supported target-specific dependencies?
- [ ] Is the exact SPDX allowlist evidence-based and finite, with unknown or missing license data denied?
- [ ] Do advisory defaults explicitly fail vulnerabilities, unsoundness, unmaintained crates, and yanked versions?
- [ ] Are registry, Git, canonical path, duplicate (including dev-only), wildcard, and finite source-cited architecture-derived package bans explicit rather than dependent on tool defaults?
- [ ] Are the ADR-0005 TLS-backend package bans non-waivable through the exception register?
- [ ] Does every exception map to one exact finding, have mitigation and approval evidence, and expire within 90 days?
- [ ] Does every pull request run the policy job regardless of changed paths, with `--locked`, successful online advisory refresh, and emitted RustSec commit SHA/time?
- [ ] Does the scheduled job inspect the configured active release ref, and is its default-branch activation behavior documented?
- [ ] Can contributors reproduce the same root and fuzz checks with the documented command?
- [ ] Are CI permissions read-only, secret-free, and free of a mutable or unreviewed checker action?
- [ ] Are the limitations of automated license metadata stated without implying legal completeness?
- [ ] Are branch protection, release attestations, OASIS integrity, and language adapters clearly outside this feature?
