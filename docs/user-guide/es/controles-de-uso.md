# Controles de política y uso

KMIPKit implementa las operaciones iniciadas por el cliente Get Constraints, Get Usage Allocation, Set Constraints y Set Defaults de OASIS KMIP v2.1. Las formas de petición y respuesta siguen §§6.1.22, 6.1.23, 6.1.52 y 6.1.53. Estas operaciones transmiten y exponen la política del servidor; KMIPKit no evalúa ni aplica localmente las restricciones del servidor.

Get Usage Allocation está mapeada en Rust, C, Java y Python. La petición
conserva el Unique Identifier opcional y el Usage Limits Count exacto de 64
bits con signo; el resultado correcto expone el identificador resuelto. El
resultado común del lote conserva Pending y Failure, la correlación y el estado
de entrega sin consultar ni reintentar automáticamente. Cuando termina con
éxito, la respuesta también expone el TTLV genérico completo y ordenado,
incluidos los campos desconocidos. Rust presta una vista; C devuelve una
estructura que pertenece al llamador y debe liberarse con
`kmipkit_ttlv_structure_release`; Java devuelve un `TtlvStructure` propiedad
del resultado cerrable; y Python ofrece `response_ttlv` en la respuesta
cerrable. Los accesores tipados de C, Java y Python no devuelven un TTLV
genérico para resultados Failure o Pending.

## Construir peticiones de política

La capa de protocolo acepta la `Structure` TTLV ordenada que proporciona el llamador. Completa Constraints y Defaults Information con valores adecuados para el servidor y tu política. Este ejemplo construye los payloads sin elegir valores de política:

```rust
use kmipkit_protocol::{
    GetConstraintsRequest, GetUsageAllocationRequest, SetConstraintsRequest,
    SetDefaultsRequest, UniqueIdentifier,
};
use kmipkit_ttlv::Structure;

fn build_policy_payloads(
    object_id: UniqueIdentifier,
    constraints: Structure,
    defaults: Structure,
) -> Result<(), Box<dyn std::error::Error>> {
    let _get_constraints = GetConstraintsRequest::new().to_ttlv_payload()?;

    // Usage Limits Count es Long Integer con signo (i64), según §7.40, Tabla 392.
    let _get_allocation = GetUsageAllocationRequest::new(25)
        .with_unique_identifier(object_id)
        .to_ttlv_payload()?;

    let _set_constraints = SetConstraintsRequest::new(constraints).to_ttlv_payload()?;

    // Omitir el campo solicita eliminar todos los Object Defaults (Tabla 328).
    let _remove_all_defaults = SetDefaultsRequest::new().to_ttlv_payload()?;

    // Una Structure vacía explícita es distinta de omitir el campo.
    let _set_empty_defaults = SetDefaultsRequest::new()
        .with_defaults_information(defaults)
        .to_ttlv_payload()?;
    Ok(())
}
```

Con un `Client` configurado, llama a `get_constraints`, `get_usage_allocation`, `set_constraints` o `set_defaults`. Cada método de conveniencia realiza un único intercambio; no reintenta ni consulta el estado asíncrono automáticamente. Rust también ofrece un constructor fluido para Get Constraints, cuya petición de la Tabla 229 está vacía:

```rust
use kmipkit_client::{Client, ClientBatchItemResponse, ClientError};
use kmipkit_ttlv::codec::CodecLimits;

fn get_constraints(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let response = client.request().get_constraints().execute(limits)?;
    let _typed_response = response.outcome().response().get_constraints();
    Ok(response)
}
```

El constructor reutiliza la respuesta tipada existente y realiza como máximo un envío. `Pending` se devuelve como un resultado explícito, sin consulta ni reintento automático. Para Get Usage Allocation, indica el Unique Identifier en una llamada individual; puede omitirse cuando una operación anterior del mismo lote establece el KMIP ID Placeholder:

```rust
use kmipkit_client::Client;
use kmipkit_protocol::UniqueIdentifier;
use kmipkit_ttlv::codec::CodecLimits;

fn allocate(
    client: &mut Client,
    object_id: UniqueIdentifier,
    requested_units: i64,
    limits: &CodecLimits,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .request()
        .get_usage_allocation()
        .unique_identifier(object_id)
        .usage_limits_count(requested_units)
        .execute(limits)?;
    let _outcome = response.outcome(); // Inspecciona Success, Pending o Failure.
    Ok(())
}
```

Inspecciona la respuesta tipada mediante `response.outcome().response()` y conserva `Pending` y su valor de correlación al usar un lote asíncrono.

## Proteger operaciones limitadas por uso

§6.1.23 de OASIS establece obligaciones de asignación de uso para el cliente. El llamador proporciona el número positivo exacto de unidades asignadas y el Usage Limits Unit obtenido de metadatos de objeto fiables; la respuesta de asignación proporciona el identificador resuelto. KMIPKit no obtiene esos atributos del objeto ni deduce el coste de una operación.

```rust
use kmipkit_client::{
    Client, ClientBatchItemResponse, ClientError, ClientRequest, UsageAllocationContext,
};
use kmipkit_protocol::{EncryptRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn allocate_for_object(
    client: &mut Client,
    object_id: UniqueIdentifier,
    requested_units: i64,
    usage_limits_unit: u32,
    limits: &CodecLimits,
) -> Result<UsageAllocationContext, Box<dyn std::error::Error>> {
    let response = client
        .request()
        .get_usage_allocation()
        .unique_identifier(object_id)
        .usage_limits_count(requested_units)
        .execute(limits)?;
    let response_view = response.outcome().response();
    let allocation = response_view
        .get_usage_allocation()
        .ok_or("se esperaba una respuesta Get Usage Allocation")?;
    Ok(UsageAllocationContext::from_successful_allocation(
        allocation,
        requested_units,
        usage_limits_unit,
    )?)
}

fn encrypt_with_allocation(
    client: &mut Client,
    allocation: &mut UsageAllocationContext,
    request: EncryptRequest,
    caller_supplied_cost: i64,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    // Construye `request` con parámetros criptográficos explícitos y el mismo
    // identificador de objeto asociado a `allocation`.
    client.execute_guarded_protection(
        allocation,
        ClientRequest::encrypt(request),
        caller_supplied_cost,
        limits,
    )
}
```

La guardia admite solo peticiones tipadas Encrypt, MAC y Sign con un identificador explícito que coincida con la asignación. Rechaza los costes no positivos y los que superan el saldo local antes de llamar al transporte. Solo libera una reserva cuando existe evidencia de que el envío no comenzó (`NotSent`); una entrega ambigua, una respuesta o `Pending` mantienen la reserva local.

Este contador cubre únicamente las peticiones enviadas mediante el contexto proporcionado en este proceso. No observa llamadas sin guardia, Decrypt, Hash, operaciones de verificación, otros clientes, otros procesos ni el consumo en el servidor. Una reserva local no demuestra que el servidor haya utilizado esas unidades. No solicites asignación fuera del periodo entre Activation Date y Protect Stop Date del objeto; el llamador debe conocer y respetar ese periodo.

Son ejemplos derivados del proyecto a partir de las tablas de OASIS, no pruebas oficiales de conformidad. Consulta [Ejecución del cliente](ejecucion-cliente.md) para configurar el transporte y conocer los resultados de entrega.
