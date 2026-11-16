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
