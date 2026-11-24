# 06. Guide du Pont Legacy KMIP 1.X (1.0–1.4) -> KMIP 2.0 / 2.1 / 3.0

Le module **`kmip1x`** de `kmip-proxy` (feature `kmip1x-bridge` activée par défaut) permet aux applications legacy **KMIP 1.0–1.4** de communiquer de manière transparente avec les serveurs modernes **KMIP 2.0, 2.1 et 3.0** sans aucune modification applicative.

* **77 attributs standards KMIP 1.0–1.4** traduits bidirectionnellement (`Attribute` `0x420008` <-> `Attributes` `0x420125`), incluant les attributs personnalisés `x-*`/`y-*` <-> `Vendor Attribute` et les 14 liens typés KMIP 3.0 (`0x420190..=0x42019D`).
* **Émulation locale `Template` (`TemplateStore`):** `Register`, `Locate`, `Get`, `GetAttributes`, `GetAttributeList`, `Destroy` et fusion automatique dans `Create`/`Register`/`Import`.
* **Read-Modify-Write automatique** pour `Modify Attribute` et `Delete Attribute` utilisant `Attribute Index`.
* **Interception locale** de `Discover Versions`, filtrage par liste blanche de `Query` (masquant les opérations 2.0/3.0 et les algorithmes PQC), rétrogradation des `Result Reason`, respect strict de `Maximum Response Size` sur le TTLV 1.X final, et support complet **S2C** (`Kmip1xS2cBridge`).

```bash
kmipkit-proxy --listen 0.0.0.0:5696 --provider kmip-upstream \
  --upstream-endpoint hsm-kms.internal:5697 --upstream-version 3.0 --allow-tls12
```
