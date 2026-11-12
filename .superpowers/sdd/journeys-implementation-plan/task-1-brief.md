### Task 1: Shared fixture contract and symmetric-key journey

Add failing consumer tests for Create → Get → Destroy in all four languages.
The fixture must use a real TLS 1.3/mTLS socket, decode and assert each request,
and return encoded, typed KMIP responses with identifiers that the next request
actually uses. Keep fixture certificates scoped to tests and test-only.

