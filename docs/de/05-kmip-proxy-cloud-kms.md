# 05. Cloud KMS Proxy Leitfaden (`kmipkit-proxy`)

**`kmipkit-proxy`** ist ein **Zero-DB Stateless Sidecar**, der KMIP 2.0, 2.1 und 3.0 (sowie KMIP 1.X über die 1.X-Bridge) in native API-Aufrufe für **10 Cloud- und Enterprise-KMS-Anbieter** übersetzt:

* **AWS**, **Azure**, **Google Cloud**, **Oracle OCI**, **HashiCorp Vault / OpenBao**, **Infisical**, **IBM Cloud Key Protect**, **Alibaba Cloud KMS**, **Scaleway Key Manager** und **Open Telekom Cloud**.
* **Cloud Tag-Stuffing (`tag_codec.rs`):** Deterministische Speicherung von KMIP-Attributen in Cloud-Tags/Labels (RFC 4648 Base32Hex, Blöcke `<= 60` Zeichen).
* **Auto-Envelope (`envelope.rs`):** Modi `native_kms`, `auto_envelope` (AES-256-GCM im zeroisierten RAM) und `auto`.
