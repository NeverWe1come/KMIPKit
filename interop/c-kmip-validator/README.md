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
API limitation. The public C Login builder only supports KMIP 2.1, so the
KMIP 2.0 Login case is reported as a public API limitation. The KMIP 2.0 Logout
case is not run because the C validator cannot obtain the server-issued Login
Ticket required by Logout. For KMIP 2.1 Login, the consumer checks the complete
response TTLV structure shape without copying Ticket bytes into route evidence.
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
  --output interop/c-kmip-validator/build/route-execution-evidence.json
python3 interop/shared/tools/validate_route_evidence.py \
  --report interop/c-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 "$(sha256sum interop/.local/c-live/install/489fafc79ef89db2524eafa7a6c6482f5cf75d97/lib/libkmipkit_ffi.so | cut -d ' ' -f 1)"
```

Both completeness commands currently exit nonzero because semantic branch,
auxiliary model/accessor, cryptographic, asynchronous, and additional typed
construction recipes remain pending. CI deliberately fails on these gaps.
The `*.gaps.json` sidecar identifies every unexecuted matrix obligation rather
than misclassifying it as an unavailable library route. Other sidecars retain
installed-artifact availability checks and sanitized controlled-peer outcomes.
The evidence JSON passes the shared schema independently of completeness.
The latest full local C matrix run (`c-route-5b7720d1-936c-42fa-bf3d-e32cf252bb18`)
has 66,736 expected cells, 43,097 invoked, zero failed, and 23,639 pending.
The MAC and Sign response TTLV accessors and the Continue batch-option
constructor contribute four local cells each across KMIP 2.0/2.1. The
Query Async Requests correlation builder contributes two KMIP 2.1 model cells.
The Opaque Object and PGP Key Register recipes each invoke the installed C
consumer across all 12 KMIP 2.0/2.1 encoding and transport combinations and
verify typed response access plus independent Destroy cleanup.
The evidence report passes schema validation, but the completeness gate
remains open because validator recipes are still pending.

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

The `test-op-poll` controlled route remains pending: the approved catalog does
not yet provide a complete request, response, correlation, and cleanup recipe
for it. No Poll recipe is inferred from the catalog's operation name alone.

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

Comparing the full report with the previous 43,035-event run by matrix cell
identity retained every prior cell and added exactly 62 cells for the single-
item default-Stop route. Catalog, route-manifest, installed-artifact, and
library-commit fingerprints match the baseline. CTest passes 7/7, including
191 route-matrix contract tests and both controlled TLS peers. The global
route-completeness gate remains open for the other 23,639 pending cells.

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

The subsequent C run `c-route-3df0ddc1-8a8a-4d90-aa50-addbab9194f4` is aligned
with the updated shared coverage and evidence schemas. It emitted 43,255
evidence rows: 43,239 actual invocations and 16 explicit N/A dispositions,
with zero failed invocations. The 66,736 planned C cells comprise those rows
and 23,481 pending recipe cells. The shared validator accepted the emitted
43,255-cell scope (`selected_scope_complete=true`,
`all_invocations_passed=true`, `complete_matrix=false`); the global gate remains
incomplete until the pending recipes are implemented.
