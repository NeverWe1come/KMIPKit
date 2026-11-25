# KMIPKit Language Bindings

KMIPKit provides **1:1 functional parity** across **C (C11 ABI)**, **Java (JNI)**, and **Python (CFFI)**, all backed by the same authoritative, memory-safe Rust core (`kmipkit-ffi`).

| Language | Adapter Technology | Minimum Runtime | Entrypoint |
| :--- | :--- | :--- | :--- |
| **[C](c/README.md)** | Stable C11 ABI (`kmipkit.h`, `kmipkit_fluent.h`) | C11 / CMake 3.20+ | `libkmipkit_ffi.{so,dylib,dll}` |
| **[Java](java/README.md)** | JNI C++ Bridge (`kmipkit_jni`) | JDK 17+ | `org.kmipkit:kmipkit-java:1.0.0` |
| **[Python](python/README.md)** | CFFI Native Adapter | Python 3.12 | `pip install kmipkit` |

## Shared Guarantees Across All Bindings

* **Zero Protocol Drift:** TTLV/JSON/XML framing, OASIS KMIP 2.0/2.1/3.0 validation, and TLS 1.3/1.2 mTLS live exclusively in Rust.
* **Opaque Permuted Handles:** Every native handle is an 8-round Feistel-permuted token backed by a live-handle table (prevents use-after-free and forged pointers).
* **Panic Containment:** Every FFI boundary catches Rust panics and maps them to `KMIPKIT_ERROR_INTERNAL (7)` without unwinding into C/JVM/Python.
* **Zeroization & Redaction:** Secret buffers are zeroized on free/close and redacted (`[REDACTED]`) from all diagnostics.
