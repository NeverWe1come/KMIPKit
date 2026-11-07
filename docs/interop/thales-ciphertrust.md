# Thales CipherTrust Manager Deployment & Interop Guide

This guide details how to configure Thales CipherTrust Manager for OASIS KMIP 2.1 interoperability and run the KMIPKit test suite over mutual TLS.

---

## 1. Overview & Architecture

Thales CipherTrust Manager (formerly Vormetric / KeySecure) provides unified data security, enterprise key management, and OASIS KMIP 2.1 compliance.

| Parameter | Specification |
|---|---|
| **KMIP Version** | OASIS KMIP 2.1 |
| **Port** | `5696` (raw TTLV over TLS 1.3) |
| **Authentication** | Registration Authority (RA) / Local CA signed client certificate |
| **Profile** | `interop/shared/profiles/thales-ciphertrust.json` |

---

## 2. Setting Up KMIP Interface & Local CA

### Step 1: Verify KMIP Interface is Active
1. In the CipherTrust Manager GUI, go to **Admin Settings** &rarr; **Interfaces**.
2. Locate the **KMIP** interface:
   - Ensure Status is **Active**.
   - Port is set to `5696`.
   - Mode is set to **TLS, verify client cert**.

### Step 2: Configure Local CA & Registration Token
1. Go to **CA** &rarr; **Local CAs** and create or select a CA for KMIP (e.g. `KMIP-Local-CA`).
2. Go to **Keys & Access Management** &rarr; **Registration Tokens** &rarr; **Create Registration Token**:
   - Name: `kmipkit-token`
   - CA: `KMIP-Local-CA`
   - Max Uses: 10
   - Lifetime: 7 days
3. Copy the generated registration token string.

---

## 3. Client Certificate Enrollment

Use the CipherTrust REST API or CLI to enroll the KMIPKit client certificate using the registration token:

```bash
# 1. Generate Client Key and CSR
openssl ecparam -name prime256v1 -genkey -noout -out ciphertrust-client.key
openssl req -new -key ciphertrust-client.key -out ciphertrust-client.csr \
  -subj "/CN=kmipkit-client"

# 2. Submit CSR to CipherTrust Manager using Registration Token
CSR_CONTENT=$(cat ciphertrust-client.csr | tr -d '\r')
curl -k -X POST "https://<ciphertrust-host>/api/v1/ca/tokens/<REGISTRATION_TOKEN>/sign" \
  -H "Content-Type: application/json" \
  -d "{\"csr\": \"$CSR_CONTENT\"}" \
  -o ciphertrust-client.crt

# 3. Download CipherTrust KMIP Server CA
curl -k -X GET "https://<ciphertrust-host>/api/v1/ca/local-cas/KMIP-Local-CA/certificate" \
  -o ciphertrust-ca.crt
```

---

## 4. KMIPKit Connection Configuration

Create `interop/shared/profiles/ciphertrust-connection.json`:

```json
{
  "server_identifier": "thales-ciphertrust-2.14",
  "host": "ciphertrust.lab.internal",
  "port": 5696,
  "transport": "raw_tls",
  "encoding": "TTLV",
  "tls": {
    "ca_certificate": "certs/ciphertrust-ca.crt",
    "client_certificate": "certs/ciphertrust-client.crt",
    "client_key": "certs/ciphertrust-client.key",
    "verify_hostname": true,
    "min_version": "1.3"
  }
}
```

---

## 5. Execution Workflow

### 1. Preflight Verification
```bash
python3 interop/shared/tools/preflight_and_audit.py \
  --connection interop/shared/profiles/ciphertrust-connection.json \
  --profile interop/shared/profiles/thales-ciphertrust.json
```

### 2. Live Capability Discovery
```bash
python3 interop/shared/tools/live_discover_capabilities.py \
  --connection interop/shared/profiles/ciphertrust-connection.json \
  --output interop/results/ciphertrust-discovered.json
```

### 3. Run Multi-Language Suite
```bash
python3 interop/shared/tools/run_all.py \
  --profile interop/shared/profiles/thales-ciphertrust.json \
  --connection interop/shared/profiles/ciphertrust-connection.json \
  --languages rust,c,java,python
```

### 4. Cleanup Test Artifacts
```bash
python3 interop/shared/tools/kms_garbage_collector.py \
  --connection interop/shared/profiles/ciphertrust-connection.json \
  --prefix kmipkit- \
  --confirm
```
