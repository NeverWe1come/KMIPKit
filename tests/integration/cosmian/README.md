# Local Cosmian KMS integration tests

This deployment runs Cosmian KMS `5.28.0` locally and exercises KMIPKit's
production Rust client against its binary TTLV KMIP socket on
`127.0.0.1:5696`. The image is pinned by digest. Docker Compose generates a
local CA and short-lived mTLS certificates; the tests do not contact the public
demo service or require a Cosmian account.

The versioned capability ledger is
[`capability-matrix.json`](capability-matrix.json). It covers the 35 operation
paths in Cosmian's 34 published support rows (Set and Modify are separate),
plus separate Ping, Adjust Attribute and Re-certify probes. It crosses
valid operation/object/format/encoding/algorithm/parameter dimensions, follows
pinned source restrictions, and records each case independently. The ledger
has 6,429 records: 6,417 executable cases and 12 normative exclusions. A targeted follow-up sent all 15 previously fixture-blocked cryptographic findings: 12 ConfigurableKEM probes completed typed postconditions and three AES/RSA hybrid probes received typed rejections. These findings are not counted as KMIP 2.1 interoperability successes.

## Prerequisites and local files

- Docker Engine/Desktop with Docker Compose.
- Rust 1.94 or later and the repository-pinned toolchain.

The PKI generator keeps certificates and private keys under ignored
`.local/cosmian-kms/certs/`. The server receives its own identity and the public
client CA; KMIPKit reads the local CA and client identity. Cosmian's SQLite
database is container-local (`clear_database = true`) with no mounted volume,
and the HTTP service is not published to the host.

Before regenerating, the PKI script verifies its ownership marker. It accepts a
new empty output directory or a directory previously initialized by KMIPKit,
and refuses to clean a non-empty unmarked directory. This protects existing
files when `KMIPKIT_COSMIAN_CERT_DIR` points at an unexpected location.

The PKI generator also creates PEM and DER PKCS#10 fixtures for Certify under
`.local/cosmian-kms/certs/validation/`. It records whether the container
OpenSSL can create ML-DSA and SLH-DSA requests. Unavailable algorithms remain
explicit fixture limitations. Generated keys, certificates, CSR content, and
KMIP payloads must not be copied into test output or reports.

## Run the full live suite

From the repository root in PowerShell:

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action up
.\scripts\integration\cosmian-kms.ps1 -Action test
.\scripts\integration\cosmian-kms.ps1 -Action status
```

`-Action test` runs every ignored test serially and tears down Compose in a
`finally` block, including when Cargo fails. It verifies that Compose has no
remaining services. To run the equivalent Cargo command directly, start the
server first and preserve cleanup explicitly:

```powershell
try {
    $env:KMIPKIT_COSMIAN_CERT_DIR = (Resolve-Path .local/cosmian-kms/certs).Path
    cargo test -p kmipkit-client --test cosmian_kms -- --ignored --test-threads=1 --nocapture
}
finally {
    .\scripts\integration\cosmian-kms.ps1 -Action down
    .\scripts\integration\cosmian-kms.ps1 -Action status
}
```

The live tests use typed `ClientRequest` values over KMIPKit's raw TTLV TLS
transport with mutual TLS. They do not construct TTLV by hand, use another
wire client, retry requests, or use production key material. Each case owns its
fixtures and object identifiers, tracks linked objects, and records sanitized
status/reason/delivery fields. No raw response bodies, TTLV bytes, generated
private keys, passwords, RNG output, or seed data are retained.

## Run one family

Use `-Action up` before a direct family command and `-Action down` in a
PowerShell `finally` block afterward. The full wrapper above is preferred when
you want every family. These commands correspond to the exhaustive runners:

```powershell
$env:KMIPKIT_COSMIAN_CERT_DIR = (Resolve-Path .local/cosmian-kms/certs).Path
cargo test -p kmipkit-client --test cosmian_kms lifecycle:: -- --ignored --test-threads=1 --nocapture
cargo test -p kmipkit-client --test cosmian_kms key_management::cosmian_528_key_management_matrix -- --ignored --exact --test-threads=1 --nocapture
cargo test -p kmipkit-client --test cosmian_kms random::cosmian_528_random_matrix_no_parameter_cases -- --ignored --exact --test-threads=1 --nocapture
cargo test -p kmipkit-client --test cosmian_kms cryptographic::cosmian_528_cryptographic_matrix_serial_live_runner -- --ignored --exact --test-threads=1 --nocapture
cargo test -p kmipkit-client --test cosmian_kms transfer::cosmian_528_transfer_matrix -- --ignored --exact --test-threads=1 --nocapture
```

For a single family, use the same cleanup guarantee around the command:

```powershell
try {
    $env:KMIPKIT_COSMIAN_CERT_DIR = (Resolve-Path .local/cosmian-kms/certs).Path
    cargo test -p kmipkit-client --test cosmian_kms cryptographic::cosmian_528_cryptographic_matrix_serial_live_runner -- --ignored --exact --test-threads=1 --nocapture
}
finally {
    .\scripts\integration\cosmian-kms.ps1 -Action down
    .\scripts\integration\cosmian-kms.ps1 -Action status
}
```

For a single ignored case, add its exact test name before `-- --ignored
--exact --test-threads=1`. Offline inventory and parity checks run without a
server and are not ignored:

```powershell
cargo test --offline -p kmipkit-client --test cosmian_kms
```

The offline suite does not require the ignored runtime certificate directory.
Without the generated validation CSRs, the 30 post-quantum Certify finding
probes remain `NotSent`; with the full local fixture set, they are eligible for
exact request construction. A partial PQC CSR fixture set is rejected by the
inventory test.

## Measured result and limitations

The latest complete serial run against the pinned
`ghcr.io/cosmian/kms:5.28.0` image
(`sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`)
used rustls 0.23.46 and ended with 101 passed and 9 failed test functions after
6,383.81 seconds. The sanitized log is an ignored local artifact at
`.local/cosmian-kms/final-rustls-0.23.46-full-suite.log` (SHA-256
`84947A36B3557C22402DA12F918898F1743F913E4153A4A025120A7E4B2C0051`). The
nine failing functions are the crypto, key-management and transfer matrix
runners; six specific lifecycle cases; and the RNG runner. Their failures
remain visible. An expected-finding assertion alone does not count as success;
a finding probe that returns a typed success and satisfies its operation-specific
postconditions can count as observed interoperability. The run emitted 6,412 unique `case_id` results;
five standalone lifecycle tests have their observations recorded by test name
in the report and manifest. The six valid RNG responses are classified as
response-validation failures because the typed response omitted Operation;
none counts as an interoperability success. Compose teardown
completed, and `docker compose --file tests/integration/cosmian/compose.yaml
ps --all` returned no services.

The targeted Hash follow-up accepts typed multipart responses and sends a, b, and c as explicit Data
blocks while reusing the first response's Correlation Value. OASIS KMIP v2.1 §6.1 defines multipart
sequencing, §7.8 defines Correlation Value reuse, and Hash §6.1.24, Table 235 marks Data as not
required for multipart Hash. The general §6.1 text says Data is always required except when Init or
Final Indicator is true, so this project makes no conformance claim for the disputed omission of
Data from an intermediate Hash request. This test supplies explicit inputs on all three parts.
Seven server responses are typed Success but their final
digests do not match the independent abc vectors; three SHA-224 requests are rejected although
Cosmian's published algorithm list advertises SHA-224. Hash remains 14/24 interoperable cases.

| Test family | Cases | Typed successes | Interoperability successes | Other measured outcomes |
|---|---:|---:|---:|---|
| Lifecycle and probes | 89 | 80 | 80 | 3 rejected; 2 response-validation failures; 2 malformed responses; 1 setup-blocked; 1 unsupported |
| Cryptographic | 1,466 | 670 | 650 | 383 rejected; 325 setup-rejected; 81 response-validation failures; 7 multipart semantic mismatches |
| Key management | 216 | 12 | 12 | 201 rejected; 3 setup-rejected |
| Import/Export/Register transfer | 4,640 | 2,986 | 2,986 | 778 rejected; 876 setup-rejected; 0 response-validation failures |
| RNG | 6 | 0 | 0 | 6 response-validation failures (missing Operation) |
| **Total executable rows** | **6,417** | **3,748** | **3,728** | **1,365 rejected; 1,204 setup-rejected; 89 response-validation failures; 7 semantic mismatches; 4 other findings** |

Twelve normative exclusions were not sent: six invalid RNG Parameters
combinations and six invalid Certify subject-OID-only CSR combinations. Of the
3,748 typed responses, 20 cryptographic source-gap/discrepancy observations
remain outside the interoperability count. A live follow-up reran 52 native SymmetricKey Import cases after correcting the Symmetric Key and transparent Key TTLV tags in the test builder; all 52 returned typed success. The 54 transfer finding rows that
returned typed success and passed operation postconditions now count as observed
interoperability; their documentation/source classifications remain attached,
and this is not a KMIP 2.1 conformance claim. The 24 Create Key
Pair source-discrepancy probes returned typed success with both key identifiers
validated, so they count as observed KMIPKit–Cosmian interoperability; their
algorithm values remain outside the pinned KMIP 2.1 enumeration and are not a
KMIP conformance claim. A targeted run measured all 15 cipher finding rows:
12 ConfigurableKEM responses passed their typed postconditions and three
RSA/AES requests were rejected; these rows are included in the aggregate above
and remain outside the interoperability count. Decrypt now has 222/234
interoperable cases: three AES-ECB fixture corrections and one CoverCrypt
framing correction passed; two pure-classical ConfigurableKEM cases fail during
prerequisite Create Key Pair, and one RSA/AES probe is rejected as Operation Not
Supported. Its ordinary RSA-OAEP payload is not a valid hybrid ciphertext. The
three corresponding standalone AES-ECB Encrypt cases also pass after the same
16-byte fixture correction; Encrypt now records 287/300 interoperability
successes. Six ConfigurableKEM opaque-fixture results and three ML-KEM source-only
results are typed successes outside the KMIP 2.1 interoperability count. Two RSA/AES
requests were rejected with Operation Failed / Operation Not Supported (Status 1,
Reason 5); two pure-classical ConfigurableKEM cases fail at key creation with
Cryptographic Failure before Encrypt is sent. The matrix stores all per-case
classifications and safe reason codes. See the
[Decrypt case re-evaluation](../../../specs/015-cosmian-kms-integration/task-6-report.md#decrypt-case-re-evaluation)
for evidence and attribution.

### Register retest

The 446-case Register retest added 154 successful target outcomes: 52 former
`Invalid Message` failures came from incorrect SymmetricKey/Key TTLV tags in the
integration fixture and passed after correction; 102 cases previously blocked
by setup `Cryptographic Failure` also passed. Register now records 737/1,029
interoperable cases. The remaining 286 cases fail during prerequisite setup
(165 `Operation Not Supported`, 115 `Cryptographic Failure`, six `Sensitive`);
six more reach Register and receive four `Cryptographic Failure` and two
`Operation Not Supported` responses. For the six Sensitive setup cases, typed
Get Attributes reads back `Sensitive=False`, but unwrapped Export still rejects
the key. The full per-case breakdown is in the
[Register retest attribution report](../../../specs/015-cosmian-kms-integration/task-6-report.md#register-retest-and-attribution).

| Operation | Interoperable / executable | Target rejections | Setup rejections | Target rejection breakdown |
|---|---:|---:|---:|---|
| Get | **797 / 1,279** | 382 | 100 | 160 `Operation Not Supported`; 8 `Sensitive`; 214 `Cryptographic Failure` |
| Export | **807 / 1,291** | 384 | 100 | 160 `Operation Not Supported`; 8 `Sensitive`; 216 `Cryptographic Failure` |

A focused run selected 66 cases and confirmed the legacy-PKCS#12 selector fix:
the test sends Cosmian's `KeyFormatType=0x88800001`; 62 target requests
returned typed success, including all 58 cases that newly send the selector.
Four other selected cases did not reach Get/Export because password-linked PFX
Import setup was rejected. The detailed [Get/Export attribution tables in
KMIPKIT-0015](../../../specs/015-cosmian-kms-integration/traceability.md#get-and-export-attribution)
classify the rejections against KMIP 2.1 and Cosmian 5.28.0 documentation and
source.

Notable results include: Discover Versions passes; Check SplitKey passes with
the optional usage mask omitted; Query returns Success but omits the required
Protection Storage Masks field; Ping and Adjust Attribute return malformed
failure responses without Operation; Re-certify is rejected as unsupported;
and all six valid RNG cases return responses missing Operation. Password-linked
PFX setup failures are recorded before dependent Get/Export requests are sent.
See
[`traceability.md`](../../../specs/015-cosmian-kms-integration/traceability.md)
and the [Task 6 evidence report](../../../specs/015-cosmian-kms-integration/task-6-report.md)
for source references and per-case interpretation.

These observations test selected request contracts against one pinned product
configuration. They do not establish complete KMIP 2.1 conformance or formal
certification.

## Stop the server

```powershell
.\scripts\integration\cosmian-kms.ps1 -Action down
.\scripts\integration\cosmian-kms.ps1 -Action status
```

This removes the containers and disposable database. Generated local
certificates and fixtures remain ignored under `.local/cosmian-kms/`; remove
that directory separately if you want to discard them.
