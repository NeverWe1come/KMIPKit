# 07. Intégrations Entreprise, Déploiement et Exploitation

* **MongoDB Enterprise:** Pointer `security.kmip.serverName` sur `kmipkit-proxy` (port `5696`) en mode `auto_envelope` ou pont KMIP 1.X.
* **MySQL Enterprise / Percona (`keyring_kmip`):** Configurer `keyring_kmip_server_addr=127.0.0.1` et `keyring_kmip_port=5696`.
* **VMware vSphere & NetApp ONTAP:** Activer `allow_tls12 = true` sur le listener pour les clients legacy KMIP 1.1–1.4.
* **Déploiement Kubernetes Sidecar:** Exécuter l'image distroless en mode `runAsNonRoot: true` et `readOnlyRootFilesystem: true` sur `127.0.0.1:5696`.
* **Vérification complète:**
```bash
cargo test --workspace
cargo test --manifest-path kmip-proxy/Cargo.toml \
  --test foundation_tests --test kmip1x_bridge_tests \
  --test kmip1x_exhaustive_roundtrip_tests \
  --test kmip1x_edge_security_and_interop_tests \
  --test kmip1x_full_matrix_and_chaos_tests
```
