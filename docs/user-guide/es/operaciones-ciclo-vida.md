# Operaciones de ciclo de vida de objetos

KMIPKit ofrece llamadas explícitas del cliente para Activate, Archive, Destroy
y Recover. Cada llamada envía una petición tipada, realiza un intercambio y
devuelve el resultado de la operación indicado por el servidor. El cliente no
simula el estado remoto, no reintenta peticiones ni ejecuta Poll/Get de forma
automática. Para configurar la conexión y la política TLS, consulta la
[guía de transportes de producción](transportes-produccion.md).

Las estructuras de las operaciones siguen OASIS KMIP Specification v2.1 §6.1.1,
Tablas 164–166 (Activate), §6.1.4, Tablas 173–175 (Archive), §6.1.15, Tablas
208–210 (Destroy) y §6.1.42, Tablas 288–290 (Recover). Las formas admitidas de
Unique Identifier y su etiqueta están definidas en §4.58, Tablas 145–146, y
§11.56, Tabla 487.

Las cuatro peticiones aceptan un `UniqueIdentifier` tipado opcional. KMIPKit
envía la representación proporcionada sin modificarla; si se omite, el cliente
no selecciona ni busca un objeto localmente. Una respuesta correcta expone el
identificador devuelto por el servidor. Las respuestas no correctas conservan
el resultado sin interpretar como estado del objeto un payload de éxito.

## Activate

Activate solicita al servidor que active el objeto identificado. El cliente
devuelve el resultado del servidor y, si hay éxito, el identificador obligatorio
de la respuesta. No afirma que el estado del servidor haya cambiado localmente.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{ActivateRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn activate(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = ActivateRequest::new(Some(UniqueIdentifier::TextString(
        "registro-archivo-42".to_owned(),
    )));
    let item = client.activate(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Activate);
    let response = outcome
        .response()
        .activate()
        .expect("Activate devuelve su respuesta tipada");
    let _server_status = response.result().status().raw();
    let _returned_identifier = response.unique_identifier();
    Ok(())
}

fn main() {
    let _example = activate;
}
```

## Archive

Archive expresa la preferencia de archivo del llamador. Una respuesta correcta
es el resultado del servidor, pero no demuestra que el archivado haya
terminado. La política del servidor determina qué significa la petición en
esa instalación.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{ArchiveRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn archive(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = ArchiveRequest::new(Some(UniqueIdentifier::TextString(
        "registro-archivo-42".to_owned(),
    )));
    let item = client.archive(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Archive);
    let response = outcome
        .response()
        .archive()
        .expect("Archive devuelve su respuesta tipada");
    let _server_status = response.result().status().raw();
    let _server_identifier = response.unique_identifier();
    Ok(())
}

fn main() {
    let _example = archive;
}
```

## Destroy

Destroy solicita la destrucción del objeto en el servidor. KMIPKit comunica la
respuesta, pero no deduce que el objeto ni sus metadatos hayan sido eliminados.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{DestroyRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn destroy(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = DestroyRequest::new(Some(UniqueIdentifier::TextString(
        "registro-archivo-42".to_owned(),
    )));
    let item = client.destroy(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Destroy);
    let response = outcome
        .response()
        .destroy()
        .expect("Destroy devuelve su respuesta tipada");
    let _server_status = response.result().status().raw();
    let _server_identifier = response.unique_identifier();
    Ok(())
}

fn main() {
    let _example = destroy;
}
```

## Recover

Recover solicita al servidor que recupere el objeto y devuelve el resultado y,
si hay éxito, el identificador que devuelve el servidor. Cualquier Get o Poll
posterior es una acción separada del llamador.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RecoverRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn recover(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = RecoverRequest::new(Some(UniqueIdentifier::TextString(
        "registro-archivo-42".to_owned(),
    )));
    let item = client.recover(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Recover);
    let response = outcome
        .response()
        .recover()
        .expect("Recover devuelve su respuesta tipada");
    let _server_status = response.result().status().raw();
    let _recovered_identifier = response.unique_identifier();
    Ok(())
}

fn main() {
    let _example = recover;
}
```

## Resultados, trabajo pendiente y errores

Consulta `item.outcome().result()` para obtener el estado KMIP bruto exacto, el
motivo bruto opcional y el mensaje opcional. El texto de Result Message solo se
lee mediante su acceso explícito; Debug y el formato predeterminado de errores
lo redactan. Los estados y motivos desconocidos se conservan como valores
brutos. Las Message Extensions no críticas aceptadas se pueden inspeccionar
mediante `item.extensions()` como TTLV genérico opaco; esto no valida su
significado específico del fabricante.

Las llamadas de conveniencia anteriores usan las opciones predeterminadas del
lote. Para permitir `Operation Pending`, usa `Client::execute` con un
`ClientBatch` de un elemento cuyo Asynchronous Indicator se establezca
explícitamente en Mandatory u Optional. El resultado Pending expone los bytes
opacos exactos de correlación mediante un acceso prestado. El llamador decide
si ejecuta Poll u otra operación; KMIPKit no reintenta, consulta ni espera
automáticamente. La [guía de ejecución del cliente](ejecucion-cliente.md)
describe Pending y el estado de entrega.

Los fallos locales de validación, protocolo y transporte devuelven un
`ClientError` redactado con evidencia de entrega `NotSent`, `PossiblySent` o
`ResponseStarted`. Esa evidencia no demuestra que reintentar sea seguro. Un
resultado Failure completo del servidor es el resultado de la operación, no
un error local.

Los ejemplos marcados de esta guía se compilan con
`python scripts/test_user_guide_examples.py`.
