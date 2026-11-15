# KMIPKIT-0105 Quickstart Validation

1. Build the Python extension from the workspace using the repository's
   pinned binding workflow.
2. Run the focused crypto fluent tests from `bindings/python`:
   `python -m pytest tests/test_fluent_crypto.py`.
3. The local mutual-TLS fixture must record the expected KMIP operation and
   semantic payload, then return an encoded KMIP 2.1 response.
4. Verify all seven builders return the existing `BatchResult` outcome and
   make one exchange only. Repeat with malformed/incomplete requests and
   confirm the fixture receives no connection.
5. Run the Python binding suite, API-manifest generation/check, exact export
   audit, format/lint checks, and CI for the pushed PR head.

The builder API is expected to follow this shape (field names are operation
specific and must match the published type information):

```python
with client.request().encrypt() as request:
    request.unique_identifier(key_id)
    request.cryptographic_parameters(parameters)
    request.data(plaintext)
    result = request.execute(limits)
```

The example does not select an algorithm, mode, IV, or usage policy implicitly.
