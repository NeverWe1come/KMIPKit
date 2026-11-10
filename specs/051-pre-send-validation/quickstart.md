# KMIPKIT-0051 Validation Quickstart

## Local contract checks

Run focused protocol and client validation tests:

```powershell
cargo test -p kmipkit-protocol --all-features validation
cargo test -p kmipkit-client --all-features request_validation
```

The tests construct both valid and invalid typed requests, call the public report API, and assert issue codes, paths, severities, and exact source references without including input values.

## Pre-send behavior

Use the scripted transport fixture to execute a locally invalid request. The expected result is `RequestDeliveryState::NotSent`, zero exchange calls, and zero written bytes. Execute a valid request through the same fixture and compare its TTLV bytes with the pre-feature fixture to ensure validation does not change encoding.

## Traceability and public API checks

```powershell
python -B -m unittest scripts.tests.test_requirement_traceability -v
python -B -m unittest discover -s tools/normative_catalog/tests -p 'test_feature_traceability.py' -v
python tools/api_manifest/generate.py --check
python tools/api_manifest/audit_rust_exports.py --check
```

The full workspace test, format, Clippy, coverage, and platform checks remain required before the stacked draft PR can be considered for integration.
