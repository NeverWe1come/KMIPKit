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

Individual Query probes selected Server Information, Extension Information,
RNG Parameters, Validation Information, Profile Information and Capability
Information. In both versions Cosmian returned a populated Server
Information structure and omitted Protection Storage Masks. All four
generic TTLV routes verified the Server Information structure. The typed
routes rejected that response. Each of the other five functions returned
Success with an empty Response Payload. Those replies did not verify the
requested structure or demonstrate a KMIP unsupported result.

In the binary TTLV tests, `Set Attribute` and `Modify Attribute` changed
`Description` on an isolated object and returned the new value on readback.
For an object without `Description`, Cosmian returned an empty `Attributes`
structure alongside the identifier. The standard requires only the
identifier in that case. The strict Add recipe records this as a failed
prerequisite. In a separate probe, Cosmian acknowledged adding the
description and returned it on readback. It also acknowledged
`Delete Attribute` with an `Attribute Reference` for Description, but the
next read still returned the field. These observations were reproduced in
both 2.0 and 2.1. [KMIP 2.0 §6.1.19](https://docs.oasis-open.org/kmip/kmip-spec/v2.0/os/kmip-spec-v2.0-os.html) · [KMIP 2.1 §6.1.20](https://docs.oasis-open.org/kmip/kmip-spec/v2.1/os/kmip-spec-v2.1-os.html)

The binary `RNG Retrieve` probe returned exactly 32 bytes in both versions.
For `RNG Seed`, the pinned server returned Operation Failed with Invalid
Message and omitted the Operation field. The report preserves this as a
failed exchange. The controlled peers separately verify a valid
zero-length response and reject a count greater than the 32 bytes sent.

Identifier-based `Certify` against an isolated RSA public key returned
Operation Not Supported in both versions over generic TTLV in all four
languages. That result leaves `Re-certify` and `Validate` unexecuted in these
workflows, since they require the certificate identifier from `Certify`.

The controlled Locate probes exercise all seven Rotate attributes in both
versions through generic TTLV, typed protocol and Fluent in C, Java, Python
and Rust, plus the direct Rust client method. Under KMIP 2.0 the clients
reject the attribute before sending a request. Under KMIP 2.1 the controlled
peer verifies the exact attribute and returns an empty Locate result.

Cosmian KMS 5.28.0 accepted Locate with Rotate Automatic, Date, Generation,
Interval, Latest and Offset on all 13 routes under KMIP 2.1. It rejected
Rotate Name because its TTLV parser did not recognize Rotate Name Value
(`0x420170`). The resulting Invalid Message response omitted Operation.
All 13 routes reported the exchange as failed.

## 6. Cosmian attribute extensions

The pinned image acknowledges the following `SetAttribute` requests at its
KMIP 2.1 HTTPS JSON TTLV endpoint.

| JSON attribute | Representation | Observed result after a successful write |
|---|---|---|
| `AttributeIndex` | Integer | `GetAttributes` and `Destroy` return HTTP 422 with an unknown-tag diagnostic. |
| `Critical` | Boolean | `GetAttributes` and `Destroy` return HTTP 422 with an unknown-tag diagnostic. |
| `ObjectGroupMember` | Enumeration, `Group_Member_Fresh` or `Group_Member_Default` | `GetAttributes` succeeds and omits the field. `Destroy` succeeds. |

The observations apply to the provider's JSON attribute context. The
standard enumerated `Object Group Member` field in a Locate request is
tested separately. The provider's published [KMIP support table](https://docs.cosmian.com/key_management_system/kmip_support/support.html)
lists these three attributes under extensions.

To reproduce on a disposable instance, add
`--publish 127.0.0.1:8443:9998` to the Docker options in section 3.
Use `{"https": {"url": "https://localhost:8443/kmip/2_1"}}` as
`endpoints` in a copy of the configuration from section 4. The tests also
verify rejection of an incorrect type or enumeration for each extension.
Discard the instance's temporary filesystem after the test because objects
with the two legacy named extensions fail to load for Destroy.

```bash
KMIPKIT_COSMIAN_EXTENSION_TEST_CONFIG=/path/to/extension-connection.json \
  KMIPKIT_COSMIAN_EXTENSION_TEST_REPORT=/path/to/cosmian-extensions.json \
  python3 -m unittest discover -s verification/interop/shared/tests \
  -p test_cosmian_attribute_extensions.py -v
```

Pass `--table-vendor-extensions-report /path/to/cosmian-extensions.json` to
`campaign.py --cosmian-minimum` to attach the observations to the three KMIP
2.1 rows. They show how the provider's HTTPS JSON endpoint behaved. The
binding-specific cells still require evidence from their own API routes.
