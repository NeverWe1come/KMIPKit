# Rust interoperability consumer

The independent Rust 1.94 workspace consumes installed `.crate` packages built
from release commit `fdafef8a3eb0c7b4b18d6433b78fde4151493ceb`, targeting KMIP
2.0 and 2.1. No deployed vendor KMS or vendor compatibility claim is included.

From the repository root:

```sh
python3 interop/rust-kmip-validator/tools/install_artifact.py
python3 interop/rust-kmip-validator/tools/verify_artifact.py
cargo +1.94 build --locked --manifest-path interop/rust-kmip-validator/Cargo.toml
```

The installer archives the exact Git tree, packages five public crates and
bundles their `.crate` distributions, required release documentation and pinned
aws-lc-sys patch. Cargo links extracted packages under ignored
`interop/.local/rust-live/install/`, with no dependency on `../../crates/`.
The build verifies release identity, bundle SHA-256, complete installed file
inventory and every installed byte. Reports embed the consumed artifact hash.
Python 3.12 or newer is needed for packaging and build-time verification.

```sh
interop/rust-kmip-validator/target/debug/rust-kmip-validator plan --capabilities /path/profile.json
interop/rust-kmip-validator/target/debug/rust-kmip-validator preflight --connection /path/connection.json
interop/rust-kmip-validator/target/debug/rust-kmip-validator run --connection /path/connection.json --capabilities /path/profile.json --output /path/results.json --markdown /path/results.md
```

Planning and preflight are offline. Relative TLS file paths resolve from the
invoking working directory. Authentication reads shared environment references;
values never enter reports or diagnostics. A run requires a confirmed dedicated
test environment. Catalog mutations, session operations and Log additionally
require `execution_policy.allow_mutations=true`. Profiles never suppress probes,
including when `probe_unsupported=false`.

One invocation emits one scalar protocol/transport/encoding/API tuple. Repeat
runs with separate configurations for a matrix. The policy encoding/API lists
are planning metadata, not implicit additional runs. The frozen shared
aggregator requires identical transport, encoding, API level and fingerprint,
so cross-axis comparison currently uses the independent offline tests. The
fingerprint hashes the exact connection JSON bytes, including execution axes.

Schema-2.0 reports distinguish library correctness, server compatibility,
policy, parity and non-run. They contain no Result Message, KMIP body, key
material, ticket or correlation bytes. Exit 1 means local failure or a failed
case; exit 0 means report completion without failed cases, not that every case
ran or passed. Cross-language parity stays `not_compared` in this branch.

For server-to-client cases, configure the peer to call the socket selected by:

```sh
--listen 127.0.0.1:0 --listener-cert /path/cert.pem --listener-key /path/key.pem --listener-ready-file /path/address.txt
```

The readiness file contains only a socket address. The listener certificate
needs server-authentication EKU and the peer needs a client certificate trusted
by the configured CA. Public `ClientListener`/`ServerRequestDispatcher` perform
reception and dispatch. Without an incoming channel those cases are non-run.
The pinned raw-TLS listener hardcodes TTLV; JSON/XML reception on that path is
an explicit API limitation. HTTPS uses the public listener's encoding path.

Recipes use isolated names and identifiers, explicit cryptographic parameters,
server-provided tickets/correlations, distinct batch IDs and ordered steps.
Poll, Process, Cancel, multipart continuation and teardown are explicit recipe
steps. Failed requests are never retried; cleanup failure is visible.

Generic TTLV construction/navigation and bounded decoding run locally. Generic
outbound KMIP 2.0/2.1 submission is unavailable under KMIPKIT-0083-FR-007.
The State server-negative case is different: its typed constructor exists, but
preflight rejects direct State modification under KMIPKIT-0051-R282 and OASIS
KMIP 2.1 §4.57, independently of mutation opt-in. Its status remains `not_run`,
library correctness passes, and server compatibility stays unknown.

The facade omits `WireEncoding`/listener config exports, so this consumer also
links the installed public transport package. Public async follow-up methods
cannot attach Request Header Authentication; authenticated async recipes report
that explicit API limitation. No pinned library source is changed here.

```sh
cargo +1.94 fmt --manifest-path interop/rust-kmip-validator/Cargo.toml --check
cargo +1.94 clippy --locked --manifest-path interop/rust-kmip-validator/Cargo.toml --all-targets --all-features -- -D warnings
cargo +1.94 test --locked --manifest-path interop/rust-kmip-validator/Cargo.toml --all-features
python3 interop/rust-kmip-validator/tools/build_test_server.py
python3 -m unittest discover -s interop/rust-kmip-validator/tests -p test_live.py -v
```

The fixture suite needs `jsonschema==4.19.2`; CI uses an isolated virtual
environment. The local-server harness uses its public async shutdown receiver
because the stock CLI joins stdin on a current-thread runtime. No release
source is modified. Controlled fixtures establish consumer behavior and request
reachability, not vendor or cryptographic certification.

`KMIPKIT_EVIDENCE_DIR` saves a redacted 207-row report. The 2.1 scenario has 107
passes, 92 observed unsupported rejections and 8 non-run rows: five incoming
channels omitted in that scenario, two 2.0-only probes and the State rejection.
Separate tests execute all five incoming operations and both version probes.
CI uploads only redacted results and package provenance.

The full controlled inventory and all five incoming operations are verified in
both 2.0 and 2.1. Twelve Python integration tests cover the matrix and rejection
classifications. In 2.0, Ping, Process, Query Asynchronous Requests and Set
Defaults are non-applicable under the exact 2.0 §§6.1/11.36 inventories, despite
the shared catalog's broader version lists. Login, Logout and Delegated Login
are present in 2.0, but the pinned library incorrectly rejects their Ticket/
Rights tags as newer-version values. Those three rows are explicitly non-run
with library correctness failed and server compatibility unknown; this branch
does not change the package or mislabel that result as server incompatibility.
The 2.0 report has 106 passed, 86 unsupported and 15 non-run rows; 2.1 retains
the 107/92/8 split. Authentication failure is failed/unknown, Feature Not
Supported is unsupported, and a malformed server response is failed with
library correctness not_checked. The pre-active Encrypt negative accepts only
Invalid Field (7) or Cryptographic Failure (10), as specified by its catalog.

The complete catalog is also executed through both high-level and typed APIs
in each version, comparing normalized per-case outcomes. Markdown is checked
for the same case/outcome and for server-text redaction. Reproduce the 90 percent
line-coverage gate with Rust 1.94 llvm-tools, cargo-llvm-cov 0.9.1 and:

```sh
python3 interop/rust-kmip-validator/tools/check_coverage.py
```

The script isolates compilation/profiles, runs the Rust and controlled subprocess
suites, and fails below 90 percent. Only `local_cases_generated.rs` is excluded:
it is a generated data-dispatch table verified by regeneration and all 67 local
model cases. Fresh measured handwritten coverage is 90.41% (1819/2012 lines).
