# 05. KMIPKit Cloud KMS Proxy Leitfaden

Unternehmensdatenbanken (MongoDB, MySQL, PostgreSQL, Oracle), Hypervisoren (VMware vSphere, Nutanix) und Speicher-Arrays (NetApp, Dell EMC, Pure Storage, Ceph) verlangen **OASIS KMIP** — öffentliche Cloud-KMS-Dienste bieten jedoch proprietäre REST/gRPC-APIs.

**`kmipkit-proxy`** ist ein **Zero-DB Stateless Sidecar**, der KMIP 2.0, 2.1 und 3.0 (sowie KMIP 1.X über die Legacy-Bridge) direkt in Aufrufe für **10 Cloud- und Enterprise-KMS-Anbieter** übersetzt.

---

## 1. Zentrale Architektursäulen

```mermaid
flowchart LR
    APP[KMIP Client\nTTLV · JSON · XML\nRaw TLS oder HTTPS] --> SRV[ProxyServer\nmTLS 1.3 / 1.2\nProtokoll-Sniffer]
    SRV --> DISP[RequestDispatcher\nMetadataCache TTL]
    DISP --> TAG[TagCodec\nKMIP Attrs <-> Cloud Tags]
    DISP --> ENV[AutoEnvelope\nAES-256-GCM + Zeroize]
    DISP --> PROV[CloudKmsProvider Trait]
    PROV --> P1[AWS KMS + Secrets Manager]
    PROV --> P2[Azure Key Vault]
    PROV --> P3[Google Cloud KMS + Secret Manager]
    PROV --> P4[Oracle OCI Vault]
    PROV --> P5[HashiCorp Vault / OpenBao]
    PROV --> P6[IBM · Alibaba · Scaleway · OTC · Infisical]
```

### Säule 1: Cloud Tag-Stuffing (`tag_codec.rs`)
Cloud-KMS-Dienste speichern nativ keine KMIP-Attribute (`State`, `Cryptographic Usage Mask`, `Activation Date`, `Name`, `Link` usw.). `TagCodec` serialisiert diese deterministisch in normierte Tags/Labels (`kmip_state`, `kmip_usage_mask`, `kmip_name_0` usw.) mit **RFC 4648 Base32Hex** (`<= 60` Zeichen pro Chunk, z. B. für GCPs 63-Zeichen-Limit).

### Säule 2: Auto-Envelope Key Custody (`envelope.rs`)
Viele Legacy-Systeme erwarten, dass `Create` + `Get` rohe AES-256-Schlüsselbytes exportiert. Drei Betriebsmodi (`key_custody_mode`) stehen bereit:

| Modus | Verhalten | Anwendungsfall |
| :--- | :--- | :--- |
| **`native_kms`** | Schlüssel verbleiben im Cloud-HSM. `Encrypt`/`Decrypt`/`Sign`/`Verify` laufen im Cloud-KMS. `Get` liefert nur öffentliche Schlüssel. | Maximale HSM-Compliance. |
| **`auto_envelope`** | Erzeugt einen 256-Bit-DEK im Zeroized RAM, verpackt ihn mit **AES-256-GCM** unter einer Master KEK und speichert den Blob im Secret Store. `Get` entpackt dynamisch. | Datenbanken/Arrays (MongoDB, vSphere, NetApp), die DEK-Export via `Get` verlangen. |
| **`auto`** (Standard) | `auto_envelope` für exportierbare symmetrische Schlüssel; `native_kms` für nicht-exportierbare/asymmetrische HSM-Schlüssel. | Gemischte Enterprise-Workloads. |

---

## 2. Unterstützte Cloud- & Enterprise-Anbieter (10 Backends)

| Anbieter | Cargo Feature | Nativer KMS-Dienst | Secret Store (Auto-Envelope) |
| :--- | :--- | :--- | :--- |
| **AWS** | `--features aws` | AWS KMS | AWS Secrets Manager |
| **Azure** | `--features azure` | Azure Key Vault Keys / Managed HSM | Azure Key Vault Secrets |
| **Google Cloud** | `--features gcp` | Cloud KMS | Cloud Secret Manager |
| **Oracle OCI** | `--features oci` | OCI Vault (Management + Crypto) | OCI Vault Secrets |
| **HashiCorp Vault / OpenBao** | `--features vault` | Transit Secrets Engine | KV v2 Secrets Engine |
| **Infisical** | `--features infisical` | Infisical KMS | Infisical Secrets |
| **IBM Cloud** | `--features ibm` | IBM Key Protect | IBM Key Protect Standard Keys |
| **Alibaba Cloud** | `--features alibaba` | Alibaba Cloud KMS | Alibaba Secrets Manager |
| **Scaleway** | `--features scaleway` | Scaleway Key Manager | Scaleway Secret Manager |
| **Open Telekom Cloud** | `--features otc` | OTC KMS (DEW) | OTC CSMS |

Build mit allen Anbietern:
```bash
cargo build --release --manifest-path kmip-proxy/Cargo.toml --features all-clouds
```

---

## 3. Konfigurationsbeispiele (`proxy.toml`)

### Beispiel A: AWS KMS + Secrets Manager

```toml
[listener]
bind_addr = "0.0.0.0:5696"
tls_enabled = true
require_client_cert = true
tls_cert_path = "/etc/kmip-proxy/server.crt"
tls_key_path = "/etc/kmip-proxy/server.key"
tls_client_ca_path = "/etc/kmip-proxy/client-ca.crt"

[proxy_behavior]
key_custody_mode = "auto"
metadata_cache_ttl_secs = 5

[backend]
provider = "aws"

[backend.aws]
region = "eu-west-1"
envelope_master_kek_id = "arn:aws:kms:eu-west-1:123456789012:key/mrk-1234abcd"
secret_name_prefix = "kmipkit/"
```

### Beispiel B: HashiCorp Vault / OpenBao

```toml
[listener]
bind_addr = "0.0.0.0:5696"
tls_enabled = true
require_client_cert = true

[backend]
provider = "vault"

[backend.vault]
endpoint_url = "https://vault.internal.example.com:8200"
namespace = "prod-kms"
transit_mount = "transit"
kv_v2_mount = "secret"
item_prefix = "kmipkit/"
allow_transit_export = true
auth_method = "approle"
role_id = "your-approle-id"
role_cred = "your-approle-secret"
```

### Proxy starten

```bash
kmipkit-proxy --config /etc/kmip-proxy/proxy.toml
```
