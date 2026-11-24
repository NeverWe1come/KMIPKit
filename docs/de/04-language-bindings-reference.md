# 04. Sprach-Bindings-Referenz (Rust · C · Java · Python)

KMIPKit bietet **1:1-Funktionsparität** zwischen Rust, C (C11 ABI), Java (JNI) und Python (CFFI), abgesichert durch ein gemeinsames API-Manifest (`verification/api/`).

* **Rust (`kmipkit`):** RAII `Drop` + `Zeroize`, `Result<T, ClientError>` mit `RequestDeliveryState`, lokale Vorabprüfung via `validation_report()`.
* **C (`kmipkit.h`):** Versionierte Strukturen (`struct_size`, `abi_version = 1`), dedizierte `kmipkit_*_free()`-Destruktoren, Thread-lokale Fehler-Snapshots über `kmipkit_last_error_get_v1()`.
* **Java (`org.kmipkit`):** `AutoCloseable` (`try-with-resources`), typisierte Exceptions, Fluent Builders.
* **Python (`kmipkit`):** Context Manager (`with Client(...) as client:`), typisierte Exceptions, konfigurierbarer Bibliothekspfad über `KMIPKIT_NATIVE_LIB`.
