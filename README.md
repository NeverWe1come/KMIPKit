# KMIPKit

> **Open-Source OASIS KMIP 2.0 / 2.1 / 3.0 Client Suite + Zero-DB Stateless Cloud KMS & KMIP 1.X Legacy Proxy**  
> **Core:** Rust (Edition 2024, `#![forbid(unsafe_code)]`) | **Bindings:** C (C11 ABI) · Java (JNI) · Python (CFFI)  
> **Encodings:** TTLV (default) · JSON · XML | **Transports:** Raw TLS 1.3/1.2 mTLS · HTTPS (HTTP/1.1)

**Documentation Portal:** [English](docs/en/00-index.md) · [Español](docs/es/00-index.md) · [Français](docs/fr/00-index.md) · [Deutsch](docs/de/00-index.md)

---

## Overview

**KMIPKit** is a memory-safe, specification-driven **OASIS KMIP** client library and proxy ecosystem built around a single authoritative Rust core:

* **KMIP Client Core (`crates/`):** Targets **KMIP 2.1** by default with explicit version-gated support for **KMIP 2.0** and **KMIP 3.0** (including NIST Post-Quantum Cryptography: `ML-KEM`, `ML-DSA`, `SLH-DSA`). Supports all 59 C2S and S2C operations, atomic batches with `ID Placeholder`, explicit asynchronous control (`Poll`/`Cancel`/`Process`), mandatory mTLS, zeroization, and full parity across **Rust, C, Java, and Python**.
* **KMIPKit Cloud KMS Proxy (`kmip-proxy/`):** Independent Zero-DB Stateless sidecar translating KMIP to **10 Cloud & Enterprise KMS providers** (AWS, Azure, Google Cloud, Oracle OCI, HashiCorp Vault / OpenBao, Infisical, IBM Cloud, Alibaba Cloud, Scaleway, and Open Telekom Cloud) with *Cloud Tag-Stuffing* and *Auto-Envelope* AES-256-GCM custody.
* **KMIP 1.X Legacy Bridge (`kmip-proxy/src/kmip1x/`):** Transparent bidirectional protocol translator allowing legacy **KMIP 1.0–1.4** applications (MongoDB, MySQL, VMware vSphere, NetApp ONTAP, banking HSM clients) to connect to modern **KMIP 2.0, 2.1, and 3.0** servers without modifying legacy code—covering all 77 standard attributes, `Template` emulation, Read-Modify-Write `Attribute Index`, `Query` allowlists, and S2C bridging.

---

## Quick Verification

```bash
# 1. Build checksum-pinned OpenSSL 3.5.9
python3 verification/security/openssl/build.py
export OPENSSL_DIR="$HOME/.cache/kmipkit/openssl/3.5.9"
export OPENSSL_STATIC=1

# 2. Check and test the 8-crate core workspace
cargo check --workspace --all-targets --all-features
cargo test --workspace

# 3. Check and test the standalone KMIP Proxy & KMIP 1.X Legacy Bridge
cargo check --manifest-path kmip-proxy/Cargo.toml --all-targets
cargo test --manifest-path kmip-proxy/Cargo.toml
```

---

## Documentation & Governance

* **Multilingual Documentation Portal:** [English](docs/en/00-index.md) · [Español](docs/es/00-index.md) · [Français](docs/fr/00-index.md) · [Deutsch](docs/de/00-index.md)
* **Language Bindings:** [C](bindings/c/README.md) · [Java](bindings/java/README.md) · [Python](bindings/python/README.md)
* **Project Status & Governance:** [Project Status](docs/project-status.md) · [Contributing](CONTRIBUTING.md) · [Governance](GOVERNANCE.md) · [Security Policy](SECURITY.md)

## License

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
