# Cosmian KMS 5.28 Deployment & Interop Guide

This guide details how to deploy Cosmian KMS 5.28 in a local or CI environment and execute the KMIPKit multi-language interoperability suite against it over mutual TLS (mTLS).

---

## 1. Overview & Architecture

[Cosmian KMS](https://cosmian.com) accepts KMIP requests over raw TLS and HTTPS. The suite records the operations and fields observed for each version and API.

| Parameter | Specification |
|---|---|
| **KMIP versions in this campaign** | 2.0 and 2.1 |
| **Raw TLS Port** | `5696` (TTLV encoding) |
| **HTTPS Port** | `8443` (JSON encoding at `/kmip/2_1`) |
| **Authentication** | Mutual TLS (mTLS) with client certificate |


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

## 3. Isolated Docker deployment

After generating the certificates, return to the repository root. Bind the
binary listener to loopback and keep test objects in a container-owned
temporary filesystem. The three mounted files are the server certificate,
server key and client trust anchor.

```bash
cert_dir="$PWD/certs"
docker run --detach --rm --name kmipkit-cosmian-528 \
  --publish 127.0.0.1:5696:5696 \
  --user "$(id -u):$(id -g)" --cap-drop ALL --security-opt no-new-privileges \
  --tmpfs "/kmipkit-data:uid=$(id -u),gid=$(id -g),mode=0700" \
  --workdir /kmipkit-data --env HOME=/kmipkit-data \
  --mount "type=bind,source=$cert_dir/server.crt,target=/server.crt,readonly" \
  --mount "type=bind,source=$cert_dir/server.key,target=/server.key,readonly" \
  --mount "type=bind,source=$cert_dir/ca.crt,target=/ca.crt,readonly" \
  ghcr.io/cosmian/kms@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e \
  --socket-server-start --socket-server-hostname 0.0.0.0 --socket-server-port 5696 \
  --tls-cert-file /server.crt --tls-key-file /server.key --clients-ca-cert-file /ca.crt
```

[Provider TLS documentation](https://docs.cosmian.com/key_management_system/configuration/tls/)

## 4. KMIPKit connection configuration

Write a connection JSON file outside the tracked profiles. Use absolute TLS
paths and assign the same configuration fingerprint to every API and KMIP
version that uses this deployment. For this configuration, the SHA-256 of
the server certificate and immutable image digest gives a reproducible
fingerprint. Replace the example fingerprint with the calculated value.

```json
{
  "server_id": "cosmian-kms-5-28",
  "server_product": "Cosmian KMS",
  "server_version": "5.28.0",
  "server_configuration_fingerprint": "0000000000000000000000000000000000000000000000000000000000000000",
  "protocol_version": "2.1",
  "transport": "raw_tls",
  "encoding": "TTLV",
  "api_level": "typed-protocol",
  "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": 5696}},
  "tls": {
    "ca_cert_path": "/path/to/certs/ca.crt",
    "client_cert_path": "/path/to/certs/client.crt",
    "client_key_path": "/path/to/certs/client.key",
    "server_name_override": "localhost",
    "read_timeout_seconds": 5
  },
  "environment": {"kind": "dedicated_test", "dedicated_test_environment_confirmed": true},
  "execution_policy": {"probe_unsupported": true, "allow_mutations": true}
}
```

## 5. Running the KMIPKit table campaign

Build and install the four validators first. Pass one connection file for
each version and route. `--enable-mutations` opts in to the recipes that
create and remove test objects. `--require-complete` writes missing recipes
and case evidence to `completion.json`.

```bash
python3 verification/interop/shared/tools/run_all.py --all-operations --cosmian-minimum \
  --enable-mutations --require-complete --campaign \
  --profile verification/interop/shared/profiles/cosmian-kms.json \
  --connection /path/to/2.0-generic-ttlv.json --connection /path/to/2.1-generic-ttlv.json \
  --connection /path/to/2.0-typed-protocol.json --connection /path/to/2.1-typed-protocol.json \
  --connection /path/to/2.0-fluent.json --connection /path/to/2.1-fluent.json \
  --connection /path/to/2.0-high-level.json --connection /path/to/2.1-high-level.json \
  --output-dir /path/to/results
```

Cosmian KMS 5.28.0 returned an Operation list for `Query Operations` in both
versions. The response omitted the Protection Storage Masks field marked
required by the KMIP 2.0 and 2.1 structured Query response tables. The
generic probe records the returned operations and the typed validation
reports the missing field. [KMIP 2.0 §6.1.37](https://docs.oasis-open.org/kmip/kmip-spec/v2.0/os/kmip-spec-v2.0-os.html) · [KMIP 2.1 §6.1.39](https://docs.oasis-open.org/kmip/kmip-spec/v2.1/os/kmip-spec-v2.1-os.html)
