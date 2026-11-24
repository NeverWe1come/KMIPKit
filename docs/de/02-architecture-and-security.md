# 02. Architektur & Sicherheitsmodell

KMIPKit ist für regulierte Unternehmensumgebungen (Banken, Zahlungs-HSMs, souveräne Clouds) konzipiert.

* **Zwei strikt isolierte Cargo-Workspaces:** Kern-Client (`crates/`, 8 Crates) und eigenständiger Proxy (`kmip-proxy/`), sodass Cloud-SDK-Abhängigkeiten niemals in die Kernbibliothek gelangen.
* **TLS-Sicherheit:** Standardmäßig **TLS 1.3 Only**, optionales TLS 1.2 ausschließlich mit OASIS-KMIP-ECDHE-GCM-Suites, **obligatorisches mTLS**, Post-Quantum-Zertifikate (**ML-DSA-44/65/87**) über OpenSSL 3.5; 0-RTT und TLS Key Logging sind dauerhaft deaktiviert.
* **Speichersicherheit & Zeroization:** `#![forbid(unsafe_code)]` in allen Crates außer `kmipkit-ffi`; explizite Zeroization von Schlüsseln/Passwörtern beim Drop sowie vollständige Redaction (`[REDACTED]`) in Logs.
* **Keine stillen Retries:** Jeder Netzwerkfehler liefert einen deterministischen `RequestDeliveryState` (`NotSent`, `PossiblySent`, `ResponseStarted`).
* **FFI-Handle-Isolation:** Opake Handles über eine 8-Runden-Feistel-Permutation (HMAC-SHA-256 pro Prozess) sowie vollständiges `catch_unwind` an jeder `extern "C"`-Grenze.
