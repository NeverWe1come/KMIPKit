# KMIPKit Dokumentationsportal (Deutsch)

> **Version:** 1.0.0 | **Protokollunterstützung:** OASIS KMIP 2.0, 2.1 (Standard), 3.0 + Legacy KMIP 1.0–1.4 Bridge  
> **Sprachen:** Rust · C (C11 ABI) · Java (JNI) · Python (CFFI)  
> **Kodierungen:** TTLV (Standard) · JSON · XML | **Transporte:** Raw TLS 1.3/1.2 · HTTPS (HTTP/1.1)

**Sprache / Language / Idioma / Langue:** [English](../en/00-index.md) · [Español](../es/00-index.md) · [Français](../fr/00-index.md) · **Deutsch**

---

## Überblick

**KMIPKit** ist eine produktionsreife, speichersichere Open-Source-Suite für **OASIS KMIP**, aufgebaut um einen einzigen maßgeblichen Rust-Kern. Sie verhindert Protokoll-Drift in polyglotten Unternehmensumgebungen durch eine **einheitliche, spezifikationsgeprüfte Implementierung** für Rust, C, Java und Python — ergänzt durch einen **Zero-DB Stateless Proxy**, der sowohl **Cloud-KMS-APIs** als auch **Legacy-KMIP-1.X-Anwendungen** transparent an moderne KMIP-2.0-, 2.1- und 3.0-Server anbindet.

---

## Dokumentationsübersicht

| Leitfaden | Zielgruppe | Inhalt |
| :--- | :--- | :--- |
| **[01. Schnelleinstieg](01-getting-started.md)** | Entwickler, DevOps | Build mit gepinntem OpenSSL 3.5.9, mTLS-PKI-Setup und vollständiger Schlüssel-Lebenszyklus in **Rust, C, Java und Python**. |
| **[02. Architektur & Sicherheitsmodell](02-architecture-and-security.md)** | Sicherheitsarchitekten, Auditoren | 8-Crate-Workspace, 3 API-Ebenen, TLS-1.3/1.2-mTLS-Richtlinie, Zeroization, Redaction, `RequestDeliveryState` und FFI-Handle-Isolation. |
| **[03. KMIP 2.0 / 2.1 / 3.0 Protokoll & Operationen](03-operations-and-protocol-guide.md)** | Kryptographie-Ingenieure | Alle 59 C2S/S2C-Operationen, atomare Batches, `ID Placeholder`, asynchrone Operationen (`Poll`/`Cancel`/`Process`), Vendor-Extensions und KMIP-3.0-PQC (`ML-KEM`, `ML-DSA`, `SLH-DSA`). |
| **[04. Sprach-Bindings-Referenz](04-language-bindings-reference.md)** | Rust / C / Java / Python Teams | 1:1-Funktionsparität, Speicherbesitz, Fehler-Snapshots, CMake, Gradle/Maven und Python CFFI. |
| **[05. Cloud KMS Proxy Leitfaden](05-kmip-proxy-cloud-kms.md)** | Cloud- & Plattform-Ingenieure | Zero-DB Stateless Sidecar, *Cloud Tag-Stuffing*, *Auto-Envelope* AES-256-GCM und 10 Cloud-/Enterprise-KMS-Anbieter. |
| **[06. KMIP 1.X Legacy Bridge Leitfaden](06-kmip1x-legacy-bridge.md)** | Modernisierungs- & IAM-Teams | Transparente Übersetzung KMIP 1.0–1.4 zu KMIP 2.0/2.1/3.0, 77 Attribute, `Template`-Emulation, *Read-Modify-Write* für `Attribute Index` und S2C-Bridge. |
| **[07. Enterprise-Integrationen & Betrieb](07-enterprise-integrations-and-deployment.md)** | SREs, DBAs, Storage-Admins | Playbooks für MongoDB, MySQL, VMware vSphere, NetApp ONTAP, Kubernetes-Distroless-Sidecar und Fehlerbehebung. |
