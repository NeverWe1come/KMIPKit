# Operaciones de atributos

KMIPKit ofrece modelos Rust tipados de petición y respuesta para las siete operaciones de atributos iniciadas por el cliente en KMIP 2.1. Los modelos construyen el payload de la operación; la API de ejecución del cliente lo envía mediante el transporte configurado. Consulta [Ejecución del cliente](client-execution.md) para configurar el transporte, interpretar el estado de entrega y tratar resultados asíncronos.

| Operación | Campos de petición | Respuesta satisfactoria |
| --- | --- | --- |
| Add Attribute | Unique Identifier opcional; un New Attribute | Unique Identifier |
| Adjust Attribute | Unique Identifier opcional; Attribute Reference; Adjustment Type; Adjustment Value opcional | Unique Identifier |
| Delete Attribute | Unique Identifier, Current Attribute y Attribute Reference opcionales | Unique Identifier |
| Get Attributes | Unique Identifier opcional y Attribute References ordenados; una lista vacía solicita todos los atributos | Unique Identifier e Items de atributos directos ordenados |
| Get Attribute List | Unique Identifier opcional | Unique Identifier y Attribute References ordenados |
| Modify Attribute | Unique Identifier y Current Attribute opcionales; un New Attribute | Unique Identifier |
| Set Attribute | Unique Identifier opcional; un New Attribute | Unique Identifier |

Estas formas siguen OASIS KMIP v2.1 §§5.5–5.7 y §6.1.2 Tablas 167–169, §6.1.3 Tablas 170–172, §6.1.13 Tablas 202–204, §6.1.20 Tablas 223–225, §6.1.21 Tablas 226–228, §6.1.34 Tablas 265–267 y §6.1.51 Tablas 322–324.

## Construir peticiones tipadas

El ejemplo construye los siete payloads. Un atributo es un `Item` TTLV genérico: su tag identifica el atributo y su valor conserva el tipo KMIP. Current Attribute y New Attribute envuelven ese único Item directo. El ejemplo usa el tag asignado a `Comment` (`0x4200FD`).

```rust
use kmipkit_protocol::{
    AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, AttributeReference,
    CurrentAttribute, DeleteAttributeRequest, GetAttributeListRequest,
    GetAttributesRequest, ModifyAttributeRequest, NewAttribute, SetAttributeRequest,
};
use kmipkit_ttlv::{Item, RawTag, Tag, Value};

fn allocated_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("el ejemplo usa un tag KMIP de 24 bits")
        .try_checked()
        .expect("Comment está asignado en KMIP 2.1")
}

fn comment(value: &str) -> Item {
    Item::new(
        allocated_tag(0x0042_00FD),
        Value::text_string(value.to_owned()),
    )
    .expect("el Item del ejemplo es estructuralmente válido")
}

fn build_attribute_payloads() -> Result<(), Box<dyn std::error::Error>> {
    let object = Some("object-id-17".to_owned());
    let by_tag = AttributeReference::tag(0x0042_00FD);

    let add = AddAttributeRequest::new(
        object.clone(),
        NewAttribute::new(comment("finance")),
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
        Some(CurrentAttribute::new(comment("legacy"))),
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
        Some(CurrentAttribute::new(comment("legacy"))),
        NewAttribute::new(comment("current")),
    );
    let _modify_payload = modify.to_ttlv_payload()?;

    let set = SetAttributeRequest::new(object, NewAttribute::new(comment("current")));
    let _set_payload = set.to_ttlv_payload()?;

    Ok(())
}
```

`AttributeReference::tag` conserva la identidad raw del tag; `AttributeReference::name` conserva exactamente las cadenas Vendor Identification y Attribute Name. Elige la forma que corresponda al atributo y al servidor. KMIPKit no deduce un identificador de fabricante a partir de un tag.

## Conservar valores y consultar resultados

`AttributeSet` conserva los Items directos, los valores repetidos y el orden del mensaje. Los tags y valores Enumeration desconocidos se mantienen en el modelo TTLV genérico cuando la política de asignación permite representarlos. Adjustment Type admite Increment, Decrement y Negate, además del rango de extensiones KMIP; los valores Reserved se rechazan en peticiones tipadas de salida.

Los modelos de respuesta exponen el `KmipOperationResult` del servidor, incluidos Result Status, Result Reason opcional y Result Message opcional. Los accesores de payload satisfactorio exponen Unique Identifier y, en las lecturas, los atributos o referencias devueltos. KMIPKit no aplica localmente Adjust Attribute ni mantiene una copia local de los atributos del servidor.

Las comprobaciones de política de mutación rechazan solo prohibiciones incondicionales respaldadas por la fuente que puedan determinarse con la petición suministrada. Esos fallos tienen estado de entrega `NotSent` y no invocan el transporte. Los casos dependientes del estado remoto o no reconocidos se envían al servidor. Las peticiones no se reintentan automáticamente. Un resultado `Pending` conserva la respuesta tipada y el valor de correlación asíncrona; consulta el flujo asíncrono en [Ejecución del cliente](client-execution.md).

Las pruebas del protocolo usan vectores derivados y entradas malformadas. No afirman que todos los servidores KMIP admitan todos los atributos ni que se haya superado un caso oficial de OASIS cuyo fixture no está disponible.
