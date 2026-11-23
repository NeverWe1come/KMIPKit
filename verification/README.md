# KMIPKit verification

This directory groups projects and tools used to examine KMIPKit from independent consumers, API and catalog checks, robustness campaigns, and CI support. These checks provide reproducible engineering evidence; passing them does not by itself certify KMIP conformance or security.

## Where to start

| I want to… | Go to |
| --- | --- |
| Inspect the planned coverage matrix for KMIP 2.0, 2.1 and 3.0 | [Shared interoperability matrix](interop/shared/README.md) |
| Run a language validator against a test KMIP server or inspect its reports | [Interoperability validators](interop/README.md) |
| Find a normative source, requirement or protocol catalog | [Conformance sources and catalogs](conformance/README.md) |
| Check KMIP 2.1 request-field validation | [Request validation](request_validation/README.md) |
| Find the Rust, C, Java and Python public API inventories | [API catalogs](api/catalog/README.md) |
| Exercise the published bindings from a standalone consumer | [Consumer projects](consumers/) |
| Run fuzzing, sanitizers or security regression checks | [Security verification](security/README.md) |
| Collect Rust, C, Java, Python or JNI coverage | [Coverage tools](coverage/) |
| Check dependency policy or documentation examples | [Dependency checks](dependencies/) and [example checks](examples/) |
| Find CI selection and diagnostics or their tests | [CI tools](ci/) and [verification script tests](tests/) |

## Discover commands

From the repository root, run `python verification/run.py` to list the checks.
`python verification/run.py interop-plan` prints a local execution plan.
`python verification/run.py security-smoke` runs the bounded hardening campaign.
For prerequisites and CI commands, see the project READMEs and
`.github/workflows/`. Live interoperability checks require an explicit command
and a configured test KMIP server.

Generated reports and fuzz artifacts belong under `target/verification/` or the relevant Cargo target directory; do not commit transient output as source evidence. Record the commit, tool versions, server identity, protocol version, and configuration whenever reporting a result.
