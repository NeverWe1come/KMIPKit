# KMIPKIT-0049 Implementation Evidence

## Source and branch baseline

- Authorized feature chain root: `feature/KMIPKIT-0043-t076-shared-client-mapping` (PR #104).
- Immediate predecessor: `feature/KMIPKIT-0048-typed-attributes`, PR #115, commit `48f8ceac29eea37ac6a65fa81fc9580af33930bd`.
- Current feature branch: `feature/KMIPKIT-0049-managed-objects`; the maintainer authorized the phase stack to start from #104 and to use predecessor branches until eventual integration into `release/1.0.0`.
- Pinned OASIS source: `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`, SHA-256 `8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF`.
- Normative catalog: `specification/catalog/kmip-2.1.json`, SHA-256 `0D68D403956FE0BA4057BED346FDE57D1BC8938D975338E02BE2D435EC16FBEE`.
- Source check: `traceability.md` contains 53 unique operative requirement IDs; all 53 exist in the pinned catalog. The 9 structure tag associations were independently checked against catalog `wire_value` entries sourced from KMIP 2.1 §11.56.
- No clarification question remained: KMIP 2.1 tables resolve structure shape, ordering, tags, and conditional requirements; unresolved server behavior and out-of-scope capabilities remain explicit exclusions.

## Red — tests before implementation

The following tests were added before production code. Their compile failures are the expected Red state because KMIPKIT-0049 types and additive request helpers do not yet exist.

| Command | Result | Expected Red evidence |
|---|---|---|
| `cargo test --offline -p kmipkit-protocol --tests --no-run` | Failed as expected | `E0432` for missing managed-object, Key Block/Key Material, wrapping, and transparent key types; no implementation has been added yet. |
| `cargo test --offline -p kmipkit-client --no-run` | Failed as expected | `E0432` for missing typed models and `E0599` for the not-yet-added `RegisterRequest::from_managed_object` and `ImportRequest::from_managed_object`. |
| `cargo fmt --all --check` | Passed | All Red tests and current Rust workspace files are formatted. |

The Red tests cover all nine object tags and fields, the §3.1–§3.3 key structures, the nine transparent material structures, conversion preservation, validation negatives, redaction, and fake-transport Register/Import requests. The exact test output is retained in the task command history; implementation acceptance will be based on Green results, not on these expected compile failures.

## Development commits

- Specification/design: `686394817a34467bf4757fc9c7af7069df422575`.
- Normative order/tag correction: `d83cfec699ddf913258c37479f2bc78c2ca4e1d3`.
- Red — initial object/key structure contracts: `403466131c31c2b162eef7f88444cd65bfad8709`.
- Green — typed object/key structure implementation: `9f04e1fee39280f7ae2c7501a3495ad06f891266`.
- Red — parsed SplitKey/SecretData constraint rejection: `d27e322ac15cdc0a2e7859d330c36fc8e9f066c8`.
- Green — parsed object constraint validation: `8a7a3cb4886c18b4679dcef50ead2b30a0d884d6`.
- Red — wrapped Key Value key-information requirement: `351827162a644be11aa6fe65f2763c434414f6e1`.
- Green — enforce wrapping key information: `452751a1fdf814c1fa5e1d4181ee607374590ffc`.
- Red — reject malformed transparent key structures: `a78fd41f1fd2a94d41ed9505e3e90952dd59dda7`.
- Green — validate transparent formats and zeroize buffers: `d4d60044e1e6146b325da5bfeb545aa26515ae50`.
- Refactor — validate nested secret TTLV by reference, avoiding temporary clones: `56a6cbe58`.

## Local verification

- `cargo fmt --all --check`: passed.
- `cargo clippy --offline -p kmipkit-protocol --all-targets --all-features -- -D warnings`: passed.
- `cargo test --offline -p kmipkit-protocol --all-features --quiet`: passed; 1,273 unit tests plus all protocol integration and documentation tests.
- `cargo test --offline -p kmipkit-client --all-features --quiet`: passed; 635 unit tests plus client integration, UI/trybuild, and documentation tests. Windows emitted non-fatal `aws-lc-sys` compiler-feature and missing NASM warnings.
- `cargo test --offline -p kmipkit-protocol --test managed_object_examples --quiet`: passed; plaintext, wrapped, transparent, unknown-object, and secret-owner examples compile and run.
- `python -B -m unittest tools.api_manifest.tests.test_reconcile_0049_managed_objects`: passed; 2 tests.
- `python -B tools/api_manifest/generate.py --check`: passed; all 12 generated outputs are current.
- Rust public-route audit: passed; 6,621 manifest declarations matched, 0 missing, 0 issues, and 15,748 exact exclusions.
- Binding public-route audit: passed with exit code 0.
- Raw local protocol diagnostic: `cargo llvm-cov --offline -p kmipkit-protocol --all-features --summary-only` reported 87.59% across the crate before the coverage-gate exclusions. The new managed-object modules are below the required changed-code threshold in this diagnostic. This is not a passing coverage-gate result; T040 remains open until the repository coverage gate passes on the required platforms and scopes.
- Independent QA identified malformed parsed SplitKey/SecretData and transparent RSA structures, missing wrapping key information, and documentation gaps. Red tests demonstrated the defects; the corresponding Green commits, expanded examples, and borrowed validation refactor remediate them. Independent security review also identified avoidable secret-bearing TTLV clones; these validation routes now borrow the source Item. No FFI, transport, or unsafe code changed.
- The Codex Security diff-scan launcher rejected its selected working-tree artifact as stale before creating a scan. No automated security scan is claimed. The independent static security review and local source review are recorded above; the automated review limitation remains explicit.
- Hosted cross-platform CI has not been used as evidence for this branch. The draft PR is intentionally not merge-ready while coverage and platform checks remain outstanding. No profile conformance, certification, or interoperability claim is made.

## Remaining evidence

T040 coverage thresholds and hosted platform checks remain outstanding. T043 will record the draft pull request URL, base, and head after terminal-based publication. The phase has not been declared complete.
