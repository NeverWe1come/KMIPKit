# Operaciones Query y Ping

Query de KMIP 2.1 permite pedir al servidor información de protocolo concreta.
Ping envía un payload de operación vacío y recibe el resultado KMIP del
servidor. Ambas operaciones usan el flujo de ejecución existente y realizan un
intercambio por llamada.

## Seleccionar información con Query

Crea un `QueryRequest` con uno o más valores `QueryFunction`. Se conservan el
orden y las repeticiones. Los catorce valores estándar tienen constantes con
nombre, y se pueden indicar valores de extensión válidos mediante `from_raw`.
Los valores reservados y una lista vacía de funciones se rechazan localmente
antes de enviar bytes de la petición.

`Object Groups` es opcional. Si se indica, se codifica como una estructura con
los atributos de texto `Object Group` en el orden elegido por el llamador. Una
lista vacía codifica una estructura presente sin miembros; si no se usa el
builder, el campo queda ausente. KMIPKit no añade grupos ni elimina valores
repetidos.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatchItemResponse, ClientError};
use kmipkit_protocol::{QueryFunction, QueryRequest};
use kmipkit_ttlv::codec::CodecLimits;

#[allow(dead_code)]
fn consultar_servidor(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = QueryRequest::new([
        QueryFunction::OPERATIONS,
        QueryFunction::OBJECTS,
        QueryFunction::SERVER_INFORMATION,
    ])
    .with_object_groups(["claves-simétricas", "certificados"]);
    client.query(request, limits)
}

fn main() {}
```

La respuesta tipada expone cada miembro de la Tabla 283 en el orden recibido.
Los valores de enumeración se conservan en bruto para no perder valores que
esta versión de KMIPKit desconozca. Los miembros complejos, como Server
Information, Extension Information y Protection Storage Masks, siguen
disponibles como elementos TTLV genéricos.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientError};
use kmipkit_protocol::{QueryFunction, QueryRequest};
use kmipkit_ttlv::codec::CodecLimits;

#[allow(dead_code)]
fn consultar_operaciones(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<Vec<u32>, ClientError> {
    let response = client.query(
        QueryRequest::new([QueryFunction::OPERATIONS]),
        limits,
    )?;
    let response_view = response.outcome().response();
    let query = response_view.query().expect("la petición era Query");
    Ok(query.operations().collect())
}

fn main() {}
```

Query informa de los datos que devuelve el servidor. No demuestra que el
servidor aplique cada capacidad anunciada, que este cliente la implemente ni
que la política local permita usarla. Si se piden Query Extension List y Query
Extension Map a la vez, KMIPKit envía ambos valores en el orden recibido y no
interpreta ni reescribe la respuesta.

La sección 6.1.40 describe un payload de respuesta vacío cuando no hay valores
que devolver. La Tabla 283 también marca Protection Storage Masks como
obligatorio en una respuesta estructurada, aunque permite que su lista esté
vacía. KMIPKit acepta ambas formas y conserva cuál llegó. La discrepancia
sigue abierta como `KMIPKIT-DISC-047`; aceptarlas no determina la conformidad
del servidor.

## Ping

Ping tiene un payload de petición y respuesta exitosa vacío. Un resultado
tipado exitoso significa que el servidor devolvió un resultado KMIP Ping
exitoso. No garantiza la salud general, la disponibilidad ni el éxito de una
operación futura.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatchItemResponse, ClientError};
use kmipkit_ttlv::codec::CodecLimits;

#[allow(dead_code)]
fn comprobar_ping(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    client.ping(limits)
}

fn main() {}
```

## Resultados, errores y reintentos

Los errores de operación KMIP se conservan como respuestas tipadas con
Result Status, Result Reason opcional y Result Message permitido. Los errores
locales de validación, decodificación de protocolo y transporte devuelven
`ClientError` con la evidencia de entrega más precisa disponible. Una lista
vacía de funciones Query tiene estado de entrega `NotSent`.

Cada llamada a `Client::query` o `Client::ping` realiza como máximo un
intercambio. El cliente no reintenta, no ejecuta Poll, no envía otra Query ni
ejecuta Discover Versions como paso previo oculto. El llamador decide si y
cuándo hacer otra petición.
