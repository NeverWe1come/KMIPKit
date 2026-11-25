# 07. Enterprise Integrations, Deployment & Operations

This playbook provides copy-pasteable integration guides for connecting major enterprise workloads to **KMIPKit Proxy** and operating it reliably in production.

---

## 1. Enterprise Workload Playbooks

### MongoDB Enterprise (WiredTiger Encryption at Rest)

Configure `/etc/mongod.conf` to point MongoDB at `kmipkit-proxy` (running either in Cloud KMS mode with `auto_envelope` or in KMIP 1.X Bridge mode):

```yaml
security:
  enableEncryption: true
  kmip:
    serverName: kmip-proxy.internal.example.com
    port: 5696
    serverCAFile: /etc/mongodb/kmip-ca.pem
    clientCertificateFile: /etc/mongodb/mongodb-client.pem
    keyIdentifier: "" # Leave empty to auto-create AES-256 master key on first boot
```

### MySQL Enterprise / Percona Server (`keyring_kmip`)

```ini
# /etc/my.cnf
[mysqld]
early-plugin-load=keyring_kmip.so
keyring_kmip_server_addr=127.0.0.1
keyring_kmip_port=5696
keyring_kmip_ca_file=/etc/mysql/kmip-ca.pem
keyring_kmip_cert_file=/etc/mysql/mysql-client.pem
keyring_kmip_key_file=/etc/mysql/mysql-client.key
```

### VMware vSphere Standard Key Provider (KMIP 1.1 / 1.4)

1. Deploy `kmipkit-proxy` with `allow_tls12 = true` (vSphere KMS connections often negotiate TLS 1.2).
2. In **vCenter Server -> Configure -> Security -> Key Providers -> Add Standard Key Provider**:
   * **KMS Address:** `kmip-proxy.internal.example.com`
   * **Port:** `5696`
   * Upload the vCenter client certificate signed by `tls_client_ca_path` and trust the proxy's server CA.

### NetApp ONTAP External Key Manager (KMIP 1.2 / 1.4)

```bash
security key-manager external enable \
  -vserver svm_prod \
  -key-servers kmip-proxy.internal.example.com:5696 \
  -client-cert ontap_kmip_client \
  -server-ca-certs kmip_proxy_ca
```

---

## 2. Kubernetes Sidecar Deployment (Distroless)

Running `kmipkit-proxy` as a localhost sidecar (`127.0.0.1:5696`) eliminates unencrypted network hops and keeps application credentials scoped to the Pod's IAM role (IRSA / Workload Identity):

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: payments-service
spec:
  template:
    spec:
      containers:
        - name: app
          image: registry.internal/payments-legacy:latest
          env:
            - name: KMIP_ENDPOINT
              value: "127.0.0.1:5696"
        - name: kmipkit-proxy
          image: ghcr.io/neverwe1come/kmipkit-proxy:1.0.0
          args: ["--config", "/etc/kmip-proxy/proxy.toml"]
          ports:
            - containerPort: 5696
          securityContext:
            runAsNonRoot: true
            readOnlyRootFilesystem: true
            allowPrivilegeEscalation: false
          volumeMounts:
            - name: proxy-config
              mountPath: /etc/kmip-proxy
              readOnly: true
      volumes:
        - name: proxy-config
          secret:
            secretName: kmip-proxy-mtls-and-config
```

---

## 3. Troubleshooting & Diagnostics Reference

| Symptom | Most Likely Cause | Resolution |
| :--- | :--- | :--- |
| **TLS handshake fails from legacy KMIP 1.X app** | Listener defaults to TLS 1.3 only (`allow_tls12 = false`). | Set `allow_tls12 = true` under `[listener]` or pass `--allow-tls12`. |
| **`Result Reason: Response Too Large (0x02)`** | Client set a tight `Maximum Response Size` header smaller than the serialized 1.X TTLV response. | Increase client `Maximum Response Size` or filter `Get Attributes` / `Locate` (`Maximum Items`). |
| **`Operation Not Supported` on `Create` with `Operation Policy Name`** | `operation_policy_mode = "reject"` is configured. | Switch to `"ignore"` or `"store_as_vendor_attribute"` in `[backend.kmip_upstream]`. |
| **`Item Not Found` on `Modify/Delete Attribute` with `Attribute Index`** | Requested `Attribute Index` exceeds the number of instances on the upstream object. | Call `Get Attributes` first to inspect valid 0-based indices. |
| **Cloud KMS `Get` returns `Operation Not Supported`** | `key_custody_mode = "native_kms"` prevents exporting symmetric key bytes from cloud HSMs. | Switch `key_custody_mode = "auto_envelope"` (or `"auto"`) and configure `envelope_master_kek_id`. |
| **Client error shows `RequestDeliveryState::PossiblySent`** | TCP reset or timeout occurred after request bytes were written. | Do **not** blindly retry `Create`/`Destroy`; call `Locate` or `Get Attributes` to verify server state first. |

---

## 4. Verification & Test Suite Commands

```bash
# Core client workspace (Rust + FFI + protocol + transport)
cargo test --workspace

# KMIP Proxy: Cloud foundation + KMIP 1.X Bridge (all 4 test suites)
cargo test --manifest-path kmip-proxy/Cargo.toml \
  --test foundation_tests \
  --test kmip1x_bridge_tests \
  --test kmip1x_exhaustive_roundtrip_tests \
  --test kmip1x_edge_security_and_interop_tests \
  --test kmip1x_full_matrix_and_chaos_tests
```
