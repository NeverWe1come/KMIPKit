# Proxy Testing & Troubleshooting Guide

---

## 1. Test Suites & Verification Commands

### Prerequisites

Always ensure the repository's pinned OpenSSL build (`3.5.9`) is active before compiling `kmip-proxy`:

```sh
python3 verification/security/openssl/build.py
export OPENSSL_DIR="$HOME/.cache/kmipkit/openssl/3.5.9"
export OPENSSL_STATIC=1
```

### Core & KMIP 1.X Bridge Suites (83 Tests)

```sh
cargo test --manifest-path kmip-proxy/Cargo.toml --features kmip1x-bridge --tests
```

This executes unit tests plus 7 dedicated integration test suites in `kmip-proxy/tests/`:

| Test Suite File | Focus Area |
| :--- | :--- |
| `kmip1x_bridge_tests.rs` | All 41 client ops × `(2.0, 2.1, 3.0)`, `Discover Versions`, `Query` allowlists, `Add`→`Set`, RMW `Modify`/`Delete`, `Template` lifecycle, `ID Placeholder`, `Max Response Size`, S2C `Notify`, pass-through, live TCP E2E. |
| `kmip1x_exhaustive_roundtrip_tests.rs` | Bidirectional roundtrip of **all 77 standard attributes**, `GetAttributes`/`GetAttributeList`, version-gated downgrading (`1.0..1.4`), **all 14 KMIP 3.0 Typed Links**, `Create Key Pair` templates, `Export` wrapping specs, sequential indices, S2C all ops. |
| `kmip1x_bugfix_and_regression_tests.rs` | Chained `ID Placeholder` + RMW batch orchestration, `(vendor, name)` custom attr RMW, KMIP 3.0 link RMW & `Get Attribute List` deduplication, concurrent duplicate `TemplateStore` overwrites, `Key Block` ↔ `Key Value` recursion + wrapped `ByteString` preservation, `TemplateStore` restart persistence. |
| `kmip1x_edge_security_and_interop_tests.rs` | Mixed orchestrated batches, KMIP 1.4 → Cloud KMS chaining, HTTP TTLV/JSON/XML over `kmip1x` bridge, TLS 1.2 compat + AutoPKI mTLS, authentication/message extensions, AEAD/DeriveKey, empty/adversarial batches, 32 concurrent clients. |
| `kmip1x_full_matrix_and_chaos_tests.rs` | Matrix of 9 `Template-Attribute` ops × 77 attrs, `Locate` with all 77 attrs, `Add Attribute` cardinality matrix × 77 attrs, unknown ops, 1.0 cert aliases, 50-instance RMW stress, exact `Maximum Response Size` ±1 byte boundary, UTF-8 custom attrs, network chaos/truncated frames. |
| `kmip1x_protocol_conformance_tests.rs` | 5×3 version matrix (`1.0..1.4` × `2.0/2.1/3.0`), all 11 TTLV value types, async pending/`Poll`/`Cancel`, batch header options, `Discover Versions` intersections, `Query` PQC filtering, S2C `Put` replace + errors. |
| `kmip1x_wire_tls_and_security_tests.rs` | Real mTLS TLS 1.3/1.2 handshakes + unauth rejection, HTTP keep-alive & malformed `Content-Length` rejection, TOML/CLI config parsing, 256-case deterministic fuzzing, 64 concurrent TCP clients, real double-hop mTLS (PEM client → proxy PEM/DER → pooled `KmipKitUpstreamTransport` → real `kmipkit_server::KmipServer`). |

### Cloud Provider Feature Suites

Every backend workflow in `.github/workflows/proxy-*.yml` can be verified locally against in-process mock HTTP/KMS endpoints:

```sh
for feat in aws azure gcp oci vault infisical ibm alibaba scaleway otc; do
  cargo clippy --manifest-path kmip-proxy/Cargo.toml --all-targets --features "$feat" -- -D warnings
  cargo test --manifest-path kmip-proxy/Cargo.toml --features "$feat"
done
```

### Strict Formatting, Clippy & Rustdoc Verification

```sh
cargo fmt --manifest-path kmip-proxy/Cargo.toml --check
cargo clippy --manifest-path kmip-proxy/Cargo.toml --features kmip1x-bridge --all-targets -- -D warnings
RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo doc --manifest-path kmip-proxy/Cargo.toml --no-deps
```

---

## 2. Troubleshooting Common Issues

| Symptom | Root Cause | Resolution |
| :--- | :--- | :--- |
| Build fails finding OpenSSL headers / `libssl` | `OPENSSL_DIR` or `OPENSSL_STATIC=1` not exported in the shell | Run `python3 verification/security/openssl/build.py` and export `OPENSSL_DIR="$HOME/.cache/kmipkit/openssl/3.5.9"` and `OPENSSL_STATIC=1`. |
| Legacy KMIP 1.X client fails TLS handshake | Listener defaults to TLS 1.3 only (`allow_tls12 = false`) | Enable `allow_tls12 = true` under `[listener]` (or pass `--allow-tls12`). |
| TLS fails to parse `.der` / `.cer` certificates | Old proxy versions expected PEM; current version auto-detects both | Verify the file is valid PEM or raw X.509/PKCS#8 DER (`openssl x509 -in cert -inform DER -noout`), and confirm client/server keys match. |
| Server rejects `Template-Attribute (0x420091)` if testing proxy internals | Strict KMIP 2.0 TTLV decoding rejects KMIP 1.X reserved tags | Ensure incoming KMIP 1.X frames go through `kmip-proxy`'s codec / `kmipkit_ttlv::decode_allow_reserved` path rather than strict `decode()`. |
| `Create` fails with `Operation Not Supported` on legacy clients sending `Operation Policy Name` | `operation_policy_mode` is set to `"reject"` | Set `operation_policy_mode = "ignore"` (default) or `"store_as_vendor_attribute"` under `[backend.kmip_upstream]`. |
| Upstream KMIP server throws Read-Only attribute error on `Create`/`Register` | Custom or caller-forced read-only attrs passed directly | Keep default `TranslatorConfig { strip_readonly_on_create: true, .. }` so read-only KMIP attrs (`State`, `Initial Date`, `Unique Identifier`, etc.) are stripped on creation ops. |
| Client receives `Response Too Large (0x02)` | Client's `Maximum Response Size` header is smaller than the re-encoded KMIP 1.X TTLV byte count | Even if the 2.0+ upstream response fit, wrapping tags back into 1.X `Attribute` structures adds bytes; increase client `Maximum Response Size`. |
| Templates vanish after proxy restart | `template_store_path` is unset (`TemplateStore::new()` is in-memory only) | Set `template_store_path = "/var/lib/kmip-bridge/templates.json"` under `[backend.kmip_upstream]` for atomic on-disk persistence. |
| CLI prints `Unknown or unconfigured provider: vault, defaulting to in-memory mock` | `src/main.rs` CLI dispatcher directly constructs `kmip-upstream`, `mock`, `aws`, `azure`, `gcp`, and `oci` | To run `vault`, `infisical`, `ibm`, `alibaba`, `scaleway`, or `otc`, instantiate them programmatically via `kmipkit_proxy::providers::create_provider(&config.backend)`. |
