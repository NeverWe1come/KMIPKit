# 07. Enterprise-Integrationen, Deployment & Betrieb

Dieses Handbuch liefert praxisfertige Konfigurationsvorlagen für die Anbindung gängiger Unternehmenssysteme an den **KMIPKit Proxy**.

---

## 1. Enterprise-Integrations-Playbooks

### MongoDB Enterprise (WiredTiger Encryption at Rest)

```yaml
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

1. `kmipkit-proxy` mit `allow_tls12 = true` starten.
2. In **vCenter Server `->` Configure `->` Security `->` Key Providers `->` Add Standard Key Provider** Hostname, Port `5696`, das von `tls_client_ca_path` signierte Client-Zertifikat sowie die Proxy-CA hinterlegen.

### NetApp ONTAP External Key Manager (KMIP 1.2 / 1.4)

```bash
security key-manager external enable \
  -vserver svm_prod \
  -key-servers kmip-proxy.internal.example.com:5696 \
  -client-cert ontap_kmip_client \
  -server-ca-certs kmip_proxy_ca
```

---

## 2. Kubernetes-Sidecar-Deployment (Distroless)

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

## 3. Fehlerbehebung & Diagnose

| Symptom | Wahrscheinlichste Ursache | Lösung |
| :--- | :--- | :--- |
| **TLS-Handshake scheitert bei KMIP-1.X-App** | Listener erzwingt TLS 1.3 (`allow_tls12 = false`). | `allow_tls12 = true` unter `[listener]` setzen oder `--allow-tls12` übergeben. |
| **`Result Reason: Response Too Large (0x02)`** | Client-Limit `Maximum Response Size` kleiner als KMIP-1.X-Antwort. | Client-Limit erhöhen oder `Get Attributes` / `Locate` (`Maximum Items`) einschränken. |
| **`Operation Not Supported` bei `Operation Policy Name`** | `operation_policy_mode = "reject"` aktiv. | Auf `"ignore"` oder `"store_as_vendor_attribute"` umstellen. |
| **`Item Not Found` bei `Modify/Delete Attribute` mit `Attribute Index`** | Index überschreitet Anzahl vorhandener Attribut-Instanzen. | Zuerst `Get Attributes` aufrufen, um gültige Indizes zu prüfen. |
| **Cloud-KMS `Get` liefert `Operation Not Supported`** | `key_custody_mode = "native_kms"` verhindert Export. | `key_custody_mode = "auto_envelope"` (oder `"auto"`) + `envelope_master_kek_id` konfigurieren. |
| **Fehler meldet `RequestDeliveryState::PossiblySent`** | TCP-Verbindungsabbruch nach Senden der Anfrage. | `Create`/`Destroy` nicht blind wiederholen; zuerst Serverzustand mit `Locate` prüfen. |

---

## 4. Verifikations- & Testsuite-Befehle

```bash
# Core Client Workspace
cargo test --workspace

# KMIP Proxy: Cloud Foundation + alle 7 KMIP-1.X-Bridge-Testsuiten
cargo test --manifest-path kmip-proxy/Cargo.toml --features kmip1x-bridge --tests
```
