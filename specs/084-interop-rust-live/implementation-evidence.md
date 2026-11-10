# Implementation evidence

The maintainer implementation plan authorizes task 1; KMIPKIT-0059's mock-only
scope is superseded for this consumer.

## Signed phases

- Red `233b00577`: JSON plan, missing preflight, and mock success without TLS.
- Green `6906b945b`: installed client and shared CLI/result contract.
- Red `954a0638d`: exhaustive validated synchronous builders expose omissions.
- Red `cb16271d5`: controlled mTLS/incoming regression fixture.
- Green `fa74578c6`: catalog recipes, incoming dispatcher, verified artifact and
  Rust 1.94 clean CI workflow. Four Rust and nine Python integration tests pass.
- Separate Refactor commit: remove obsolete uncompiled matcher/isolation,
  document usage, traceability and exact evidence limitations.

The reproducible commands are in `interop/rust-kmip-validator/README.md`.
The locally measured bundle SHA-256 is
`b03bb7e3e374faf775e57415fcab8c2add5da160573b266dc05c56b92e664f44`.
Rebuilds may differ in archive metadata/hash; each run verifies and embeds the
actual consumed bundle, always from the exact pinned release commit.

The controlled 2.1 report contains all 207 IDs: 107 passed, 92 observed Operation
Not Supported responses, and 8 non-run. These eight are five unconfigured
incoming channels, two 2.0-only probes and the State preflight rejection.
Dedicated tests execute all five incoming operations and both 2.0 boundaries.
The full-catalog test requires no unexplained prerequisite gap and no failed
case. Constructor tests validate all synchronous request shapes. Native Query
checks cover two versions, three HTTPS encodings and two API routes; a separate
request test compares deterministic payloads to hand-derived expected values.

These are offline library/consumer and controlled-fixture observations. There
is no live vendor, profile certification, FIPS or cryptographic-conformance
claim. Cross-language parity remains not_compared.

The frozen schema has scalar execution axes, while the shared aggregator
requires equality on transport/encoding/API and on the exact connection-file
fingerprint. Cross-axis parity therefore currently requires independent tests;
this branch changes no shared contract file.

State is not a mutation-policy skip: the public typed constructor exists, but
KMIPKIT-0051-R282 / OASIS KMIP 2.1 §4.57 rejects direct mutation before sending.
The server-negative row stays not_run with library passed and server unknown.
Outbound generic TTLV, raw-TLS text reception and authenticated async follow-up
limitations remain explicitly distinguished from server incompatibility.

## Independent review corrections

Controlled tests now run the complete catalog in both 2.0 and 2.1 and incoming
operations in both versions. Eleven Python tests additionally prove that
Authentication Not Successful is failed/unknown; Feature Not Supported is
unsupported; malformed server input does not automatically blame the library;
and Encrypt-on-pre-active accepts only catalog reasons 7/10 and rejects 0x43.
The 2.0 inventory has 106 passed, 86 unsupported and 15 non-run rows. Four extra
non-applicabilities follow OASIS 2.0 §§6.1/11.36 (Ping, Process, Query Async,
Set Defaults). Three additional local API failures are the pinned VersionGate's
incorrect Ticket/Rights tag restriction for Login/Logout/Delegated Login, which
are defined in OASIS 2.0 §§6.1.12, 6.1.29, 6.1.30, 7.29 and 7.33. These record
library correctness failed and server compatibility unknown. No shared input
or pinned library source is modified.

## Full API matrix and coverage gate

Both 207-row inventories now execute through **both** high-level and typed APIs
and compare normalized outcomes per version. Twelve integration tests include
Markdown identity/outcome/redaction checks. `tools/check_coverage.py` instruments
an isolated target, runs Rust tests plus the real subprocess/controlled-peer
suite, and enforces >=90 percent line coverage. A fresh run measures 90.41%
(1819/2012 lines). The sole exclusion is the generated local-case table; its
regeneration and all 67 model cases are tested. CI pins cargo-llvm-cov 0.9.1 and
the Rust 1.94 llvm-tools component and runs this same script.

## Complete structural request/response parity

Review exposed an outcome-only comparison. Signed Red `2335a8c05` fails on
changed deterministic requests and omitted KMIP Status/Reason; Green `54ae862c1`
compares the peer's actual request trees and canonical numeric result codes.
Red `f72799703` exposes omitted response payloads; Green `2e7654f89` additionally
compares every response tree. The 19-test Python suite passes (12 integration,
seven focused parity/privacy tests). Both catalog versions and API routes
compare complete ordered exchange sequences, including setup and teardown.

Only deterministic structure and public numeric/boolean values are compared.
Time Stamp is omitted. Resource identifiers, names and batch IDs become stable
aliases preserving relationships; Credential Value, Key Material, Attribute
Value, remaining text, byte strings and big integers are redacted. Secret and
cryptographic output values are outside this structural comparison. Normalized
trees and raw alias keys remain ephemeral memory and are never written to
reports/artifacts or displayed by the matrix assertion. Incoming operations
retain their separate two-version listener verification. These comparisons do
not establish live vendor compatibility or cross-language parity.

## Revalidation against the merged KMIP 2.0 session fix

After KMIPKIT-0089 merged into `release/1.0.0`, the independent Rust consumer
was rebuilt from release commit
`3017cfda4dbe1ccdf94c827e318e57688f858165`. The installed artifact SHA-256 was
`8cb649af459ea7a37d8ecbe4da3cfe65686a324e8719f3facfaa446c2e31b123`; the
installer and verifier both confirmed this exact provenance. A clean Cargo
rebuild ensured no artifacts from the previous release pin were reused.

The controlled 207-row catalog now passes for KMIP 2.0 and 2.1 through both
high-level and typed-protocol APIs. Login, Logout and Delegated Login succeed
in the 2.0 scenario, including Login/Logout setup and teardown. The earlier
2.0 non-run classification documented above applied to the pre-fix release pin
and is superseded by this run. No deployed KMS was involved.

Verification on the clean rebuild: Rust formatting, Clippy with warnings
denied, one adapter unit test, three CLI tests, the 19-test controlled Python
suite, artifact verification and `git diff --check` pass. The isolated
coverage gate measures 90.37% (1812/2005 handwritten consumer lines), above
the 90% requirement.
