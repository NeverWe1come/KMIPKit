# Quickstart: Local Cosmian KMS Smoke Test

This quickstart deploys Cosmian KMS 5.28.0 locally and provides an opt-in
Discover Versions test. It does not claim broad protocol conformance.

## Prerequisites

- Docker Engine/Desktop with Docker Compose.
- Rust 1.94 or later and the repository-pinned toolchain.
- KMIPKIT-0013 production transport changes available in the checkout.

## Start the local listener

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action up
.\scripts\integration\cosmian-kms.ps1 -Action status
```

The server exposes raw KMIP binary TTLV over TLS on `127.0.0.1:5696`. Compose
generates mTLS certificates in ignored `.local/cosmian-kms/certs/` before the
server starts. Only the test CA and client identity are read by KMIPKit; the
Cosmian container receives its server identity and the public client CA.

## Run the live test

The test is deliberately ignored by default. With the local server running,
run the live test explicitly:

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action test
```

The test sends one typed Discover Versions request, defined by OASIS KMIP
Specification v2.1 §6.1.16, Tables 211–213, and asserts a successful result
with returned version `(2, 1)` as encoded under §9.16, Table 421.

## Recorded result

On 2026-10-08 the first live attempt reached Cosmian over verified mTLS but
failed because KMIPKit expected the Request Message root tag on the response
path. After correcting the raw-TLS response validation to use the OASIS
Response Message tag, the live test passed (1 passed, 0 failed). The pinned
image resolved to
`ghcr.io/cosmian/kms@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`.
This result covers only Discover Versions and is not a broad conformance
claim.

## Stop and remove the server

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action down
```

The database is container-local and is discarded when the container is
removed. Delete `.local/cosmian-kms/` separately if you also want to discard
the generated certificate files.
