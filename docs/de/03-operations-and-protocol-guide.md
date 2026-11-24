# 03. KMIP 2.0 / 2.1 / 3.0 Protokoll & Operationen

KMIPKit verwendet standardmäßig **OASIS KMIP 2.1** mit versionsgeprüfter Unterstützung für **KMIP 2.0** und **KMIP 3.0** (inklusive NIST-Post-Quantum-Kryptographie).

* **59 vollständige Operationen:** Alle C2S-Operationen (Erstellung, Lebenszyklus, Zertifikate, Attribute, Krypto, PQC-KEM `Encapsulate`/`Decapsulate` mit `ML-KEM-512/768/1024`, Sitzungen, Administration, asynchrones `Poll`/`Cancel`/`Process`) und S2C-Operationen (`Discover Versions`, `Query`, `Notify`, `Put`, `Set Endpoint Role`).
* **Kodierungen:** `TTLV` (Standard), `JSON` und `XML`, global oder pro Aufruf über `RequestOptions`.
* **Atomare Batches:** Mehrere abhängige Operationen in 1 RTT via `ClientBatch` und `UniqueIdentifier::id_placeholder()`.
* **Vendor-Extensions:** Normative Prüfung des `Criticality Indicator` und unveränderliche `ClientExtensionRegistry`.
