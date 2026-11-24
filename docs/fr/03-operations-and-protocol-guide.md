# 03. Protocole et Opérations KMIP 2.0 / 2.1 / 3.0

KMIPKit cible **OASIS KMIP 2.1 par défaut**, avec support versionné pour **KMIP 2.0** et **KMIP 3.0** (incluant la cryptographie post-quantique NIST FIPS 203/204/205).

* **59 opérations complètes:** Toutes les opérations C2S (création, cycle de vie, certificats, attributs, chiffrement/signature/MAC/hash, KEM post-quantique `Encapsulate`/`Decapsulate` avec `ML-KEM-512/768/1024`, sessions, administration, contrôle asynchrone `Poll`/`Cancel`/`Process`) et S2C (`Discover Versions`, `Query`, `Notify`, `Put`, `Set Endpoint Role`).
* **Encodages:** `TTLV` (par défaut), `JSON` et `XML`, configurables globalement ou par requête via `RequestOptions`.
* **Lots Atomiques:** Exécution multi-opérations en 1 RTT avec `ClientBatch` et `UniqueIdentifier::id_placeholder()`.
* **Extensions Vendeur:** Respect strict de `Criticality Indicator` et registre immuable `ClientExtensionRegistry`.
