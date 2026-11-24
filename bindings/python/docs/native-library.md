# Python Native Library Handling

The wheel bundles `libkmipkit_ffi` for Linux, macOS, and Windows. To point to a custom build:

```bash
export KMIPKIT_NATIVE_LIB=/opt/kmipkit/libkmipkit_ffi.so
```

Always use `with Client(config, transport) as client:` so native handles and secret buffers are zeroized on exit.
