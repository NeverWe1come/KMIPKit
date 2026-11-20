# Governance

## Initial governance

NeverWe1come is the initial owner and maintainer. The maintainer controls
repository settings, appoints reviewers, accepts contributors, merges PRs, and
coordinates releases.

AI agents are implementation tools. Their reviews do not count as human
approval, and they cannot merge or publish.

## Decisions

- Repository policies live in maintained project documentation.
- Feature behavior and user-visible changes are described in pull requests.
- Protocol interpretations cite OASIS sources and include relevant tests.
- Maintainer approval is required before merging changes.

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
Changes are documented publicly in the repository. DCO is used initially; no
CLA is required.
