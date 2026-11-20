# KMIPKit Proxy

Standalone KMIP service that bridges KMIP clients to cloud key-management providers. This crate has its own Cargo workspace and lockfile; it is kept outside the core Rust workspace and language bindings.

Provider integrations are selected with Cargo features such as `aws`, `azure`, `gcp`, `oci`, and `vault`. `all-clouds` enables all provider features.

From the repository root:

```sh
cargo check --manifest-path kmip-proxy/Cargo.toml
cargo test --manifest-path kmip-proxy/Cargo.toml
```

Provider tests use local mock services except the AWS LocalStack integration test, which is retained in source but disabled on the review branch because it requires that external service.
