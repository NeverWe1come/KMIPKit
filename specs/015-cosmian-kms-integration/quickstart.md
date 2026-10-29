# Quickstart: Local Cosmian KMS Integration Tests

This quickstart deploys Cosmian KMS 5.28.0 locally and runs KMIPKit's opt-in,
typed integration suite. Its exhaustive manifest contains 6,429 rows across
all 35 advertised operation paths and applicable object, format, encoding,
algorithm, and parameter combinations. It records 3,728 successful typed
interoperability outcomes; remaining results include target rejections, setup
failures, malformed or invalid responses, and explicit fixture blockers. This
does not claim complete KMIP 2.1 conformance or formal certification.

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
The generator initializes an empty certificate directory with a regular
KMIPKit ownership marker and rejects non-empty unmarked directories and
symlinked cleanup paths; use a new empty directory for custom
`KMIPKIT_COSMIAN_CERT_DIR` paths.

## Run the live test

The test is deliberately ignored by default. With the local server running,
run the live test explicitly:

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action test
```

The full ignored target covers lifecycle, cryptographic, random-number,
key-management, transfer, and separate operation-probe cases. The wrapper's
`-Action test` runs the ignored suite serially and tears down Compose in a
`finally` block, even when Cargo returns a failure. To run Cargo directly,
start the server first and stop it with `-Action down` in a PowerShell
`finally` block.

```powershell
cargo test -p kmipkit-client --test cosmian_kms -- --ignored --test-threads=1 --nocapture
```

The matrix's family runners can also be selected by their exact test names;
commands and safe-output rules are listed in the [integration README](../../tests/integration/cosmian/README.md).

To run one operation test, add its exact Rust test name before `-- --ignored --exact --test-threads=1`, for example:

```powershell
cargo test -p kmipkit-client --test cosmian_kms cosmian_kms_accepts_kmip_2_1_discover_versions_over_mutual_tls -- --ignored --exact --test-threads=1
```

The operation payloads follow OASIS KMIP Specification v2.1 §6.1.16, Tables
211–213; §6.1.8, Tables 186–188; §6.1.9, Tables 189–192; §6.1.10, Tables
193–195; §6.1.2, Tables 167–169; §6.1.13, Tables 202–204; §6.1.20, Tables
223–225; §6.1.21, Tables 226–228; §6.1.34, Tables 265–267; §6.1.51, Tables
322–324; §6.1.1, Tables 163–166; §6.1.40, Tables 282–284; §6.1.36, Tables
271–272; §6.1.17, Tables 214–216; §6.1.11, Tables 196–198; §6.1.24, Tables
235–237; §6.1.32, Tables 259–261; §6.1.33, Tables 262–264; §6.1.55, Tables
334–336; and §6.1.56, Tables 337–339. Cryptographic data tags follow §11.56,
Table 487. Discover Versions must return success and include version `(2, 1)`
as encoded under §9.16, Table 421. Eighteen tests assert successful typed
results. Query and Ping assert the observed Cosmian limitations: Query omits
Protection Storage Masks required by Table 283, and Cosmian rejects Ping as
unsupported. The tests create server-generated objects and test metadata in
the container-local database; the database is discarded when the container is
removed.

## Exhaustive matrix result

The pinned local image was exercised with the complete ignored suite. The
manifest contains 6,429 records: 6,417 executable cases and 12 non-executable
normative exclusions. All executable rows have a measured outcome or an
explicit fixture/setup blocker; 3,728 rows passed typed-response and
operation-specific checks and count as interoperability successes. This
includes 24 successful Create Key Pair source-discrepancy probes; their
algorithm values remain outside the KMIP 2.1 enumeration. The latest
serial suite ended with 101 passed and 9 failed test functions, so the report
preserves server rejections and KMIPKit response-validation failures rather
than calling the run green.
The seven multipart Hash cases are separately classified as semantic
mismatches: KMIPKit accepted typed Success, but the server's final digest did
not match the independent `abc` known-answer vector. The three SHA-224 requests
remain target rejections despite Cosmian's published algorithm list; pinned
5.28.0 source omits SHA-224 in Hash dispatch.

The 446 previously non-interoperable Register rows were rerun with the typed client: 154 now passed their Register postconditions, 286 failed during prerequisite setup, and six reached Register but were rejected. See the [Register retest report](task-6-report.md#register-retest-and-attribution) for the full attribution.

| Family | Cases | Interoperability successes | Main non-success observations |
|---|---:|---:|---|
| Lifecycle and probes | 89 | 80 | 3 rejected; Query/Locate validation failures; Ping/Adjust malformed responses; Activate setup blocked; Re-certify unsupported |
| Cryptographic | 1,466 | 650 | 383 rejected; 325 setup rejected; 81 response validation failed; 7 multipart semantic mismatches; 20 typed finding responses |
| Key management | 216 | 12 | 201 rejected; 3 setup rejected |
| Import/Export/Register transfer | 4,640 | 2,986 | 778 rejected; 876 setup rejected; 0 response validation failed |
| RNG | 6 | 0 | 6 response-validation failures (missing Operation) |

Encrypt records 287/300 interoperable cases. Three AES-ECB Encrypt failures were test-fixture defects: ECB has no default padding, and the old 19-byte input was not block-aligned. All three now pass with a 16-byte input. Six ConfigurableKEM opaque-fixture results and three ML-KEM source-only results are typed successes outside the KMIP 2.1 interoperability count. Two RSA/AES requests were rejected with Operation Failed / Operation Not Supported (Status 1, Reason 5); two pure-classical ConfigurableKEM cases fail at key setup with Cryptographic Failure before Encrypt is sent.

The Decrypt subset records 222/234 interoperable cases. Three AES-ECB and one
CoverCrypt case passed after correcting KMIPKit test fixtures/postconditions.
Two pure-classical ConfigurableKEM rows fail at prerequisite Create Key Pair,
before Decrypt is sent; one RSA/AES probe is rejected as Operation Not
Supported. See the [case re-evaluation](task-6-report.md#decrypt-case-re-evaluation)
for root-cause evidence and limits.

The 12 exclusions are six standards-invalid RNG Parameters combinations and
six standards-invalid Certify request combinations. Matrix and summary details
are recorded in [traceability](traceability.md) and the
[Task 6 report](task-6-report.md).

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
supported-operation test target. On 2026-10-15, the expanded 20-case target
passed 20/20: 18 successful typed exchanges and two expected-limitation probes
for Query and Ping. The integration also exposed KMIPKit's incorrect MAC Data,
Signature Data, and Validity Indicator tags; these were corrected under
KMIPKIT-0021 using §11.56, Table 487. See
`specs/015-cosmian-kms-integration/traceability.md` for operation-level
results. A final run after the test-helper and `SecretBytes` changes again
passed 20/20. Cosmian and PKI containers were then removed with `-Action down`,
and `-Action status` showed no running services. These selected operations are
not a broad conformance claim. The final local workspace coverage was 92.92%
line coverage (`kmipkit-protocol` 96.17%; `kmipkit-ttlv` 98.29%). Hosted CI run
`38809840838` passed the core, bindings, scripts, traceability, inventory,
sanitizer, and coverage jobs. Its dependency-policy gate failed on newly issued
RustSec advisories for the existing `rustls 0.23.45`; see
`traceability.md`. Keep the draft PR open until that separate dependency issue
is resolved under an approved scope.

## Stop and remove the server

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action down
```

The database is container-local and is discarded when the container is
removed. Delete `.local/cosmian-kms/` separately if you also want to discard
the generated certificate files.
