# C interoperability consumer

This standalone C11 consumer links the public KMIPKit C ABI installed from
release commit `9a16cc90a12205101d22a38501a5e74c665df4dc`. It targets KMIP 2.0
and 2.1 and uses the shared 2,689-case catalog. A successful build or controlled
fixture does not assert compatibility with a deployed vendor KMS.

Build the exact library artifact and the consumer from the repository root:

```sh
python3 interop/c-kmip-validator/tools/install_artifact.py
python3 interop/c-kmip-validator/tools/verify_artifact.py
cmake -S interop/c-kmip-validator -B interop/c-kmip-validator/build \
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
