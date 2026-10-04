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
coverage-guided fuzzing follows later.

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
- Test Java from the packaged native JAR.
- Test Python from the built wheel in a clean environment.
- Run shared behavioral vectors through Rust, C, Java, and Python.

### Interoperability

Before 1.0, test against at least two independent server implementations. One
open source implementation runs in CI. A second may use a licensed or manual
environment. Record server product, version, transport, profile, operation,
result, and known workaround.

## Coverage

- TTLV/protocol: 95 percent line minimum.
- Transport/FFI/bindings: 85 percent line minimum.
- Workspace: 90 percent line minimum.
- Changed code: 95 percent line minimum.
- Normative requirement traceability: 100 percent.

Use `cargo llvm-cov`. CI collects LLVM JSON coverage on Linux, Windows, and
macOS for the checked-out pull-request merge commit. The changed-code gate
compares that exact tree with the pull-request base and derives executable Rust
line counts from LLVM file segments across the three reports. It validates the
parsed line and covered-line totals as lower bounds against each file's LLVM
summary while checking function code-region schemas, file references, and
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
coverage. The 95 percent package gate
applies to `kmipkit-ttlv` and `kmipkit-protocol`; 85 percent applies to
`kmipkit-transport` and `kmipkit-ffi`; the workspace gate is 90 percent.
An LLVM report that repeats a workspace source path across export mappings
fails closed until those mappings can be reconciled independently; function
regions from distinct `CoverageMapping` objects are never combined to explain
one another's summaries.

The normalizer accepts the reviewed LLVM JSON export schema versions 3.0.x and
3.1.x. Other major or minor versions fail closed until their consumed file,
segment, region, and summary fields have been checked and covered by a fixture.

The changed-code metric is `not applicable` when a pull request changes no
executable Rust lines; the package and workspace gates still apply. Coverage
is `unavailable` only when a complete scan finds no production function
bodies. Each required platform then uploads an explicit unavailable status.
Unreadable or unclassifiable source requires coverage, inline `#[cfg(test)]`
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

## CI levels

### Every PR

- Formatting and linting.
- MSRV and stable Rust tests.
- Linux, Windows, and macOS tests.
- Coverage and traceability.
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

## Fuzzing

Initial targets include TTLV frame parsing, tree decoding, typed conversion,
unknown extensions, and response correlation. Seed corpora use public OASIS and
project fixtures with secrets removed.

## Performance

Benchmarks are added before 1.0 after correctness stabilizes. Measure codec
throughput and latency, allocations, batch behavior, FFI overhead, and small,
medium, large, and configured maximum messages. Establish targets from real
measurements rather than unsupported claims.
