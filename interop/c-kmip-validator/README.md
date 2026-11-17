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
Delegated Login, Logout, and Interop. Login, Delegated Login, and the
Ticket-backed Logout recipe run under both KMIP 2.0 and 2.1. Login and
Delegated Login check their catalogued request fields and the typed Ticket;
the Login and Delegated Login responses also preserve the generic TTLV Ticket
structure. Logout returns that server-issued Ticket and checks the empty
response payload. Interop sends Begin, End, and Reset in an ordered batch with
distinct batch-item IDs and checks each successful empty response.
The ordered Create/Destroy, Create/Activate/Revoke/Destroy,
Create/Check/Revoke/Destroy, Create/Revoke/Destroy, Create/Destroy/Revoke/Destroy,
Create/Get/Destroy, Create/Get Attributes/Destroy, Create/Get Attribute
List/Destroy and Create/Get Constraints/Revoke/Destroy lifecycles under KMIP
2.1. Under KMIP 2.0 they exercise Login, Logout, Locate, Query, Log and all nine lifecycle
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
material without including the key bytes in reports. The public C Discover
Versions builder emits a fixed KMIP 2.1 offer, which the controlled fixture
exercises under both KMIP 2.0 and 2.1 message headers. With the KMIP 2.0
header, the fixture returns the empty intersection between that offer and its
supported versions. Login and the typed/TTLV Ticket readers are exercised for
both protocol versions. The public C API exposes generic TTLV constructors,
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
The latest full local C matrix run on the integrated Archive stack at
`055fa2715` has 66,692 expected cells, 43,372 invoked, zero failed, and 23,320
pending. The Archive lifecycle adds 38 invoked cells. Follow-up commits add
focused response-item-ID correlation coverage and do not change that matrix
snapshot; the exhaustive gate remains open.

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
establish cryptographic validity of the supplied MAC.

The `test-op-sign` and `test-op-signature-verify` routes create an RSA-2048 key
pair with separate usage masks: Sign on the Private Key and Verify on the Public
Key. Each ordered batch activates the selected key, executes the operation, then
revokes and destroys the selected key and destroys its pair. The request uses
the explicit SHA-256 with RSA signature algorithm. The peer checks the full
request tree and typed response shape. Create generates a random pair, so the
fixed signature inputs and response do not establish cryptographic validity.

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
certificate by ID Placeholder before revoking and destroying both keys. It
also checks that the operation's generic response TTLV contains the same
certificate identifier. The
controlled peer verifies the request tree, signer lifecycle, placeholder
mapping, typed results, and final state. This validates KMIP request/response
shape and lifecycle only; the fixture does not validate a real certificate or
signature.

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
