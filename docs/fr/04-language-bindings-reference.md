# 04. Référence des Bindings Multi-Langages (Rust · C · Java · Python)

KMIPKit assure une **parité fonctionnelle 1:1** entre Rust, C (ABI C11), Java (JNI) et Python (CFFI), vérifiée par un manifeste d'API commun (`verification/api/`).

* **Rust (`kmipkit`):** RAII `Drop` + `Zeroize`, `Result<T, ClientError>` avec `RequestDeliveryState`, validation locale `validation_report()`.
* **C (`kmipkit.h`):** Structures versionnées (`struct_size`, `abi_version = 1`), destructeurs dédiés `kmipkit_*_free()`, snapshots d'erreur par thread via `kmipkit_last_error_get_v1()`.
* **Java (`org.kmipkit`):** `AutoCloseable` (`try-with-resources`), exceptions typées (`ProtocolException`, `TransportException`, `ServerResultException`), builders fluides.
* **Python (`kmipkit`):** Gestionnaires de contexte (`with Client(...) as client:`), exceptions typées, chargement natif configurable via `KMIPKIT_NATIVE_LIB`.
