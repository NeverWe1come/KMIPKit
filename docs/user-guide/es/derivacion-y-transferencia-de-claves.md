# Derivación y transferencia de claves

KMIPKit envía operaciones Derive Key, Export, Import y Join Split Key a un
servidor KMIP 2.1. Estas operaciones siguen la especificación OASIS KMIP
Specification Version 2.1 §6.1.14 Tablas 205–207 y §7.12 Tabla 364,
§6.1.18 Tablas 217–219, §6.1.25 Tablas 238–240 y §6.1.27 Tablas 244–246.
Cada método explícito del cliente realiza un intercambio. KMIPKit no deriva
claves, envuelve ni desenvuelve objetos, importa material localmente, consulta
atributos del objeto ni reintenta peticiones.
El llamador elige Cryptographic Parameters; KMIPKit conserva los valores
aportados, incluido Block Cipher Mode, y no infiere el estado criptográfico de
un objeto de origen remoto. El servidor KMIP evalúa las condiciones de
derivación que dependen de ese contexto.

## Preparar valores controlados por el llamador

Elige los objetos de origen, el método de derivación, los atributos de salida,
la política de reemplazo, los selectores de exportación y el contexto de Split
Key según la política de tu aplicación. El siguiente ejemplo usa solo bytes
sintéticos: no contiene una clave ni contraseña utilizables. Sus funciones
auxiliares construyen los atributos y el objeto KMIP genérico que se envía a
Import.

Antes de llamar a Derive Key, selecciona objetos de origen cuya Cryptographic
Usage Mask incluya Derive Key. KMIPKit envía los identificadores indicados y
no consulta atributos remotos para comprobar esta política elegida por el
llamador.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatchItemResponse, ClientError};
use kmipkit_protocol::{
    AttributeSet, DerivationMethod, DerivationParameters, DeriveKeyRequest,
    ExportRequest, ImportRequest, JoinSplitKeyRequest, ObjectType, SecretBytes,
    SplitKeyReference, UniqueIdentifier,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const ATTR_HASHING_ALGORITHM: u32 = 0x0042_0038;

fn item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag)
        .expect("el tag cabe en TTLV")
        .try_checked()
        .expect("el tag está asignado");
    Item::new(tag, value).expect("el elemento TTLV es válido")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut value = Structure::new();
    for child in items {
        value.try_push(child).expect("la estructura sintética es válida");
    }
    value
}

fn output_attributes() -> AttributeSet {
    AttributeSet::try_new([
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
    ])
    .expect("los atributos sintéticos de salida son válidos")
}

fn synthetic_symmetric_key() -> Item {
    let key_value = structure([item(
        KEY_MATERIAL,
        Value::byte_string(b"synthetic-import-material".to_vec()),
    )]);
    let key_block = structure([
        item(KEY_FORMAT_TYPE, Value::enumeration(1)),
        item(KEY_VALUE, Value::structure(key_value)),
    ]);
    item(
        SYMMETRIC_KEY,
        Value::structure(structure([item(0x0042_0040, Value::structure(key_block))])),
    )
}

#[allow(dead_code)]
fn derive_key(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let parameters = structure([item(ATTR_HASHING_ALGORITHM, Value::enumeration(6))]);
    let derivation = DerivationParameters::new()
        .with_cryptographic_parameters(parameters)
        .with_salt(SecretBytes::new(b"synthetic-salt".to_vec()))
        .with_iteration_count(10_000);
    let request = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source-key-example".to_owned())],
        DerivationMethod::from_raw(1),
        derivation,
        output_attributes(),
    );
    client.derive_key(request, limits)
}

#[allow(dead_code)]
fn export_key(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = ExportRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("key-to-export-example".to_owned()));
    client.export(request, limits)
}

#[allow(dead_code)]
fn import_key(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = ImportRequest::new(
        UniqueIdentifier::TextString("imported-key-example".to_owned()),
        ObjectType::from_raw(2),
        output_attributes(),
        synthetic_symmetric_key(),
    )
    .with_replace_existing(false);
    client.import(request, limits)
}

#[allow(dead_code)]
fn join_split_key(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let references = vec![
        SplitKeyReference::new(UniqueIdentifier::TextString("split-a-example".to_owned()))
            .with_split_key_threshold(2),
        SplitKeyReference::new(UniqueIdentifier::TextString("split-b-example".to_owned()))
            .with_split_key_threshold(2),
    ];
    client.join_split_key(JoinSplitKeyRequest::new(ObjectType::from_raw(2), references), limits)
}

#[allow(dead_code)]
fn result_status(response: &ClientBatchItemResponse) -> u32 {
    response.outcome().result().status().raw()
}

#[allow(dead_code)]
fn inspect_export_without_formatting_material(response: &ClientBatchItemResponse) -> Option<usize> {
    response
        .outcome()
        .response()
        .export()
        .map(|typed| typed.with_ttlv(|item| item.children().len()))
}

fn main() {}
```

## API de Export en los cuatro lenguajes

Las cuatro API usan las mismas opciones de KMIP 2.1 §6.1.18 Tabla 217. El
identificador, Key Format Type, Key Wrap Type, Key Compression Type y Key
Wrapping Specification son opcionales; los selectores de enumeración conservan
los valores raw desconocidos.

| Lenguaje | Crear la petición | Revisar el resultado tipado |
|---|---|---|
| Rust | `Client::export(ExportRequest)` | `ClientBatchOutcome::Export` y `ExportResponse` |
| C | `kmipkit_client_batch_item_export` | `kmipkit_client_batch_item_response_export` y `kmipkit_client_batch_item_response_export_ttlv` |
| Java | `ClientBatchItem.export(...)` | `ClientBatchItemResult.exportResponse()` y `ExportResponse.close()` |
| Python | `client_batch_item_export(...)` | `BatchItemResult.export_response` y `ExportResponse.close()` |

Comprueba `Result Status` del elemento de lote antes de usar los campos de una
respuesta correcta. Un KMIP Failure sigue siendo un resultado Export tipado y
conserva el TTLV completo de respuesta en su orden original. En caso de éxito,
la Tabla 218 exige Object Type, Unique Identifier, Attributes y Any Object.
Object Type se expone como valor unsigned raw, Unique Identifier conserva su
tipo wire, y Attributes y Any Object permanecen como TTLV genérico. En C, Java
y Python, Any Object es una Structure de un elemento cuyo hijo conserva la
etiqueta original del elemento polimórfico y el orden.

Cierra las copias de respuesta propiedad del llamador al terminar. KMIPKit pone
a cero la memoria que posee, pero los runtimes de Java y Python pueden conservar
copias externas fuera de ese control. No formatees, registres ni incluyas los
datos del objeto, Attributes o el TTLV raw de respuesta en errores.

## API de Import en los cuatro lenguajes

Import sigue OASIS KMIP Specification v2.1 §6.1.25, Tablas 238–240. El
llamador aporta el Unique Identifier, el Object Type raw, Attributes y un
elemento Any Object completo. `Replace Existing` se omite hasta que el llamador
lo selecciona; por eso `false` explícito se distingue de la ausencia. Para un
objeto de clave envuelto, aporta el Key Wrap Type raw cuando el objeto contiene
Key Wrapping Data. KMIPKit conserva la representación recibida y no realiza
unwrap ni operaciones criptográficas locales.

Any Object se construye con el modelo TTLV genérico. Los valores Big Integer
están disponibles en Rust, C, Java y Python; desde C se puede usar
`kmipkit_ttlv_value_big_integer(limits, bytes, length, &value)`. La cabecera C
documenta cuándo los constructores transfieren la propiedad de handles opacos.

| Lenguaje | Crear la petición | Revisar un resultado correcto |
|---|---|---|
| Rust | `ImportRequest::new(...)` y `Client::import(...)` | `ClientBatchOutcome::Import` e `ImportResponse` |
| C | `kmipkit_client_batch_item_import` | `kmipkit_client_batch_item_response_import` y `kmipkit_client_batch_item_response_import_ttlv` |
| Java | `ClientBatchItem.importObject(...)` | `ClientBatchItemResult.importResponse()` y `ImportResponse.close()` |
| Python | `client_batch_item_import(...)` | `BatchItemResult.import_response` e `ImportResponse.close()` |

En Success, el resultado tipado expone el Unique Identifier resultante y una
copia propia del TTLV completo y ordenado, incluidos los campos desconocidos.
Pending y Failure siguen siendo resultados comunes del lote y no exponen un
TTLV de respuesta de éxito. Comprueba `Result Status` antes de usar datos
tipados de éxito. Cada operación Import se envía una vez; KMIPKit no reintenta,
consulta Pending ni envía una operación posterior. Cierra las copias de
respuesta al terminar. KMIPKit pone a cero la memoria que posee; los runtimes de
Java y Python pueden conservar copias externas fuera de ese control.

El llamador aporta cada identificador exacto y el umbral conocido de su Split
Key. KMIPKit requiere un contexto por ocurrencia y, antes de enviar, comprueba
que el número de entradas —incluidos identificadores repetidos— alcance el
mayor umbral suministrado. Los umbrales son metadatos locales: no se codifican
ni se consultan al servidor. La petición conserva el orden y las repeticiones.

## Revisar resultados y valores sensibles

Comprueba `Result Status` antes de utilizar el resultado de una operación. El
objeto exportado y el TTLV de respuesta pueden contener material secreto. No
los registres, formatees ni copies en cadenas ordinarias. Los bytes secretos
propiedad de KMIPKit se ponen a cero al liberarse; las copias creadas por el
llamador quedan fuera de esa garantía. Al importar una clave envuelta, indica
el Key Wrap Type elegido por el llamador exactamente cuando el Key Block del
objeto contiene Key Wrapping Data. `Replace Existing` solo se establece si el
llamador lo elige explícitamente.

Cada método devuelve el resultado del servidor para una sola petición. Un
fallo de validación local indica `NotSent`; los fallos de transporte conservan
la evidencia de entrega más precisa disponible. Los resultados Pending siguen
siendo explícitos si el llamador habilita el manejo asíncrono; KMIPKit no
consulta su estado automáticamente.

## API de Join Split Key en los cuatro lenguajes

Join Split Key sigue OASIS KMIP Specification v2.1 §6.1.27, Tablas 244–246.
Cada referencia empareja una ocurrencia de Unique Identifier con el Split Key
Threshold conocido por quien llama para esa clave. Crea una referencia por
cada entrada transmitida, incluidos los identificadores repetidos. Antes de
enviar, KMIPKit exige al menos una referencia y comprueba que el número de
entradas sea mayor o igual que el mayor umbral suministrado. Los umbrales son
locales: nunca se serializan ni se consultan al servidor, y pueden diferir
entre entradas. KMIPKit no combina localmente el material de las claves.

Los bindings C, Java y Python limitan a 16 MiB el total de bytes UTF-8 de los
identificadores de texto de una petición Join Split Key, antes de copiarlos a
la petición nativa. Cada ocurrencia repetida cuenta para este límite; si se
supera, la llamada devuelve un error de límite de recursos.

La Tabla 244 exige Object Type y una o más entradas Unique Identifier. La
petición conserva su orden y las repeticiones. Secret Data Type, Attributes y
Protection Storage Masks son opcionales. En Success, la Tabla 245 exige el
Unique Identifier resultante. Failure y Pending siguen siendo resultados
comunes del lote; comprueba Result Status y conserva los metadatos compartidos
de resultado y entrega antes de usar los campos de una respuesta correcta.

| Lenguaje | Emparejar identificadores y umbrales locales | Revisar un resultado correcto |
|---|---|---|
| Rust | `SplitKeyReference::new(identifier).with_split_key_threshold(threshold)` | `ClientBatchOutcome::JoinSplitKey` y `JoinSplitKeyResponse::with_ttlv` |
| C | Crear cada valor opaco con `kmipkit_split_key_reference_create`; pasar el array ordenado a `kmipkit_client_batch_item_join_split_key`; liberar cada handle después de la llamada | `kmipkit_client_batch_item_response_join_split_key` y `kmipkit_client_batch_item_response_join_split_key_ttlv` |
| Java | `new SplitKeyReference(identifier, threshold)` en la lista ordenada de `ClientBatchItem.joinSplitKey(...)`; cerrar cada referencia | `ClientBatchItemResult.joinSplitKeyResponse()` y `JoinSplitKeyResponse.close()` |
| Python | `extensions.SplitKeyReference(identifier, threshold)` en la secuencia ordenada de `extensions.client_batch_item_join_split_key(...)`; cerrar cada referencia | `BatchItemResult.join_split_key_response` y `JoinSplitKeyResponse.close()` |

El accesor C de TTLV devuelve una copia propia e independiente de la respuesta
completa y ordenada, incluidos los campos desconocidos; libérala con
`kmipkit_ttlv_structure_release`. Las respuestas Java y Python poseen sus
copias TTLV y deben cerrarse. Los arrays C solo toman prestados los handles de
referencia durante la llamada al constructor, que copia al pedido el
identificador y su contexto de umbral. Java y Python poseen sus handles de
referencia y pueden conservar copias externas en el runtime, que no garantiza
su puesta a cero determinista. Evita registrar o formatear identificadores y
el TTLV raw de respuesta. Estas pruebas de API no afirman haber superado
fixtures oficiales, conformidad OASIS, interoperabilidad, soporte de perfil
ni certificación.
