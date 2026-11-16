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
