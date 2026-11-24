# 03. Guide Protocole et Opérations KMIP 2.0 / 2.1 / 3.0

KMIPKit cible **OASIS KMIP 2.1 par défaut**, avec support explicite et vérifié par version pour **KMIP 2.0** et **KMIP 3.0** (y compris les primitives Post-Quantiques du NIST).

---

## 1. Sélection de Version et d'Encodage Fil

```rust
use kmipkit::client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit::client::{RequestOptions, WireEncoding};
use kmipkit::protocol::ProtocolVersion;

let registry = client_extension_registry(Vec::new(), kmipkit::protocol::extension::defaults())?;

// Configuration version par défaut (KMIP 3.0) et encodage (TTLV)
let config = ClientConfiguration::new(registry)
    .with_protocol_version(ProtocolVersion::V3_0)
    .with_wire_encoding(WireEncoding::Ttlv);

// Surcharge par requête (JSON sur HTTPS)
let json_opts = RequestOptions::default().with_wire_encoding(WireEncoding::Json);
```

| Encodage | Content-Type (HTTPS) | Cadrage Raw TLS | Meilleur Cas d'Usage |
| :--- | :--- | :--- | :--- |
| **TTLV** (défaut) | `application/octet-stream` | En-tête 8 octets + alignement 8 octets | HSM natifs, latence minimale |
| **JSON** | `application/json` | Scanner incrémental d'accolades | Passerelles Cloud, inspection REST |
| **XML** | `text/xml` | Scanner XML sans DTD | Environnements SOAP / XML |

---

## 2. Matrice Complète de Couverture (59 Opérations)

### Opérations Client-vers-Serveur (C2S)

| Catégorie | Opérations | Version KMIP |
| :--- | :--- | :--- |
| **Découverte & Santé** | `Discover Versions`, `Query`, `Ping`, `Interop`, `Log` | 2.0 / 2.1 / 3.0 |
| **Création d'Objets & Clés** | `Create`, `Create Key Pair`, `Create Split Key`, `Register`, `Import`, `Derive Key`, `Join Split Key` | 2.0 / 2.1 / 3.0 |
| **Cycle de Vie & Rotation** | `Activate`, `Deactivate` (3.0), `Revoke`, `Destroy`, `Obliterate` (3.0), `Archive`, `Recover`, `Re-key`, `Re-key Key Pair`, `Re-Provision` | 2.0 / 2.1 / 3.0 |
| **Certificats** | `Certify`, `Re-certify`, `Validate` | 2.0 / 2.1 / 3.0 |
| **Récupération & Recherche** | `Locate`, `Get`, `Export`, `Check`, `Obtain Lease`, `Get Usage Allocation` | 2.0 / 2.1 / 3.0 |
| **Gestion d'Attributs** | `Get Attributes`, `Get Attribute List`, `Set Attribute`, `Add Attribute`, `Modify Attribute`, `Adjust Attribute`, `Delete Attribute` | 2.0 / 2.1 / 3.0 |
| **Opérations Cryptographiques** | `Encrypt`, `Decrypt`, `Sign`, `Signature Verify`, `MAC`, `MAC Verify`, `Hash`, `RNG Retrieve`, `RNG Seed` | 2.0 / 2.1 / 3.0 |
| **KEM Post-Quantique (3.0)** | `Encapsulate`, `Decapsulate` (`ML-KEM-512/768/1024`, `X25519MLKEM768`, `SECP256R1MLKEM768`) | **3.0** |
| **Session & Accès** | `Login`, `Logout`, `Delegated Login`, `Create User` (3.0), `Create Group` (3.0), `Create Credential` (3.0) | 2.0 / 2.1 / 3.0 |
| **Politiques & Admin** | `Get Constraints`, `Set Constraints`, `Set Defaults`, `PKCS#11`, `Set Endpoint Role` | 2.0 / 2.1 / 3.0 |
| **Contrôle Asynchrone** | `Poll`, `Cancel`, `Process`, `Query Asynchronous Requests` | 2.0 / 2.1 / 3.0 |

### Opérations Serveur-vers-Client (S2C)

| Opération | Rôle | Support Transport |
| :--- | :--- | :--- |
| **`Discover Versions`** | Le serveur sonde les versions KMIP supportées par le client | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Query`** | Le serveur interroge les capacités et espaces de noms du client | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Notify`** | Le serveur pousse des notifications de changement d'attribut/état | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Put`** | Le serveur pousse de nouvelles clés/certificats ou remplacements | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Set Endpoint Role`** | Négocie l'inversion de rôle sur une connexion TLS existante | Raw TLS Role-Reversed Stream |

---

## 3. Lots Atomiques Multi-Opérations & `ID Placeholder`

Exécutez plusieurs opérations dépendantes en un seul aller-retour réseau avec `ClientBatch` et `UniqueIdentifier::id_placeholder()` :

```rust
use kmipkit::client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit::protocol::{ActivateRequest, CreateRequest, GetRequest, ObjectType, UniqueIdentifier};

let mut batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::create(
    CreateRequest::builder(ObjectType::SymmetricKey).build()?,
)));

// Item 2 : activer la clé créée à l'Item 1 via ID Placeholder
batch.push(ClientBatchItem::new(ClientRequest::activate(
    ActivateRequest::new(UniqueIdentifier::id_placeholder()),
)));

// Item 3 : récupérer la clé activée dans le même échange atomique
batch.push(ClientBatchItem::new(ClientRequest::get(
    GetRequest::new(UniqueIdentifier::id_placeholder()),
)));

let outcome = client.execute(&batch)?;
assert_eq!(outcome.response().items().len(), 3);
```

---

## 4. Opérations Asynchrones (`Poll`, `Cancel`, `Process`)

Lorsqu'un serveur KMIP renvoie `ResultStatus::OperationPending` avec une `Asynchronous Correlation Value`, KMIPKit expose des méthodes explicites :

```rust
use kmipkit::protocol::{CancelRequest, PollRequest};

let poll_outcome = client.poll(&PollRequest::new(correlation_value.clone()))?;
let cancel_outcome = client.cancel(&CancelRequest::new(correlation_value))?;
```

---

## 5. Cryptographie Post-Quantique KMIP 3.0 (`ML-KEM` & `ML-DSA`)

Sous `ProtocolVersion::V3_0`, KMIPKit prend en charge NIST FIPS 203 (`ML-KEM`), FIPS 204 (`ML-DSA`) et FIPS 205 (`SLH-DSA`) :

```rust
let kp = client
    .create_key_pair()
    .algorithm(CryptographicAlgorithm::MlKem768)
    .send()?;

let encap = client.encapsulate(kp.public_key_unique_identifier().unwrap().clone()).send()?;
let decap = client
    .decapsulate(
        kp.private_key_unique_identifier().unwrap().clone(),
        encap.ciphertext().unwrap(),
    )
    .send()?;
```

---

## 6. Extensions Vendeur & Gestion de la Criticité

KMIPKit applique les règles OASIS **Message Extension** :
* **Extensions non critiques inconnues (`Criticality Indicator = false`) :** préservées intactes dans l'arbre TTLV générique.
* **Extensions critiques inconnues (`Criticality Indicator = true`) :** rejetées avec erreur de protocole.
* **Extensions vendeur enregistrées :** déclarez des schémas `ExtensionDefinition` dans `ClientExtensionRegistry` pour valider tags, énumérations et structures dans Rust, C, Java et Python.
