# Objetos gestionados y material de clave

El núcleo de KMIPKit proporciona modelos para las nueve estructuras de objetos
gestionados de KMIP 2.1. Los builders Rust emiten los campos conocidos en el
orden de las tablas OASIS. C, Java y Python también ofrecen vistas tipadas de
los objetos y estructuras de clave sobre el modelo TTLV compartido. El Item
Structure específico del objeto se pasa directamente a Register o Import; la
Enumeration `Object Type` es un valor separado.

## Material de clave en claro

Elige explícitamente el Key Format Type. Para bytes Raw, puedes construir un
valor tipado así:

```rust
use kmipkit_protocol::{
    AttributeSet, KeyBlock, KeyBlockValue, KeyFormatType, KeyMaterial, KeyValue,
    SymmetricKey,
};

let material = KeyMaterial::byte_string(1, vec![0xA1, 0xA2])?;
let value = KeyValue::try_new(material, Some(AttributeSet::new()))?;
let block = KeyBlock::try_new(KeyFormatType::from_raw(1))?
    .with_key_value(KeyBlockValue::Plaintext(value));
let symmetric_key = SymmetricKey::try_new(block)?;
let object_item = symmetric_key.into_item();
assert_eq!(object_item.tag().raw(), 0x0042_008F);
# Ok::<(), Box<dyn std::error::Error>>(())
```

El formato y los atributos criptográficos son decisiones del llamador. KMIPKit
no infiere el algoritmo, tamaño de clave, uso, política de protección ni
compatibilidad del servidor.

## Material de clave envuelto

Los bytes envueltos usan un `Key Value` TTLV de tipo Byte String y conservan
los metadatos de envoltura suministrados. KMIPKit transporta esos datos; no
envuelve, desenvuelve, cifra, firma ni calcula MACs.

```rust
use kmipkit_protocol::{
    EncryptionKeyInformation, KeyBlock, KeyBlockValue, KeyFormatType,
    KeyWrappingData, SymmetricKey,
};

let wrapping = KeyWrappingData::try_new(1)?
    .with_encryption_key_information(
        EncryptionKeyInformation::try_new("wrapping-key".to_owned())?,
    );
let block = KeyBlock::try_new(KeyFormatType::from_raw(1))?
    .with_key_value(KeyBlockValue::Wrapped(vec![0xB1, 0xB2]))
    .with_key_wrapping_data(wrapping);
let symmetric_key = SymmetricKey::try_new(block)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Lectura y conservación de datos

`try_from_item` lee estructuras tipadas sin descartar descendientes
desconocidos. `with_raw` presta su `StructureView` ordenado solo durante la
ejecución del callback. Así se conservan tags de fabricante, elementos
repetidos, el orden original y valores futuros desconocidos. La vista
representa el árbol TTLV decodificado, no una copia byte por byte de la trama
de red.

`TransparentSymmetricKey`, `TransparentDsaPrivateKey`,
`TransparentRsaPrivateKey` y los demás modelos transparentes ofrecen builders
con la forma de las tablas. Los demás formatos de Key Material siguen
disponibles como bytes o Structures genéricas elegidas por el llamador. Un
Object Type desconocido puede transportarse con `ManagedObject::unknown` sin
interpretar su contenido.

### Material de clave transparente

```rust
use kmipkit_protocol::TransparentSymmetricKey;

let transparent = TransparentSymmetricKey::try_new(vec![0xC1, 0xC2])?;
let key_material_item = transparent.into_item();
assert_eq!(key_material_item.tag().raw(), 0x0042_0043);
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Fallback para objetos desconocidos

Los Object Type desconocidos y las estructuras de fabricante siguen usando TTLV
genérico:

```rust
use kmipkit_protocol::{ManagedObject, ObjectType};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

let tag = RawTag::new(0x0054_0102)?.try_checked()?;
let extension = Item::new(tag, Value::structure(Structure::new()))?;
let object = ManagedObject::unknown(ObjectType::from_raw(0x8000_0001), extension)?;
assert_eq!(object.object_type().raw(), 0x8000_0001);
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Acceso desde C, Java y Python

Los wrappers extranjeros copian y validan el `Item` TTLV de origen; no lo
consumen. Cada resultado es un handle propio y debe liberarse. Sus vistas TTLV
permiten recorrer los campos en el orden original y llegar a tags de fabricante
anidados, sin convertirlos a una representación conocida.

| Estructura | C | Java | Python |
|---|---|---|---|
| Objeto gestionado | `kmipkit_managed_object_from_item` | `ManagedObject.fromItem` | `ManagedObject.from_item` |
| Key Block | `kmipkit_key_block_from_item` | `KeyBlock.fromItem` | `KeyBlock.from_item` |
| Key Value | `kmipkit_key_value_from_item` | `KeyValue.fromItem` | `KeyValue.from_item` |
| Key Material | `kmipkit_key_material_from_item` | `KeyMaterial.fromItem` | `KeyMaterial.from_item` |
| Key Wrapping Data | `kmipkit_key_wrapping_data_from_item` | `KeyWrappingData.fromItem` | `KeyWrappingData.from_item` |

En C, libera cada handle con su función `*_release`; los punteros de una vista
prestada dejan de ser válidos al liberar el propietario. Los wrappers Java
implementan `AutoCloseable`; los Python admiten `with` y `close()`. Si copias
bytes de material secreto a un `byte[]` de Java o a un `bytes` de Python, esa
copia queda fuera de la garantía de puesta a cero de KMIPKit. Java permite
sobrescribir su array; Python no ofrece borrado determinista de `bytes`.
Mantén los datos como vistas prestadas cuando sea posible y no los registres en
logs ni en diagnósticos.

## Vida útil de los secretos

Los payloads `Item` y `Value` propiedad de KMIPKit ocultan los bytes en `Debug`
y ponen a cero sus buffers propios al liberarse. Mover un modelo tipado
transfiere ese mismo propietario. KMIPKit no puede limpiar el buffer original
del llamador, las copias retenidas por la aplicación o un runtime externo, ni
el almacenamiento de TLS y las bibliotecas de plataforma.

Este ejemplo transfiere el buffer al propietario TTLV de KMIPKit. Al liberarlo,
KMIPKit pone a cero su propia copia:

```rust
use kmipkit_protocol::KeyMaterial;

let caller_owned_secret = vec![0xD1, 0xD2];
let secret_owner = KeyMaterial::byte_string(1, caller_owned_secret)?;
drop(secret_owner);
# Ok::<(), Box<dyn std::error::Error>>(())
```
