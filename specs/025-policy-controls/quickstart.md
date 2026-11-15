# Quickstart: KMIP 2.1 Policy and Usage Controls

This is the implementation and verification checklist for KMIPKIT-0039. Record command outcomes from the current revision; a passing local suite does not claim formal KMIP profile conformance.

## Baseline and dependency status

1. The implementation branch is `feature/KMIPKIT-0039-policy-controls-implementation`, based on the approved `release/1.0.0` branch.
2. Verify pinned source checksums in `specification/oasis/kmip-2.1/CHECKSUMS.sha256` and review `spec.md`, `plan.md`, `traceability.md`, and the two API contracts.
3. KMIPKIT-0043's approved public API manifest contract has not merged into `release/1.0.0`. Rust protocol/client work and documentation are complete; C, Java, Python, manifest, and generated parity tasks remain blocked until that contract is approved and merged.
4. Typed Encrypt, MAC, and Sign request variants were present on the release baseline and are reused by the guard; no duplicate wire models were introduced.

## Implementation order and current disposition

1. Get Constraints and Get Usage Allocation, their client paths, and the local caller guard are implemented with focused model/client tests.
2. Set Constraints and Set Defaults are implemented in separate Red/Green/Refactor sequences. Set Endpoint Role (§6.1.54) and server-to-client §6.2.5 are deferred from 1.0 because successful completion changes endpoint roles; no partial client-only API is exposed.
3. Requirement traceability and normative catalog implementation/verification references are updated. Generate catalog reports and code with the pinned generators; never edit generated reports by hand.
4. After 0043 merges, add public manifest entries, adapters, generated outputs, and C/Java/Python parity tests in a coordinated slice.
5. English/Spanish examples and the interop matrix are updated. Record real server/version evidence only after tests actually run.

## Focused checks

Run from the repository root after the matching tests exist:

```text
cargo test -p kmipkit-protocol --lib get_constraints
cargo test -p kmipkit-client --lib get_constraints
cargo test -p kmipkit-protocol --lib get_usage_allocation
cargo test -p kmipkit-client --lib usage_allocation_guard
cargo test -p kmipkit-client --lib set_constraints
cargo test -p kmipkit-client --lib set_defaults
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --write
python tools/normative_catalog/report.py --check
python tools/api_manifest/generate.py --check
git diff --check
```

Before requesting review, run the complete repository checks required by `AGENTS.md`:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
```

Run the API-manifest unit tests and generated-manifest check now. Run the binding-specific build/parity commands listed in `docs/development/api-manifest.md` after 0043 is available; the generated manifest check is not a substitute for testing handwritten adapters.

## 1.0 scope disposition

The 1.0 feature implements four operations plus the §6.1.23 usage-allocation guard. Set Endpoint Role remains inventoried as an OASIS operation but is unsupported and unadvertised in 1.0. Its required post-success role transition (§6.1.54 and §11.19) and reverse-direction operation (§6.2.5) need a separate 1.1 specification.

## §6.1.23 focused acceptance checklist

- The request encodes optional Unique Identifier and required Usage Limits Count in Table 232 order and type.
- Usage Limits Count is encoded as TTLV Long Integer (`i64`) per §7.40, Table 392.
- The response requires and returns the concrete Unique Identifier in Table 233.
- A context requires a complete successful allocation plus the exact request count and caller-known unit.
- Guarded Encrypt/MAC/Sign requires a matching concrete object identifier and positive caller-supplied cost.
- Cost equal to known available units is accepted; cost above it, zero, negative, mismatched ID, and ID Placeholder fail before transport.
- A proven `NotSent` releases the reservation; `PossiblySent`, `ResponseStarted`, complete results, and Pending retain it locally.
- No test or documentation equates local reservation with server consumption or claims coverage of unguarded/raw/external activity.
- Decrypt, MAC Verify, Signature Verify, and Hash are not charged by the usage guard.

Report command, platform, and result for each Red, Green, and Refactor evidence commit. Do not claim a direct official OASIS Test Cases pass for these operations; none is cataloged in the approved source inventory.
