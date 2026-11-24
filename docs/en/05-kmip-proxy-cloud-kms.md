# 05. KMIPKit Cloud KMS Proxy Guide

Enterprise databases (MongoDB, MySQL, PostgreSQL, Oracle), hypervisors (VMware vSphere, Nutanix), and storage arrays (NetApp, Dell EMC, Pure Storage, Ceph) require **OASIS KMIP**—yet public cloud KMS services expose proprietary REST/gRPC APIs.

**`kmipkit-proxy`** is a **Zero-DB Stateless Sidecar** that translates KMIP 2.0, 2.1, and 3.0 (and KMIP 1.X when chained with the KMIP 1.X Bridge) into native calls against **10 cloud and enterprise KMS providers** without requiring any local database.

---

## 1. Core Architectural Pillars

```mermaid
flowchart LR
    APP[KMIP Client\nTTLV · JSON · XML\nRaw TLS or HTTPS] --> SRV[ProxyServer\nmTLS 1.3 / 1.2\nProtocol Sniffer]
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

### Pillar 1: Cloud Tag-Stuffing (`tag_codec.rs`)
Cloud KMS services do not natively store KMIP attributes (`State`, `Cryptographic Usage Mask`, `Activation Date`, `Protect Stop Date`, `Deactivation Date`, `Revocation Reason`, `Name`, `Link`). `TagCodec` deterministically serializes these into normalized, lowercase cloud tags/labels (`kmip_state`, `kmip_usage_mask`, `kmip_name_0`, etc.), using **RFC 4648 Base32Hex** chunked at `<= 60` characters to satisfy strict provider limits (such as GCP's 63-character label restriction).

### Pillar 2: Auto-Envelope Key Custody (`envelope.rs`)
Many legacy KMIP applications expect `Create` + `Get` to export raw AES-256 key bytes, whereas cloud KMS HSMs forbid exporting symmetric master keys. `kmipkit-proxy` supports three custody modes (`key_custody_mode`):

| Mode | Behavior | When to Use |
| :--- | :--- | :--- |
| **`native_kms`** | Keys stay strictly inside the cloud HSM. `Encrypt`/`Decrypt`/`Sign`/`Verify` execute remotely in the cloud KMS. `Get` returns public keys only. | Maximum HSM compliance; apps delegating crypto operations. |
| **`auto_envelope`** | Generates a random 256-bit DEK in zeroized RAM, wraps it with **AES-256-GCM** using a cloud Master KEK, and stores the encrypted blob in the provider's native Secret Store. `Get` unwraps on the fly. | Databases/storage arrays (MongoDB, vSphere, NetApp) that require exporting raw symmetric DEKs via KMIP `Get`. |
| **`auto`** (default) | Uses `auto_envelope` for exportable symmetric keys and `native_kms` for non-exportable/asymmetric HSM keys. | Mixed enterprise workloads. |

---

## 2. Supported Cloud & Enterprise Providers (10 Backends)

| Provider | Cargo Feature | Native KMS Service | Secret Store (Auto-Envelope) |
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

Build with all providers enabled:
```bash
cargo build --release --manifest-path kmip-proxy/Cargo.toml --features all-clouds
```

---

## 3. Configuration Examples (`proxy.toml`)

### Example A: AWS KMS + Secrets Manager

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

### Example B: HashiCorp Vault / OpenBao

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

### Running the Proxy

```bash
kmipkit-proxy --config /etc/kmip-proxy/proxy.toml
```
