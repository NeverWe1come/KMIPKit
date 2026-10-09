# Quickstart Validation: KMIP 2.1 Query and Ping

This document specifies future implementation checks. It does not imply that KMIPKIT-0020 has been implemented.

## Deterministic operation checks

Use a fake transport; do not require a network server for focused tests.

- Ping sends one empty request and accepts an empty successful response.
- Query rejects an empty Query Function list before exchange.
- Query emits each standard Query Function and a valid extension value; repeated functions and optional Object Groups retain caller order. Object Groups preserves zero or more repeated Object Group attributes as Text String values (§6.1.40 Table 282, §7.23 Table 375, §4.35 Tables 99–100).
- Query round-trips all Table 283 response members, including repeated entries, absent entries, unknown enum values, and nested Items.
- Both source-described successful Query forms are accepted: an empty response payload under §6.1.40 and a structured Table 283 response with an empty Protection Storage Masks list. The conflict remains tracked as `KMIPKIT-DISC-045`; this does not claim server conformance.
- KMIP and transport failures preserve result and delivery evidence; no test observes an automatic retry or follow-up request.
- Debug/error output does not expose raw request/response payloads.

## Normative catalog checks

```powershell
python -B tools/normative_catalog/validate.py --repo-root .
python -B tools/normative_catalog/report.py --repo-root . --check
python -B tools/normative_catalog/check_immutable_sources.py --repo-root . --base-sha <active-release-base>
```

Expected: Query, Ping, Query Function, its named values, Object Groups structure/member and Object Group attribute, and both Query client requirements are assigned to KMIPKIT-0020; `KMIPKIT-DISC-045` records the open conflict; generated report is current; OASIS source copies are unchanged.

## Workspace checks

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
```

Run the repository's full PR CI matrix and report coverage results before the implementation PR is considered ready. An unavailable official fixture is not reported as a pass.