# C Binding Installation

## 1. Build `kmipkit-ffi` with Pinned OpenSSL

```bash
python3 verification/security/openssl/build.py
export OPENSSL_DIR="$HOME/.cache/kmipkit/openssl/3.5.9"
export OPENSSL_STATIC=1
cargo build --release -p kmipkit-ffi
```

## 2. CMake Integration

```cmake
find_package(KMIPKit REQUIRED)
add_executable(my_kmip_app main.c)
target_link_libraries(my_kmip_app PRIVATE KMIPKit::kmipkit_ffi)
```
