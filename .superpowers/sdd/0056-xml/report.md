# KMIPKIT-0056 XML codec report

Date: 2026-11-05. Branch: `feature/KMIPKIT-0056-xml-encoding`.
Worktree: `C:\Users\ramp1953\.codex\worktrees\kmipkit-0056-xml\KMIPKit`.
Original implementation verified at `a1ca02e6a84e62f24cc461f75ce100812228950f`.
Independent-review corrections verified at `fa90504dd` (see review follow-up).

## Result and scope

Implemented the approved private XML codec in `kmipkit-client` over the
existing ordered `kmipkit_ttlv::Item` model. The stable private entry points
are `xml_codec::encode(&Item, &CodecLimits)` and
`xml_codec::decode(&[u8], &CodecLimits)`, with the 0054 shared redacted error
categories. Output is owned by `Zeroizing<Vec<u8>>`. No public selector,
client/transport dispatch, public API manifest or generated public API was
changed. No agent was spawned and no PR was approved or merged.

The implementation supports all eleven scalar/Structure types; assigned
catalog names; general TTLV elements; standard namespaces; fixed-width hex;
space-separated mask components across all three families and all 33 assigned
flags; numeric/hex mask forms; unknown bits/tags/enumerations; repeated and
ordered children; XML escaping/normalization; zoned DateTime throughout the
model's i64 range; and strict malformed-input/resource rejection. Unknown
symbolic identities without raw values and lossy non-integral-second dates
cannot enter the model and fail as safe value errors.

Implementation/ownership/dependency details and the six stable requirement
identifiers are recorded in `specs/056-xml-encoding/implementation.md`.
Normative references: OASIS KMIP Specification v2.1 §10.2.3 and OASIS KMIP
Profiles v2.1 §§5.4.1.1–5.4.1.6.13, 5.4.2 and 5.4.4.1. ADR-0018 controls
the accepted 1.0 scope and contextual HTTPS interpretation.

## Signed TDD evidence

| Phase | Commit | Evidence |
|---|---|---|
| Red | `ffa7565e8` | `cargo test -p kmipkit-client --lib xml_codec -- --test-threads=1`: 13 positive/limit/value failures against the deliberately rejecting stub; 3 rejection-only cases passed. |
| Red extension | `e8133745a` | After initial implementation, added depth-before-payload and character-reference/prolog strictness tests. Observed 17 passed / 4 failed: these two new regressions plus two cases awaiting the common QueryFunction mapping. |
| Green | `7df3dd0bf` | After the common mapping fix, 21/21 XML tests passed, including all four official vector messages; all-target/all-feature client Clippy passed. |
| Refactor | `802d304a8` | Split names, characters, scalar values, encoder and decoder; removed per-row metadata Vec allocations; 21/21 tests and Clippy remained green. Preserved the exact fixture bytes using per-directory attributes. |
| Red declaration | `42aaf609a` | Added malformed declaration grammar cases; observed rejection test fail because `standalone='maybe'` was accepted. |
| Green hardening / audit adaptation | `a1ca02e6a` | Strict declaration state machine plus compatible syntax for the existing fail-closed writer audit; 23/23 focused tests, Clippy and final full client regression passed. |

All development commits contain DCO `Signed-off-by`. The existing complete
client regression initially found the syntactic boundary audit rejecting the
new Result alias, `format!` macros and a subordinate writer call named
`encoder::encode`. The auditor was not weakened: the alias was removed,
formatting uses direct hex octets and zeroizing `write!`, and the subordinate
helper became `emit_document`. The contracted `xml_codec::encode`/`decode`
entry points remain unchanged. The exact production inventory gate passes.

The common 0054 direct Enumeration family fix was consumed via approved
cherry-picks `4b6527f10`, `0781aef70`, `f757a2221` (originals `d1ca945b3`,
`61df55166`, `80133d08f`). Those dependency commits account for generated
metadata/generator and 0054 contract changes in this branch. XML-specific
commits do not hand-edit generated output or infer extra families.

## Original implementation verification

All commands ran from this worktree on `x86_64-pc-windows-msvc`.

| Command | Result |
|---|---|
| `cargo test -p kmipkit-client --lib xml_codec -- --test-threads=1` | 23 passed, zero failures. Includes 200 generated ordered scalar trees and 200 full-range DateTime values. |
| `cargo test -p kmipkit-client --all-features` | Exit 0: 704 passed across library/integration/compile tests/doctests; zero failed; 3 pre-existing Cosmian KMS tests ignored because local Cosmian KMS 5.28.0 is required. Library result: 630 passed. |
| `cargo fmt --all --check` | Exit 0. |
| `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings` | Exit 0; no Rust warning/lint exemptions added. |
| `cargo llvm-cov --package kmipkit-client --lib --json --output-path .superpowers/sdd/0056-xml/coverage-final.json -- xml_codec --test-threads=1` | Exit 0, 23 focused tests passed. New XML plus common error code: 720/740 executable lines, 97.30%. |
| `pwsh -NoProfile -File scripts/Test-DependencyPolicy.ps1` | Exit 0, root and fuzz policy passed with unchanged lockfile hashes during the check. Fresh RustSec remote verified as `https://github.com/RustSec/advisory-db`, commit `26fd2d43f7280a1fb925a3743ec463017afe7119`, timestamp `2026-11-05T09:50:49+01:00`. Existing validated exceptions: KMIPKIT-0011-EX-003 and -004; none added. |
| `git diff --check` | Exit 0. |
| Committed fixture SHA-256 | `5ca1a4533bb8e911231ea9e43c446894ff11f9004e2ca74531515327145e75d3`, verified from both working file and `git show HEAD:<fixture>` bytes. |

Final XML module coverage: entry points 12/12 (100%); characters 54/54
(100%); decoder 209/219 (95.43%); encoder 151/154 (98.05%); names 106/106
(100%); scalar values 182/186 (97.85%). Shared error code is 6/9 because the
constructor is evaluated as constants; combined changed-line coverage is
97.30%. No generated-code or allocation-branch coverage exclusion was added.
Untaken defensive allocation-failure/non-exhaustive paths remain present.

The vendored aws-lc build emits its existing Windows compiler-feature/NASM
probe warnings. The fallback build and all checks above completed successfully.
The workspace-wide release/other-platform gates belong to integration;
these are complete client-scoped verification results, not a whole-workspace
coverage or real-server interoperability claim.

Local detailed logs and coverage JSON are retained beside this report and
ignored by the report directory's `.gitignore` to avoid committing bulky
machine artifacts.

## Official case evidence

The exact OASIS download is pinned at
`crates/kmipkit-client/tests/fixtures/xml/MSGENC-XML-M-1-21.xml`; source and
notices are retained. The test binds `$NOW` to `2023-11-14T22:13:20Z`, replays
all two requests and two responses, and verifies semantic parse/emit/parse
identity, maximum response sizes 256 and 2048, then failure and success
statuses. This is codec fixture replay, not proof of actual server response
size enforcement, complete XML Client profile conformance or certification.

## Self-review

- Confirmed only the generated F mapping, not codec-side family inference,
  resolves QueryFunction. Masks consume only assigned M/B rows; reserved and
  extension placeholders preserve raw bits through hex.
- Confirmed decoder byte preflight, count/depth preflight before payload
  decoding, bounded iterative traversal, and encoder checked appends.
- Confirmed DTD/entity/schema/network resolution is absent, malformed XML
  declarations and reference grammar fail, and source/parser payloads never
  enter retained errors.
- Confirmed borrowed input is not retained, initialized scratch and output
  use zeroizing owners, and model ownership/runtime/allocation-growth limits
  are documented rather than overstated.
- Confirmed exact upstream OASIS sources and `public-api.json` are unchanged;
  no dispatch or 0057 files were edited. The fixture hash is stable in Git.
- Reviewed the tests against independent literal expectations and the pinned
  source, including all eleven types, negative ranges, exact configured bounds,
  namespaces, all assigned mask flags, unknown values and full DateTime range.

Implementation is ready for the parent integration review. Human acceptance,
draft PR/release branch management and KMIPKIT-0057 integration remain under
the orchestrating task; no merge or approval was performed here.

## Independent review follow-up

Two findings were verified and corrected within the same private codec scope.
Inspection of pinned quick-xml 0.42.0 `ReaderState::emit_start` confirmed it
unconditionally appends nonempty element names to its ordinary internal String
and index Vec even when `check_end_names` is false. The earlier report's claim
that the parser stack was disabled was incorrect and is corrected in
`implementation.md`. A complete, allocation-free lexical pass now runs before
Reader construction. The byte cap applies first; the lexical pass bounds Item
count and nesting, including escaped Structure type metadata, before any
parser/model allocation. It borrows input, skips non-item markup, recognizes
quoted delimiters, and uses borrowed attributes with duplicate storage disabled.
Full XML/KMIP validation, including duplicate rejection and matching end tags,
still runs afterward. Reader name bytes are bounded by message size and its
entries by element count and Structure depth plus at most one scalar frame;
these dependency-internal allocations are infallible and not zeroized.

The QName helper previously accepted an empty prefix in `:RequestPayload` and
treated a namespace declaration suffix as a QName, accepting `xmlns:a:b`.
It now requires nonempty NCName prefix/local components and a single NCName
declaration prefix, using the exact XML 1.0 fifth edition section 2.3 [4]/[4a]
character ranges. Unicode positive controls cover combining marks, middle dot,
connector characters, zero-width non-joiner and supplementary name characters.
References: [XML 1.0 section 2.3](https://www.w3.org/TR/REC-xml/#sec-common-syn)
and [Namespaces in XML 1.0 sections 3-4](https://www.w3.org/TR/REC-xml-names/#ns-qualnames).

| Phase | Signed commit | Evidence |
|---|---|---|
| RED | `228856652` | 23 focused tests passed; 4 new regressions failed as expected: complete count/depth checks occurred after earlier invalid payload/namespace processing, malformed QName was accepted, valid XML Unicode NCNames were rejected. |
| GREEN | `adbced8c3` | 27/27 focused XML tests, complete client suite and client all-target/all-feature Clippy passed after the borrowed preflight and exact QName/NCName fix. |
| REFACTOR | `fa90504dd` | Removed the redundant allocating event-local type preflight, preserving defensive model checks. Added delimiter/escaped-type and malformed lexical regression controls; 29/29 focused tests and Clippy passed. |

Fresh final checks on `x86_64-pc-windows-msvc`:

| Command / check | Result |
|---|---|
| `cargo test -p kmipkit-client --all-features` | Exit 0; 710 passed, 0 failed, 3 pre-existing Cosmian KMS tests ignored; 636 library tests passed. Includes all official XML fixture replay cases. |
| `cargo fmt --all --check` | Exit 0. |
| `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings` | Exit 0; no exemptions added. |
| `cargo llvm-cov --package kmipkit-client --lib --json --output-path .superpowers/sdd/0056-xml/coverage-review.json -- xml_codec --test-threads=1` | Exit 0; 29 tests passed. New lexical preflight 66/66 lines; characters 66/66; decoder 200/210. All XML plus common error code: 789/809 (97.53%). |
| `cargo llvm-cov report --lcov --output-path .superpowers/sdd/0056-xml/review-coverage.lcov` | Exit 0; intersecting LCOV DA records with production added lines in `git diff --unified=0 c5f3c46fd` gives 102/102 corrective changed executable lines (100%). |
| `git diff --check` | Exit 0. |
| Worktree and committed fixture SHA-256 | Both remain `5ca1a4533bb8e911231ea9e43c446894ff11f9004e2ca74531515327145e75d3`. |

No dependency/lockfile, public API, dispatch, generated output or pinned OASIS
source changed in this follow-up. The earlier fresh dependency-policy result
remains the dependency evidence; no new dependency exception was introduced.
Detailed final logs and coverage artifacts are retained locally alongside this
report. No PR was created, approved or merged during this follow-up.
