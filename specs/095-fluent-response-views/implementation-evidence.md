# KMIPKIT-0095 binding implementation evidence

## Red

- `cargo test -p kmipkit-ffi client_response_extension_abi_returns_ordered_generic_ttlv_copy --lib`: exit 101, E0425 for the three missing C response renderers.
- `mvn -q -f bindings/java/pom.xml test-compile`: exit 1, missing `ClientBatchItemResult.ttlvHex/json/xml(CodecLimits)`.
- Python contracts request the same three explicit methods and verify native ownership transfer with no transport available in the fake bridge.

The approved scope is T009 only. Exact remote CI remains T008 and is owned by the parent task.

## Green

- Red commit: `525ffa212` (DCO signed off).
- `CARGO_BUILD_JOBS=2 cargo test -p kmipkit-ffi --all-features --lib`: 473 passed, including compiled C response-view consumers, independent owners and category translation.
- `/tmp/kmipkit-0095-py312/bin/python -m pytest bindings/python/tests/test_response_views.py bindings/python/tests/test_response_raw_navigation.py -q`: 16 passed.
- `/tmp/kmipkit-0095-py312/bin/python -m pytest bindings/python/tests -q`: 1299 passed, 119 subtests passed.
- `sh bindings/java/native/build.sh`: passed native helper tests and JNI compilation.
- `mvn -q -f bindings/java/pom.xml -Dexec.skip=true '-Dtest=ResponseViewsTest,ClientFacadeTlsIntegrationTest#pingQueryAndLogoutRoundTripThroughNativeJniOverDeterministicTls13Peer' test`: 2 passed. The skip reuses the immediately built JNI library.
- `python3 tools/api_manifest/reconcile_0095_response_views.py` and `python3 tools/api_manifest/generate.py --write`: 12 generated outputs, without manual generated edits.

The tests compare complete response bytes and multi-item ordering, retained
vendor fields, size/depth/element limits, owned results after source/client
closure, scoped callback erasure (including exceptions), dead handles,
payload-free failures, and the no-send inspection boundary.

A workspace build and independent audits initially exhausted the 7.6 GiB host
and 2 GiB swap. They were stopped and reissued sequentially with two Cargo jobs
and a disk-backed temporary directory where supported. The no-feature FFI test
attempt also exposed the existing requirement to enable the compiled-consumer
feature; all-feature testing is the applicable command used above.
