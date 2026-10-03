# Contributing to KMIPKit

KMIPKit is currently developed in a private foundation phase. These rules
apply to maintainers, invited reviewers, and AI agents and will remain the
baseline when the repository becomes public.

## Before contributing

Read:

1. [`AGENTS.md`](AGENTS.md)
2. [`docs/README.md`](docs/README.md)
3. The approved specification for the change
4. Relevant ADRs and OASIS sources

Every implementation change requires an approved Spec Kit specification. The
maintainer owns the installed Spec Kit framework.

## Development flow

1. Choose or obtain a stable `KMIPKIT-NNNN` specification.
2. Create a feature branch and worktree from the active release branch.
3. Write failing tests and confirm the expected Red state.
4. Implement the minimum Green behavior.
5. Refactor while keeping tests green.
6. Update traceability, generated output, and documentation.
7. Run all required checks.
8. Open a draft PR using the template.

Do not push directly to `master` or a release branch. Only a human may approve
and merge.

## Commit sign-off

The project uses the Developer Certificate of Origin 1.1. Sign every commit:

```text
Signed-off-by: Your Name <your-email@example.com>
```

Using `git commit -s` adds the line. By signing off, you certify the Developer
Certificate of Origin at <https://developercertificate.org/>.

## Code and tests

Follow [`docs/development/coding-standards.md`](docs/development/coding-standards.md)
and [`docs/development/testing.md`](docs/development/testing.md). Protocol
changes require exact requirement IDs and normative citations.

Do not submit real keys, credentials, certificates, server addresses, customer
data, or proprietary vendor documentation. Test PKI and credentials must be
generated specifically for tests.

## Dependencies

Explain why a dependency is required, alternatives considered, its license,
maintenance status, security history, MSRV, supported targets, and transitive
cost. Stable releases cannot contain Git dependencies.

## Generated files

Change the reviewed input and run the pinned generator. Never edit generated
output by hand. Commit input and output together.

## Reporting security problems

Follow [`SECURITY.md`](SECURITY.md). Do not open a public issue containing a
suspected vulnerability.
