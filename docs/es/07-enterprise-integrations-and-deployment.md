# 07. Integraciones Empresariales, Despliegue y Operaciones

Playbooks listos para copiar y pegar para conectar cargas de trabajo empresariales a **KMIPKit Proxy** en producción.

---

## 1. Playbooks Empresariales

### MongoDB Enterprise (WiredTiger Encryption at Rest)

```yaml
# /etc/mongod.conf
security:
  enableEncryption: true
  kmip:
    serverName: kmip-proxy.internal.example.com
    port: 5696
    serverCAFile: /etc/mongodb/kmip-ca.pem
    clientCertificateFile: /etc/mongodb/mongodb-client.pem
    keyIdentifier: ""
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
1. Desplegar `kmipkit-proxy` con `allow_tls12 = true`.
2. En **vCenter Server -> Configure -> Security -> Key Providers -> Add Standard Key Provider**, introducir `kmip-proxy.internal.example.com:5696` y subir el certificado cliente firmado por `tls_client_ca_path`.

### NetApp ONTAP External Key Manager (KMIP 1.2 / 1.4)

```bash
security key-manager external enable \
  -vserver svm_prod \
  -key-servers kmip-proxy.internal.example.com:5696 \
  -client-cert ontap_kmip_client \
  -server-ca-certs kmip_proxy_ca
```

---

## 2. Despliegue Sidecar en Kubernetes (Distroless)

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

## 3. Diagnóstico y Resolución de Problemas

| Síntoma | Causa Más Probable | Solución |
| :--- | :--- | :--- |
| **Fallo de handshake TLS desde app legacy 1.X** | Listener en modo TLS 1.3 exclusivo (`allow_tls12 = false`). | Activar `allow_tls12 = true` en `[listener]` o pasar `--allow-tls12`. |
| **`Result Reason: Response Too Large (0x02)`** | `Maximum Response Size` menor que la respuesta TTLV 1.X serializada. | Aumentar `Maximum Response Size` o filtrar `Get Attributes` / `Locate`. |
| **`Operation Not Supported` con `Operation Policy Name`** | `operation_policy_mode = "reject"` activo. | Cambiar a `"ignore"` o `"store_as_vendor_attribute"`. |
| **`Item Not Found` en `Modify/Delete Attribute` con índice** | `Attribute Index` fuera de rango en el objeto upstream. | Consultar índices válidos con `Get Attributes`. |
| **`Operation Not Supported` en `Get` sobre Cloud KMS** | `key_custody_mode = "native_kms"` impide exportar bytes simétricos. | Cambiar a `auto_envelope` o `auto` y definir `envelope_master_kek_id`. |
| **Error con `RequestDeliveryState::PossiblySent`** | Corte TCP o timeout tras enviar bytes de petición. | **No** reintentar ciegamente `Create`/`Destroy`; comprobar estado con `Locate`. |

---

## 4. Comandos de Verificación

```bash
cargo test --workspace
cargo test --manifest-path kmip-proxy/Cargo.toml \
  --test foundation_tests \
  --test kmip1x_bridge_tests \
  --test kmip1x_exhaustive_roundtrip_tests \
  --test kmip1x_edge_security_and_interop_tests \
  --test kmip1x_full_matrix_and_chaos_tests
```
