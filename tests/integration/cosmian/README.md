# Local Cosmian KMS integration tests

This test deployment runs Cosmian KMS `5.28.0` locally and exposes only its
binary TTLV KMIP socket on `127.0.0.1:5696`. It generates a local test CA and
short-lived mTLS certificates; no public server or account is used.

The ignored Rust integration target contains operation-specific typed test
cases for twenty KMIPKit operations:
Discover Versions, Create, Create Key Pair, Create Split Key, Add Attribute,
Delete Attribute, Get Attribute List, Get Attributes, Modify Attribute, Set
Attribute, Activate, Query, Ping, Encrypt, Decrypt, Hash, MAC, MAC Verify, Sign,
and Signature Verify. Eighteen return successful typed results. The Query test
records Cosmian's missing required Protection Storage Masks response field; the
Ping test records Cosmian's unsupported-operation rejection. Adjust Attribute
was probed separately and is also rejected as unsupported by the pinned
Cosmian 5.28.0 parser. Exact OASIS operation clauses and per-operation results
are recorded in `specs/015-cosmian-kms-integration/traceability.md`. Passing this selected
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

The PKI generator also keeps actual PEM and DER PKCS#10 requests for the
cryptographic Certify matrix under `.local/cosmian-kms/certs/validation/`.
It records whether the container's OpenSSL can generate ML-DSA and SLH-DSA
request fixtures in `cryptographic-fixture-capabilities.txt`; unavailable
algorithms remain explicit fixture limitations. The files are generated
inside the `test-pki` container, are ignored by Git, and must never be copied
into test output or reports. Regeneration uses the `fixture-set-v3` marker so
an older local fixture set cannot skip the new CSR generation.

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

The exhaustive Task 6 matrix has separate serial live runners so its outcomes
can be captured by family without starting the other ignored operations. After
`-Action up`, use these commands from the repository root (the RNG and crypto
families are deliberately run separately and with one test thread):

```powershell
$env:KMIPKIT_COSMIAN_CERT_DIR = (Resolve-Path .local/cosmian-kms/certs).Path
cargo test -p kmipkit-client --test cosmian_kms random::cosmian_528_random_matrix_no_parameter_cases -- --ignored --test-threads=1 --nocapture
cargo test -p kmipkit-client --test cosmian_kms cryptographic::cosmian_528_cryptographic_matrix_serial_live_runner -- --ignored --test-threads=1 --nocapture
```

The crypto inventory includes 1,466 standards-valid target rows: 1,220
`typed_success` expectations, six source gaps, and 240 source/documentation
discrepancies. Six standards-invalid Certify rows are excluded. The runner keeps
source-gap and discrepancy observations separate from interoperability; its
preflight can report exact probes or `NotSent` when a matching fixture is not
available. A finding assertion passing is never a crypto interoperability
success. Offline request-construction contracts run before the live families:

```powershell
cargo test --offline -p kmipkit-client --test cosmian_kms cryptographic::
cargo test --offline -p kmipkit-client --test cosmian_kms random::
```

Capture the case-ID/classification lines and sanitized error categories from
each explicit live run. Never record response bodies, TTLV payloads, seed data,
private keys, or generated CSR contents. Always run `-Action down` and verify
`-Action status` is empty after both families, including after a failed run.

Task 6 has a complete serial-suite run against the pinned image; its sanitized
evidence and limitations are recorded in
`.superpowers/sdd/2026-10-16-cosmian-exhaustive-matrix/task-6-report.md` and
`.local/cosmian-kms/task14-full-ignored-suite.log`. The suite ended with 98
tests passed and 10 failed. The transfer runner counted 2,668 verified
interoperability successes, 378 expected-finding observations, and 1,594
failures across 4,640 cases. Rejections, setup failures, malformed responses,
and finding observations are not interoperability successes. The six live RNG
rows received typed-response validation failures (Operation absent, Status 1,
Reason 4); the offline regression now classifies these as
`response_validation_failed`, with the request and response recorded and
`interoperability=false`. The full run preceded that formatter change, so it
was not rerun. The 50 transfer Key Format Type postcondition failures and the
other live failures remain open; Task 6 is not complete.

The extra cryptographic operations use server-generated and activated test
keys. MAC Data, Signature Data, and Validity Indicator use the OASIS KMIP v2.1
§11.56, Table 487 tags. The Cosmian run exposed and helped correct those three
KMIPKit tag assignments. Query's Table 283 response requires Protection
Storage Masks; Cosmian returned Success and its Operations list but omitted
that field, so KMIPKit rejects the typed payload. Ping is rejected by this
Cosmian 5.28.0 deployment as unsupported; its error Batch Item omits Operation.

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
On 2026-10-15 the expanded ignored suite reported 20 passed and 0 failed. A
final rerun after the helper/secret-buffer changes produced the same result.
This
means 18 operations returned successful typed responses and the Query/Ping
limitation probes observed the expected server behavior; it does not mean all
20 operations interoperated successfully. See
`specs/015-cosmian-kms-integration/traceability.md` for operation-by-operation
evidence. These results validate only the tested requests and do not establish
full KMIP 2.1 conformance or certification. The local deployment was stopped
after the final run; `-Action status` showed no running services.

## Stop the server

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action down
```

This removes the container and its disposable database. To discard generated
certificates as well, remove `.local/cosmian-kms/`; the next start regenerates
the PKI.
