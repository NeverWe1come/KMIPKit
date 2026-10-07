# Testing strategy

## Goals

Tests demonstrate protocol correctness, parser robustness, cross-language
parity, ABI stability, interoperability, and safe failure. Coverage is a gate,
not a substitute for behavior and requirement traceability.

## Test layers

### Unit tests

Small tests cover individual types, validation rules, encodings, state
transitions, and errors. They are independent and descriptive.

### TTLV conformance

- Official OASIS vectors are immutable fixtures.
- Project vectors fill identified gaps.
- Every primitive type includes boundaries and invalid forms.
- Canonical encode and strict decode are tested separately.
- Generic valid decode/encode roundtrips produce identical bytes.

### Negative parser tests

Include truncated headers and values, integer overflow, invalid lengths,
invalid type lengths, invalid padding, excessive depth, excessive count,
unknown values, malformed structures, trailing/incomplete frames, and limits.

### Property tests

Generate bounded valid TTLV values and assert encode/decode properties. Use
shrinking to retain minimal failures. Property tests begin in the foundation;
coverage-guided fuzzing complements them for parser and extension-schema
validation.

### Extension schema fuzzing

The `extension_schema` libFuzzer target decodes at most 4 KiB of input with a
64-Structure and 512-Item limit, then validates any decoded root Structure
against a fixed nested extension schema. Schema mismatches are expected; a
panic, partial value, or limit bypass is not. Run it with the repository's
nightly toolchain and pinned `cargo-fuzz` 0.13.2 installed:

```text
cargo +nightly install cargo-fuzz --version 0.13.2 --locked
cargo +nightly fuzz run extension_schema -- -runs=1000 -max_len=4096 -timeout=5
```

Pull-request CI runs the same bounded smoke campaign in `fuzz-smoke`. Keep
minimized crashes under `fuzz/artifacts/extension_schema/` for triage, and
promote any regression input into a deterministic protocol test before fixing
it.

### Client tests

A deterministic fake transport verifies exact sent bytes, response handling,
connection invalidation, delivery state, timeouts, batch outcomes, pending
operations, and redaction without network I/O.

The shared error contract has focused external integration suites:

- `crates/kmipkit-protocol/tests/result_contract.rs` checks all assigned
  Result Status and Result Reason values generated from the normative catalog,
  unknown-value retention, the §9.18 Success/Failure reason invariant, and
  exact optional Result Message text. Error-format tests verify that server
  text and arbitrary protocol causes remain redacted.
- `crates/kmipkit-transport/tests/delivery_state.rs` checks the write-started,
  zero-byte-read, and first-response-byte boundaries and verifies that an
  arbitrary transport cause is dropped and unreachable from the public chain.
- `crates/kmipkit-client/tests/error_contract.rs` checks local failure
  categories, safe cause and delivery metadata, secret sentinels, source
  destruction, and redaction of a complete server Result Message.
- `crates/kmipkit/tests/error_api.rs` exercises the same result and delivery
  contracts through the public facade.

Regenerate assigned result-value lookup tables with
`python -B tools/normative_catalog/generate_result_values.py --repo-root . --write`.
CI runs the same generator in `--check` mode after validating the normative
catalog. Generated Rust output must not be edited by hand.

### Transport tests

An ephemeral PKI creates CA, server, and client material. Test valid mTLS,
unknown CA, hostname mismatch, expiry, CRL, incomplete I/O, timeout, HTTP
framing, forbidden redirect/compression behavior, and connection reuse.

### FFI and adapters

- Compile and execute a C consumer.
- Run panic-containment and memory ownership tests.
- Run sanitizer jobs for the FFI surface.
- Pull-request CI runs the Linux C consumer through the Rust ABI under
  AddressSanitizer and runs the JNI zeroizing-owner test under AddressSanitizer
  and UndefinedBehaviorSanitizer in `ffi-sanitizer`.
- Pull-request CI checks generated API and cross-adapter fixture outputs and
  runs the extension fixture generator's rejection tests in `script-contracts`
  using the Python 3.12 executable inside the uv-managed `VIRTUAL_ENV` and the
  pinned `tools/api_manifest/requirements-test.txt`.
- Test Java from the packaged native JAR.
- Test Python from the built wheel in a clean environment.
- Run shared behavioral vectors through Rust, C, Java, and Python.

### Interoperability

Before 1.0, test against at least two independent server implementations. One
open source implementation runs in CI. A second may use a licensed or manual
environment. Record server product, version, transport, profile, operation,
result, and known workaround.

The [KMIP server interoperability matrix](kmip-interoperability.md) records
published operation claims and candidate servers separately from executed
KMIPKit integration-test results.

## Coverage

- `kmipkit-ttlv` and `kmipkit-protocol`: 95 percent line minimum each.
- `kmipkit-transport`, `kmipkit-ffi`, Java, Python, and JNI: 85 percent
  line minimum for each separately measured scope.
- Rust workspace: 90 percent line minimum.
- Changed production code across Rust, Java, Python, and JNI: 95 percent line
  minimum.
- Normative requirement traceability: 100 percent.

Use `cargo llvm-cov`. CI collects LLVM JSON coverage on Linux, Windows, and
macOS for the checked-out pull-request merge commit. The changed-code gate
compares that exact tree with the pull-request base, derives executable Rust
line counts from LLVM file segments across the three reports, and checks added
Java, Python, and JNI source lines against their own collector reports. It
validates the parsed line and covered-line totals as lower bounds against each
file's LLVM summary while checking function code-region schemas, file references, and
each code region's start line against the file segment map. The parser checks
region start lines because a region may span structural source lines that have
no executable code. LLVM sums line summaries by source-level function group,
while file segments merge coverage by physical source location; shared lines
can make the summary larger than the unique segment line map. Thresholds use
the physical line map and first reconcile per-function region coverage at
shared physical lines. Any remaining summary-reported uncovered-line residual
counts as uncovered. Residuals from each platform are summed because
platform-specific code may omit different source lines. For changed files, the
same residual is included in changed-code coverage so an omitted uncovered
line cannot become `not applicable`. Region data explains covered and
uncovered duplicate groups at a mapped physical line; unexplained summary
reserves are conservative because LLVM does not report their physical
locations. Missing region start lines fail closed. This preserves
nested-region counts and prevents missing function records from hiding
uncovered lines. It requires at least 95 percent changed executable-line
coverage. Each Rust crate is gated separately: `kmipkit-ttlv` and
`kmipkit-protocol` at 95 percent, and `kmipkit-transport` and `kmipkit-ffi`
at 85 percent. The Rust workspace gate is 90 percent and includes Rust
production crates only.
An LLVM report that repeats a workspace source path across export mappings
fails closed until those mappings can be reconciled independently; function
regions from distinct `CoverageMapping` objects are never combined to explain
one another's summaries.

### Adapter and FFI coverage

The coverage aggregator also requires one JaCoCo XML report at
`coverage-java/jacoco.xml`, one coverage.py Cobertura report at
`coverage-python/coverage.xml`, and one LLVM JSON report at
`coverage-jni/coverage.json` whenever these source trees exist. It checks each
scope independently at 85 percent, then includes all reported production
lines in the 95 percent changed-code gate. Adapter report paths must resolve to
the source checkout, and every selected production source must be present in
its report.

JaCoCo 0.8.12 measures compiled Java product code below
`bindings/java/src/main/java/org/kmipkit`; its Maven check enforces 85 percent
line coverage. Test classes are compiled under `src/test/java` and examples
remain under `bindings/java/examples`, so neither is product coverage input.
Generated Java package code remains included. JaCoCo measures JVM bytecode;
it does not measure the native JNI implementation.

Python uses the single exact-pinned test/build requirements set in
`bindings/python/requirements-coverage.txt`: coverage 7.10.6, pytest-cov
6.2.1, pytest 8.4.2, CFFI 1.17.1, and Maturin 1.9.4. The committed
coverage.py configuration selects `kmipkit` as the measured source package
with no omissions, so handwritten and generated files under
`bindings/python/src/kmipkit` remain in the 85 percent gate. Tests and example
consumers live outside that package and are not measured as product code.
Cross-platform adapter CI pins uv 0.12.23, installs CPython 3.12 through uv,
and activates a clean virtual environment before installing this requirements
set. This provides the same Python minor version on the self-hosted Debian
ARM64 runner and GitHub-hosted platforms. The C consumer CI pins CMake 3.31.6
and Ninja 1.13.2 and selects Ninja explicitly on every platform, so the build
does not depend on the runner image's default CMake generator.

The Rust C ABI implementation, including generated Rust FFI code, is measured
by `cargo llvm-cov` in `kmipkit-ffi`. Its Linux-only `coverage-c-consumer`
feature compiles the existing public C consumer test into the Rust integration
test binary and calls it under the same coverage process. This keeps the
behavior assertions in C while making their Rust ABI calls visible to LLVM.
The ordinary CMake consumer test still runs in the language-bindings job. The
coverage build uses the exact-pinned `cc` 1.6.0 build dependency only when the
coverage feature is enabled. The C header contains declarations, not
executable lines. The handwritten JNI implementation in
`bindings/java/native/kmipkit_jni.cpp` is compiled with LLVM source-based
coverage instrumentation and measured from its own report; JaCoCo and Rust
coverage do not count those C++ lines. The JNI collector is Linux-only and
requires LLVM/Clang 20.1.8. There are no generated-source coverage exclusions
in the current configuration.

The aggregate job requires these inputs at the report root:

```text
coverage-reports/
  coverage-ubuntu/coverage.json
  coverage-windows/coverage.json
  coverage-macos/coverage.json
  coverage-ffi/coverage.json
  coverage-java/jacoco.xml
  coverage-python/coverage.xml
  coverage-jni/coverage.json
```

Collect them with the following commands. Run the Rust commands in each
platform matrix job; cargo-llvm-cov is pinned at 0.9.1. The Python commands
run in the Python 3.12 environment provisioned by CI or an active Python 3.12
virtual environment.

```sh
cargo llvm-cov --workspace --all-features --locked --json --output-path coverage-raw.json
python scripts/coverage_gate.py normalize --workspace . --input coverage-raw.json --output coverage.json

mvn -B -f bindings/java/pom.xml clean verify

python -m pip install -r bindings/python/requirements-coverage.txt
python -m pip install --no-build-isolation --editable bindings/python
python -m pytest -q bindings/python/tests --cov=kmipkit --cov-config=bindings/python/pyproject.toml --cov-report=xml:coverage-reports/coverage-python/coverage.xml

# Linux only; requires clang/LLVM 20.1.8 and runs the Java suite against instrumented JNI.
bash scripts/collect_jni_coverage.sh target/coverage-jni

# Linux only; runs the C ABI consumer against the instrumented Rust library.
bash scripts/collect_ffi_coverage.sh target/coverage-ffi

mkdir -p coverage-reports/coverage-java coverage-reports/coverage-jni coverage-reports/coverage-ffi
cp bindings/java/target/site/jacoco/jacoco.xml coverage-reports/coverage-java/jacoco.xml
cp target/coverage-jni/coverage.json coverage-reports/coverage-jni/coverage.json
cp target/coverage-ffi/coverage.json coverage-reports/coverage-ffi/coverage.json
python scripts/coverage_gate.py aggregate --workspace . --report-dir coverage-reports --base <full-base-sha> --merge <full-merge-sha>
```

The pull-request workflow's Linux `coverage` job runs the C ABI collector and
uploads `coverage-ffi`; its Linux `adapter-coverage` job runs the Java, Python,
and JNI collectors and uploads their three reports. The `coverage-gate` job
requires successful Rust and adapter collection, downloads all seven coverage
artifacts into `coverage-reports`, and runs the aggregate command with the pull request
base and merge commit SHAs shown above. A local aggregate run must supply real
reports from Ubuntu, Windows, and macOS; missing platform or adapter artifacts
fail closed.

The normalizer accepts LLVM JSON export schema 2.0.x for the pinned JNI
collector (LLVM 20.1.8) and schemas 3.0.x and 3.1.x for Rust coverage.
Schema 2.0.x is accepted only with a captured native fixture; other major or
minor versions fail closed until their consumed file, segment, region, and
summary fields have been checked and covered by a fixture. LLVM 20.1.8's
export command has no include-filename option, so the Linux JNI collector
excludes only the JDK `jni.h` wrapper declarations when they resolve under
`/usr/lib/jvm/<jdk>/include` or GitHub-hosted
`/opt/hostedtoolcache/Java_<distribution>/<version>/<architecture>/include`,
plus `bindings/c/include/kmipkit.h` declarations. The Rust `kmipkit-ffi` sources
measure the C ABI implementation separately; these headers contain no
handwritten JNI implementation. Every other reported source path is retained
and checked by the normalizer, so the JNI denominator includes the actual
`bindings/java/native/kmipkit_jni.cpp` lines and fails closed on unexpected
headers.

The changed-code metric is `not applicable` when a pull request changes no
executable production lines; the package and workspace gates still apply.
Coverage is `unavailable` only when a complete Rust scan finds no production
function bodies and no Java, Python, or JNI production package exists. Adapter
source files conservatively require measured coverage even when Rust has no
function bodies. Each required platform then uploads an explicit unavailable
status only for the complete no-code case. Unreadable or unclassifiable source
requires coverage, inline `#[cfg(test)]`
modules fail preflight, and missing or malformed reports fail once production
code is eligible. Keep tests in crate-level external test directories outside
`src/`. Every Rust file under `src/` is included regardless of its name or
nested directory; for example, a file named `parser_tests.rs` can still be
selected as a production module with `#[path]`. Generated source may be excluded
only with a documented reason.

CI attempts branch coverage separately with nightly Rust only on a daily
schedule; it does not run on pull requests. That job is informational and
cannot gate pull-request success; promote branch coverage to a required check
only in a separately reviewed change after its reliability has been
demonstrated. Reproduce the local source, workflow-contract, and coverage
parser checks with:

```powershell
pwsh -File .\scripts\tests\Test-Wsl.ps1
python -m unittest discover -s scripts/tests -p 'test_*.py' -v
```

## Cargo dependency policy

Run the complete dependency-policy check from the repository root in
PowerShell with:

```powershell
pwsh -File .\scripts\Test-DependencyPolicy.ps1
```

The runner installs and verifies the exact `cargo-deny` **0.20.2** release,
then checks the root and fuzz Cargo workspaces separately against their
committed lockfiles. It includes all features, development dependencies, and
all resolved target-specific edges; it does not filter Cargo metadata by the
policy runner's host platform. The runner validates its observed `rustc -vV`
host triple and fails if the triple is absent from the reviewed CI runner set.
For each successful workspace check it attempts an online RustSec advisory
database refresh and reports that invocation's database commit SHA and ISO
timestamp. The refresh evidence is per workspace, and cached-only or failed
refreshes do not count as a passing fresh scan. The command checks lockfile
hashes and fails if policy execution changes either lockfile.

An unexcepted license, advisory, source, ban, duplicate, wildcard, invalid
path, tool, host, or database-evidence finding fails the run. Policy exceptions
require an exact, reviewed, expiring record and a matching cargo-deny entry;
see the [dependency-policy guide](../security/dependency-policy.md) for the
record fields, evidence, renewal, and removal process. Automated license
metadata checks are not a complete legal audit and do not validate license
terms against every package source file.

The daily scheduled dependency review scans the configured active release ref.
GitHub starts scheduled workflow runs from the repository's default branch,
so this workflow change becomes active for scheduled runs only after it is
integrated into that default branch. The schedule then checks out the
configured release ref and reports the scanned commit with root and fuzz
RustSec evidence separately.

## CI run summaries

Every pull-request and scheduled run includes a final **CI summary / at a glance** job. The workflow run Summary shows whether required checks passed, the event, ref, commit, run link, and outcomes grouped by the checks that apply to that event. Checks belonging to the other event type are marked not applicable. Failed, cancelled, or missing required checks remain failures in the final summary.

The coverage gate adds its own job Summary with measured percentages and thresholds, an explicit reason when coverage is unavailable and no threshold is claimed, or a failure diagnostic. Scheduled branch coverage is called out as informational and does not affect the required result. Detailed matrix rows and logs remain in each job. These summaries only present existing workflow results; they do not change the jobs, runner routing, or thresholds.

## CI levels

### Every PR

- Formatting and linting.
- MSRV and stable Rust tests.
- Linux, Windows, and macOS tests.
- Coverage and traceability, including the coverage gate summary.
- Final at-a-glance run summary with relevant job-group outcomes.
- Documentation with warnings denied.
- Affected ABI and adapter tests.
- Dependency, advisory, source, and license policy.

### Scheduled

- Wide feature combinations.
- Miri on compatible safe crates.
- FFI sanitizers.
- Bounded fuzzing.
- External server integration.
- Updated vulnerability review.

### Release

- Complete target matrix.
- Build, install, and smoke test final packages.
- ABI and public API comparisons.
- Signatures, hashes, SBOM, and provenance.
- Independent security review findings resolved or explicitly accepted.

### Self-hosted Linux runner

For pull requests whose head branch is in this repository, Linux jobs run on
the repository's self-hosted Linux ARM64 runner. Pull requests from forks use
GitHub-hosted Linux runners so fork-controlled code never runs on the persistent
runner at home. Windows and macOS jobs remain GitHub-hosted, preserving the
three-platform checks and coverage reports. The scheduled informational
branch-coverage job also uses the self-hosted Linux ARM64 runner.

The self-hosted runner must have the default `self-hosted`, `Linux`, and
`ARM64` labels and remain online while jobs are queued. Its execution account
needs Git, Python 3, PowerShell 7, Rustup, a C toolchain/linker, and Cargo's
`bin` directory on the service `PATH`. The workflow installs the required Rust
toolchains and `cargo-llvm-cov`. Run the runner as an unprivileged account and
install it as a system service for unattended pull-request and scheduled jobs.
Do not store production credentials or sensitive files on the runner.

## Fuzzing

Initial targets include TTLV frame parsing, tree decoding, typed conversion,
unknown extensions, and response correlation. Seed corpora use public OASIS and
project fixtures with secrets removed.

## Performance

Benchmarks are added before 1.0 after correctness stabilizes. Measure codec
throughput and latency, allocations, batch behavior, FFI overhead, and small,
medium, large, and configured maximum messages. Establish targets from real
measurements rather than unsupported claims.
