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

Para los valores de clave envueltos, el llamador aporta IV/Counter/Nonce
siempre que lo requiera el método de envoltura elegido. Si falta Encoding
Option, debe codificar como TTLV la estructura Key Value antes de envolverla.
Estas obligaciones proceden de KMIP Specification v2.1 §3.3, Tablas 14–15.
KMIPKit conserva la representación aportada: no deduce parámetros a partir de
claves referenciadas del servidor ni inspecciona el texto claro o cifrado
envuelto.

Cada método devuelve el resultado del servidor para una sola petición. Un
fallo de validación local indica `NotSent`; los fallos de transporte conservan
la evidencia de entrega más precisa disponible. Los resultados Pending siguen
siendo explícitos si el llamador habilita el manejo asíncrono; KMIPKit no
consulta su estado automáticamente.
