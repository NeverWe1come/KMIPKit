# KMIPKIT-0115 Quickstart Validation

## Java

1. Create the existing mutual-TLS `KmipClient`, `CodecLimits`, `UniqueIdentifier`,
   `AttributeSet`, and `RevocationReason` values.
2. Call each `client.request()` builder's `build()` and `execute(limits)` path.
3. Verify the controlled peer decodes the expected operation and supplied
   optional fields, then return one encoded typed response.
4. Verify direct methods use the same result path and invalid fields send no
   bytes.

## Python

1. Create the existing mutual-TLS `Client`, `CodecLimits`, identifier,
   `AttributeSet`, and reason values.
2. Call each `client.request()` builder's `build()` and `execute(limits)` path.
3. Repeat through the named direct client methods and verify the fixture's
   decoded requests and typed responses.
4. Verify invalid fields are rejected before transport and all temporary
   handles close on success and failure.

Run focused tests with the Java fluent test task and
`python -m pytest bindings/python/tests/test_fluent_lifecycle.py`.
