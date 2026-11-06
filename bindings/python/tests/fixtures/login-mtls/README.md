These self-signed PEM identities are test-only TLS fixtures for
`test_login_native_integration.py`. The client and server trust the opposing
test certificate directly and require TLS 1.3 client authentication. The
private keys are disposable test material and must never be reused.
