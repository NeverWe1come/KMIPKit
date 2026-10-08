# Local Cosmian KMS integration test

This test deployment runs Cosmian KMS `5.28.0` locally and exposes only its
binary TTLV KMIP socket on `127.0.0.1:5696`. It generates a local test CA and
short-lived mTLS certificates; no public server or account is used.

The integration target covers one typed Discover Versions exchange only. It
checks OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213, and Protocol
Version structure encoding in §9.16, Table 421. Passing it does not establish
full KMIP 2.1 support, profile conformance, or certification.

## Start and inspect the server

From the repository root in PowerShell:

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action up
.\scripts\integration\cosmian-kms.ps1 -Action status
.\scripts\integration\cosmian-kms.ps1 -Action logs
```

Docker Compose runs the PKI generator before KMS. The generated private keys
are stored under the ignored `.local/cosmian-kms/certs/` directory. Cosmian
uses SQLite under its container filesystem with `clear_database = true`; no
database volume is mounted. The HTTP service is not published to the host.

## Run the test explicitly

With the production transport and local server available, run the ignored
live test explicitly:

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action test
```

Equivalent Cargo command:

```powershell
$env:KMIPKIT_COSMIAN_CERT_DIR = (Resolve-Path .local/cosmian-kms/certs).Path
cargo test -p kmipkit-client --test cosmian_kms -- --ignored
```

The test does not create, import, modify or destroy key objects. It has no
retry or server fallback. Ordinary test runs skip it because the test is
marked ignored.

## Recorded result

On 2026-10-08, the first live attempt reached Cosmian over verified mTLS and
Cosmian logged the `DiscoverVersions` request, but KMIPKit rejected the
response because it expected the Request Message root tag. The raw-TLS
response validator was corrected to require the OASIS Response Message tag
`0x42007B` (KMIP v2.1 §8.4, Table 397, and §11.56). The rerun passed: 1
passed, 0 failed. The pinned image resolved to
`ghcr.io/cosmian/kms@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`.

The live request used `cargo test -p kmipkit-client --test cosmian_kms
-- --ignored`. After the transport correction, verification also passed for
the raw-TLS suite (90 tests) and production-client suite (14 tests, with
`--features kmipkit-test-support/fixtures`). This validates one operation on
this server version; it does not establish full KMIP 2.1 conformance or
certification.

## Stop the server

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action down
```

This removes the container and its disposable database. To discard generated
certificates as well, remove `.local/cosmian-kms/`; the next start regenerates
the PKI.
