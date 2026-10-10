# Quickstart: Local Cosmian KMS Integration Tests

This quickstart deploys Cosmian KMS 5.28.0 locally and provides opt-in typed
test cases for the ten operation paths in KMIPKIT-0015. It does not claim
broad protocol conformance.

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

The test target sends typed requests for Discover Versions; Create; Create Key
Pair; Create Split Key; Add Attribute; Delete Attribute; Get Attributes; Get
Attribute List; Modify Attribute; and Set Attribute. Start
with the full ignored integration target:

```powershell
cargo test -p kmipkit-client --test cosmian_kms -- --ignored --test-threads=1
```

To run one operation test, add its exact Rust test name before `-- --ignored --exact --test-threads=1`, for example:

```powershell
cargo test -p kmipkit-client --test cosmian_kms cosmian_kms_accepts_kmip_2_1_discover_versions_over_mutual_tls -- --ignored --exact --test-threads=1
```

The test requests follow OASIS KMIP Specification v2.1 §6.1.16, Tables
211–213; §6.1.8, Tables 186–188; §6.1.9, Tables 189–192; §6.1.10, Tables
193–195; §6.1.2, Tables 167–169; §6.1.13, Tables 202–204; §6.1.20, Tables
223–225; §6.1.21, Tables 226–228; §6.1.34, Tables
265–267; and §6.1.51, Tables 322–324. Discover Versions must return success
and include version `(2, 1)` as encoded under §9.16, Table 421. Each other
operation must return a successful typed response with its required response
fields. The tests create server-generated objects and test metadata in the
container-local database; the database is discarded when the container is
removed.

## Recorded result

On 2026-10-08 the first live attempt reached Cosmian over verified mTLS but
failed because KMIPKit expected the Request Message root tag on the response
path. After correcting raw-TLS response validation to use the OASIS Response
Message tag, the Discover Versions test passed (1 passed, 0 failed). Compose
pins the exact image references
`ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`
and
`alpine:3.22.2@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412`.
The initial expanded run on the unpatched KMIPKIT-0015 branch reported 8
passed and 2 failed: Create used the wrong Attributes tag, and Create Split
Key stopped during setup. On 2026-10-10, PR #56 was synchronized with current
`release/1.0.0`, which includes the separately reviewed KMIPKIT-0014 corrections
from PR #69. The serial test command passed 10/10 on merge head `a1945f4e`,
including Create and Create Split Key. After pinning both Compose images by
digest, it passed 10/10 again on commit `9a6a6341`. The corrections are inherited
from release and are not duplicated in PR #56. Adjust Attribute was probed
separately and Cosmian 5.28.0 rejects it as unsupported, so it is not included in the
supported-operation test target. See
`specs/015-cosmian-kms-integration/traceability.md` for operation-level
results. These selected operations are not a broad conformance claim. The
hosted multi-platform CI and coverage gates passed on 2026-10-09; they must be
rerun on the synchronized PR head before review. The local raw Windows coverage
summary is not the platform-aggregated coverage gate.

## Stop and remove the server

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action down
```

The database is container-local and is discarded when the container is
removed. Delete `.local/cosmian-kms/` separately if you also want to discard
the generated certificate files.
