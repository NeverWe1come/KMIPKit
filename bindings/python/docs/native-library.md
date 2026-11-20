# Python native library

Maturin builds the CFFI declarations and native library under the installed
`kmipkit._native_ffi` package. The binding uses CFFI to call the shared Rust C ABI.
The loader selects a single adjacent regular `.dll`, `.dylib`, or `.so` file;
missing or ambiguous files and symlinks are rejected. It checks
`kmipkit_version_get` and requires ABI version 1 before exposing the library.

The explicit administrator override `KMIPKIT_NATIVE_LIBRARY` selects a native
library by absolute path. It replaces bundled-library selection. An invalid
or incompatible override fails without falling back to the bundled library.
Use it only for a trusted, matching build. CFFI declarations from the installed
package remain required even when overriding the native library.

For import failures, check that Python 3.12 is active, the binding was installed
through its build backend, and the library matches the interpreter's platform
and architecture. Copying the Python sources alone does not install the generated
CFFI module or library. Check and remove unintended administrator overrides.

Use context managers or explicit close methods for owned native handles.
Python strings and bytes can outlive native owners and require application-level
secret handling. See the [binding architecture](../../../docs/architecture/ffi-and-bindings.md)
and [loader implementation](../src/kmipkit/_native.py).
