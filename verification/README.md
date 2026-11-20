# KMIPKit verification

This directory groups projects and tools used to examine KMIPKit from independent consumers, API and catalog checks, robustness campaigns, and CI support. These checks provide reproducible engineering evidence; passing them does not by itself certify KMIP conformance or security.

## Layout

- `interop/` contains independent Rust, C, Java, and Python consumers that can be run against KMIP servers, plus shared catalogs, schemas, profiles, vectors, and result aggregation. `python verification/run.py interop-plan` prints a local execution plan without contacting a server. Live runs require a dedicated test server and explicit connection configuration.
- `consumers/` contains small standalone applications that exercise the published language bindings against a local controlled responder, and a Rust single-dependency consumer.
- `security/` contains bounded coverage-guided fuzz targets and the Miri, sanitizer, and negative-test campaign runner. Crate and binding regression tests remain next to the code they exercise.
- `api/` audits and generates language API manifests and parity evidence.
- `request_validation/` contains the audited KMIP v2.1 operation-field matrix, its deterministic Rust schema generator, and generator tests. The generated descriptors are compiled from `crates/kmipkit-protocol`; run `python verification/request_validation/generate.py --check` to detect stale output.
- `conformance/` contains normative catalog validation and report tooling. Its normative catalogs, requirement matrices, and checksum-pinned OASIS documents are under `conformance/catalog/`, `conformance/requirements/`, and `conformance/oasis/`. The shared binding contract and schema are under `bindings/shared/`; draft API and fluent operation inventories are under `api/catalog/`.
- `dependencies/` checks the repository dependency policy.
- `coverage/` collects and evaluates Rust, C, and JNI coverage.
- `examples/` executes selected documentation examples and checks example alignment.
- `ci/` contains the change-impact classifier, diagnostics, run summary, branch-protection normalization, and PowerShell WSL helpers used by CI.
- `tests/` contains tests of verification scripts and CI tooling.

## Discover commands

From the repository root, run `python verification/run.py` to list checks exposed by the local dispatcher. For example, `python verification/run.py security-smoke` runs the bounded hardening campaign. Individual projects retain their own scripts and tool requirements; see their README files and `.github/workflows/` for the CI invocation. Live interoperability checks contact the configured KMIP server and are not run by the dispatcher without an explicit command.

Generated reports and fuzz artifacts belong under `target/verification/` or the relevant Cargo target directory; do not commit transient output as source evidence. Record the commit, tool versions, server identity, protocol version, and configuration whenever reporting a result.
