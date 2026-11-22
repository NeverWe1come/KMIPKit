# Shared mTLS test identities

These self-signed PEM identities are shared test-only TLS fixtures used by
the Java and Python bindings. The client and server trust the opposing test
certificate directly and require TLS 1.3 client authentication.

- `client-cert.pem` and `client-key.pem` identify the test client.
- `server-cert.pem` and `server-key.pem` identify the controlled test server.

Consumers include the Python
[login integration test](../../../python/tests/test_login_native_integration.py)
and the Java
[client TLS integration test](../../../java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java).

The private keys are disposable test material and must never be reused
outside tests. Keep these fixtures in this shared directory so bindings can
exercise the same test identities.
