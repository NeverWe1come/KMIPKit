# Portail de Documentation KMIPKit (Français)

> **Version :** 1.0.0 | **Protocoles Supportés :** OASIS KMIP 2.0, 2.1 (par défaut), 3.0 + Pont Legacy KMIP 1.0–1.4  
> **Langages :** Rust · C (ABI C11) · Java (JNI) · Python (CFFI)  
> **Encodages :** TTLV (par défaut) · JSON · XML | **Transports :** Raw TLS 1.3/1.2 · HTTPS (HTTP/1.1)

**Langue / Language / Idioma / Sprache :** [English](../en/00-index.md) · [Español](../es/00-index.md) · **Français** · [Deutsch](../de/00-index.md)

---

## Vue d'Ensemble Exécutive

**KMIPKit** est une suite cryptographique open-source **OASIS KMIP** conçue pour la production et sûre en mémoire, articulée autour d'un unique noyau Rust faisant autorité. Elle élimine toute dérive protocolaire dans les environnements polyglottes en exposant une **implémentation unifiée et auditée** pour Rust, C, Java et Python — accompagnée d'un **Proxy Stateless sans Base de Données (Zero-DB)** reliant les **API Cloud KMS** ainsi que les **applications legacy KMIP 1.X** aux serveurs modernes KMIP 2.0, 2.1 et 3.0.

---

## Plan de la Documentation

| Guide | Public | Contenu Principal |
| :--- | :--- | :--- |
| **[01. Démarrage Rapide](01-getting-started.md)** | Développeurs, DevOps | Compilation avec OpenSSL épinglé, configuration PKI mTLS et cycle de vie complet en **Rust, C, Java et Python**. |
| **[02. Architecture et Modèle de Sécurité](02-architecture-and-security.md)** | Architectes Sécurité, Auditeurs | Workspace de 8 crates, 3 niveaux d'API, politique mTLS TLS 1.3/1.2, zéroïsation, masquage des secrets, `RequestDeliveryState` et isolation des handles FFI. |
| **[03. Protocole et Opérations KMIP 2.0 / 2.1 / 3.0](03-operations-and-protocol-guide.md)** | Ingénieurs Cryptographie | Référence des 59 opérations C2S/S2C, lots atomiques, `ID Placeholder`, opérations asynchrones (`Poll`/`Cancel`/`Process`), extensions vendeur et PQC KMIP 3.0 (`ML-KEM`, `ML-DSA`, `SLH-DSA`). |
| **[04. Référence des Bindings Multi-Langages](04-language-bindings-reference.md)** | Équipes Rust / C / Java / Python | Parité fonctionnelle, gestion mémoire, snapshots d'erreurs, Gradle/Maven, CMake et Python CFFI. |
| **[05. Guide du Proxy Cloud KMS](05-kmip-proxy-cloud-kms.md)** | Ingénieurs Cloud & Plateforme | Sidecar Stateless Zero-DB, *Cloud Tag-Stuffing*, chiffrement *Auto-Envelope* AES-256-GCM et 10 fournisseurs Cloud/Enterprise KMS. |
| **[06. Guide du Pont Legacy KMIP 1.X](06-kmip1x-legacy-bridge.md)** | Équipes Modernisation & IAM | Traduction transparente KMIP 1.0–1.4 vers KMIP 2.0/2.1/3.0, 77 attributs, émulation `Template`, *Read-Modify-Write* `Attribute Index` et pont S2C. |
| **[07. Intégrations Entreprise et Déploiement](07-enterprise-integrations-and-deployment.md)** | SRE, DBA, Admin Stockage | Playbooks MongoDB, MySQL, VMware vSphere, NetApp ONTAP, sidecar Kubernetes distroless et diagnostic. |
