# KMIPKit agent rules

This file is the operational contract for every AI agent working in this
repository. Read it before inspecting or modifying code. The project is in a
design and foundation phase; do not infer that missing code is authorized.

## 1. Authority and source order

Follow instructions in this order:

1. Direct human instructions for the current task.
2. The approved Spec Kit constitution in `.specify/memory/constitution.md`.
3. The approved specification for the assigned feature.
4. This file.
5. Accepted ADRs and the architecture documentation.
6. The OASIS source hierarchy in
   `docs/compliance/document-hierarchy.md`.

If sources conflict, stop the affected work, record the conflict, and ask the
human reviewer. Never silently invent a KMIP requirement.

## 2. Product boundaries

- Product name: KMIPKit.
- License: Apache-2.0.
- Protocol: OASIS KMIP 2.1 only for the 1.x line.
- 1.0 encoding: TTLV only.
- 1.0 direction: client initiated operations. Protocol asynchronous behavior
  is included. Server initiated operations are planned for 1.1.
- 1.0 transports: raw TTLV over TLS and TTLV over HTTPS/HTTP 1.1.
- TLS: TLS 1.3 only, mutual TLS, rustls with aws-lc-rs.
- 1.0 languages: Rust, C, Java, and Python with functional parity.
- The library transports and manages cryptographic material. It is not a
  local cryptographic algorithm provider.

Do not expand these boundaries inside a feature PR. Propose a new
specification and ADR when a boundary must change.

## 3. Required workflow

Every implementation change requires an approved specification. The
maintainer owns the installed Spec Kit framework files.

For each specification:

1. Work in a dedicated worktree and `feature/<spec-id>-<slug>` branch created
   from the active `release/<version>` branch.
2. Confirm its normative references, dependencies, acceptance criteria, and
   exclusions before coding.
3. Use strict Red, Green, Refactor TDD.
4. Keep Red, Green, and Refactor evidence in distinct development commits and
   summarize commands and results in the PR.
5. Update requirement traceability, documentation, and generated artifacts in
   the same PR.
6. Rebase or otherwise update from the release branch, resolve conflicts, and
   run all required checks before requesting review.
7. Open a draft PR. Only a human may approve or merge it.

Feature PRs are squash merged into the release branch. Release branches are
integrated into `master`, which must remain stable and publishable.

## 4. Agent permissions

Agents may create worktrees and feature branches, edit files, run tests, make
commits, push their feature branch, and open or update draft PRs.

Agents must not:

- Merge a PR or approve their own work.
- Push directly to `master` or a `release/*` branch.
- Force push shared or foreign branches.
- Publish packages or releases.
- Change repository protections, credentials, or secrets.
- Change the constitution or an accepted architectural boundary without an
  explicitly approved task.
- Edit files under `specification/oasis/`; they are immutable upstream copies.
- Modify generated files manually.

## 5. OASIS requirements and conformance

- Cite the exact OASIS document and section in every implementation
  specification and conformance test.
- Give every normative requirement a stable traceability identifier.
- Implement every applicable client `MUST` and `SHALL`.
- Add negative tests for `MUST NOT` and `SHALL NOT`.
- Implement `SHOULD` requirements unless an approved record justifies a
  deviation.
- Represent and expose every client `MAY` and `OPTIONAL` capability in scope.
- Treat Usage Guide text and examples as informative. They cannot create a
  requirement absent from normative sources.
- Claim profile support only after all applicable clauses and official test
  cases pass.
- Never claim formal certification or FIPS status without exact external
  evidence.

## 6. TDD and testing

Tests are executable specifications. Prefer small, independent tests with
descriptive names and Given/When/Then or Arrange/Act/Assert structure.

Required layers include:

- Focused unit tests.
- OASIS conformance vectors.
- Negative malformed input tests.
- Property based TTLV roundtrip tests.
- Deterministic client tests through a fake transport.
- C ABI tests compiled and called from C.
- Java and Python parity tests.
- Integration tests against at least two independent KMIP implementations
  before 1.0.

Coverage gates:

- `kmipkit-ttlv` and protocol/model code: at least 95 percent line coverage.
- Transport, FFI, and bindings: at least 85 percent line coverage.
- Rust workspace overall: at least 90 percent line coverage.
- New or changed code: at least 95 percent line coverage.
- Normative requirement traceability: 100 percent.

Generated code may be excluded from raw coverage, but its behavior must be
tested. Every exclusion requires a documented reason.

## 7. Rust and code quality

- Rust Edition 2024; initial MSRV 1.94.
- All crates except `kmipkit-ffi` use `#![forbid(unsafe_code)]`.
- Unsafe code is restricted to the smallest possible FFI scope and every
  block requires a `SAFETY` explanation.
- Do not use `unwrap`, `expect`, `panic!`, `todo!`, or `unimplemented!` in
  production paths.
- Tests may use them only to express a test precondition clearly.
- Use `rustfmt`; enforce `clippy::all` and `clippy::pedantic`. Document narrow
  lint exceptions next to the exception.
- Public APIs require documentation. Private comments explain invariants and
  reasons, not obvious syntax.
- Prefer small modules with one responsibility. Use patterns only when they
  reduce real complexity.
- Avoid global mutable state.
- Preserve error sources and add redacted context.

When the workspace exists, the minimum local checks are expected to include:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
```

Use the repository automation task when it becomes available instead of
duplicating command logic.

## 8. Security invariants

- Never log, format, serialize, or include in errors: credentials, private
  keys, secret key material, OTPs, tickets, raw KMIP bodies, or TLS private
  keys.
- Use explicit secret types and zeroize memory owned by KMIPKit.
- Document runtime limitations when Java or Python may retain external copies.
- Enforce TLS certificate chain, validity, and hostname checks. There is no
  production option to accept arbitrary certificates.
- Disable TLS 0-RTT, redirects, HTTP compression, proxies, and TLS key logging.
- Treat all network bytes, server messages, extension manifests, and callback
  behavior as untrusted.
- Apply decoder limits before allocation: 16 MiB message size, depth 64, and
  100,000 elements by default, all configurable.
- Never retry a KMIP request automatically.
- Report whether a failed request was not sent, possibly sent, or had begun
  receiving a response.
- Run dependency, license, supply chain, FFI, and parser security checks
  defined in the testing and security documentation.

## 9. Public API and compatibility

KMIPKit exposes three levels: high level builders, complete typed KMIP
requests/responses, and generic structurally valid TTLV.

- Preserve unknown tags, enum values, bitmask bits, and extensions.
- Public enums must allow future or vendor values.
- Do not make cryptographic choices implicitly. Algorithms, sizes, usage, and
  protection policy are explicit.
- Stable C symbols use the `kmipkit_` prefix and fixed width types.
- Rust, C, Java, and Python APIs in 1.0 must have equivalent capabilities.
- All official package versions are synchronized.
- Breaking public API or ABI changes require a new major version.

## 10. Generated artifacts

The checked in normative catalog and public API manifest are generation
inputs. Generated Rust, C, Java, Python, tests, and documentation are committed
for review.

- Never edit generated output by hand.
- Regenerate with the pinned repository tool.
- CI must regenerate and fail on a diff.
- A catalog change and all affected generated output belong in one PR.
- Builds must not scrape or download OASIS pages.

## 11. Documentation

- Code, API names, rustdoc, errors, specifications, ADRs, and technical files
  are written in English.
- Human discussion and learning may be in Spanish.
- The 1.0 user guide must be complete in English and Spanish; English becomes
  canonical at 1.0.
- Keep examples executable and tested.
- Update affected docs in the same PR as behavior.
- Link to an ADR or canonical document instead of copying rules into multiple
  places.

## 12. Definition of done

A PR is ready for human review only when:

- Its approved scope and acceptance criteria are satisfied.
- Red, Green, Refactor evidence is present.
- Relevant tests pass on supported platforms.
- Coverage and traceability gates pass.
- Formatting, linting, documentation, security, and compatibility checks pass.
- Generated output is current.
- Secrets are redacted and security effects are documented.
- No placeholder or unresolved ambiguity remains.
- The PR describes changes, reasons, verification evidence, risks, and known
  limitations.
