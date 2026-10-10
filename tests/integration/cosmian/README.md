# Local Cosmian KMS integration tests

This test deployment runs Cosmian KMS `5.28.0` locally and exposes only its
binary TTLV KMIP socket on `127.0.0.1:5696`. It generates a local test CA and
short-lived mTLS certificates; no public server or account is used.

The ignored Rust integration target contains operation-specific typed test
cases for ten KMIPKit operations:
Discover Versions, Create, Create Key Pair, Create Split Key, Add Attribute,
Delete Attribute, Get Attribute List, Get Attributes, Modify Attribute, and
Set Attribute. Adjust Attribute was probed but is rejected as unsupported by
the pinned Cosmian 5.28.0 parser. Exact OASIS operation clauses and
per-operation results are recorded in
`specs/015-cosmian-kms-integration/traceability.md`. Passing this selected
suite does not establish full KMIP 2.1 support, profile conformance, or
certification.

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
cargo test -p kmipkit-client --test cosmian_kms -- --ignored --test-threads=1
```

The tests use KMIPKit's typed production client over raw TTLV TLS with mTLS;
they do not hand-encode messages, import key material, or destroy objects.
Create tests generate server-side test objects, and the attribute tests change
metadata on their own test objects. All data remains in the container-local
database, which is removed by `-Action down`. The tests have no retry or server
fallback. Ordinary test runs skip them because each one is marked ignored.

## Recorded result

On 2026-10-08, the first live attempt reached Cosmian over verified mTLS and
Cosmian logged the `DiscoverVersions` request, but KMIPKit rejected the
response because it expected the Request Message root tag. The raw-TLS
response validator was corrected to require the OASIS Response Message tag
`0x42007B` (KMIP v2.1 §8.4, Table 397, and §11.56). The rerun passed: 1
passed, 0 failed. Compose pins the exact image references
`ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`
and
`alpine:3.22.2@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412`.

The initial expanded run on the unpatched KMIPKIT-0015 branch reported 8
passed and 2 failed: Create used the wrong tag (`0x420008`, `Attribute`, rather
than `0x420125`, `Attributes`), and Create Split Key stopped while creating its
source key. On 2026-10-10, PR #56 was synchronized with `release/1.0.0`, which
includes the separately reviewed KMIPKIT-0014 correction from PR #69. The same
serial command passed 10/10 on merge head `a1945f4e`, including Create and
Create Split Key. After pinning both Compose images by digest, it passed 10/10
again on commit `9a6a6341`. The correction is inherited from release and is not
duplicated in PR #56. Adjust Attribute was probed separately and Cosmian 5.28.0 rejects
it as unsupported.
See `specs/015-cosmian-kms-integration/traceability.md` for operation-by-
operation evidence. These results validate only the tested requests and do
not establish full KMIP 2.1 conformance or certification.

## Stop the server

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action down
```

This removes the container and its disposable database. To discard generated
certificates as well, remove `.local/cosmian-kms/`; the next start regenerates
the PKI.
