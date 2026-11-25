# KMIP 1.X Legacy Bridge Engineering Reference (`src/kmip1x/`)

The **`kmip1x`** module (`kmip1x-bridge` Cargo feature, enabled by default) transparently bridges **KMIP 1.0, 1.1, 1.2, 1.3, and 1.4** clients to **KMIP 2.0, 2.1, and 3.0** servers (or chained to `RequestDispatcher` / Cloud KMS) while passing native KMIP >= 2.0 requests through untouched.

---

## 1. Module Architecture (`kmip-proxy/src/kmip1x/`)

| File | Responsibility |
| :--- | :--- |
| `mod.rs` | `Kmip1xBridge` orchestrator: batch splitting, implicit `ID Placeholder` chaining across batch items, RMW loop (`Modify`/`Delete Attribute`), `TemplateStore` integration, and error `Batch Item` construction. |
| `translator.rs` | Core request/response message translation: `Template-Attribute ↔ Attributes`, `Key Block → Key Value` recursive attributes, `Key Wrapping Specification` (`Attribute Name → Attribute Reference`), `Discover Versions`, `Query` allowlists, `Result Reason` downgrading, `Maximum Response Size` re-checking, and S2C translation. |
| `mapping.rs` | Static lookup catalog for all **77 standard KMIP 1.X attributes**, cardinality rules (`SingleInstance` vs `MultiInstance`), deprecated KMIP 1.0 X.509 certificate aliases, custom `x-*`/`y-*` prefixes, **14 KMIP 3.0 Typed Links**, `Query` per-minor allowlists, and `Result Reason` downgrade matrix. |
| `template_store.rs` | Thread-safe single-`RwLock` `TemplateStore` with optional atomic temp-file-and-rename JSON persistence (`template_store_path`), overwrite-safe duplicate registration, and attribute merging. |
| `upstream.rs` | `UpstreamKmipTransport` trait, `KmipKitUpstreamTransport` (pooled mTLS client over raw TLS/HTTPS with `spawn_blocking`), and `LocalDispatcherUpstream` (chaining into Cloud KMS `RequestDispatcher`). |
| `s2c.rs` | `Kmip1xS2cBridge` & `S2cClientTransport` for Server-to-Client operations (`Notify`, `Put`, `Query`, `Discover Versions`). |
| `tags.rs` & `ttlv_helpers.rs` | Canonical tag/operation/status constants and safe TTLV tree builders/extractors using `RawTag::try_checked_allow_reserved()`. |

---

## 2. Protocol Paradigm Conversions

### A. Attribute Model & All 77 Standard Attributes

* **KMIP 1.X → 2.0/2.1/3.0:** Unwraps `Attribute (0x420008) { Attribute Name, Attribute Index?, Attribute Value }` into `Attributes (0x420125)` with direct attribute tags (`Cryptographic Algorithm`, `State`, `Usage Limits`, etc.).
* **KMIP 2.0/2.1/3.0 → 1.X:** Wraps direct tags back into `Attribute { Attribute Name, Attribute Index?, Attribute Value }`. Multi-instance attributes in responses receive deterministic sequential `Attribute Index` values (`0, 1, 2, ...`).
* **Version Gating (`1.0`..`1.4`):** Only attributes valid for the client's minor version are returned in `Get Attributes` / `Get Attribute List` responses (for instance, `Alternative Name` requires >= 1.2, `Description` >= 1.3, `Sensitive`/`Key Role Type` >= 1.4).
* **KMIP 1.0 Deprecated Certificate Aliases:** When serving a KMIP 1.0 client (`minor == 0`), `X.509 Certificate Identifier`, `X.509 Certificate Issuer`, and `X.509 Certificate Subject` are mapped to `Certificate Identifier`, `Certificate Issuer`, and `Certificate Subject` respectively; inbound 1.0 aliases normalize cleanly toward 2.X/3.0.
* **Read-Only Sanitization:** When `strip_readonly_on_create = true` (default) for object-creation operations (`Create`, `Register`, `Import`, `Rekey`, `Derive Key`, `Certify`, `Recertify`, `Create Split Key`, `Join Split Key`), read-only attributes rejected by KMIP 3.0 servers (e.g., `State`, `Initial Date`, `Digest`, `Unique Identifier`, `Object Type`) are automatically stripped, while `Locate` retains all attributes including read-only filters.

### B. `Template-Attribute`, `Create Key Pair` & `Key Block` -> `Key Value`

* `Template-Attribute` (`0x420091`), `Common Template-Attribute` (`0x42001F`), `Private Key Template-Attribute` (`0x420065`), and `Public Key Template-Attribute` (`0x42006E`) map to `Attributes` (`0x420125`), `Common Attributes` (`0x420126`), `Private Attributes` (`0x420127`), and `Public Attributes` (`0x420128`).
* Inside **`Key Block` → `Key Value`**, `translator.rs` recursively translates nested `Attribute (0x420008)` items into 2.0+ `Attributes (0x420125)` on plain `Structure` `Key Value` items in requests and back to KMIP 1.X `Attribute` lists in responses (`Get`, `Register`, `Import`, `Export`), while leaving wrapped `ByteString` `Key Value` payloads completely untouched.

### C. Emulated `Template` Object Lifecycle (`TemplateStore`)

KMIP 2.0 removed stored `Template` managed objects (`Object Type = 0x06`). When `backend.kmip_upstream.template_store_path` is set, templates survive proxy restarts via atomic JSON serialization (`*.tmp` + `rename`):

1. **`Register` (Object Type = Template `0x06`)**: Stored locally in `TemplateStore`; returns a local unique identifier without hitting the upstream 2.0+ server. Duplicate registered names overwrite prior entries safely.
2. **`Locate` / `Get` / `Get Attributes` / `Get Attribute List` / `Destroy`**: Resolved directly against `TemplateStore` when targeting a stored template UID or template filter.
3. **Batch `Create` / `Create Key Pair` Merge**: Any template referenced by `Name` inside `Template-Attribute` is merged into the outgoing object attributes before translation.

### D. `Add` / `Modify` / `Delete` Attribute Semantics

| Client 1.X Request | Target 2.0/2.1/3.0 Translation | Bridge Action |
| :--- | :--- | :--- |
| **`Add Attribute`** (`SingleInstance` attr) | **`Set Attribute` (`0x31`)** | Routes by cardinality; rewrites response operation back to `Add Attribute (0x0D)`. |
| **`Add Attribute`** (`MultiInstance` attr) | **`Add Attribute` (`0x0D`)** | Preserves operation; assigns index. |
| **`Modify Attribute`** (with/without `Attribute Index`) | **`Get Attributes` → `Modify Attribute`** | Automatic **Read-Modify-Write**: queries current attributes, resolves exact `Current Attribute` matching `Attribute Index` (default `0`), matches custom attributes strictly by `(vendor, name)`, handles KMIP 3.0 typed links, and restores `Attribute Index` in the 1.X response. |
| **`Delete Attribute`** (`Name` + optional `Index`) | **`Get Attributes` → `Delete Attribute`** | Automatic **Read-Modify-Write**: resolves target instance (or first matching attribute when omitted; handles index mismatch cleanly with `Item Not Found`), forwards `Current Attribute`, and returns the deleted index. |

### E. Custom Vendor Attributes, KMIP 3.0 Links & Policies

* **Custom Attributes (`x-*` / `y-*`)**: Mapped losslessly to KMIP 2.0+ **Vendor Attribute** (`Vendor Identification = "x"` or `"y"`, `Attribute Name = <suffix>`, `Attribute Value = <val>`), and reconstructed identically in reverse.
* **Links (`Link 0x42004A` ↔ KMIP 3.0 Typed Links `0x420190..=0x42019D`)**: All 14 `Link Type` variants (`Certificate`, `Public Key`, `Private Key`, `Derivation Base Object`, `Derived Object`, `Replacement Object`, `Replaced Object`, `Parent`, `Child`, `Previous`, `Next`, `PKCS#12 Certificate`, `PKCS#12 Password`, `Wrapping Key`) map 1:1 when targeting `UpstreamKmipVersion::V3_0` and are deduplicated cleanly in `Get Attribute List`.
* **`Operation Policy Name` (`0x42005D`)**: Controlled by `operation_policy_mode`:
  * `"ignore"` *(default)*: stripped silently.
  * `"store_as_vendor_attribute"`: preserved as vendor `"kmip1x"` attribute `"Operation Policy Name"`.
  * `"reject"`: fails the specific `Batch Item` cleanly with `STATUS_OPERATION_FAILED` + `REASON_OPERATION_NOT_SUPPORTED` without aborting sibling batch items.

### F. Chained Batch `ID Placeholder`, Async Header, `Query`, `Discover Versions` & Size Guard

* **Implicit `ID Placeholder`**: In multi-item batches (e.g., `Create` followed by `Activate` or `Get` omitting `Unique Identifier`), the bridge tracks the upstream `last_uid` across sequential and orchestrated items (notably during RMW batch execution) and injects `Unique Identifier` appropriately.
* **`Asynchronous Indicator` & `Maximum Response Size`**: Boolean/enum translation in headers, plus re-verification of **`Maximum Response Size`** (`> 0`) against the **final serialized KMIP 1.X TTLV byte length**, returning `Result Status = Operation Failed`, `Result Reason = Response Too Large (0x02)` when exceeded.
* **`Discover Versions` (`0x1E`)**: Intercepted locally (never reaches upstream) and intersected against `[1.4, 1.3, 1.2, 1.1, 1.0]`.
* **`Query` (`0x18`)**: Filters upstream `Query` responses through per-minor allowlists so KMIP 1.X clients never receive 2.0+/3.0 operations (`Set Attribute`, `Encapsulate`, `Decapsulate`, `RNG Seed` gating where applicable, etc.) or PQC algorithms (`ML-KEM`, `ML-DSA`, `SLH-DSA`, `LMS`, `XMSS`, `Quantum Safe Capability`).
* **`Result Reason` Downgrade**: Extended 2.0/2.1/3.0 reasons are deterministically downgraded to valid KMIP 1.X codes (`0x00..=0x13` depending on minor version) before reaching legacy parsers.

---

## 3. Server-to-Client (S2C) Translation (`Kmip1xS2cBridge`)

For environments where the KMIP 2.0/2.1/3.0 server initiates S2C requests (`Notify`, `Put`, `Query`, `Discover Versions`) toward a legacy KMIP 1.X client:

1. Outbound S2C `Discover Versions` is answered locally with the configured `UpstreamKmipVersion`.
2. Outbound `Notify` and `Put` (`Attributes 0x420125`) payloads are downgraded to `Protocol Version = client (major, minor)` and KMIP 1.X `Attribute` lists (`0x420008`).
3. Legacy `Response Message` payloads from the client are promoted back to the upstream server's `2.0 / 2.1 / 3.0` header version and `Result Reason` format.
