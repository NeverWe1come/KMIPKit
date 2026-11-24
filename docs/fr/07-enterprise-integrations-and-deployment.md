# 07. Intégrations Entreprise, Déploiement & Exploitation

Ce guide fournit des configurations prêtes à l'emploi pour connecter les principales charges d'entreprise à **KMIPKit Proxy** en production.

---

## 1. Playbooks d'Intégration Entreprise

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

1. Déployez `kmipkit-proxy` avec `allow_tls12 = true`.
2. Dans **vCenter Server `->` Configure `->` Security `->` Key Providers `->` Add Standard Key Provider**, saisissez l'adresse, le port `5696`, le certificat client signé par `tls_client_ca_path` et la CA du proxy.

### NetApp ONTAP External Key Manager (KMIP 1.2 / 1.4)

```bash
security key-manager external enable \
  -vserver svm_prod \
  -key-servers kmip-proxy.internal.example.com:5696 \
  -client-cert ontap_kmip_client \
  -server-ca-certs kmip_proxy_ca
```

---

## 2. Déploiement Sidecar Kubernetes (Distroless)

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

## 3. Dépannage & Diagnostics

| Symptôme | Cause la Plus Probable | Résolution |
| :--- | :--- | :--- |
| **Échec du handshake TLS depuis une app KMIP 1.X** | Le listener impose TLS 1.3 (`allow_tls12 = false`). | Définir `allow_tls12 = true` sous `[listener]` ou passer `--allow-tls12`. |
| **`Result Reason: Response Too Large (0x02)`** | `Maximum Response Size` trop petit pour la trame TTLV 1.X. | Augmenter la limite cliente ou filtrer `Get Attributes` / `Locate` (`Maximum Items`). |
| **`Operation Not Supported` avec `Operation Policy Name`** | `operation_policy_mode = "reject"` est configuré. | Basculer sur `"ignore"` ou `"store_as_vendor_attribute"`. |
| **`Item Not Found` sur `Modify/Delete Attribute` avec `Attribute Index`** | Index hors limites sur l'objet en amont. | Appeler `Get Attributes` d'abord pour vérifier les indices. |
| **Cloud KMS `Get` renvoie `Operation Not Supported`** | `key_custody_mode = "native_kms"` interdit l'export symétrique. | Configurer `key_custody_mode = "auto_envelope"` (ou `"auto"`) avec `envelope_master_kek_id`. |
| **Erreur avec `RequestDeliveryState::PossiblySent`** | Coupure TCP après écriture de la requête. | Ne pas relancer aveuglément `Create`/`Destroy` ; vérifier via `Locate`. |

---

## 4. Commandes de Vérification & Suites de Tests

```bash
# Workspace client principal
cargo test --workspace

# KMIP Proxy : Cloud foundation + toutes les 7 suites du Pont KMIP 1.X
cargo test --manifest-path kmip-proxy/Cargo.toml --features kmip1x-bridge --tests
```
