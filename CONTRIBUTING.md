# Contributing to KMIPKit

Contributions are welcome through GitHub issues and pull requests. These
guidelines apply to contributors, maintainers, and reviewers.

## Before contributing

Read:

1. [`docs/README.md`](docs/README.md)
2. The relevant source code, tests, and verification material
3. Applicable KMIP normative sources for protocol changes

## Development flow

1. Choose a focused change and identify its observable behavior.
2. Create a branch from the intended base branch.
3. Update implementation and relevant tests.
4. Update generated output and documentation when affected.
5. Run the checks relevant to the change and report their results.
6. Open a pull request using the repository template.

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
changes should cite applicable normative sources and include tests for the
behavior being changed.

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
