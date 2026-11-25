# 03. Guía de Protocolo y Operaciones KMIP 2.0 / 2.1 / 3.0

KMIPKit utiliza **OASIS KMIP 2.1 por defecto**, con soporte explícito y validado por versión para **KMIP 2.0** y **KMIP 3.0** (incluyendo criptografía post-cuántica NIST).

---

## 1. Selección de Versión y Codificación de Cable

| Codificación | Content-Type (HTTPS) | Framing Raw TLS | Caso de Uso Ideal |
| :--- | :--- | :--- | :--- |
| **TTLV** (defecto) | `application/octet-stream` | Cabecera 8 bytes Tag/Type/Length + padding 8 bytes | HSMs nativos, mínima latencia y menor tamaño |
| **JSON** | `application/json` | Escáner incremental de llaves balanceadas | Gateways cloud, inspección REST/HTTPS |
| **XML** | `text/xml` | Escáner XML incremental sin DTD | Entornos empresariales XML/SOAP |

---

## 2. Matriz Completa de Operaciones (59 Operaciones)

### Operaciones Cliente-a-Servidor (C2S)

| Categoría | Operaciones | Versión KMIP |
| :--- | :--- | :--- |
| **Descubrimiento y Salud** | `Discover Versions`, `Query`, `Ping`, `Interop`, `Log` | 2.0 / 2.1 / 3.0 |
| **Creación e Importación** | `Create`, `Create Key Pair`, `Create Split Key`, `Register`, `Import`, `Derive Key`, `Join Split Key` | 2.0 / 2.1 / 3.0 |
| **Ciclo de Vida y Rotación** | `Activate`, `Deactivate` (3.0), `Revoke`, `Destroy`, `Obliterate` (3.0), `Archive`, `Recover`, `Re-key`, `Re-key Key Pair`, `Re-Provision` | 2.0 / 2.1 / 3.0 |
| **Certificados** | `Certify`, `Re-certify`, `Validate` | 2.0 / 2.1 / 3.0 |
| **Consulta y Extracción** | `Locate`, `Get`, `Export`, `Check`, `Obtain Lease`, `Get Usage Allocation` | 2.0 / 2.1 / 3.0 |
| **Gestión de Atributos** | `Get Attributes`, `Get Attribute List`, `Set Attribute`, `Add Attribute`, `Modify Attribute`, `Adjust Attribute`, `Delete Attribute` | 2.0 / 2.1 / 3.0 |
| **Operaciones Criptográficas** | `Encrypt`, `Decrypt`, `Sign`, `Signature Verify`, `MAC`, `MAC Verify`, `Hash`, `RNG Retrieve`, `RNG Seed` | 2.0 / 2.1 / 3.0 |
| **KEM Post-Cuántico (3.0)** | `Encapsulate`, `Decapsulate` (`ML-KEM-512/768/1024`, `X25519MLKEM768`, `SECP256R1MLKEM768`) | **3.0** |
| **Sesión y Acceso** | `Login`, `Logout`, `Delegated Login`, `Create User` (3.0), `Create Group` (3.0), `Create Credential` (3.0) | 2.0 / 2.1 / 3.0 |
| **Políticas y Admin** | `Get Constraints`, `Set Constraints`, `Set Defaults`, `PKCS#11`, `Set Endpoint Role` | 2.0 / 2.1 / 3.0 |
| **Control Asíncrono** | `Poll`, `Cancel`, `Process`, `Query Asynchronous Requests` | 2.0 / 2.1 / 3.0 |

### Operaciones Servidor-a-Cliente (S2C)
`Discover Versions`, `Query`, `Notify`, `Put` y `Set Endpoint Role` (soportadas sobre Raw TLS Listener, HTTPS Listener y Role-Reversed TLS).

---

## 3. Lotes Atómicos con `ID Placeholder` y Criptografía Post-Cuántica (KMIP 3.0)

```rust
// Lote atómico: Crear + Activar + Obtener en 1 solo RTT
let mut batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::create(
    CreateRequest::builder(ObjectType::SymmetricKey).build()?,
)));
batch.push(ClientBatchItem::new(ClientRequest::activate(
    ActivateRequest::new(UniqueIdentifier::id_placeholder()),
)));
batch.push(ClientBatchItem::new(ClientRequest::get(
    GetRequest::new(UniqueIdentifier::id_placeholder()),
)));
let outcome = client.execute(&batch)?;

// KMIP 3.0 PQC: ML-KEM-768 Encapsulate / Decapsulate
let kp = client.create_key_pair().algorithm(CryptographicAlgorithm::MlKem768).send()?;
let encap = client.encapsulate(kp.public_key_unique_identifier().unwrap().clone()).send()?;
let decap = client.decapsulate(
    kp.private_key_unique_identifier().unwrap().clone(),
    encap.ciphertext().unwrap(),
).send()?;
```
