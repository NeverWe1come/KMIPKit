# 07. Enterprise-Integrationen, Deployment & Betrieb

* **MongoDB Enterprise:** `security.kmip.serverName` auf `kmipkit-proxy` (Port `5696`) im Modus `auto_envelope` oder als KMIP-1.X-Bridge konfigurieren.
* **MySQL Enterprise / Percona (`keyring_kmip`):** `keyring_kmip_server_addr=127.0.0.1`, `keyring_kmip_port=5696`.
* **VMware vSphere & NetApp ONTAP:** `allow_tls12 = true` im `[listener]` für Legacy-KMIP-1.1–1.4-Clients aktivieren.
* **Kubernetes Sidecar:** Distroless-Container mit `runAsNonRoot: true` und `readOnlyRootFilesystem: true` auf `127.0.0.1:5696`.
* **Vollständige Testverifikation:**
```bash
cargo test --workspace
cargo test --manifest-path kmip-proxy/Cargo.toml \
  --test foundation_tests --test kmip1x_bridge_tests \
  --test kmip1x_exhaustive_roundtrip_tests \
  --test kmip1x_edge_security_and_interop_tests \
  --test kmip1x_full_matrix_and_chaos_tests
```
