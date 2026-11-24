# Testing the C Bindings

```bash
cmake -S bindings/c -B bindings/c/build -DKMIPKIT_FFI_LIB_DIR=target/release
cmake --build bindings/c/build
ctest --test-dir bindings/c/build --output-on-failure
```
