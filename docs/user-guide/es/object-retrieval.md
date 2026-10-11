# Recuperar objetos y localizar identificadores

Usa `Locate` para pedir al servidor KMIP los identificadores que coincidan con
los atributos y restricciones elegidos por la aplicación. `Get` solicita un
objeto por identificador. `Locate` devuelve los identificadores encontrados
por el servidor; no recupera sus objetos. `Get` es una petición posterior e
independiente.

Las formas de petición y respuesta siguen OASIS KMIP Specification v2.1:
Get está en §6.1.19, Tablas 220–222; Locate, en §6.1.28, Tablas 247–249. Los
resultados `Pending` y sus valores de correlación siguen §§9.1–9.2 y §8.6,
Tabla 399.

## Construir una petición `Locate`

`LocateRequest` siempre contiene un `AttributeSet`. Puede estar vacío o
contener elementos `Item` genéricos TTLV validados. El ejemplo envía el
atributo `Comment`, limita el número de resultados y conserva un desplazamiento
explícito de cero.

```rust,kmipkit-test
use std::error::Error;

use kmipkit_protocol::{AttributeSet, LocateRequest};
use kmipkit_ttlv::{Item, RawTag, Value};

fn main() -> Result<(), Box<dyn Error>> {
    let comment_tag = RawTag::new(0x0042_00FD)?.try_checked()?;
    let attributes = AttributeSet::try_new([Item::new(
        comment_tag,
        Value::text_string("ledger".to_owned()),
    )?])?;
    let query = LocateRequest::new(attributes)
        .with_maximum_items(25)
        .with_offset_items(0);

    assert_eq!(query.attributes().len(), 1);
    assert_eq!(query.maximum_items(), Some(25));
    assert_eq!(query.offset_items(), Some(0));
    Ok(())
}
```

Un conjunto vacío pide al servidor que busque en todos los objetos. Otros
selectores opcionales son `Storage Status Mask` y `Object Group Member`. Envía
los valores elegidos por la aplicación. KMIPKit transmite los criterios al
servidor; no evalúa atributos, busca en un estado local, ordena ni deduplica
los identificadores recibidos. Conserva el orden y las repeticiones de la
respuesta.

## Construir una petición `Get`

`Unique Identifier` es opcional en el modelo. Su ausencia se conserva cuando
el servidor debe resolver un `ID Placeholder` en el contexto del protocolo.
El formato de clave, la compresión, el tipo de envoltura y la especificación
de envoltura son opciones explícitas del llamador; KMIPKit las transmite sin
convertir ni envolver objetos localmente.

Para `Key Format Type`, KMIP 2.1 §6.1.19 exige que el servidor pueda devolver
una clave en el mismo formato en que se registró. El servidor MAY admitir la
conversión a otro formato; esto no significa que todos los servidores admitan
cualquier formato o conversión. KMIPKit transmite el `Key Format Type`
solicitado por el llamador y conserva la respuesta como TTLV opaco. No analiza
PKCS#12 ni otros formatos de clave, ni convierte localmente el material
criptográfico.

```rust,kmipkit-test
use kmipkit_protocol::{
    GetRequest, KeyFormatType, KeyWrapType, UniqueIdentifier,
};
use kmipkit_ttlv::Structure;

fn build_get(
    identifier: UniqueIdentifier,
    format: KeyFormatType,
    wrap_type: KeyWrapType,
    wrapping_specification: Structure,
) -> GetRequest {
    GetRequest::new()
        .with_unique_identifier(identifier)
        .with_key_format_type(format)
        .with_key_wrap_type(wrap_type)
        .with_key_wrapping_specification(wrapping_specification)
}

fn main() {
    let _ = build_get;
}
```

Los valores de enumeración y la estructura de envoltura del ejemplo son
entradas proporcionadas por su llamador; no son valores predeterminados ni
recomendaciones de KMIPKit. Elige los valores adecuados para el servidor y la
política de protección de tu aplicación. Consulta la
[guía de operaciones de atributos](attribute-operations.md) para crear
elementos de atributo validados y la
[guía de transportes de producción](transportes-produccion.md) para configurar
un cliente con TLS 1.3 mutuo y verificado.

## Ejecutar un lote tipado e interpretar los resultados

`Client::execute` realiza un único intercambio para el lote enviado. El
ejemplo supone que `client` se construyó como describe la
[guía de transportes de producción](transportes-produccion.md). Envía
peticiones explícitas `Get` y `Locate` y consulta las respuestas en el orden
de las peticiones. `Asynchronous Indicator` con valor `1` es `Mandatory`
(OASIS KMIP v2.1 §9.2, Tabla 401; §11.3, Tablas 431–432). El servidor SHALL
procesar de forma asíncrona todos los elementos del lote y devolver un
`Asynchronous Correlation Value` para cada elemento. Esta es una obligación
del servidor, no solo un permiso para responder `Pending`. Los resultados
`Pending` y sus valores de correlación se describen en §§9.1–9.2 y §8.6,
Tabla 399.

```rust,kmipkit-test
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientError, ClientRequest,
};
use kmipkit_protocol::{
    AttributeSet, GetRequest, LocateRequest, UniqueIdentifier,
};
use kmipkit_ttlv::codec::CodecLimits;

fn execute_reads(client: &mut Client, limits: &CodecLimits) -> Result<(), ClientError> {
    let get = GetRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("object-123".to_owned()));
    let locate = LocateRequest::new(AttributeSet::new()).with_offset_items(0);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Get(get)),
        ClientBatchItem::new(ClientRequest::Locate(locate)),
    ])
    .with_asynchronous_indicator(1);

    let response = client.execute(batch, limits)?;
    if let Some(item) = response.get(0) {
        let outcome = item.outcome();
        match outcome.result().status().known_name() {
            Some("Success") => {
                if let Some(get) = outcome.get_response() {
                    let metadata = (get.object_type(), get.unique_identifier());
                    if let Some(object) = get.object() {
                        // El Any Object es opaco; consulta solo su tag, no su contenido.
                        let object_tag = object.tag();
                        let _ = (metadata, object_tag);
                    }
                }
            }
            Some("Operation Failed") => {
                // El error expone el resultado compartido, no campos exclusivos de Get.
                let reason = outcome.result().reason();
                let message = outcome.result().message();
                let _ = (reason, message);
            }
            Some("Operation Pending") => {
                // El acceso de operación completada es None; la vista compartida conserva Get.
                let pending_get = outcome.response().get();
                let correlation = outcome.asynchronous_correlation_value();
                let _ = (pending_get, correlation);
            }
            _ => {}
        }
    }

    if let Some(item) = response.get(1) {
        let outcome = item.outcome();
        match outcome.result().status().known_name() {
            Some("Success") => {
                if let Some(locate) = outcome.locate_response() {
                    let count = locate.located_items();
                    let identifiers = locate.unique_identifiers();
                    // Conserva el orden del servidor, incluidas las repeticiones.
                    let _ = (count, identifiers);
                }
            }
            Some("Operation Failed") => {
                let reason = outcome.result().reason();
                let message = outcome.result().message();
                let _ = (reason, message);
            }
            Some("Operation Pending") => {
                let pending_locate = outcome.response().locate();
                let correlation = outcome.asynchronous_correlation_value();
                let _ = (pending_locate, correlation);
            }
            _ => {}
        }
    }
    Ok(())
}

fn main() {
    let _ = execute_reads;
}
```

`get_response()` y `locate_response()` devuelven una respuesta tipada solo
cuando la operación ha terminado. Las vistas compartidas
`outcome.response().get()` y `.locate()` pueden exponer la respuesta tipada
transportada por `Pending`. Un resultado `Pending` no es todavía un objeto
recuperado ni una lista final de identificadores. El valor de correlación es
opaco; KMIPKit no ejecuta `Poll` ni espera automáticamente. Si la aplicación
elige una operación posterior, debe solicitarla de forma explícita con el
valor de correlación requerido por ese flujo.

La respuesta `Get` lleva el `Any Object` completo como un elemento genérico
TTLV opaco. KMIPKit conserva su estructura y valores, pero no decodifica
material de claves ni realiza operaciones criptográficas. Limita la
inspección a los metadatos permitidos, como `Object Type` y `Unique
Identifier`, y al tag del elemento genérico `Any Object`. No inspecciones la
estructura de `Any Object` ni sus descendientes, que pueden contener datos
secretos. Trata el contenido como secreto: no formatees ni registres el
objeto, material de claves, especificaciones de envoltura ni cuerpos KMIP sin
procesar. El almacenamiento TTLV propiedad de KMIPKit se pone a cero al
descartarlo; las copias creadas por el llamador quedan fuera de esa garantía.

Un error del servidor es un resultado de operación tipado con `Result
Status`, `Result Reason` y `Result Message` opcional; no es un `ClientError`
local. Inspecciona un mensaje no confiable solo si hace falta y no lo registres
sin revisarlo. Los valores locales `ClientError` representan errores de
validación, protocolo y transporte, y conservan la clasificación de entrega
descrita en la [guía de ejecución del cliente](ejecucion-cliente.md). KMIPKit
no reintenta `Get` ni `Locate`, no busca localmente ni ejecuta `Recover`
automáticamente para objetos archivados. Para obtener el contenido archivado,
la aplicación debe solicitar `Recover` y después `Get`.

Los ejemplos Rust marcados de esta guía se compilan sin conexión mediante
`python -B scripts/test_user_guide_examples.py`. La compilación demuestra el
uso de la API; esta guía no afirma interoperabilidad con servidores,
ejecución completa de los casos OASIS, compatibilidad con perfiles ni
certificación.
