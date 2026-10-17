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
has 6,429 records: 6,417 executable cases and 12 normative exclusions. Fifteen
cryptographic finding rows are explicitly marked `fixture_unavailable`; they
were not sent. A typed response on a finding row is not an interoperability
success.

## Prerequisites and local files

- Docker Engine/Desktop with Docker Compose.
- Rust 1.94 or later and the repository-pinned toolchain.

The PKI generator keeps certificates and private keys under ignored
`.local/cosmian-kms/certs/`. The server receives its own identity and the public
client CA; KMIPKit reads the local CA and client identity. Cosmian's SQLite
database is container-local (`clear_database = true`) with no mounted volume,
and the HTTP service is not published to the host.

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
remain visible; expected-finding assertions do not turn them into
interoperability successes. The run emitted 6,412 unique `case_id` results;
five standalone lifecycle tests have their observations recorded by test name
in the report and manifest. The six valid RNG responses are classified as
response-validation failures because the typed response omitted Operation;
none counts as an interoperability success. Compose teardown
completed, and `docker compose --file tests/integration/cosmian/compose.yaml
ps --all` returned no services.

| Test family | Cases | Typed successes | Interoperability successes | Other measured outcomes |
|---|---:|---:|---:|---|
| Lifecycle and probes | 89 | 80 | 80 | 3 rejected; 2 response-validation failures; 2 malformed responses; 1 setup-blocked; 1 unsupported |
| Cryptographic | 1,466 | 648 | 616 | 384 rejected; 331 setup-rejected; 88 response-validation failures; 15 fixture-unavailable |
| Key management | 216 | 12 | 12 | 201 rejected; 3 setup-rejected |
| Import/Export/Register transfer | 4,640 | 2,722 | 2,668 | 890 rejected; 978 setup-rejected; 50 response-validation failures |
| RNG | 6 | 0 | 0 | 6 malformed responses |
| **Total executable rows** | **6,417** | **3,462** | **3,376** | **All other rows remain non-success outcomes or explicit blockers** |

Twelve normative exclusions were not sent: six invalid RNG Parameters
combinations and six invalid Certify subject-OID-only CSR combinations. Of the
3,462 typed responses, 86 belong to source-gap/discrepancy finding rows and
therefore have `interoperability=false`. The matrix stores all per-case
classifications and safe reason codes.

Notable results include: Discover Versions passes; Check SplitKey passes with
the optional usage mask omitted; Query returns Success but omits the required
Protection Storage Masks field; Ping and Adjust Attribute return malformed
failure responses without Operation; Re-certify is rejected as unsupported;
and all six valid RNG cases return responses missing Operation. Fifty PrivateKey
Get/Export checks for `Cosmian-Pkcs12Legacy` fail the typed Key Format Type
postcondition. The password-linked PFX import setup is separately recorded as
rejected before dependent Get/Export requests are sent. See
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
