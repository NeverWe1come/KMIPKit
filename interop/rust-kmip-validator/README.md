# Rust interoperability consumer

The independent Rust 1.94 workspace consumes installed `.crate` packages built
from release commit `9a16cc90a12205101d22a38501a5e74c665df4dc`, targeting KMIP
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

`KMIPKIT_EVIDENCE_DIR` saves a redacted report with one row for every selected
catalog case. The shared catalog currently contains 2,689 cases: 2,507 local
model checks and 182 server exchanges. Counts come from the selected catalog
and protocol version so catalog growth cannot silently reduce coverage.
Separate tests execute all five incoming operations and every KMIP 2.0 version
boundary probe. CI uploads only redacted results and package provenance.

The full controlled inventory and all five incoming operations are verified in
both 2.0 and 2.1. The current package pin includes the KMIP 2.0 session-tag
correction merged in `3017cfda4dbe1ccdf94c827e318e57688f858165`. Login, Logout
and Delegated Login run through the controlled peer in 2.0 as well as 2.1;
older evidence from the pre-fix pin is superseded by the current controlled
run. Ping, Process, Query Asynchronous Requests and Set Defaults remain
non-applicable under the exact 2.0 §§6.1/11.36 inventories, despite the shared
catalog's broader version lists. Authentication failure is failed/unknown,
Feature Not
Supported is unsupported, and a malformed server response is failed with
library correctness not_checked. The pre-active Encrypt negative accepts only
Invalid Field (7) or Cryptographic Failure (10), as specified by its catalog.

The complete catalog is also executed through both high-level and typed APIs
in each version, comparing case ID, classification, library/server outcome and
canonical numeric KMIP Result Status/Reason. The controlled peer also captures
and compares the ordered normalized TTLV trees for every outgoing request and
response, including setup, target and cleanup exchanges. Tags, types and public
numeric/boolean scalars remain comparable. Timestamps are omitted; identifiers,
names and batch IDs use stable aliases preserving resource relationships.
Credential Value, Key Material and Attribute Value subtrees, and all remaining
text, byte strings and big integers, are redacted. Thus secret values and
cryptographic output bytes are outside this structural parity check. Trees and
alias keys exist only in test-process memory and never enter reports, artifacts
or failure diagnostics. The five incoming operations have separate two-version
listener tests rather than a high-level/typed outgoing comparison. Seven unit
regressions detect changed requests, responses or KMIP codes and check redaction
and resource-reference preservation. Markdown is checked for the same
case/outcome and for server-text redaction. Reproduce the 90 percent
line-coverage gate with Rust 1.94 llvm-tools, cargo-llvm-cov 0.9.1 and:

```sh
python3 interop/rust-kmip-validator/tools/check_coverage.py
```

The script isolates compilation/profiles, runs the Rust and controlled subprocess
suites, and fails below 90 percent. The generated `local_cases_generated.rs`
data table and `route_calls_generated.rs` route-dispatch table are excluded from
the raw line count. CI checks that both files regenerate without a diff;
local-model cases and each implemented route record actual invocation evidence.
Routes that remain pending are reported separately and are not counted as
passed coverage. Handwritten-code coverage is measured from the current build
rather than a fixed catalog count.

## KMIPKIT-0092 public route matrix (WIP)

The `matrix-local` command consumes the shared generator's Rust local JSONL
rows. At build time the consumer hashes every canonical Rust row; a modified
case, target, symbol, action, protocol version, encoding, transport, artifact
commit, or execution route is rejected before a call can be recorded. Duplicate
rows are rejected too. Artifact identity is compiled from the independently
installed immutable release packages, whose bytes are checked by `build.rs`.

Implemented model recipes invoke the exact public constructor or accessor
named by the row. They frame TTLV through the public scoped value views, invoke
the installed public decoder, re-encode, and compare complete values in memory.
The package has no public generic encoder; consumer framing is not advertised
as an additional library encoder route. Byte buffers are zeroizing and reports
contain only redacted normalized-tree hashes, never raw bodies or values.
Constructor output must also match its target fixture before it can pass.
A recipe whose prerequisites cannot be constructed emits no invocation event.
An unfinished recipe is pending coverage and cannot become an API limitation.

The following command generates canonical local rows, runs the installed Rust
consumer, validates its event schema, and applies the shared completeness gate:

```sh
interop/.local/rust-ci-venv/bin/python interop/rust-kmip-validator/tools/run_route_matrix.py
```

It writes `rust-local-route-evidence.json` and `matrix-summary.json` under
`interop/rust-kmip-validator/results/routes/`. The gate currently fails:
request-model, response-accessor, workflow-specific local fixtures, and network
cell evidence are unfinished. A passing subset is never a full-matrix claim.
The CI completeness gate deliberately preserves this failure while the PR is
WIP. The separate live catalog fixture suite covers both API levels, both
protocol versions and all six encoding/transport combinations, comparing
normalized request and response trees; these are controlled fixtures and do
not establish deployed vendor interoperability.

Generated `route_calls_generated.rs` is rebuilt by
`tools/generate_route_calls.py` from installed typed attribute metadata and the
shared route manifest. Both generated dispatch tables are excluded from raw
handwritten-source coverage; canonical route execution and freshness gates
remain required for their behavior.

Keep temporary fixture stores on the workspace disk when Rust, Java and C
validators run concurrently. CI sets `TMPDIR` to the ignored Rust fixture
folder before installing or testing. For local commands use:

```sh
mkdir -p interop/rust-kmip-validator/target/fixture-tmp
TMPDIR="$PWD/interop/rust-kmip-validator/target/fixture-tmp" python3 interop/rust-kmip-validator/tools/check_coverage.py
```

Local target hashes project only the exact, uniquely extractable target TTLV
subtree. The whole containing model must first pass encode/decode comparison;
a missing, ambiguous, or changed target fails the cell. This allows a model
builder and a generic value constructor to compare the same normative value
without substituting the parent model's hash for the target's hash.
