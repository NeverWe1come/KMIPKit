# PKCS #11 3.0 referenced by KMIP 2.0/2.1

KMIP §§11.38–11.39 (KMIP 2.1 numbering) define values by reference to the
OASIS PKCS #11 Cryptographic Token Interface Base Specification Version 3.0.
The checked-in headers below are the official OASIS work-product artifacts
linked by that specification. They are immutable input files; generators and
tests verify their SHA-256 digests and never fetch sources during CI.

Official source directory:

<https://docs.oasis-open.org/pkcs11/pkcs11-base/v3.0/os/include/pkcs11-v3.0/>

| File | Purpose | SHA-256 |
|---|---|---|
| `upstream/include/pkcs11.h` | Defines `CK_FUNCTION_LIST_3_0` layout and includes the ordered function declarations | `260bbef88360ddbc8f7f668622c688ed7e6dab939e1eb2fdd6392abb571637e5` |
| `upstream/include/pkcs11f.h` | Ordered `CK_PKCS11_FUNCTION_INFO` function list; KMIP encodes its 1-based member offset | `c52bdac9e30f982cb5ad57db946a8ff6076efea744a90d684835d7269babab9b` |
| `upstream/include/pkcs11t.h` | Defines all assigned `CK_RV` constants and `CKR_VENDOR_DEFINED` | `1f6763a72dc33a97d964d3d4b860870e1b4d4bf76b8705bdee26078caf4994cc` |

The interop inventory tests every named function offset and every assigned
return-code value. `CKR_VENDOR_DEFINED` is tested as an allocation boundary;
vendor-specific return-code semantics remain outside this project.
