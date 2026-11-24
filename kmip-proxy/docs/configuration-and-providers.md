# Configuration & Cloud Provider Reference

`kmipkit-proxy` loads settings from an optional TOML configuration file (`--config`) merged with CLI flags (`CliArgs::merge_into`) and provider-specific environment variables. If `--config` is omitted, `ProxyConfig::default()` provides safe defaults (`127.0.0.1:5696`, TLS 1.3 + mTLS enabled, `mock` provider).

---

## 1. Command-Line Interface (`kmipkit-proxy`)

Defined in `src/config.rs` (`CliArgs`):

| Flag | Short | Type | TOML Override Target | Default / Effect |
| :--- | :---: | :--- | :--- | :--- |
| `--config` | `-c` | `PathBuf` | *(base TOML file)* | Uses `ProxyConfig::default()` if omitted |
| `--listen` | `-l` | `SocketAddr` | `listener.bind_addr` | `127.0.0.1:5696` |
| `--provider` | `-p` | `String` | `backend.provider` | `mock` |
| `--upstream-endpoint` | — | `String` | `backend.kmip_upstream.endpoint` | `127.0.0.1:5697` |
| `--upstream-version` | — | `String` | `backend.kmip_upstream.target_version` | `"2.1"` (`"2.0"`, `"2.1"`, `"3.0"`) |
| `--allow-tls12` | — | `bool` flag | `listener.allow_tls12` | Sets `true` (enables TLS 1.2 + 1.3) |
| `--master-kek-hex` | — | `String` | `backend.master_kek_hex` | None (64 hex chars = 256-bit key) |
| `--no-client-auth` | — | `bool` flag | `listener.require_client_cert` | Sets `false` *(dev/test only)* |
| `--no-tls` | — | `bool` flag | `listener.tls_enabled` | Sets `false` *(plain TCP, dev/test only)* |

---

## 2. Core TOML Sections (`[listener]`, `[proxy_behavior]`, `[backend]`)

### `[listener]` (`ListenerConfig`)

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `bind_addr` | `SocketAddr` | `"127.0.0.1:5696"` | TCP listen IP and port. |
| `tls_enabled` | `bool` | `true` | Enables TLS termination (`false` = plaintext TCP). |
| `allow_tls12` | `bool` | `false` | `false` = TLS 1.3 only; `true` = TLS 1.2 + TLS 1.3 for legacy KMIP 1.X clients. |
| `tls_cert_path` | `Option<PathBuf>` | `None` | Server certificate chain (PEM or DER). Omit for ephemeral `AutoPki`. |
| `tls_key_path` | `Option<PathBuf>` | `None` | Server private key (PEM or DER; PKCS#8, RSA, or EC). |
| `tls_client_ca_path` | `Option<PathBuf>` | `None` | Trusted client CA bundle (PEM or DER) when `require_client_cert = true`. |
| `require_client_cert` | `bool` | `true` | Enforces mutual TLS (mTLS) client certificates. |

### `[proxy_behavior]` (`ProxyBehaviorConfig`)

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `key_custody_mode` | `enum` | `"auto"` | `"auto"`, `"native_kms"`, or `"auto_envelope"`. |
| `metadata_cache_ttl_secs` | `u64` | `5` | TTL in seconds for `MetadataCache`. |

### `[backend]` (`BackendConfig`)

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `provider` | `String` | `"mock"` | Target backend: `"kmip-upstream"`, `"mock"`, `"aws"`, `"azure"`, `"gcp"`, `"oci"`, `"vault"` (`"openbao"`), `"infisical"`, `"ibm"`, `"alibaba"`, `"scaleway"`, `"otc"`. |
| `master_kek_hex` | `Option<String>` | `None` | Optional 64-character hex string (32 bytes) used as local AES-256-GCM wrapping KEK. |

> **CLI vs Library Dispatch Note:** In `src/main.rs`, the CLI binary directly initializes `"kmip-upstream"` (when feature `kmip1x-bridge` is enabled), `"mock"`, `"aws"`, `"azure"`, `"gcp"`, and `"oci"` (falling back to `InMemoryMockProvider` with a stderr warning for other provider names). All 10 cloud/enterprise providers (`vault`/`openbao`, `infisical`, `ibm`, `alibaba`, `scaleway`, `otc`, plus `aws`/`azure`/`gcp`/`oci`) are fully implemented in `kmipkit_proxy::providers::create_provider(&config.backend)`.

---

## 3. KMIP 1.X Bridge Upstream Section (`[backend.kmip_upstream]`)

Enabled when `provider = "kmip-upstream"` (default Cargo feature `kmip1x-bridge`):

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `endpoint` | `String` | `"127.0.0.1:5697"` | Target server (`"host:port"` for raw TLS or `"https://host:port/kmip"` for HTTP/HTTPS). |
| `target_version` | `String` | `"2.1"` | Target upstream KMIP version (`"2.0"`, `"2.1"`, or `"3.0"`). |
| `wire_encoding` | `Option<String>` | `Some("ttlv")` | Upstream encoding: `"ttlv"`, `"json"`, or `"xml"`. |
| `client_cert_path` | `Option<PathBuf>` | `None` | Upstream client certificate (PEM or DER) for mTLS. |
| `client_key_path` | `Option<PathBuf>` | `None` | Upstream client private key (PEM or DER). |
| `ca_cert_path` | `Option<PathBuf>` | `None` | CA certificate bundle (PEM or DER) to verify the upstream server. |
| `allow_tls12` | `bool` | `false` | Allows TLS 1.2 + TLS 1.3 on the upstream client connection. |
| `timeout_secs` | `Option<u64>` | `Some(30)` | Per-request upstream timeout in seconds. |
| `operation_policy_mode` | `enum` | `"ignore"` | `"ignore"` (drop deprecated `Operation Policy Name`), `"store_as_vendor_attribute"`, or `"reject"`. |
| `max_connections` | `Option<usize>` | `Some(8)` | Bounded concurrent connection pool size in `KmipKitUpstreamTransport`. |
| `template_store_path` | `Option<PathBuf>` | `None` | File path for atomic JSON persistence of KMIP 1.X `Template` objects across restarts (in-memory if unset). |

---

## 4. Cloud & Enterprise Backends (10 Providers)

### 1. AWS KMS + Secrets Manager (`--features aws`, `[backend.aws]`)

| Field / Env | Default | Description |
| :--- | :--- | :--- |
| `region` | `"us-east-1"` (or AWS SDK chain) | AWS region for KMS and Secrets Manager. |
| `endpoint_url` | AWS standard endpoint | Optional custom/LocalStack endpoint override. |
| `envelope_master_kek_id` | `None` | ARN or key ID of the KMS CMK used to wrap `auto_envelope` DEKs. |
| `secret_name_prefix` | `"kmipkit/"` | Prefix for envelope blobs stored in AWS Secrets Manager. |

Uses standard AWS SDK credential resolution (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, IRSA/IMDS/SSO).

### 2. Azure Key Vault / Managed HSM (`--features azure`, `[backend.azure]`)

| Field / Env | Default | Description |
| :--- | :--- | :--- |
| `vault_url` | `https://localhost:8443` *(if `endpoint_url` unset)* | Key Vault URI (`https://<name>.vault.azure.net`). |
| `endpoint_url` | `None` | Optional custom endpoint override. |
| `envelope_master_kek_name` | `None` | Key Vault key name used for `auto_envelope`. |
| `secret_name_prefix` | `"kmipkit/"` | Prefix for envelope secret names. |

Environment credentials: `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET` (or `DefaultAzureCredential` via `azure_identity`).

### 3. Google Cloud KMS + Secret Manager (`--features gcp`, `[backend.gcp]`)

| Field / Env | Default | Description |
| :--- | :--- | :--- |
| `project_id` | `"my-project"` | GCP Project ID. |
| `location` | `"global"` | Cloud KMS location (e.g., `europe-west1`, `global`). |
| `key_ring` | `"kmip-keyring"` | Cloud KMS KeyRing name. |
| `endpoint_url` | `https://cloudkms.googleapis.com` | Optional custom endpoint. |
| `envelope_master_kek_name` | `None` | KMS CryptoKey name used for `auto_envelope`. |
| `secret_name_prefix` | `"kmipkit/"` | Prefix for Cloud Secret Manager secrets. |
| `access_token` / `GOOGLE_OAUTH_ACCESS_TOKEN` | `None` | Explicit OAuth2 bearer token override. |

Note: `TagCodec` chunks names at `<= 60` chars using Base32Hex specifically to comply with GCP's strict 63-character label value limit.

### 4. Oracle Cloud Infrastructure Vault (`--features oci`, `[backend.oci]`)

| Field / Env | Default | Description |
| :--- | :--- | :--- |
| `compartment_id` | `None` | OCI Compartment OCID. |
| `vault_id` | `None` | OCI Vault OCID. |
| `management_endpoint` | `https://vaults.us-ashburn-1.oci.oraclecloud.com` | Vault management plane URL. |
| `crypto_endpoint` | `https://crypto.kms.us-ashburn-1.oci.oraclecloud.com` | Vault cryptographic plane URL. |
| `secrets_endpoint` | `None` | OCI Secrets service endpoint override. |
| `endpoint_url` | `None` | Unified mock/test endpoint override for all OCI planes. |
| `envelope_master_kek_id` | `None` | OCI master key OCID for `auto_envelope`. |
| `secret_name_prefix` | `"kmipkit/"` | Secret name prefix. |
| `auth_token` / `OCI_AUTH_TOKEN` | `None` | Auth token header value. |

### 5. HashiCorp Vault / OpenBao (`--features vault`, `[backend.vault]`)

| Field / Env | Default | Description |
| :--- | :--- | :--- |
| `endpoint_url` | `"http://127.0.0.1:8200"` | Vault/OpenBao API base URL. |
| `namespace` | `None` | Optional Vault Enterprise `X-Vault-Namespace`. |
| `transit_mount` | `"transit"` | Transit Secrets Engine mount path. |
| `kv_v2_mount` | `"secret"` | KV v2 Secrets Engine mount path (for `auto_envelope` blobs). |
| `item_prefix` | `"kmipkit/"` | Key/secret prefix. |
| `allow_transit_export` | `true` | Sets `exportable=true` on created Transit keys when applicable. |
| `auth_method` | `"token"` | `"token"` or `"approle"` (TOML schema). |
| `role_id`, `role_cred` | `None` | AppRole credentials. |
| `auth_token` / `VAULT_TOKEN` | `None` | Vault token (`X-Vault-Token`). |
| `envelope_master_kek_id` | `None` | Transit key ID used as KEK. |

### 6. Infisical KMS (`--features infisical`, `[backend.infisical]`)

| Field / Env | Default | Description |
| :--- | :--- | :--- |
| `site_url` | `"https://eu.infisical.com"` | Infisical instance URL (`app.infisical.com`, `eu.infisical.com`, or self-hosted). |
| `project_id` | `"default"` | Infisical project ID. |
| `client_id`, `client_cred` | `None` | Universal Auth Machine Identity credentials. |
| `auth_token` / `INFISICAL_TOKEN` | `None` | Static/cached bearer token. |
| `item_prefix` | `"kmipkit/"` | Secret/key prefix. |

### 7. IBM Cloud Key Protect (`--features ibm`, `[backend.ibm]`)

| Field / Env | Default | Description |
| :--- | :--- | :--- |
| `region` | `"us-south"` | IBM Cloud region (computes default `https://<region>.kms.cloud.ibm.com`). |
| `instance_id` | `"default-instance"` | Key Protect service instance GUID (`Bluemix-Instance`). |
| `endpoint_url` | `https://<region>.kms.cloud.ibm.com` | Key Protect base URL override. |
| `tagging_endpoint_url`, `search_endpoint_url` | `None` | Global Tagging / Global Search endpoint overrides. |
| `iam_endpoint_url` | `"https://iam.cloud.ibm.com/identity/token"` | IBM Cloud IAM token endpoint. |
| `iam_api_cred` / `IBMCLOUD_API_KEY` | `None` | IBM Cloud IAM API key. |
| `auth_token` / `IBM_AUTH_TOKEN` | `None` | Direct IAM bearer token override. |
| `default_extractable_for_get` | `true` | Marks created keys as Standard (extractable) vs Root Key when KMIP `Get` extractability applies. |
| `envelope_master_kek_id` | `None` | Key Protect Root Key ID for envelope wrapping. |

### 8. Alibaba Cloud KMS (`--features alibaba`, `[backend.alibaba]`)

| Field / Env | Default | Description |
| :--- | :--- | :--- |
| `region_id` | `"cn-hangzhou"` | Alibaba region ID (computes `https://kms.<region_id>.aliyuncs.com`). |
| `endpoint_url` | `https://kms.<region_id>.aliyuncs.com` | Custom KMS endpoint override. |
| `access_key_id` / `ALIBABA_CLOUD_ACCESS_KEY_ID` | `None` | Alibaba AccessKey ID. |
| `access_key_cred` / `ALIBABA_CLOUD_ACCESS_KEY_SECRET` | `None` | Alibaba AccessKey secret. |
| `auth_token` / `ALIBABA_AUTH_TOKEN` | `None` | Optional bearer/mock auth token. |
| `envelope_master_kek_id` | `None` | Master CMK ID for `auto_envelope`. |
| `item_prefix` | `"kmipkit/"` | Item prefix. |

### 9. Scaleway Key Manager (`--features scaleway`, `[backend.scaleway]`)

| Field / Env | Default | Description |
| :--- | :--- | :--- |
| `region` | `"fr-par"` | Scaleway region (`fr-par`, `nl-ams`, `pl-waw`). |
| `project_id` / `SCW_DEFAULT_PROJECT_ID` | `None` | Scaleway Project UUID. |
| `endpoint_url` | `"https://api.scaleway.com"` | Scaleway API endpoint. |
| `auth_token` / `SCW_SECRET_KEY` | `None` | Scaleway secret key (`X-Auth-Token`). |
| `envelope_master_kek_id` | `None` | Master key UUID for `auto_envelope`. |
| `item_prefix` | `"kmipkit/"` | Item prefix. |

### 10. Open Telekom Cloud KMS / DEW & CSMS (`--features otc`, `[backend.otc]`)

| Field / Env | Default | Description |
| :--- | :--- | :--- |
| `region` / `OTC_REGION` | `"eu-de"` | OTC region (`eu-de`, `eu-nl`, `eu-ch2`). |
| `project_id` / `OTC_PROJECT_ID` | `"default-otc-project"` | OTC Project ID. |
| `endpoint_url` / `OTC_ENDPOINT_URL` | `https://kms.<region>.otc.t-systems.com` | KMS/DEW endpoint override. |
| `access_key_id`, `access_key_cred` | `None` | AK/SK credentials. |
| `auth_token` / `OTC_AUTH_TOKEN` | `None` | IAM token override. |
| `envelope_master_kek_id` | `None` | DEW Master Key ID for `auto_envelope`. |
| `item_prefix` | `"kmipkit-"` | Item prefix *(note hyphen default rather than slash)*. |
