# C interoperability consumer

This standalone C11 consumer links the public KMIPKit C ABI installed from the
exact `release/1.0.0` source commit pinned by
`interop/shared/coverage/route-capabilities.json`
(`489fafc79ef89db2524eafa7a6c6482f5cf75d97`). It targets KMIP 2.0 and 2.1 and
uses the shared 2,689-case catalog. A successful build or controlled fixture
does not assert compatibility with a deployed vendor KMS.

Build the exact library artifact and the consumer from the repository root:

```sh
python3 interop/c-kmip-validator/tools/install_artifact.py
python3 interop/c-kmip-validator/tools/verify_artifact.py
python3 -m pip install --only-binary ':all:' jsonschema==4.23.0
cmake -S interop/c-kmip-validator -B interop/c-kmip-validator/build \
  -DPython3_EXECUTABLE="$(command -v python3)" \
  -DKMIPKIT_C_INSTALL_DIR=interop/.local/c-live/install/489fafc79ef89db2524eafa7a6c6482f5cf75d97
cmake --build interop/c-kmip-validator/build
ctest --test-dir interop/c-kmip-validator/build --output-on-failure
```

After preparing a dedicated test KMS connection profile and mTLS files, run:

```sh
interop/c-kmip-validator/build/kmip_validator_c preflight \
  --connection path/to/connection.json
interop/c-kmip-validator/build/kmip_validator_c run \
  --connection path/to/connection.json \
  --catalog interop/shared/catalog/kmipkit-1.0-test-catalog.json \
  --capabilities interop/shared/profiles/generic-kmip-2.1.json \
  --output results-c.json --output-markdown results-c.md --output-html results-c.html
```

When the server requires KMIP Username/Password authentication, configure only
the environment variable names in `authentication.username_password`:

```json
"authentication": {
  "username_password": {
    "username_env": "KMIPKIT_USERNAME",
    "password_env": "KMIPKIT_PASSWORD"
  }
}
```

Set those variables in the process environment before `run`. The driver builds
the Credential through KMIPKit's zeroizing secret API and never writes either
credential value to its reports or console output. An unset variable stops the
run before opening a network connection.

Each mutating case is reported as `policy_restricted` unless both the connection
profile and the `--allow-mutations` command-line option enable it. Reports use
the shared schema and record the configured server identity, selected KMIP
version, catalog digest, exact library commit and installed library digest.
The capability profile is informational; it never suppresses a probe.

The pinned release artifact preserves the configured KMIP version. Controlled
TLS fixtures exercise Discover Versions, Ping, Locate, Query, Log, Login,
Logout, and Interop. The Logout recipe obtains the Ticket from a successful
KMIP 2.1 Login, returns it in the Logout request, and checks the empty Logout
response payload. Interop sends Begin, End, and Reset in an ordered batch with
distinct batch-item IDs and checks each successful empty response.
The ordered Create/Destroy, Create/Activate/Revoke/Destroy,
Create/Check/Revoke/Destroy, Create/Revoke/Destroy, Create/Destroy/Revoke/Destroy,
Create/Get/Destroy, Create/Get Attributes/Destroy, Create/Get Attribute
List/Destroy and Create/Get Constraints/Revoke/Destroy lifecycles under KMIP
2.1. Under KMIP 2.0 they exercise Locate, Query, Log and all nine lifecycle
recipes except Get Constraints, which is available starting in KMIP 2.1. The
2.0 catalog case verifies that KMIPKit rejects Get Constraints before sending
it; because no KMIP response exists, server compatibility is reported as
unknown. The attribute catalog also sends Get Attributes requests selecting
`Cryptographic Algorithm` and `Cryptographic Length` by their KMIP tags and
checks the returned AES and 128-bit values in both protocol versions. The
`Symmetric Key` object catalog variant also runs Create followed by Destroy
against an isolated object; the `Private Key` and `Public Key` variants create
RSA-2048 objects and validate the respective typed responses before cleanup.
Activate moves the isolated key out of Pre-Active before revocation; the
negative Destroy case verifies `Operation Failed` with `Wrong Key Lifecycle
State`, then revokes and destroys the active key. Check verifies the
returned object identifier, then revokes the isolated object before Destroy.
Revoke verifies its returned identifier before Destroy. Destroy verifies its
target response and treats Object Not Found on the best-effort cleanup items as
expected after deletion. Each lifecycle probe
requests a generated AES-128 key, checks the response against the created
object, and enables batch continuation so cleanup still runs after a failed
operation. Get also checks the returned key format, algorithm and nonempty key
material without including the key bytes in reports. The public C
Discover Versions builder has a fixed KMIP 2.1 offer; because it does not expose
a way to offer KMIP 2.0, that specific KMIP 2.0 case is reported as a public
API limitation. Its installed route manifest therefore limits C request
construction to KMIP 2.1. The route-matrix fixture separately exercises the
response decoder with a KMIP 2.0 message header and the builder's fixed KMIP 2.1
offer; it verifies that the server returns only that offered version. This adds
2.0 response-decoding evidence without claiming that the C builder can offer
KMIP 2.0. Controlled TLS fixtures execute the C Login and Logout lifecycle
under both KMIP 2.0 and 2.1. They verify the Login Ticket is consumed for Logout
without copying Ticket bytes into route evidence.
The public C API exposes generic TTLV constructors,
inspectors, and a decoder.
It has no generic TTLV encoder and no generic TTLV request dispatch.
It also exposes no high-level client API. The consumer records those route
limits as library/API gaps. Live runs require a dedicated test KMS, valid
mutual-TLS credentials and explicit opt-in for mutations.

## KMIPKIT-0092 per-cell route evidence (WIP)

The matrix coordinator executes exact assigned tag/enum/bit-mask values through
local public C ABI constructors and the decoder, compares the inspected values,
and records decoder rejection of reserved tags. It invokes the compiled C
consumer against a controlled peer in TTLV, JSON, and XML over raw TLS and HTTPS
for the implemented operation recipes in KMIP 2.0 and 2.1. Each positive request
has recipe-specific field assertions, response checks, and lifecycle state and
teardown checks. A negative active-Destroy recipe checks the numeric failure
outcome before revocation and deletion. The KMIP 2.0 Get Constraints boundary
checks the typed executor invocation and zero observed transport connections.

On Linux, CMake generates link wrappers from the installed public header. The
wrappers observe only calls the compiled consumer actually makes; temporary
observations contain public symbol names and numeric status codes. They never
record KMIP bodies. The coordinator credits a matrix route only if its symbol
and executor were observed. Local opaque-handle calls use typed `ctypes`
signatures against the same verified installed C library.

Run from the checkout root after building the installed consumer:

```sh
python3 -m pip install --only-binary ':all:' jsonschema==4.23.0
python3 interop/c-kmip-validator/tests/test_route_matrix.py
python3 interop/c-kmip-validator/tools/route_matrix.py \
  --binary interop/c-kmip-validator/build/kmip_validator_c \
  --output interop/c-kmip-validator/build/route-execution-evidence-set-constraints-v20-boundary.json
python3 interop/shared/tools/validate_route_evidence.py \
  --report interop/c-kmip-validator/build/route-execution-evidence-set-constraints-v20-boundary.json \
  --artifact-sha256 "$(sha256sum interop/.local/c-live/install/489fafc79ef89db2524eafa7a6c6482f5cf75d97/lib/libkmipkit_ffi.so | cut -d ' ' -f 1)"
```

For an incomplete matrix, validate the exact emitted scope (rather than
claiming the full planned matrix is complete) with the shared validator API:

```sh
python3 - <<'PY'
import hashlib
import json
import sys
from pathlib import Path

sys.path.insert(0, "interop/shared/tools")
from validate_route_evidence import validate_report

report = json.loads(Path("interop/c-kmip-validator/build/route-execution-evidence-set-constraints-v20-boundary.json").read_text())
catalog_bytes = Path("interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_bytes()
manifest_bytes = Path("interop/shared/coverage/route-capabilities.json").read_bytes()
artifact_bytes = Path(
    "interop/.local/c-live/install/489fafc79ef89db2524eafa7a6c6482f5cf75d97/lib/libkmipkit_ffi.so"
).read_bytes()
summary = validate_report(
    report, json.loads(catalog_bytes), json.loads(manifest_bytes),
    catalog_sha256=hashlib.sha256(catalog_bytes).hexdigest(),
    route_manifest_sha256=hashlib.sha256(manifest_bytes).hexdigest(),
    expected_artifact_sha256=hashlib.sha256(artifact_bytes).hexdigest(),
    scope_cells=report["events"],
)
print(json.dumps(summary, sort_keys=True))
PY
```

This scoped call validates every emitted event and its provenance. The regular
CLI command above is the full-completeness gate and reports missing planned
cells until the pending recipes below are implemented.

Both completeness commands currently exit nonzero because semantic branch,
auxiliary model/accessor, cryptographic, asynchronous, and additional typed
construction recipes remain pending. CI deliberately fails on these gaps.
The `*.gaps.json` sidecar identifies every unexecuted matrix obligation rather
than misclassifying it as an unavailable library route. Other sidecars retain
installed-artifact availability checks and sanitized controlled-peer outcomes.
The evidence JSON passes the shared schema independently of completeness.
The preceding full local C matrix report (`c-route-b193e0e8-202a-487f-8b9b-f3ee0ec48bb9`)
has 66,288 expected cells, 43,480 invoked, 40 explicitly not applicable, zero
failed, and 22,768 pending. Its JSON SHA-256 is
`2b8cd3d5b44f23f07ba0a11b51885bf6ce7241e91e62b73fee3b90e84ac214f9`; it uses
the pinned C artifact SHA-256
`c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`. The
shared validator accepts all emitted cells with
`selected_scope_complete=true` and `all_invocations_passed=true`; the full
matrix remains incomplete. The first pending cell is the KMIP 2.0 local
`kmipkit_client_batch_item_response_reprovision` decoder for `test-op-re-provision`.
The `test-op-process` route adds 23 validated cells for KMIP 2.1: one local
Process request encoder, its typed constructor and response decoder, six
encoding/transport executions, and their local request/response construction
and decode routes. The controlled TLS peer returns Create Pending with a fixed
correlation, a synchronous successful Process response for that correlation,
a terminal Poll response containing the created identifier, then successful
Revoke and Destroy responses. It verifies the five-request order and empty
final fixture state. These are controlled fixture results, not evidence from
an external KMS. Compared with the preceding full report, this run adds 21
event identities for `test-op-process`; two of its applicable cells already
had matching identities in the previous report.
The MAC and Sign response TTLV accessors and the Continue batch-option
constructor contribute four local cells each across KMIP 2.0/2.1. The
Query Async Requests correlation builder contributes two KMIP 2.1 model cells.
The Opaque Object and PGP Key Register recipes each invoke the installed C
consumer across all 12 KMIP 2.0/2.1 encoding and transport combinations and
verify typed response access plus independent Destroy cleanup.
The evidence report passes schema validation, but the completeness gate
remains open because validator recipes are still pending.

The `test-enum-credential-type` local route roundtrips all six assigned values
and one extension value through `kmipkit_credential_type_from_raw`, then checks
generic TTLV preservation under KMIP 2.0/2.1 §11.11. It adds 14 direct C ABI
cells: seven values for each protocol version. It also wraps each assigned and
extension discriminator in a generic Credential envelope, consumes it through
`kmipkit_credential_from_ttlv`, and transfers the unchanged ordered tree back
through `kmipkit_credential_into_ttlv`. This adds 24 C ABI cells for the six
assigned values across both versions; the extension value is checked in the
local roundtrip too. The Credential Value fixture is an empty generic
Structure, so this recipe exercises the outer envelope without retaining
credential material.

The `test-enum-asynchronous-indicator` and
`test-enum-batch-error-continuation-option` local cases also invoke their C
batch setters for each of three assigned values and one extension value. They
add 16 construction cells across KMIP 2.0/2.1; generic TTLV roundtrips check the
enum values independently, without sending a request.

The PKCS#11 operation now covers its typed request and response accessors for
both protocol versions across all six encoding and transport combinations.
The Obtain Lease recipe creates an AES-128 key, obtains a lease through its
batch ID Placeholder, and revokes and destroys the object; it verifies the
Unique Identifier, Lease Time, Last Change Date, and complete response TTLV.
The Create Split Key route creates two AES-128 parts with a 2-of-2 XOR policy,
reads the repeated response identifiers in wire order, and destroys each part
by its returned identifier. All three operations are checked against the
controlled peer across KMIP 2.0 and 2.1, TTLV/JSON/XML, and raw TLS/HTTPS. These
recipes use synthetic controlled-peer values and do not claim vendor
interoperability.
The companion `test-obj-split-key` case records 50 passing cells for the same
Create Split Key and cleanup workflow: 18 construction, 12 network send, and
20 decode cells. The peer verifies both returned identifiers are destroyed
exactly once; the report adds 40 new matrix identities for this object case.
The Join Split Key route creates the same isolated pair, joins the two returned
identifiers, then destroys the joined object through the batch ID Placeholder
and both source parts through their exact identifiers. It verifies the typed
Join response and complete response TTLV through
`kmipkit_client_batch_item_response_join_split_key_ttlv`, as well as the final
fixture state across the same 12 network routes. The latest matrix records 44
passed cells for this case and no pending cells.
The Re-key route creates an AES-128 Symmetric Key with explicit Encrypt usage,
then sends Re-key without an offset or additional attributes. It reads the
replacement identifier through the typed response accessor and verifies the
complete response payload through
`kmipkit_client_batch_item_response_rekey_ttlv`, then revokes and destroys the
replacement through the batch ID Placeholder and revokes and destroys the
original by its returned identifier. The controlled peer verifies both
responses and complete cleanup for KMIP 2.0/2.1 across TTLV, JSON, and XML over
raw TLS and HTTPS. The latest matrix run records 44 passed cells for the Re-key
case and no pending cells. These synthetic values do not establish external
KMS interoperability.
The Secret Data Register route builds a Password object with a Raw Key Block
and explicit Derive Key usage, reads the typed Register result and response
TTLV, then destroys the object through its batch ID Placeholder. Its synthetic
material is cleared from the input buffer after construction and is omitted
from route events and peer checks. The same 12 controlled network combinations
pass for KMIP 2.0 and 2.1; the latest full matrix records 86 passed cells for
this case and no pending cells. It does not execute Derive Key or claim external
KMS interoperability.

The Derive Key route creates and activates an AES-128 source key with explicit
Derive Key usage, then requests PBKDF2 with explicit SHA-256, derivation data,
salt, iteration count, and AES-128 output attributes with Encrypt usage. It
checks the typed result and complete response TTLV, then revokes and destroys
the derived key and source key. Synthetic derivation data and salt are omitted
from route events and peer checks. All 12 controlled network combinations pass
for KMIP 2.0 and 2.1; the latest full matrix records 44 passed cells for this
case and no pending cells. These fixtures exercise the installed C consumer
and do not claim external KMS interoperability.

In the latest matrix run, Re-key has 44 passed cells and no pending cells for
its covered targets. Its controlled network journeys pass across all 12
combinations; the response-TTLV accessor is exercised for both protocol
versions.

The `test-op-poll` route now uses a controlled asynchronous Create followed by
Pending, Pending, and successful Poll responses, then destroys the created
object. Its request encoder and typed response decoder pass under KMIP 2.0 and
2.1. The route adds 42 passed cells across the six encoding/transport pairs and
the two protocol versions; the response carries the exact correlation issued
by the fixture Create. This is controlled-peer evidence, not external KMS
interoperability.

A generic TTLV wire encoder, generic-TTLV request dispatcher, and high-level
C client are absent from the exact installed public artifact. The harness's
independent vector serialization is never credited as a public C encoder.
Reserved tags can be represented by RawTag and rejected by the decoder/checked
conversion; construction requiring an already checked Tag remains an explicit
matrix-applicability question. Cross-language normalized-tree parity and vendor
interoperability remain unverified.

The `test-op-encrypt` controlled route also creates and activates an AES-128
key with explicit Encrypt usage, sends an AES/ECB/No Padding request, checks
the typed Encrypt response, then revokes and destroys the key. Its fixed AES
fixture bytes validate request/response fields only; Create generates a random
key, so they do not establish a correct encryption result. The route does not
run cryptography locally or establish compatibility with an external KMS.
The `test-op-decrypt` route uses the matching AES-128 Decrypt usage and
explicit AES/ECB/No Padding parameters, checks the fixed ciphertext input and
typed 16-byte plaintext response, then performs the same revoke/destroy
cleanup. The fixture bytes are not cryptographically tied to the randomly
generated key; they test field handling, not cryptographic correctness or
external KMS interoperability.

The `test-op-mac` route creates an HMAC-SHA256 key with explicit MAC Generate
usage and sends a typed MAC request with an explicit HMAC-SHA256 parameter.
The peer checks the exact request and deterministic 32-byte response fixture;
the installed C consumer checks the response accessor, identifier, and length.
Because the key is generated by the server, the fixture does not validate the
cryptographic MAC value.

The `test-op-mac-verify` route creates a 256-bit HMAC-SHA256 key with explicit
MAC Verify usage, activates it, and sends MAC Verify with explicit HMAC-SHA256
parameters, fixed data, and a fixed MAC Data field. It then revokes and destroys
the key. The peer checks the complete ordered request and returns a typed
Validity Indicator of `Valid`; the C consumer checks that typed field and the
created key identifier. The generated key is random, so the fixture does not
establish cryptographic validity of the supplied MAC. It also reads and checks
the complete response-item structure through
`kmipkit_client_batch_item_response_mac_verify_ttlv`.

The `test-op-sign` and `test-op-signature-verify` routes create an RSA-2048 key
pair with separate usage masks: Sign on the Private Key and Verify on the Public
Key. Each ordered batch activates the selected key, executes the operation, then
revokes and destroys the selected key and destroys its pair. The request uses
the explicit SHA-256 with RSA signature algorithm. The peer checks the full
request tree and typed response shape. Create generates a random pair, so the
fixed signature inputs and response do not establish cryptographic validity.
The Signature Verify route also reads and checks the complete response-item
structure through `kmipkit_client_batch_item_response_signature_verify_ttlv`.

The direct `test-op-create-key-pair` route constructs an RSA-2048 pair with
explicit Sign and Verify usage masks, checks that the typed response returns
two distinct text identifiers, and destroys the Private Key and Public Key by
their Create Key Pair identifier selectors. The peer checks the exact request
attributes, response identifier fields, item-ID correlation, and cleanup
mapping. It verifies protocol shape and lifecycle only; it does not inspect or
validate generated key material.

The direct `test-op-certify` route creates an RSA-2048 pair with explicit
Certificate Sign usage on the Private Key and Verify usage on the Public Key,
activates the signer, and certifies the paired Public Key with the X.509
Certificate Type. It checks the typed certificate identifier and destroys the
certificate by ID Placeholder before revoking and destroying both keys. The
controlled peer verifies the request tree, signer lifecycle, placeholder
mapping, typed results, and final state. This validates KMIP request/response
shape and lifecycle only; the fixture does not validate a real certificate or
signature. The route also reads
`kmipkit_client_batch_item_response_certify_ttlv` and checks the complete
successful response-item structure for both KMIP 2.0 and 2.1. The accessor's
returned structure is released immediately.

The `test-op-archive` route creates an isolated AES-128 key, requests Archive
using the ID Placeholder while the key remains Pre-Active, then revokes and
destroys the key. KMIP 2.0/2.1 §6.1.4 does not require Activate before Archive.
The controlled peer checks the Archive request, required response Unique
Identifier, ordered item IDs, and teardown. KMIP 2.0/2.1 §6.1.4 defines Archive
as a client indication; the peer records receipt without claiming that the
object was archived or that an external KMS changed object state.

The two-item Get/Get Attribute List batch fixture also checks Batch Error
Continuation Option Continue. It correlates both results by Unique Batch Item
ID, verifies the failed Get result through the typed accessor with no Response
Payload, and checks the successful Get Attribute List through both its typed
fields and generic TTLV payload. The controlled response includes the fixture
key identifier and the Cryptographic Algorithm and Cryptographic Length
references; this validates response shape and accessors, not external KMS
behavior.

The `test-batch-order-required-placeholder-resolution` and
`test-batch-order-omitted-defaults-true` fixtures send Create followed by
Activate using ID Placeholder. The first explicitly enables Batch Order
Option; the second omits it and checks the KMIP §9.8 default of true. Both
verify ordered typed results, response identifier correlation, and a separate
Revoke/Destroy cleanup exchange against the controlled TLS peer. This checks
the C API and protocol behavior without claiming external KMS interoperability.

The `test-batch-count-matches-items` and
`test-batch-order-false-correlates-response-items` fixtures exercise both
direct Get Attribute List construction and the standalone
`GetAttributeListRequest` constructor through the installed C ABI. The peer
reverses the two responses for the unordered batch and verifies correlation by
Unique Batch Item ID.

The consumer also checks response metadata v1, the raw payload and header
TTLV structures, and the absent pending correlation for synchronous success.

The C `test-batch-count-mismatch-rejected` route also sends the catalog's
two-item Get/Get Attribute List case through both direct and standalone
constructors. The controlled peer checks that the request carries
`Batch Count = 2` and contains exactly two Batch Items before returning a
successful response. This is supplemental C exchange evidence for the
catalog's local construction case: it proves no inconsistent count reaches
this controlled peer and does not claim external KMS behavior.

The `test-batch-count-zero-rejected` contract contains only the Batch Count
target, with no operation or Batch Item, and records rejection before
transport. Its matrix rows are local construction checks; it creates no send
or response-decode obligation. A separate C parser test confirms that an
encoded empty Request Message with `Batch Count = 0` is rejected locally as
`invalid_batch_count`; that supplemental parser check is not counted as a
catalogued decode cell.

The negative `test-neg-modify-attribute-current-mismatch` route seeds the
Comment attribute, requests a Current Attribute value that is not present, and
expects `OperationFailed` with `Attribute Instance Not Found` (32). It exercises
direct and standalone Modify Attribute request constructors and verifies that
the peer preserves the original Comment. This follows KMIP 2.0 §6.1.33 and KMIP
2.1 §6.1.34 and §9.18/Table 267. The route covers all 12 version, encoding,
and transport combinations; cleanup revokes and destroys the seeded object.

The `test-op-import` route imports a nonsecret Opaque Object with an explicit
Unique Identifier, empty caller-supplied Attributes, and one Any Object item.
The controlled peer checks the object type, object fields, request item IDs,
response status, and returned identifier; the C consumer checks both the typed
Import result and the full response-item TTLV accessor, then destroys the
imported object. The route passes all 12 KMIP 2.0/2.1 encoding and transport
combinations. Neither the object value nor the request body is retained in
route events or peer checks.

The `test-op-query-asynchronous-requests` local model route constructs and
encodes the valid empty Query Asynchronous Requests payload. KMIP 2.1 §6.1.41
Table 285 makes both request filters optional. Its live route creates an AES
key, sends Query without filters, verifies the successful response has a
present empty Response Payload through both typed and TTLV accessors, then
revokes and destroys the created key. The controlled TLS peer verifies this
workflow across TTLV, JSON, and XML over raw TLS and HTTPS. The route adds 19
matrix cells: request construction, send, and response handling for six
network combinations, plus the typed response-TTLV accessor.

The C `QueryAsyncRequestsRequest::with_operations` model route also constructs
and encodes two Operation filter entries (Create and Get), checking the exact
repeated Enumeration tree under KMIP 2.1 §§6.1.41 and 7.25. It covers the
Operations structure, its Operation member, the Operations tag, and the
catalogued KMIP 2.1 request field, adding seven applicable local cells.

The `test-batch-single-item-default-continuation` live route exercises one
successful Get with Batch Count 1 and both batch options omitted across both
protocol versions, all three encodings, raw TLS, and HTTPS. KMIP 2.0/2.1 §9.6
defines the omitted continuation option as Stop and permits the option only
when Batch Count is greater than one. The route therefore never calls the
explicit Stop setter; its two setter cells remain pending until a valid
multi-item Stop recipe is implemented. The peer checks the request shape and
successful Get response; the C consumer checks response metadata, correlation,
header and payload TTLV views, and the typed key response. This adds 62 cells:
36 live construct/send/response cells and 26 local request/response cells.

The `test-batch-placeholder-without-producer-rejected` case uses the typed C
Activate builder with an omitted Unique Identifier in a one-item batch. Under
KMIP 2.0/2.1 §§6.1.1 and 9.7, there is no earlier item to produce the batch's
ID Placeholder. The installed client rejects the request as Validation / Invalid
Input with delivery state Not Sent before opening a transport connection. The
matrix records all 12 version/encoding/transport combinations as local
rejections; it does not claim a KMS response or response-accessor coverage.

The preceding full report (`c-route-56b0ea4e-dcd3-4332-97bc-2f265c3abc00`),
compared with its 43,035-event predecessor, retained every prior cell identity
and added exactly 62 cells for the single-item default-Stop route. Catalog,
route-manifest, installed-artifact, and library-commit fingerprints matched
that run's baseline. Its CTest passed 7/7, including 191 route-matrix contract
tests and both controlled TLS peers. The global route-completeness gate
remains open for the pending recipes.

For `test-batch-stop-on-first-failure`, the approved KMIPKIT-0007-FR-018
response policy expects one `Operation Failed` result and omits the later
unprocessed result. The route matrix marks only the omitted item's decode cells
as `not_applicable` with reason `stop_unprocessed_response_omitted`; returned
response items remain required and are decoded through the installed C API.
The event has zero invocations and no invocation ID or fabricated response
tree. OASIS KMIP 2.0 §11.5/Table 396 and KMIP 2.1 §11.5/Table 434 describe Stop
behavior but do not explicitly prescribe that omission. The omission is the
approved KMIPKit policy aligned with the KMIPKIT-0092 scenario, not an OASIS
conformance claim.

An earlier C run (`c-route-3df0ddc1-8a8a-4d90-aa50-addbab9194f4`) emitted 43,255
evidence rows: 43,239 actual invocations and 16 explicit N/A dispositions,
with zero failed invocations. It had 23,481 pending recipe cells.

The prior full run (`c-route-3115b3c5-78fc-4a1e-98fc-f8645e434d84`) emitted
43,665 evidence rows: 43,625 actual invocations and 40 explicit N/A
dispositions, with zero failed invocations. Compared with its previous full
report, no cell identities were removed and 16 were added for the two batch enum setters: eight
Asynchronous Indicator cells and eight Batch Error Continuation Option cells.
The Credential Type constructor and Credential envelope routes remain in the
report. The previously covered Current Attribute mismatch route contributes
74 cells: 24 construction calls, 24 send attempts, 24 network response
decoders, and two local response-TTLV accessors for KMIP 2.0 and 2.1.
The peer returns `OperationFailed` with `Attribute Instance Not Found` and
confirms the original attribute remains unchanged. The earlier
forward-Placeholder route contributes 24 N/A response decoders because each
matching request was rejected locally before transport; all 24 N/A cells match
a `not_sent` event for the same case, route, operation, item index, protocol
version, encoding, and transport. The 14 independent local
response accessors remain pending; six local generic TTLV decodes pass using
constructed fixtures. Of the cells shared with the previous run, 199 tree
hashes differ: 192 request hashes contain regenerated public certificate or
certificate-request bytes and seven Query Async Requests response hashes
include the fixture's current timestamp. Cell identities are unchanged for
those rows. The previous report added the KMIP 2.0 Discover Versions local
decoder cell using a 2.0 response header and the fixed 2.1 request offer; it
validates the returned 2.1 version without claiming 2.0 offer construction.
The previous report retained all previous identities and added 40 for the
`test-obj-split-key` object case after the 42 Poll-route cells and 21 Process
identities. It passed the evidence schema and scoped shared validation, while
22,768 matrix cells remained pending.

The preceding full C report is `c-route-4657eef3-12ec-4fe0-bb88-b78746886234`
with JSON SHA-256
`4f44724969f655433306a03491336577a787beb698a5c198cac9f9b538cff1dc`. It has
66,288 expected cells: 43,518 invoked, 40 explicitly not applicable, zero
failed, and 22,730 pending. It retains all 43,520 identities from the previous
report and adds 38 passing cells for
`test-batch-unique-item-id-response-correlation`: 12 construction, 12 send,
12 response decode, and two local request encodes. Across both KMIP versions
and all six encoding/transport combinations, the controlled peer checks that
the two request items (`Get`/`item-a` and `Get Attribute List`/`item-b`) each
receive a response with the matching ID and operation. The full matrix remains
incomplete; scoped shared validation of this 168-cell case reports
`selected_scope_complete=true`, `all_invocations_passed=true`, and zero
failures. The first pending cells remain the local response decoder for
`test-op-re-provision`; that recipe is unresolved against the approved source
contract and is not represented as implemented.

The preceding full C report is `c-route-a857f6db-06ed-4041-b7c5-fd119d492875`
with JSON SHA-256
`34b5003d7cad4c40cfcb35231d136bb6bfceebe24d8fa29bb9d6d6fabf961b58`. It has
66,288 expected cells: 43,520 invoked, 40 explicitly not applicable, zero
failed, and 22,728 pending. It retains all 43,558 identities from the prior
report and adds two passing local constructor cells for the explicit Stop
continuation-option builder, one per KMIP version. That API call is tested
separately from `test-batch-single-item-default-continuation`'s wire recipe:
the controlled peer still confirms the request omits the option and therefore
uses the §9.6 default. Scoped validation of the two-cell delta reports
`selected_scope_complete=true`, `all_invocations_passed=true`, and zero
failures. Full CTest passed 7/7, including the route-matrix contracts and both
controlled TLS peer suites. The first pending cells remained the local response
decoder for `test-op-re-provision`; its recipe is unresolved against the
approved source contract.

The preceding full C report is `c-route-bd097850-6359-4f42-b138-9f1f44f3a168`
with JSON SHA-256
`64d08004a8c3b34589fce047e8e7f692287473fa36a45fea2b6120556c9d81b6`. It has
66,288 expected cells: 43,530 invoked, 40 explicitly not applicable, zero
failed, and 22,718 pending. It retains all 43,560 prior identities and adds
ten passing response-decode cells for `test-batch-single-item-options-omitted`:
the typed Get result, response header TTLV, response metadata, pending
correlation accessor, and Unique Batch Item ID accessor for KMIP 2.0 and 2.1.
The controlled fixture verifies these accessors on its one successful Get
response while confirming the request omits all optional batch settings. The
new ten-cell scope passes shared validation with
`selected_scope_complete=true` and `all_invocations_passed=true`. The first
pending cells are still the local response decoder for `test-op-re-provision`;
its recipe is unresolved against the approved source contract.

The latest full C report is `c-route-7c34d7a2-6fe8-423d-888f-d5ec9deeac1f`
with JSON SHA-256
`70a0dabd711701c476cb76df64ba864f8051241b31d90b2b8b6e05ee615937ce`. It has
66,288 expected cells: 43,518 invoked, 52 explicitly not applicable, zero
failed, and 22,718 pending. It preserves all 43,570 prior cell identities.
Twelve previous `passed` decode events for
`test-batch-placeholder-without-producer-rejected` are corrected to
`not_applicable` with reason `local_rejection_no_response`: the client rejected
each matching Activate request before transport, so no response was available
to decode. The shared validator checks their matching `not_sent` calls; the
44-cell case scope is complete with 32 invocations, 12 N/A, zero failures, and
`all_invocations_passed=true`. The full CTest suite passed 7/7 in 144.99 s
after the evidence correction. The first pending cells remain the local
response decoder for `test-op-re-provision`, whose recipe is unresolved against
the approved source contract.

The latest full report is `c-route-3712618f-31eb-4f9d-ad02-6fd5af16eebe`
(`route-execution-evidence-certificate-type-extension.json`) with JSON SHA-256
`559af072feb69bb9c853fba9d4e5fd824c2ac66a89076a62b19a758280898c87`. It has
66,288 expected cells: 43,758 invoked, 52 explicitly not applicable, zero
failed, and 22,478 pending. It preserves all 43,570 prior cell identities and
adds 240 passing cells across 24 enum and bit-mask extension cases. The
attribute fixture now selects the lower bound from the catalogued extension
range (for Certificate Type, `0x80000000`) and verifies the typed attribute
constructors, decoders, and AttributeSet paths against that value. The shared
validator accepts the emitted report with `selected_scope_complete=true`,
`all_invocations_passed=true`, and zero failures; the full matrix remains
incomplete.

The latest full report is `c-route-88bb9dfc-7337-4112-98b1-c2cc05f106fb`
(`route-execution-evidence-process-v20-boundary.json`) with JSON SHA-256
`70bc94bfbf781ceecf02aead80093e241a50993c2b72cb267cfff0e4d3bae85d`. It has
66,288 expected cells: 43,776 invoked, 52 explicitly not applicable, zero
failed, and 22,460 pending. It preserves all 43,810 previous cell identities
and adds 18: 14 cells for `test-v20-reject-process` and four local constructor
and encoder cells for the KMIP 2.1 Process payload. The controlled C driver
confirms that Process under KMIP 2.0 is rejected before transport across TTLV,
JSON, and XML over raw TLS and HTTPS, while the separate request model preserves
its Asynchronous Correlation Value. The shared validator accepts the emitted
report with `selected_scope_complete=true`, `all_invocations_passed=true`, and
zero failures; the full matrix remains incomplete.

The latest full report is `c-route-bb9575c1-4edf-44d8-94ba-0d95f23df59b`
(`route-execution-evidence-set-defaults-v20-boundary.json`) with JSON SHA-256
`2b09dcfdbf16caa932d7edf6bccd85c041e477481e9b7cfcb2818b3c8a9a0abd`. It has
66,288 expected cells: 43,788 invoked, 52 explicitly not applicable, zero
failed, and 22,448 pending. It preserves all 43,828 previous emitted cell
identities and adds 12 for `test-v20-reject-set-defaults`: construct and send
across TTLV, JSON, and XML over raw TLS and HTTPS. The installed typed C route
rejects KMIP 2.1-only Set Defaults under KMIP 2.0 before transport; all six send
events report `NotSent` and zero transport calls. The report passes schema and
scoped shared validation with `selected_scope_complete=true`,
`all_invocations_passed=true`, and zero failures. Full CTest passed 7/7 in
140.15 seconds; the global matrix remains incomplete, with the Re-Provision
response decoder still the first pending cell because its source recipe is
unresolved.

The latest full report is `c-route-917bd7e5-e9ba-4b6b-bfe2-accebacb5f60`
(`route-execution-evidence-set-constraints-v20-boundary.json`) with JSON
SHA-256 `2743642fd600f85179cc9cba8c16cfb1cca2dd0d879d225e29275d7ff11bfc14`.
It has 66,288 planned cells: 43,800 invoked, 52 explicitly not applicable,
zero failed, and 22,436 pending. It preserves all 43,840 previous emitted
identities and adds 12 for `test-v20-reject-set-constraints`: construct and
send across TTLV, JSON, and XML over raw TLS and HTTPS. The installed typed C
route rejects KMIP 2.1-only Set Constraints under KMIP 2.0 before transport;
all six send events report `NotSent` and zero transport calls. Schema and scoped
shared validation pass with `selected_scope_complete=true`,
`all_invocations_passed=true`, and zero failures. Full CTest passed 7/7 in
142.35 seconds. The full matrix remains incomplete; the first pending cell is
still the Re-Provision response decoder, whose recipe is unresolved.

The latest full report is `c-route-b33618df-f4d1-48f9-bcda-fd360c1baea7`
(`route-execution-evidence-aes256-cbc-encrypt.json`) with JSON SHA-256
`fefb35e842d539b66c782ebc11f30166760b0fa65c649124affea458d48826b1`. It has
66,288 planned cells: 43,838 invoked, 52 explicitly not applicable, zero
failed, and 22,398 pending. It preserves all 43,852 prior cell identities and
adds 38 for `test-crypto-encrypt-aes-256-cbc`: 36 controlled TLS request/send/
response routes across KMIP 2.0/2.1, TTLV/JSON/XML, and raw TLS/HTTPS, plus two
local response decoders. The complete 42-cell scope also retains four prior
local construction/decode cells. Its builder creates an AES-256 key and sends
an explicit CBC mode, no-padding parameter, and 16-byte IV; the controlled peer
checks those fields and the deterministic ciphertext response. It keeps the
existing ECB recipe unchanged. Schema validation and scoped shared validation
pass with `selected_scope_complete=true`, `all_invocations_passed=true`, and
zero failures. Full CTest passed 7/7 in 142.57 seconds. The matrix remains
incomplete; the first pending cells are still the Re-Provision response
decoders, whose recipe is unresolved against the approved source contract.

The next full run is `c-route-8ee3fe83-886d-4aa1-b22b-253428dc842d` (report
SHA-256 `de38693a5a1d69e910e4e8dfa31c55bd8a59dd9317381593ded62b60ab0e688b`,
pinned artifact SHA-256 `c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`).
It preserves all 43,890 identities from the AES-256-CBC run and adds 40 new
passed cells per case; each complete case scope now has 44 passed cells for
`test-op-re-certify` and `test-op-validate`. Re-certify renews
the fixture X.509 certificate by its explicit identifier, consistent with
KMIPKIT-DEC-019 and KMIP 2.0 §6.1.41 / 2.1 §6.1.45; Validate sends the actual
DER certificate fixture in a Table 308/340 Certificate structure under KMIP
2.0 §6.1.51 / 2.1 §6.1.57. Both scopes cover local construction/decode and
all six encoding/transport combinations per protocol version. Schema and
scoped shared validation pass with `selected_scope_complete=true`,
`all_invocations_passed=true`, and zero failures. The full ledger now records
66,288 planned cells: 43,918 invoked, 52 not applicable, zero failed, and
22,318 pending. Full CTest passed 7/7 in 144.84 seconds. The first pending
cells remain the Re-Provision response decoders; the global matrix is still
incomplete.

The latest full C report is `c-route-0b591607-5597-496a-9169-92ef08abaa58`
(`route-execution-evidence-rekey-key-pair.json`) with JSON SHA-256
`1f3a2165f133459275a5219cce2788f04305b75057dd225de574b83b3e9a78b2` and
catalog SHA-256 `11d002765d807ba4232625ed19e44649db0fcdd33f0b43911ea18f18a7fe4651`.
It has 66,288 planned cells: 43,958 invoked, 52 explicitly not applicable,
zero failed, and 22,278 pending. It preserves all 43,970 prior cell identities
and adds 40 passed cells for `test-op-re-key-key-pair`; its 44-cell total also
includes four previously reported generic TTLV construction/decode cells. The
new routes cover both protocol versions, TTLV/JSON/XML, raw TLS/HTTPS, and the
typed response and response-TTLV accessors. The controlled client creates and
activates an RSA key pair, reads both original identifiers, executes Re-key Key
Pair, checks both distinct replacement identifiers, then revokes and destroys
both key pairs. Scoped shared validation passes with
`selected_scope_complete=true`, `all_invocations_passed=true`, and zero
failures. The 199 changed hashes on existing events affect only request/response
tree digests: 192 for certificate fixtures generated afresh per run and seven
for time-bearing asynchronous responses; all prior event identities remain.
The pinned artifact SHA-256 remains
`c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`. Full
CTest passed 7/7 in 145.45 seconds and the C matrix contract suite passed
226/226. The first pending cells remain the Re-Provision response decoders; the
global matrix is still incomplete.

The latest full C report is `c-route-5d37985d-a4f0-4913-a343-55a33faec466`
(`route-execution-evidence-decrypt-aes-256-cbc.json`) with JSON SHA-256
`e13355c803ae78295334b7dce6a8cfd90e516f1145289426b0616cfd33feaed9`, catalog
SHA-256 `11d002765d807ba4232625ed19e44649db0fcdd33f0b43911ea18f18a7fe4651`,
route manifest SHA-256
`28b381ebca85752ba8eb3c41c003f02b2d1d8ad8180f0e8908a7e716724732f5`, and
pinned artifact SHA-256
`c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`. It has
66,288 planned cells: 43,996 invoked, 52 explicitly not applicable, zero
failed, and 22,240 pending. It preserves all 44,010 prior cell identities and
adds 38 passed cells for `test-crypto-decrypt-aes-256-cbc`; its 42-cell total
also includes four previously reported generic TTLV construction/decode cells.
The new cells cover KMIP 2.0/2.1, TTLV/JSON/XML over raw TLS/HTTPS, and both
local and network response decoders. The controlled request specifies AES-256,
Decrypt usage, CBC mode, no padding, the 16-byte IV, and the fixed CBC
ciphertext; the peer checks those fields and returns the matching plaintext
response. The client validates the returned identifier, plaintext, and cleanup
responses. Schema validation and scoped shared validation pass with
`selected_scope_complete=true`, `all_invocations_passed=true`, and zero
failures. Full CTest passed 7/7 in 145.71 seconds. The first pending cells
remain the Re-Provision response decoders; the global matrix is still
incomplete. The shared Python test suite passed 210/210 tests. Of the preserved
prior events, 199 changed only their tree digests: 192 request hashes from
regenerated certificate fixtures and seven response hashes from time-bearing
asynchronous responses; no other prior outcome fields changed.

The latest full C report is `c-route-154bc70f-8cf3-47ff-9513-345d8247eeeb`
(`route-execution-evidence-rsa-pkcs1-sign.json`) with JSON SHA-256
`e0afe302b09216abe5622bf4c17d672a24815e4f6fc78fee3f2695f96194fb01`, catalog
SHA-256 `11d002765d807ba4232625ed19e44649db0fcdd33f0b43911ea18f18a7fe4651`,
route manifest SHA-256
`28b381ebca85752ba8eb3c41c003f02b2d1d8ad8180f0e8908a7e716724732f5`, and
pinned artifact SHA-256
`c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`. It has
66,288 planned cells: 44,036 invoked, 52 explicitly not applicable, zero
failed, and 22,200 pending. It preserves all 44,048 previous cell identities
and adds 40 passed cells for `test-crypto-sign-rsa-2048-pkcs1_v1_5`; its 44-cell
case scope also includes four previously reported generic TTLV construction
and decode cells. The new cells cover 36 request/send/response routes across
KMIP 2.0/2.1, TTLV/JSON/XML, and raw TLS/HTTPS, plus both typed response
accessors for each protocol version. The controlled request creates and
activates an RSA-2048 key pair with Sign/Verify usage, applies the SHA-256 with
RSA signature algorithm, checks Sign output shape, revokes the private key,
and destroys both keys. The peer validates the request parameters and returns
a controlled response; this does not represent an external KMS run. Schema and
scoped shared validation pass with `selected_scope_complete=true`,
`all_invocations_passed=true`, and zero failures. Full CTest passed 7/7 in
148.47 seconds. Of the preserved prior events, 199 changed only their tree
digests: 192 request hashes from regenerated certificate fixtures and seven
response hashes from time-bearing asynchronous responses; no other prior
outcome fields changed. The first pending cells remain the Re-Provision
response decoders, so the global matrix is still incomplete.

The next full C report is `c-route-cd3bb527-f2e0-49c1-9b1c-536a15030e64`
(`route-execution-evidence-rsa-pkcs1-signature-verify.json`) with JSON SHA-256
`17d6502ad9bc5dc2cb538e13dfd7e38b567ba81fead3205e7c814621c0f6dda0`, catalog
SHA-256 `11d002765d807ba4232625ed19e44649db0fcdd33f0b43911ea18f18a7fe4651`,
route manifest SHA-256
`28b381ebca85752ba8eb3c41c003f02b2d1d8ad8180f0e8908a7e716724732f5`, and
pinned artifact SHA-256
`c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`. It has
66,288 planned cells: 44,076 invoked, 52 explicitly not applicable, zero
failed, and 22,160 pending. It preserves all 44,088 prior cell identities and
adds 40 passed cells for
`test-crypto-signature-verify-rsa-2048-pkcs1_v1_5`; its 44-cell scope also
includes four previously reported generic TTLV construction/decode cells.
The new cells cover 36 request/send/response routes across KMIP 2.0/2.1,
TTLV/JSON/XML, and raw TLS/HTTPS, plus both typed response accessors for each
protocol version. The controlled request supplies an all-zero 256-byte RSA
signature, so the controlled peer now reports Validity Indicator `Invalid`
(2) while the Signature Verify Result Status remains `Success`, as defined by
OASIS KMIP 2.0 §6.1.50/Table 451 and KMIP 2.1 §6.1.56/Table 492. The client
checks the typed invalid indicator, response TTLV, and key cleanup. This is
controlled route evidence, not an external KMS run. Schema and scoped shared
validation pass with `selected_scope_complete=true`,
`all_invocations_passed=true`, and zero failures. Full CTest passed 7/7 in
151.83 seconds. The peer effect and response digest for the existing generic
Signature Verify case changed to reflect the invalid fixture; 199 other
preserved events changed only tree digests (192 request hashes from fresh
certificate fixtures and seven response hashes from time-bearing asynchronous
responses). All previous cell identities remain. The first pending cells
remain the Re-Provision response decoders, so the global matrix is still
incomplete.

### C full-matrix report after Recover and ECDSA Sign

The latest full C report is `c-route-ef0cbe1a-c819-446f-9e58-45775e433be6`
(`build/route-execution-evidence-c-current.json`), SHA-256
`be1d15ccdafa3ea73500b9404f12803b350d0e2433c20e69809e344fa9f3f4c8`. It
uses catalog SHA-256
`511468935d55435681bec13fb66d1d9327e91a5a92fb781e9450edb043951fba`, route
manifest SHA-256
`28b381ebca85752ba8eb3c41c003f02b2d1d8ad8180f0e8908a7e716724732f5`, and
pinned artifact SHA-256
`c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`. The
ledger has 66,280 planned cells: 44,146 invoked, 52 explicitly not
applicable, zero failed, and 22,082 pending. The shared validator confirms all
44,198 emitted cells, with `selected_scope_complete=true`,
`all_invocations_passed=true`, and `complete_matrix=false`.

The Recover route adds 38 network cells. The controlled consumer creates a
SymmetricKey, archives it, recovers it by the ID Placeholder, checks the typed
Recover response Unique Identifier, and destroys the recovered key across both
KMIP versions and all six TTLV/JSON/XML plus raw TLS/HTTPS combinations. The
ECDSA Sign case adds 40 invoked cells: 36 network request/send/response routes
and four typed response-accessor decodes. Four generic TTLV construct/decode
cells in its 44-event scope were already present in the previous ledger. The
case covers Create Key Pair → Activate → Sign → Revoke → Destroy for both
KMIP versions and all six encoding/transport combinations. The peer checks
ECDSA, 256-bit key length, P-256 domain parameters, and ECDSA-with-SHA256
parameters; it returns a controlled 64-byte signature fixture, while the C
response check only requires nonempty signature data and does not assume its
encoding. This is fixture evidence for OASIS KMIP 2.0 §6.1.49 and KMIP 2.1
§6.1.55, not an external KMS result.

The catalog's zero-count batch case contains only the Batch Count target and
does not plan operation, Batch Item, send, or response-decode cells. The full
report's scoped validation passes for all emitted cells; full-matrix validation
remains incomplete. Full CTest passes 7/7 in 151.07 seconds. The complete C
route matrix remains open with 22,082 pending cells, so this PR remains WIP.
