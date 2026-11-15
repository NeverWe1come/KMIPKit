# Cosmian KMS 5.28 Deployment & Interop Guide

This guide details how to deploy Cosmian KMS 5.28 in a local or CI environment and execute the KMIPKit multi-language interoperability suite against it over mutual TLS (mTLS).

---

## 1. Overview & Architecture

[Cosmian KMS](https://cosmian.com) provides an OASIS KMIP 2.1 compliant key management server with support for TTLV over TLS and JSON over HTTPS.

| Parameter | Specification |
|---|---|
| **KMIP Version** | OASIS KMIP 2.1 |
| **Raw TLS Port** | `5696` (TTLV encoding) |
| **HTTPS Port** | `8443` (JSON encoding at `/kmip/2_1`) |
| **Authentication** | Mutual TLS (mTLS) with client certificate |
| **Known Quirks** | Rejects `Archive` and `Recover` operations |

---

## 2. Certificate Generation for mTLS

Generate a local CA, server certificate, and client certificate for KMIPKit using OpenSSL:

```bash
mkdir -p certs && cd certs

# 1. Root Certificate Authority (CA)
openssl ecparam -name prime256v1 -genkey -noout -out ca.key
openssl req -new -x509 -key ca.key -out ca.crt -days 365 -subj "/CN=KMIPKit-Local-CA"

# 2. Server Certificate (with SAN for localhost)
openssl ecparam -name prime256v1 -genkey -noout -out server.key
openssl req -new -key server.key -out server.csr -subj "/CN=localhost"
cat > server_ext.cnf <<EOF
basicConstraints = CA:FALSE
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names
[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
EOF
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out server.crt -days 365 -extfile server_ext.cnf

# 3. KMIPKit Client Certificate
openssl ecparam -name prime256v1 -genkey -noout -out client.key
openssl req -new -key client.key -out client.csr -subj "/CN=kmipkit-client"
cat > client_ext.cnf <<EOF
basicConstraints = CA:FALSE
keyUsage = digitalSignature
extendedKeyUsage = clientAuth
EOF
openssl x509 -req -in client.csr -CA ca.crt -CAkey ca.key -CAcreateserial \
  -out client.crt -days 365 -extfile client_ext.cnf
```

---

## 3. Docker Compose Deployment

Create `docker-compose.yml` to launch Cosmian KMS with mTLS enabled:

```yaml
version: "3.8"

services:
  cosmian-kms:
    image: ghcr.io/cosmian/kms:5.28.0
    container_name: cosmian-kms
    restart: unless-stopped
    ports:
      - "5696:5696"   # Raw TTLV over TLS
      - "8443:8443"   # HTTPS JSON API
    environment:
      - KMS_LOG_LEVEL=info
      - KMS_KMIP_PORT=5696
      - KMS_HTTPS_PORT=8443
      - KMS_TLS_CERT_PATH=/etc/cosmian/certs/server.crt
      - KMS_TLS_KEY_PATH=/etc/cosmian/certs/server.key
      - KMS_TLS_CA_PATH=/etc/cosmian/certs/ca.crt
      - KMS_REQUIRE_CLIENT_CERT=true
    volumes:
      - ./certs:/etc/cosmian/certs:ro
      - cosmian-data:/var/lib/cosmian
    healthcheck:
      test: ["CMD-SHELL", "nc -z 127.0.0.1 5696 || exit 1"]
      interval: 5s
      timeout: 3s
      retries: 5

volumes:
  cosmian-data:
```

Launch the service:
```bash
docker compose up -d
docker compose ps
```

---

## 4. KMIPKit Connection Configuration

Create `interop/shared/profiles/cosmian-connection.json`:

```json
{
  "server_identifier": "cosmian-kms-5.28",
  "host": "127.0.0.1",
  "port": 5696,
  "transport": "raw_tls",
  "encoding": "TTLV",
  "tls": {
    "ca_certificate": "certs/ca.crt",
    "client_certificate": "certs/client.crt",
    "client_key": "certs/client.key",
    "verify_hostname": true,
    "min_version": "1.3"
  }
}
```

---

## 5. Running the KMIPKit Interop Suite

### 1. Preflight Verification
```bash
python3 interop/shared/tools/preflight_and_audit.py \
  --connection interop/shared/profiles/cosmian-connection.json \
  --profile interop/shared/profiles/cosmian-kms.json
```

### 2. Discover Server Capabilities
```bash
python3 interop/shared/tools/live_discover_capabilities.py \
  --connection interop/shared/profiles/cosmian-connection.json \
  --output interop/results/cosmian-discovered.json
```

### 3. Run Multi-Language Suite
```bash
python3 interop/shared/tools/run_all.py \
  --profile interop/shared/profiles/cosmian-kms.json \
  --connection interop/shared/profiles/cosmian-connection.json \
  --languages rust,c,java,python
```

### 4. Cleanup Test Objects
```bash
python3 interop/shared/tools/kms_garbage_collector.py \
  --connection interop/shared/profiles/cosmian-connection.json \
  --prefix kmipkit- \
  --confirm
```
