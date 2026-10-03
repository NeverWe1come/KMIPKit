# Governance

## Initial governance

NeverWe1come is the initial owner and maintainer. The maintainer controls
repository settings, approves specifications, appoints reviewers, accepts
contributors, merges PRs, and coordinates releases.

AI agents are implementation tools. Their reviews do not count as human
approval, and they cannot merge or publish.

## Decisions

- Product invariants live in the Spec Kit constitution once installed.
- Feature behavior lives in approved specifications.
- Architectural decisions live in accepted ADRs.
- Protocol interpretations require OASIS citations and conformance tests.
- Human approval is required for all of the above.

## Sensitive changes

Parser, TLS, secrets, FFI, unsafe code, normative catalog, conformance claims,
CI trust, packaging, and release changes require explicit owner review. Before
1.0, an independent qualified person reviews the security-sensitive release
surface. When a second maintainer is appointed, sensitive changes should
require two human approvals.

## Becoming a maintainer

Maintainers are appointed based on sustained high-quality contributions,
protocol understanding, security judgment, respectful collaboration, and a
documented decision by current governance. Access follows least privilege.

## Conflicts and conduct

Technical disagreement is resolved using normative sources, tests, measured
evidence, and documented tradeoffs. Conduct issues follow the Code of Conduct.

## Evolution

Governance may evolve as the community grows or the project joins a foundation.
Such a change requires an ADR and public migration plan. DCO is used initially;
no CLA is required.
