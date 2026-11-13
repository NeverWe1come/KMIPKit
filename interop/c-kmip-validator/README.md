# C interoperability consumer

This standalone C11 consumer links the public KMIPKit C ABI installed from the
exact `release/1.0.0` source commit pinned by
`interop/shared/coverage/route-capabilities.json`
(`9a16cc90a12205101d22a38501a5e74c665df4dc`). It targets KMIP 2.0 and 2.1 and
uses the shared 2,689-case catalog. A successful build or controlled fixture
does not assert compatibility with a deployed vendor KMS.

Build the exact library artifact and the consumer from the repository root:

```sh
python3 interop/c-kmip-validator/tools/install_artifact.py
python3 interop/c-kmip-validator/tools/verify_artifact.py
python3 -m pip install --only-binary ':all:' jsonschema==4.23.0
cmake -S interop/c-kmip-validator -B interop/c-kmip-validator/build \
  -DPython3_EXECUTABLE="$(command -v python3)" \
  -DKMIPKIT_C_INSTALL_DIR=interop/.local/c-live/install/9a16cc90a12205101d22a38501a5e74c665df4dc
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
TLS fixtures exercise Discover Versions, Ping, Locate, Query, Log, Login and
the ordered Create/Destroy, Create/Activate/Revoke/Destroy,
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
API limitation. The public C Login builder only supports KMIP 2.1, so its
KMIP 2.0 case is also reported as a public API limitation. The public C API exposes generic TTLV constructors, inspectors,
and a decoder, but it has no generic TTLV encoder and no generic TTLV request dispatch.
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
  --artifact-sha256 "$(sha256sum interop/.local/c-live/install/9a16cc90a12205101d22a38501a5e74c665df4dc/lib/libkmipkit_ffi.so | cut -d ' ' -f 1)"
```

Both completeness commands currently exit nonzero because semantic branch,
auxiliary model/accessor, cryptographic, asynchronous, and additional typed
construction recipes remain pending. CI deliberately fails on these gaps.
The `*.gaps.json` sidecar identifies every unexecuted matrix obligation rather
than misclassifying it as an unavailable library route. Other sidecars retain
installed-artifact availability checks and sanitized controlled-peer outcomes.
The evidence JSON passes the shared schema independently of completeness.

A generic TTLV wire encoder, generic-TTLV request dispatcher, and high-level
C client are absent from the exact installed public artifact. The harness's
independent vector serialization is never credited as a public C encoder.
Reserved tags can be represented by RawTag and rejected by the decoder/checked
conversion; construction requiring an already checked Tag remains an explicit
matrix-applicability question. Cross-language normalized-tree parity and vendor
interoperability remain unverified.
