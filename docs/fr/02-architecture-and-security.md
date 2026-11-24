# 02. Architecture et Modèle de Sécurité

KMIPKit est conçu pour les environnements réglementés (banque, HSM de paiement, clouds souverains).

---

## 1. Deux Workspaces Cargo Strictement Isolés

* **Workspace Principal (`crates/`):** 8 crates (`kmipkit-ttlv`, `kmipkit-protocol`, `kmipkit-transport`, `kmipkit-client`, `kmipkit`, `kmipkit-ffi`, `kmipkit-test-support`, `kmipkit-test-server`).
* **Workspace Proxy (`kmip-proxy/`):** Complètement indépendant afin qu'aucun SDK cloud ne fuite dans la bibliothèque cliente principale.

---

## 2. Invariants de Sécurité Transport et Mémoire

* **TLS 1.3 par défaut** (`TlsVersionPolicy::Tls13Only`), compatibilité opt-in TLS 1.2 restreinte aux suites ECDHE-GCM OASIS KMIP, **mTLS obligatoire**, certificats post-quantiques **ML-DSA-44/65/87** via OpenSSL 3.5.
* **0-RTT et TLS Key Logging désactivés** de manière permanente.
* **Zéroïsation et Masquage (`[REDACTED]`):** `#![forbid(unsafe_code)]` partout hors `kmipkit-ffi`; zéroïsation explicite des clés et secrets à la libération.
* **Aucun réessai automatique:** Chaque erreur réseau retourne un `RequestDeliveryState` déterministe (`NotSent`, `PossiblySent`, `ResponseStarted`).
* **Isolation FFI:** Handles opaques permutés par un réseau de Feistel à 8 tours (HMAC-SHA-256 par processus) et capture systématique des panics (`catch_unwind`).
