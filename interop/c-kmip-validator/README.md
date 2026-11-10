# C interoperability consumer

This standalone C11 consumer links the public KMIPKit C ABI installed from
release commit `9692057c9ade474fb0c4e0b7130539da368eb794`. It targets KMIP 2.0
and 2.1 and uses the shared 218-case catalog. A successful build or controlled
fixture does not assert compatibility with a deployed vendor KMS.

Build the exact library artifact and the consumer from the repository root:

```sh
python3 interop/c-kmip-validator/tools/install_artifact.py
python3 interop/c-kmip-validator/tools/verify_artifact.py
cmake -S interop/c-kmip-validator -B interop/c-kmip-validator/build \
  -DKMIPKIT_C_INSTALL_DIR=interop/.local/c-live/install/9692057c9ade474fb0c4e0b7130539da368eb794
cmake --build interop/c-kmip-validator/build
ctest --test-dir interop/c-kmip-validator/build --output-on-failure
```

The public C API exposes typed request builders plus generic TTLV construction
and navigation. It does not currently expose a high-level client API or a
generic TTLV request dispatch path; the consumer records those route limits as
library/API gaps. Live runs require a dedicated test KMS, valid mutual-TLS
credentials and explicit opt-in for mutations.
