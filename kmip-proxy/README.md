# KMIPKit Proxy

Standalone KMIP service that bridges KMIP clients to cloud key-management providers. This crate has its own Cargo workspace and lockfile; it is kept outside the core Rust workspace and language bindings.

Provider integrations are selected with Cargo features such as `aws`, `azure`, `gcp`, `oci`, and `vault`. `all-clouds` enables all provider features.

The proxy uses OpenSSL for its cryptographic operations and as the rustls TLS provider. This includes the AWS SDK HTTPS transport and Azure's reqwest transport. Build it with the repository's checksum-pinned OpenSSL installation so the proxy and KMIPKit use the same native OpenSSL version:

```sh
python3 verification/security/openssl/build.py
export OPENSSL_DIR="$HOME/.cache/kmipkit/openssl/3.5.9"
export OPENSSL_STATIC=1
cargo check --manifest-path kmip-proxy/Cargo.toml --all-features
```

The SDK HTTP clients use OpenSSL-backed rustls and do not enable the AWS-LC or Ring crypto-provider features.

From the repository root:

```sh
cargo check --manifest-path kmip-proxy/Cargo.toml
cargo test --manifest-path kmip-proxy/Cargo.toml
```

Provider tests use local mock services except the AWS LocalStack integration test, which is retained in source but disabled on the review branch because it requires that external service.
