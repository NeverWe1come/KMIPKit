# 05. Guide du Proxy Cloud KMS (`kmipkit-proxy`)

**`kmipkit-proxy`** est un **Sidecar Stateless sans Base de Données (Zero-DB)** traduisant KMIP 2.0, 2.1 et 3.0 (ainsi que KMIP 1.X via le pont 1.X) vers **10 fournisseurs Cloud et Enterprise KMS** :

* **AWS** (KMS + Secrets Manager), **Azure** (Key Vault Keys + Secrets), **Google Cloud** (Cloud KMS + Secret Manager), **Oracle OCI** (Vault + Secrets), **HashiCorp Vault / OpenBao** (Transit + KV v2), **Infisical**, **IBM Cloud Key Protect**, **Alibaba Cloud KMS**, **Scaleway Key Manager** et **Open Telekom Cloud (DEW/CSMS)**.
* **Cloud Tag-Stuffing (`tag_codec.rs`):** Sérialisation déterministe des attributs KMIP dans les tags cloud (RFC 4648 Base32Hex en blocs `<= 60` caractères).
* **Auto-Envelope (`envelope.rs`):** Modes `native_kms`, `auto_envelope` (AES-256-GCM en RAM zéroïsée) et `auto`.
