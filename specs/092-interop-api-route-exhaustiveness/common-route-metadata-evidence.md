# Common route metadata evidence

## Red — route assertions use the installed artifact's API inventory

The route manifest pins the comparable release artifact to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` under KMIPKIT-0092-FR-006. After
updating the common branch to `release/1.0.0` at
`e1b12188b50a689ede5daf373651d00cda3367e6`, the focused route test read
`specification/api/public-api.json` from the updated checkout instead of from
the pinned artifact. That newer manifest includes response JSON, XML, and
TTLV-hex encoders which are absent from the installed artifact. The test then
reported nine missing C, Java, and Python routes.

The failure was reproduced with
`python3 -B -m unittest interop.shared.tests.test_route_capabilities.RouteCapabilitiesTests.test_public_request_serializers_and_response_readers_are_routes`.
The same nine missing-route assertions appear in failed CI run
`43692884852`, job `132747024380`.

## Green — read API metadata from the recorded artifact commit

The route assertion now loads `specification/api/public-api.json` through the
existing pinned-source reader using `library_commit` from the route manifest.
This keeps route availability tied to the installed release artifact, as
required by KMIPKIT-0092-FR-006. The focused test passed after the change:

```text
python3 -B -m unittest interop.shared.tests.test_route_capabilities.RouteCapabilitiesTests.test_public_request_serializers_and_response_readers_are_routes
Ran 1 test — OK
```

## Refactor — centralize pinned API loading

The test class now loads public API metadata through one `_pinned_public_api`
helper, keeping future route assertions on the same artifact boundary. After
this refactor, the route-capability suite passed **63 tests**, and the workflow
contract suite passed **95 tests**.


## Shared artifact pin parity refresh (2026-11-16)

The Java route matrix refresh uses `12496d646d11d4069b58bcc88fc76cc619211ec6`,
the common manifest's `library_commit`, and the Java and Python installed-package
pins. After this refresh, the Rust installed consumer still used its earlier
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` pin, so its route matrix rejected
the common manifest with `route artifact commit mismatch`.

A new shared test,
`test_all_language_artifact_pins_match_shared_route_manifest`, failed on that
Rust mismatch. The Rust installer, verifier, test-server build, compile-time
model identity, and expected-source test now use the common `12496d...` commit.
The test passes; installing and verifying the Rust package succeeds with
artifact SHA-256
`ce3146009589b7caa11590f89cea7ca775b09e9af7e3c6a2fe1ff704fa0c55b2`.
`route_capabilities.py --check` accepts all 1,507 routes, and the Rust pin
contract and formatting checks pass. Full controlled-peer CI remains the final
verification for this refresh.

## CI follow-up: pin alignment and response views (2026-11-16)

CI on the previous #187 head found two independent gaps. The Rust validator
still expected the old `489fafc...` artifact while the shared manifest and
Java/Python validators used `12496d...`. The Python matrix left 186 cells
pending because it declared, but did not invoke, the `BatchItemResult` full
response TTLV-hex, JSON, and XML views.

The Red test commit `32a6ba574` added a shared cross-language pin-parity
contract and failed on the Rust mismatch. Green commit `76fd0cd92` aligned the
Rust installer, verifier, test server, compile-time model identity, and tests
to `12496d...`. Red commit `12d2dbb8a` added a controlled-TLS test for each
Python response view and a pinned-lock test for the Rust test-server harness;
the view test failed as a missing route recipe before its fix. The harness's
first `--locked` build attempt also exposed that its generated lock selected
`cc 1.11.2`, which is incompatible with the pinned release. The harness now
seeds its lock from the pinned release, fetches the standalone harness graph,
then validates metadata and builds offline with `--locked`, retaining the
release's `cc 1.6.0`.

The next exact-head CI run exposed a clean-runner dependency-cache gap:
`cargo metadata --offline` could not find `asn1-rs 0.7.2`, which happened to
be cached locally. Red commit `34bf5ddc3` added a regression requiring an
explicit dependency-fetch step and failed before the fix. Green commit
`980885bbb` fetches the graph against the copied pinned lock before the offline
metadata/build steps. The focused regression passed, and
`python3 interop/rust-kmip-validator/tools/build_test_server.py` built the
harness locally from the same pinned source.

Verification on the pinned Python wheel (`12496d...`, SHA-256
`18043fee5e92893bf4d3ac495552121ac8edc7cbc7f6e77472143e2b6ccb7d78`): the
three controlled-TLS response-view routes pass, and the full matrix plus the
independent shared validator accept **74,842/74,842** cells with zero failed,
pending, or duplicate events. Run ID:
`python-matrix-4e712f18-fea8-4c41-ad06-f7bf9f6478f5`; route-manifest SHA-256
`411aad566ee5f53e36d8db10a6f5792c6411559871fc53a52a9425dba9911cc8`. The
Rust route-validator test suite passed 51 tests, and the pinned test-server
harness built successfully. The Python validator suite passed all 133 tests in
369.480 seconds. Exact-head CI for the updated PR stack remains required before
removing WIP markers.
