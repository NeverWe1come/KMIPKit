# 06. KMIP 1.X (1.0–1.4) -> KMIP 2.0 / 2.1 / 3.0 Legacy Bridge

Das Modul **`kmip1x`** in `kmip-proxy` (Feature `kmip1x-bridge`, standardmäßig aktiv) verbindet Legacy-Anwendungen (**KMIP 1.0–1.4**) transparent mit modernen **KMIP 2.0 / 2.1 / 3.0**-Servern ohne Änderungen an der Altanwendung.

* **Alle 77 Standardattribute von KMIP 1.0–1.4** bidirektional übersetzt (`Attribute` `0x420008` <-> `Attributes` `0x420125`), inkl. Custom-Attributen `x-*`/`y-*` <-> `Vendor Attribute` und allen 14 typisierten KMIP-3.0-Links (`0x420190..=0x42019D`).
* **Lokale `Template`-Emulation (`TemplateStore`):** `Register`, `Locate`, `Get`, `GetAttributes`, `GetAttributeList`, `Destroy` sowie automatisches Zusammenführen bei `Create`/`Register`/`Import`.
* **Automatisches Read-Modify-Write** für `Modify Attribute` und `Delete Attribute` mit `Attribute Index`.
* **Lokales Abfangen** von `Discover Versions`, Allowlist-Filterung für `Query` (blendet 2.0/3.0-Operationen und PQC-Algorithmen aus), `Result Reason`-Downgrade, `Maximum Response Size`-Prüfung auf dem finalen 1.X-TTLV sowie vollständige **S2C-Unterstützung** (`Kmip1xS2cBridge`).

```bash
kmipkit-proxy --listen 0.0.0.0:5696 --provider kmip-upstream \
  --upstream-endpoint hsm-kms.internal:5697 --upstream-version 3.0 --allow-tls12
```
