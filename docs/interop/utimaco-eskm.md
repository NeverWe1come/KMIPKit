# Utimaco Enterprise Secure Key Manager (ESKM) Deployment & Interop Guide

This guide describes how to configure Utimaco ESKM for OASIS KMIP 2.1 interoperability and connect using KMIPKit over mutual TLS (mTLS).

---

## 1. Overview & Architecture

Utimaco ESKM is a hardware/virtual appliance providing centralized key management with enterprise-grade KMIP support.

| Parameter | Specification |
|---|---|
| **KMIP Version** | OASIS KMIP 2.1 (and legacy 1.x / 2.0) |
| **Port** | `5696` (raw TTLV over TLS 1.3) |
| **Authentication** | Mutual TLS with X.509 client certificate mapping |
| **Profile** | `interop/shared/profiles/utimaco-eskm.json` |

---

## 2. Certificate Preparation & Onboarding

Utimaco ESKM maps incoming client TLS certificates to local user accounts and access policies based on the certificate Subject or Common Name (`CN`).

### Step 1: Generate Client Private Key & CSR
```bash
# Generate EC P-256 key
openssl ecparam -name prime256v1 -genkey -noout -out eskm-kmipkit-client.key

# Generate CSR for client username 'kmipkit-interop'
openssl req -new -key eskm-kmipkit-client.key -out eskm-kmipkit-client.csr \
  -subj "/CN=kmipkit-interop/O=KMIPKit/OU=TestLab"
```

### Step 2: Sign Certificate with Enterprise or Test CA
```bash
openssl x509 -req -in eskm-kmipkit-client.csr -CA eskm-ca.crt -CAkey eskm-ca.key \
  -CAcreateserial -out eskm-kmipkit-client.crt -days 365
```

### Step 3: Register CA in Utimaco ESKM
1. Log in to the ESKM Administration Web Interface (`https://<eskm-ip>:8443`).
2. Navigate to **Security** &rarr; **Certificate Authorities**.
3. Click **Add CA** and upload `eskm-ca.crt`.
4. Ensure the CA is marked as **Trusted for KMIP Client Authentication**.

---

## 3. Configure KMIP User & Access Control

1. Navigate to **Users & Groups** &rarr; **Users** &rarr; **Add User**.
2. Set Username to `kmipkit-interop` (matching the Common Name in the client certificate).
3. Assign the user to the **KMIP Clients** group.
4. Ensure the group has permissions for:
   - `Symmetric Key`: Create, Get, Locate, Activate, Revoke, Destroy
   - `Asymmetric Key`: Create Key Pair, Get, Locate, Destroy
   - `Attributes`: Name, CryptographicAlgorithm, CryptographicLength, CryptographicUsageMask

---

## 4. KMIPKit Connection Configuration

Create `interop/shared/profiles/utimaco-connection.json`:

```json
{
  "server_identifier": "utimaco-eskm-5.5",
  "host": "eskm.lab.internal",
  "port": 5696,
  "transport": "raw_tls",
  "encoding": "TTLV",
  "tls": {
    "ca_certificate": "certs/eskm-server-ca.crt",
    "client_certificate": "certs/eskm-kmipkit-client.crt",
    "client_key": "certs/eskm-kmipkit-client.key",
    "verify_hostname": true,
    "min_version": "1.3"
  }
}
```

---

## 5. Execution Workflow

### 1. Preflight Network & TLS Audit
```bash
python3 interop/shared/tools/preflight_and_audit.py \
  --connection interop/shared/profiles/utimaco-connection.json \
  --profile interop/shared/profiles/utimaco-eskm.json
```

### 2. Live Capability Discovery
```bash
python3 interop/shared/tools/live_discover_capabilities.py \
  --connection interop/shared/profiles/utimaco-connection.json \
  --output interop/results/utimaco-discovered.json
```

### 3. Run Multi-Language Suite
```bash
python3 interop/shared/tools/run_all.py \
  --profile interop/shared/profiles/utimaco-eskm.json \
  --connection interop/shared/profiles/utimaco-connection.json \
  --languages rust,c,java,python
```

### 4. Cleanup Test Artifacts
```bash
python3 interop/shared/tools/kms_garbage_collector.py \
  --connection interop/shared/profiles/utimaco-connection.json \
  --prefix kmipkit- \
  --confirm
```
