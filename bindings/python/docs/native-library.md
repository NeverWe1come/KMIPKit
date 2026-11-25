# Python Native Library Handling

Maturin builds the CFFI declarations and native library under `kmipkit._native_ffi`.
The loader selects a single adjacent regular `.dll`, `.dylib`, or `.so` file
(symlinks and ambiguous libraries are rejected) and verifies `kmipkit_version_get`
returns ABI version `1`.

## Administrator Override

To point explicitly to a trusted native library build by absolute path:

```sh
export KMIPKIT_NATIVE_LIBRARY=/opt/kmipkit/libkmipkit_ffi.so
```

Setting `KMIPKIT_NATIVE_LIBRARY` replaces bundled-library selection without
falling back to the bundled binary. Compiled CFFI declarations in
`kmipkit._native_ffi` remain required at runtime.

## Resource Ownership

Always manage client and secret objects via context managers or explicit `close()`
calls so native handles and secret buffers are freed and zeroized deterministically:

```python
with Client(config, transport) as client:
    ...
```

See the binding architecture (`docs/architecture/ffi-and-bindings.md`)
and loader implementation (`bindings/python/src/kmipkit/_native.py`).
