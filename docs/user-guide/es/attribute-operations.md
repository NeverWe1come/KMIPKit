# Operaciones de atributos

KMIPKit ofrece modelos Rust tipados de petición y respuesta para las siete operaciones de atributos iniciadas por el cliente en KMIP 2.1. Los modelos construyen el payload de la operación; la API de ejecución del cliente lo envía mediante el transporte configurado. Consulta [Ejecución del cliente](client-execution.md) para configurar el transporte, interpretar el estado de entrega y tratar resultados asíncronos.

Las cinco operaciones de mutación también tienen modelos de petición independientes en C, Java y Python. Así, la petición inspeccionable se mantiene separada del elemento de batch que se usa para enviarla:

| Lenguaje | Modelo Add Attribute | Copia del payload | Copia del elemento de batch |
| --- | --- | --- | --- |
| C | `kmipkit_add_attribute_request_t` | `kmipkit_add_attribute_request_to_ttlv_payload` | `kmipkit_client_batch_item_add_attribute_from_request` |
| Java | `org.kmipkit.extensions.AddAttributeRequest` | `toTtlvPayload()` | `toBatchItem()` |
| Python | `kmipkit.attribute_operations.AddAttributeRequest` | `to_ttlv_payload()` | `to_batch_item()` |

Se aplica el mismo patrón a Adjust, Delete, Modify y Set Attribute. Cada constructor copia los campos recibidos en un modelo propio. Los accesores TTLV devuelven copias independientes; los identificadores, las referencias y los valores raw de Adjustment Type se pueden inspeccionar directamente. La conversión a elemento de batch deja abierto el modelo de petición; el llamador libera ambos objetos según las reglas de propiedad de su lenguaje.

Usa `create` para construir el modelo independiente, el método del payload para inspeccionar o codificar sus campos y la conversión a elemento de batch para añadir una copia independiente a la ejecución del cliente.

| Operación | Campos de petición | Respuesta satisfactoria |
| --- | --- | --- |
| Add Attribute | Unique Identifier opcional; un New Attribute | Unique Identifier |
| Adjust Attribute | Unique Identifier opcional; Attribute Reference; Adjustment Type; Adjustment Value opcional | Unique Identifier |
| Delete Attribute | Unique Identifier, Current Attribute y Attribute Reference opcionales | Unique Identifier |
| Get Attributes | Unique Identifier opcional y Attribute References ordenados; una lista vacía solicita todos los atributos | Unique Identifier; Attributes se omite si no existe ninguno de los atributos solicitados y se distingue de una Structure presente pero vacía |
| Get Attribute List | Unique Identifier opcional | Unique Identifier y Attribute References ordenados |
| Modify Attribute | Unique Identifier y Current Attribute opcionales; un New Attribute | Unique Identifier |
| Set Attribute | Unique Identifier opcional; un New Attribute | Unique Identifier |

Estas formas siguen OASIS KMIP v2.1 §§5.5–5.7 y §6.1.2 Tablas 167–169, §6.1.3 Tablas 170–172, §6.1.13 Tablas 202–204, §6.1.20 Tablas 223–225, §6.1.21 Tablas 226–228, §6.1.34 Tablas 265–267 y §6.1.51 Tablas 322–324.

## Construir peticiones tipadas

El ejemplo construye los siete payloads. Los atributos estándar pueden usar marcadores tipados derivados del catálogo; la ruta de `Item` TTLV genérico sigue disponible para tags de extensiones y fabricantes. Current Attribute y New Attribute aceptan ambas formas. El ejemplo usa el tag asignado a `Comment` (`0x4200FD`).

```rust
use kmipkit_protocol::{
    AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, AttributeReference,
    Attribute, AttributeSet, CurrentAttribute, DeleteAttributeRequest, GetAttributeListRequest,
    GetAttributesRequest, ModifyAttributeRequest, NewAttribute, SetAttributeRequest,
    attribute::typed,
};
use kmipkit_ttlv::{Item, RawTag, Tag, Value};

fn allocated_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("el ejemplo usa un tag KMIP de 24 bits")
        .try_checked()
        .expect("Comment está asignado en KMIP 2.1")
}

fn comment(value: &str) -> Result<Attribute<typed::Comment>, kmipkit_protocol::AttributeValueError> {
    Attribute::new(value.to_owned())
}

fn build_attribute_payloads() -> Result<(), Box<dyn std::error::Error>> {
    let object = Some("object-id-17".to_owned());
    let by_tag = AttributeReference::tag(0x0042_00FD);

    let mut attributes = AttributeSet::new();
    attributes.try_push_typed(comment("finance")?)?;
    let comment_value = attributes
        .typed::<typed::Comment>()
        .next()
        .map(|comment| comment.with_value(str::to_owned))
        .transpose()?
        .ok_or("se acaba de insertar el Comment tipado")?;
    assert_eq!(comment_value, "finance");

    // Los tags de fabricante desconocidos siguen disponibles mediante Item genérico.
    attributes.try_push(Item::new(
        allocated_tag(0x0054_1234),
        Value::byte_string(vec![0x01, 0x02]),
    )?)?;

    let add = AddAttributeRequest::new(
        object.clone(),
        NewAttribute::from_typed(comment("finance")?),
    );
    let _add_payload = add.to_ttlv_payload()?;

    let adjust = AdjustAttributeRequest::new(
        object.clone(),
        by_tag.clone(),
        AdjustmentType::INCREMENT,
        Some(Value::integer(1)),
    );
    let _adjust_payload = adjust.to_ttlv_payload()?;

    let delete = DeleteAttributeRequest::new(
        object.clone(),
        Some(CurrentAttribute::from_typed(comment("legacy")?)),
        None,
    );
    let _delete_payload = delete.to_ttlv_payload()?;

    let get_attributes = GetAttributesRequest::try_new(
        object.clone(),
        [by_tag.clone(), AttributeReference::name("example.org", "CostCenter")],
    )?;
    let _get_attributes_payload = get_attributes.to_ttlv_payload()?;

    let get_attribute_list = GetAttributeListRequest::new(object.clone());
    let _get_attribute_list_payload = get_attribute_list.to_ttlv_payload()?;

    let modify = ModifyAttributeRequest::new(
        object.clone(),
        Some(CurrentAttribute::from_typed(comment("legacy")?)),
        NewAttribute::from_typed(comment("current")?),
    );
    let _modify_payload = modify.to_ttlv_payload()?;

    let set = SetAttributeRequest::new(
        object,
        NewAttribute::from_typed(comment("current")?),
    );
    let _set_payload = set.to_ttlv_payload()?;

    Ok(())
}
```

`AttributeReference::tag` conserva la identidad raw del tag; `AttributeReference::name` conserva exactamente las cadenas Vendor Identification y Attribute Name. Elige la forma que corresponda al atributo y al servidor. KMIPKit no deduce un identificador de fabricante a partir de un tag.

## Conservar valores y consultar resultados

`AttributeSet::try_push_typed` y `AttributeSet::typed` construyen y leen atributos estándar sin comparar manualmente tags raw. Los valores Enumeration siguen siendo `u32` abiertos: se conservan incluso si esta versión no los conoce. La ruta de `Item` genérico sigue disponible para extensiones de fabricante. Ambas rutas conservan el orden y las instancias repetidas; la validación de cada operación sigue aplicando las reglas de cardinalidad definidas por la fuente. Adjustment Type admite Increment, Decrement y Negate, además del rango de extensiones KMIP; los valores Reserved se rechazan en peticiones tipadas de salida.

Las fachadas C, Java y Python incluyen los nombres de los 62 atributos
estándar directos del catálogo. Sus wrappers `TypedAttribute` validan y copian
un `Item` TTLV existente con respecto al atributo elegido; no sustituyen el
acceso genérico ni infieren un valor. En C se usa
`kmipkit_typed_attribute_from_item`; Java usa
`TypedAttribute.fromItem(StandardAttribute, TtlvItem)` y Python,
`TypedAttribute.from_item(StandardAttribute, TtlvItem)`. Los wrappers Java y
Python son propietarios y cerrables; C devuelve un handle que se libera con
`kmipkit_ttlv_item_release`.

```java
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TypedAttribute;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;

final class TypedAttributeExample {
    public static void main(String[] args) {
        try (CodecLimits limits = CodecLimits.defaults()) {
            try (RawTag raw = RawTag.fromRaw(0x00420001);
                    Tag tag = raw.checked();
                    TtlvValue date = TtlvValue.DateTime(1_700_000_000L);
                    TtlvItem source = TtlvItem.create(tag, date, limits);
                    TypedAttribute attribute = TypedAttribute.fromItem(
                            StandardAttribute.ACTIVATION_DATE, source);
                    TtlvItem copy = attribute.asItem()) {
                if (attribute.attribute() != StandardAttribute.ACTIVATION_DATE) {
                    throw new AssertionError("attribute identity changed");
                }
            }
        }
    }
}
```

```python
from kmipkit import ttlv
from kmipkit.typed_attributes import StandardAttribute, TypedAttribute

with ttlv.codec_limits_defaults() as limits:
    with ttlv.raw_tag_from_raw(0x0042_0001) as raw:
        with ttlv.raw_tag_try_checked(raw) as tag:
            with ttlv.ttlv_value_date_time(1_700_000_000) as value:
                with ttlv.ttlv_item_create(tag, value, limits) as source:
                    with TypedAttribute.from_item(
                        StandardAttribute.ACTIVATION_DATE, source
                    ) as attribute:
                        assert attribute.attribute is StandardAttribute.ACTIVATION_DATE
```

Para el ABI C, el consumidor crea el `Item` genérico con el codec TTLV y luego
valida/copia el atributo con la firma `kmipkit_typed_attribute_from_item(
expected_tag, item, &out_item)`. Al terminar, libera ambas copias por separado;
la conversión no consume el `Item` fuente.

KMIP 2.1 §4 recomienda que las distintas instancias de un mismo atributo tengan valores diferentes. El llamador elige esos valores; KMIPKit los conserva tal como se proporcionan y no los reescribe ni elimina duplicados.

Los modelos de respuesta exponen el `KmipOperationResult` del servidor, incluidos Result Status, Result Reason opcional y Result Message opcional. Los accesores de payload satisfactorio exponen Unique Identifier y, en las lecturas, los atributos o referencias devueltos. KMIPKit no aplica localmente Adjust Attribute ni mantiene una copia local de los atributos del servidor.

Las comprobaciones de política de mutación rechazan solo prohibiciones incondicionales respaldadas por la fuente que puedan determinarse con la petición suministrada. Esos fallos tienen estado de entrega `NotSent` y no invocan el transporte. Los casos dependientes del estado remoto o no reconocidos se envían al servidor. Las peticiones no se reintentan automáticamente. Un resultado `Pending` conserva la respuesta tipada y el valor de correlación asíncrona; consulta el flujo asíncrono en [Ejecución del cliente](client-execution.md).

Las pruebas del protocolo usan vectores derivados y entradas malformadas. No afirman que todos los servidores KMIP admitan todos los atributos ni que se haya superado un caso oficial de OASIS cuyo fixture no está disponible.

## Petición fluida Set Attribute

En Rust, se puede construir y enviar Set Attribute mediante el cliente tipado existente, sin ensamblar a mano la estructura TTLV de New Attribute ni crear un batch:

```rust
use std::error::Error;

use kmipkit_client::Client;
use kmipkit_protocol::{attribute::typed, Attribute};
use kmipkit_ttlv::codec::CodecLimits;

fn set_comment(client: &mut Client) -> Result<String, Box<dyn Error>> {
    let response = client
        .request()
        .set_attribute()
        .unique_identifier("object-id-17")
        .attribute(Attribute::<typed::Comment>::new("managed by service".to_owned())?)
        .execute(&CodecLimits::defaults())?;

    response
        .outcome()
        .response()
        .set_attribute()
        .and_then(|result| result.unique_identifier())
        .map(str::to_owned)
        .ok_or_else(|| "Set Attribute did not return a successful identifier".into())
}
```

El builder valida localmente el valor catalogado y el payload de la operación. También ofrece `build`, `validate` y `to_batch_item` para controlar cada paso, y `attribute_item` para un `Item` TTLV con tipo explícito. Usa `vendor_attribute` para construir la estructura KMIP Vendor Attribute a partir del identificador del fabricante, el nombre y el valor. La petición usa el cliente existente en un único intercambio, sin Poll ni reintentos.

## Petición fluida Add Attribute

Add Attribute implementa KMIP v2.1 §6.1.2, Table 167; su New Attribute usa la estructura definida en §5.7, Table 163. La petición fluida construye el request tipado existente para añadir un atributo a un objeto identificado y acepta un `Attribute` catalogado, un `Item` TTLV con tipo explícito o un Vendor Attribute construido a partir de sus campos:

```rust
use std::error::Error;

use kmipkit_client::Client;
use kmipkit_protocol::{attribute::typed, Attribute};
use kmipkit_ttlv::codec::CodecLimits;

fn add_comment(client: &mut Client) -> Result<String, Box<dyn Error>> {
    let response = client
        .request()
        .add_attribute()
        .unique_identifier("object-id-17")
        .attribute(Attribute::<typed::Comment>::new("reviewed by service".to_owned())?)
        .execute(&CodecLimits::defaults())?;

    response
        .outcome()
        .response()
        .add_attribute()
        .and_then(|result| result.unique_identifier())
        .map(str::to_owned)
        .ok_or_else(|| "Add Attribute did not return a successful identifier".into())
}
```

La operación requiere un único New Attribute. El builder valida localmente el tipo TTLV catalogado y el payload; ofrece `build`, `validate` y `to_batch_item` para controlar cada paso. `attribute_item` acepta un `Item` con tipo explícito y `vendor_attribute` construye la estructura KMIP Vendor Attribute.

| Lenguaje | Iniciar la petición | Añadir un atributo | Enviar y liberar |
| --- | --- | --- | --- |
| C | `kmipkit_add_attribute_builder_create` | `..._add_catalogued_attribute`, `..._add_attribute_item` o `..._add_vendor_attribute` | `..._execute_v1` o `..._build_batch_item`; liberar builder y resultados |
| Java | `new FluentRequest(client).addAttribute()` | `cataloguedAttribute`, `attributeItem` o `vendorAttribute` | `execute(limits)` o `build`; cerrar builder y resultados |
| Python | `fluent.Request(client).add_attribute()` | `catalogued_attribute`, `attribute_item` o `vendor_attribute` | `execute(limits)` o `build`; usar el builder como gestor de contexto |

El intercambio usa una sola vez el cliente existente y no realiza Poll ni reintentos. Las comprobaciones de política del cliente pueden rechazar atributos prohibidos antes del transporte y marcar el envío como `NotSent`.

El mismo builder está disponible en las fachadas de los cuatro lenguajes:

| Lenguaje | Iniciar la petición | Atributo catalogado | TTLV experto o atributo de fabricante | Enviar y liberar |
| --- | --- | --- | --- | --- |
| C | `kmipkit_set_attribute_builder_create` | `kmipkit_set_attribute_builder_add_catalogued_attribute(..., KMIPKIT_STANDARD_ATTRIBUTE_DESCRIPTION, value)` | `..._add_attribute_item` o `..._add_vendor_attribute` | `..._execute_v1` o `..._build_batch_item`; liberar builder y resultados |
| Java | `new FluentRequest(client).setAttribute()` | `cataloguedAttribute(StandardAttribute.DESCRIPTION, value)` | `attributeItem` o `vendorAttribute` | `execute(limits)` o `build`; cerrar el builder y sus resultados |
| Python | `fluent.Request(client).set_attribute()` | `catalogued_attribute(StandardAttribute.DESCRIPTION, value)` | `attribute_item` o `vendor_attribute` | `execute(limits)` o `build`; usar el builder como gestor de contexto |

Los valores catalogados se validan según el tipo TTLV del atributo. Los tags desconocidos requieren un `Item` con tipo explícito. El helper de fabricante construye la estructura Vendor Attribute del protocolo; un tag de fabricante por sí solo no representa esa estructura. Las pruebas del consumidor C compilado, de Java/JNI y de Python/CFFI verifican que la petición tipada y su conversión a batch conservan el mismo tipo de operación.

## Petición fluida Delete Attribute

Delete Attribute sigue OASIS KMIP v2.1 §6.1.13, Tablas 202–204. Current Attribute usa el `Item` directo definido en §5.6, Tabla 162, y Attribute Reference sigue §5.5, Tabla 161. El builder fluido crea el request tipado existente; no envía TTLV arbitrario. El llamador puede seleccionar el atributo por su valor actual, por una referencia o por ambos. El validador de la petición y el servidor determinan si la combinación es válida y si se puede eliminar ese atributo.

En Rust se puede usar un atributo catalogado sin construir a mano la estructura TTLV de Current Attribute ni un batch:

```rust
use std::error::Error;

use kmipkit_client::Client;
use kmipkit_protocol::{attribute::typed, Attribute};
use kmipkit_ttlv::codec::CodecLimits;

fn delete_comment(client: &mut Client) -> Result<String, Box<dyn Error>> {
    let response = client
        .request()
        .delete_attribute()
        .unique_identifier("object-id-17")
        .current_attribute(Attribute::<typed::Comment>::new("legacy value".to_owned())?)
        .execute(&CodecLimits::defaults())?;

    response
        .outcome()
        .response()
        .delete_attribute()
        .and_then(|result| result.unique_identifier())
        .map(str::to_owned)
        .ok_or_else(|| "Delete Attribute did not return a successful identifier".into())
}
```

| Lenguaje | Iniciar la petición | Seleccionar el atributo actual | Validar o enviar |
| --- | --- | --- | --- |
| Rust | `client.request().delete_attribute()` | `current_attribute(Attribute<D>)`, `current_attribute_item(Item)`, `current_vendor_attribute(...)`, `attribute_reference(...)` | `validate`, `build_request`, `to_batch_item` o un único `execute` |
| C | `kmipkit_delete_attribute_builder_create` | `..._set_catalogued_attribute`, `..._set_current_attribute_item`, `..._set_vendor_attribute`, `..._set_attribute_reference` | `..._validate`, `..._build_request`, `..._build_batch_item` o un único `..._execute_v1`; liberar cada handle propietario |
| Java | `new FluentRequest(client).deleteAttribute()` | `cataloguedAttribute`, `currentAttribute(TtlvItem)`, `currentVendorAttribute`, `attributeReference` | `validate`, `buildRequest`, `toBatchItem` o un único `execute(limits)`; cerrar los valores propios |
| Python | `fluent.Request(client).delete_attribute()` | `catalogued_attribute`, `current_attribute_item`, `current_vendor_attribute`, `attribute_reference` | `validate`, `build_request`, `to_batch_item` o un único `execute(limits)`; usar el builder como gestor de contexto |

Los valores catalogados se comprueban contra el tipo TTLV del atributo. Los tags desconocidos requieren un `Item` con tipo explícito y el helper de fabricante construye la estructura KMIP Vendor Attribute. Las opciones de transporte se configuran con `with_options`; el Request Header Asynchronous Indicator tiene su propio setter. Cada `execute` realiza un intercambio mediante el cliente existente, sin Poll ni reintentos. Java y Python mantienen copias gestionadas por sus runtimes además del propietario nativo; cierra el request, el elemento de batch, el builder y la respuesta según la API de cada lenguaje.
