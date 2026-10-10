# Ejecución tipada del cliente

Esta guía describe la base de ejecución tipada introducida por KMIPKIT-0007 y
sus operaciones: Discover Versions, Create, Create Key Pair, Create Split Key,
Add Attribute, Adjust Attribute, Delete Attribute, Get Attributes, Get
Attribute List, Modify Attribute, Set Attribute, Activate, Archive, Destroy,
Recover, Query y Ping. Para el comportamiento de ciclo de vida, consulta la
[guía de operaciones de ciclo de vida](operaciones-ciclo-vida.md); para Query
y Ping, consulta la [guía de Query y Ping](operaciones-query-ping.md). Para
configurar una conexión de producción, consulta la [guía de transportes TLS y
HTTPS](transportes-produccion.md).

## Límite de peticiones tipadas

La API síncrona `kmipkit_client::Client::execute` acepta un `ClientBatch` que
contiene únicamente variantes del conjunto cerrado `ClientRequest`. El cliente
admite peticiones explícitas del cliente al servidor para Discover Versions,
Create, Create Key Pair, Create Split Key, Add Attribute, Adjust Attribute,
Delete Attribute, Get Attributes, Get Attribute List, Modify Attribute, Set
Attribute, Activate, Archive, Destroy, Recover, Query y Ping. Discover Versions
anuncia el par de versión KMIP 2.1 (2, 1), conforme a
OASIS KMIP Specification v2.1 §6.1.16, Tablas 211–213. Get Attributes y Get
Attribute List usan sus modelos tipados de petición y respuesta de
§§6.1.20–6.1.21. La API no admite valores TTLV genéricos `Item` o `Structure`,
bytes de mensajes codificados ni hooks de
conversión definidos por el llamador.

Discover Versions es una operación normal que el llamador debe solicitar. El
cliente no la ejecuta como paso previo oculto ni la usa para negociar otra
operación. La respuesta comunica versiones, pero no demuestra compatibilidad
con otras operaciones. Una lista vacía o el resultado `Operation Not
Supported` del servidor se devuelve como un resultado tipado normal.

Cada elemento del lote conserva su respuesta tipada por operación. La llamada
`ClientBatchOutcome::response()` devuelve una vista prestada
`ClientResponseView`, con el accesor común `result()` y accesores tipados para
Discover Versions, Create, Create Key Pair, Create Split Key, Add Attribute,
Adjust Attribute, Delete Attribute, Get Attributes, Get Attribute List, Modify
Attribute, Set Attribute, Activate, Archive, Destroy y Recover. Cuando una
operación queda en estado Pending, `PendingOutcome` conserva esa respuesta
tipada y presta el valor de correlación opaco mediante
`asynchronous_correlation_value()`. KMIPKit no ejecuta Poll ni reintenta
automáticamente.

KMIPKit 1.0 se limita a KMIP 2.1. Esta parte emite y acepta únicamente la
versión de protocolo 2.1 conforme a la decisión de producto registrada para
[`KMIPKIT-DISC-022`](../../../specs/007-client-execution/spec.md); esto no afirma
compatibilidad retroactiva entre versiones principales conforme a KMIP §9.16.
Consulta [ADR-0002](../../adr/0002-kmip-21-release-scope.md) para conocer el
alcance de la versión.

## Operaciones de creación en el servidor

Create, Create Key Pair y Create Split Key envían peticiones tipadas elegidas
por el llamador mediante el mismo escritor `ClientBatch`. Construir un lote no
abre una conexión: pásalo a `Client::execute` cuando tengas un cliente de
producción y una configuración de transporte. El comando
`python scripts/test_user_guide_examples.py` compila todos los ejemplos
marcados.

### Create

Indica el Object Type KMIP y un `AttributeSet`. Create siempre codifica la
estructura Attributes externa obligatoria, aunque no tenga miembros. KMIPKit
no elige atributos criptográficos ni una política de protección.

```rust,kmipkit-test
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::{AttributeSet, CreateRequest, ObjectType};

fn main() {
    let request = CreateRequest::new(ObjectType::from_raw(7), AttributeSet::new());
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Create(request)));
    assert_eq!(batch.items().len(), 1);
}
```

### Create Key Pair

Los tres grupos de atributos se mantienen separados. Añade solo los valores
elegidos por el llamador; KMIPKit no selecciona algoritmo, longitud,
parámetros ni uso de las claves.

```rust,kmipkit-test
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::{AttributeSet, CreateKeyPairRequest};

fn main() {
    let request = CreateKeyPairRequest::new()
        .with_common_attributes(AttributeSet::new());
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::CreateKeyPair(request)));
    assert_eq!(batch.items().len(), 1);
}
```

### Create Split Key

Indica el tipo de objeto, el número de partes, el umbral, el método de
separación y la estructura Attributes obligatoria. El Unique Identifier de la
clave de entrada solo se envía si se proporciona. Según la política FR-015 de
KMIPKit, para Polynomial Sharing Prime Field el llamador debe proporcionar
Prime Field Size explícitamente.

```rust,kmipkit-test
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::{AttributeSet, CreateSplitKeyRequest, ObjectType, SplitKeyMethod};

fn main() {
    let request = CreateSplitKeyRequest::new(
        ObjectType::from_raw(7),
        3,
        2,
        SplitKeyMethod::XOR,
        AttributeSet::new(),
    );
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::CreateSplitKey(request)));
    assert_eq!(batch.items().len(), 1);
}
```

Create Split Key puede devolver varios Unique Identifier. Su petición anuncia
`Maximum Response Size` como el menor valor entre el límite local de bytes de
respuesta y el mayor KMIP Integer con signo. El cliente también impone el
límite local antes de entrar en el decoder TTLV, aunque el servidor ignore el
tamaño anunciado.

`Client::new` construye un cliente de producción a partir de una configuración
inmutable del cliente y un `TransportConfig` validado. Acepta TTLV sobre TLS
directo o TTLV sobre HTTPS/HTTP 1.1; no acepta transportes implementados por el
llamador ni bytes de petición sin procesar. Consulta la [guía de transporte](transportes-produccion.md)
para configurar la confianza, los plazos y el comportamiento de las conexiones.

## Credenciales y capacidad de atestación

KMIPKIT-0008 proporciona valores tipados de Credential y Authentication que
conservan el árbol TTLV original en `kmipkit-protocol`. Authentication
contiene una o más Credential en el orden indicado por el llamador. En este
incremento son modelos solo en memoria; el payload de petición no incluye
datos de Authentication o Credential.

Una Credential tipada Hashed Password requiere la marca de tiempo y los bytes
hash proporcionados por el llamador. Hashing Algorithm es opcional. Si se
omite, el modelo informa SHA-256 como algoritmo efectivo (valor bruto `6`) y
conserva el campo como ausente en el árbol TTLV. KMIPKit no calcula el hash.

Una Credential tipada Device debe incluir al menos uno de estos campos:
Device Serial Number, Network Identifier, Machine Identifier o Media
Identifier. El llamador es responsable de elegir un identificador o una
combinación que realmente sea única. KMIPKit no define el ámbito de
comparación ni comprueba la unicidad. Los campos Password y Device Identifier
no sustituyen ese requisito; su presencia
tampoco exige que el texto no esté vacío.

Los constructores síncronos y asíncronos de la cabecera Request emiten
`Attestation Capable Indicator = True` porque la API de Rust puede construir
una Credential Attestation. Esto anuncia únicamente capacidad de construcción:
no genera ni verifica evidencia de atestación, no envía una Credential ni predice si el
servidor la aceptará. No existe una opción para cambiarlo en cada petición. El
El indicador solo anuncia capacidad de construcción de la API. No envía una
Credential, no genera ni verifica evidencia y no predice si el servidor la
aceptará.

KMIPKit no registra el contenido de las Credential; Debug, Display y los
diagnósticos de validación también lo redactan. `SecretText` y `SecretBytes`
ponen a cero los bytes inicializados de la asignación actual propiedad de
KMIPKit cuando se destruye su propietario; la conversión a TTLV transfiere la
propiedad sin clonar. La capacidad libre o no inicializada solo queda
cubierta si se inicializa y se verifica su limpieza. Esto no borra copias
creadas por el llamador, asignaciones anteriores que queden tras ampliar un buffer, copias
hechas desde vistas prestadas en callbacks, copias temporales en pila o
registros, ni copias conservadas por runtimes externos o dependencias.

## Metadatos de petición y resultados de lote

La marca de tiempo opcional de la cabecera Request se omite de forma
predeterminada. Si la proporciona el llamador, el valor KMIP Date-Time se
envía exactamente como se recibió. El cliente no genera una marca de tiempo a
partir del reloj ni de un temporizador de cuenta atrás. Las peticiones
iniciadas por el cliente omiten Server Correlation Value. Si se proporciona
Client Correlation Value, es metadato independiente y nunca identifica un
elemento del lote.

Cada elemento de una petición de varios elementos tiene un Unique Batch Item
ID. El cliente asocia cada respuesta con su petición mediante ese ID, aunque
el servidor devuelva los elementos en otro orden. Los resultados se devuelven
en el orden de la petición. Si se proporcionó un ID, el servidor debe
devolverlo; una respuesta de un solo elemento puede omitir el ID cuando la
petición también lo omitió. Las identidades duplicadas, ausentes,
inesperadas o no coincidentes, así como las operaciones distintas, producen
errores de protocolo redactados. Batch Order Option controla el orden de
ejecución del servidor; no cambia la regla de asociación de respuestas.

Cada resultado de elemento válido se representa como completado o Pending.
Pending solo se acepta si el valor efectivo de Asynchronous Indicator de la
petición lo permite y la respuesta contiene el Asynchronous Correlation Value
obligatorio. El valor de correlación es opaco: KMIPKit conserva sus bytes y
solo los presta mediante un accesor explícito. Se redacta en el formato, los
errores y los logs, y el almacenamiento propiedad de KMIPKit se pone a cero
al destruir el valor Pending.

## Operaciones asíncronas explícitas

`Client::execute_poll`, `Client::execute_cancel`,
`Client::execute_process` y `Client::execute_query_async_requests` realizan,
cada una, un único intercambio síncrono explícito. No vuelven a hacer Poll,
reintentar, esperar ni programar tareas en segundo plano. Un error de transporte
conserva el significado del estado de entrega descrito más abajo; ese estado
no determina si es seguro reintentar.

El llamador construye `PollRequest`, `CancelRequest` o `ProcessRequest` con los
bytes exactos obtenidos mediante el acceso prestado al valor de correlación de
un resultado Pending. `execute_poll` devuelve Pending sin enviar otra petición.
El payload de una finalización correcta se conserva como TTLV genérico hasta
que exista el modelo tipado de la operación original; una finalización con
Failure expone el resultado y su razón sin payload. `execute_cancel` comprueba
que una respuesta correcta repita exactamente los bytes de correlación
solicitados y expone el valor Cancellation Result asignado o desconocido. Se
rechaza una respuesta Cancel Pending conforme a OASIS KMIP v2.1 §6.1.5,
Tablas 176–178, y §11.7, Tablas 437–438.

`execute_process` es una operación de servidor independiente. El llamador
selecciona si la petición Process permite una respuesta asíncrona. Cada
respuesta Process distinta de Failure, incluida Pending, contiene el Response
Payload vacío requerido por OASIS KMIP v2.1 §8.6, Tabla 399, y definido por
§6.1.39, Tabla 279. Failure no lleva payload según §8.6. Si el resultado es
Pending, se devuelve al llamador; KMIPKit no afirma que un Poll posterior vaya
a completarse. El texto de §6.1.39 indica que Process puede afectar a otros
elementos del lote cuando Batch Order Option es true, su valor predeterminado.
KMIPKit no afirma controlar esos efectos del servidor.

`execute_query_async_requests` admite los filtros opcionales de valor de
correlación y operación de OASIS KMIP v2.1 §6.1.41, Tabla 285. Su payload de
respuesta se expone como TTLV genérico mientras siga abierto
`KMIPKIT-DISC-039`. KMIPKit no presenta una interpretación tipada del pie de
tabla 286 en disputa. También sigue abierto bajo `OD-002` el hueco del
catálogo para el campo obligatorio de la petición Process de la Tabla 278; no
se modificaron el catálogo generado ni la fuente OASIS upstream.

## Límites de recursos

`CodecLimits` se aplica a cada ejecución de una petición. Sus valores
predeterminados son:

- Tamaño del mensaje completo: 16 MiB.
- Profundidad de Structures anidadas: 64.
- Número de Items TTLV, incluido el Item raíz: 100.000.

El llamador puede configurar los límites de tamaño del mensaje y cantidad de
Items, incluso a cero. La profundidad de Structure se puede reducir, pero no
superar el máximo de 64 del modelo. El mismo valor prestado de `CodecLimits`
se usa para codificar la petición y decodificar la respuesta. Su
`max_message_bytes()` también es exactamente el límite de bytes de respuesta
que se pasa al transporte, y el cliente comprueba la longitud recibida antes
de decodificar. No hay un límite independiente por valor: el límite del
mensaje completo también acota cada valor individual. Son límites locales de
recursos que se aplican a todas las operaciones.

Discover Versions no anuncia el campo Maximum Response Size visible para el
servidor. Create Split Key sí lo anuncia porque su respuesta puede contener
identificadores repetidos. Ese campo es distinto del límite local de bytes: el
cliente sigue rechazando antes de decodificar cualquier respuesta que supere
el límite configurado. La [guía de transporte](transportes-produccion.md) documenta los plazos de conexión,
escritura, lectura e intercambio completo.

## Errores, estado de entrega y redacción

Los `ClientError` locales distinguen errores de validación, protocolo y
transporte. Conservan solo categorías de causa seguras y el estado de entrega
`RequestDeliveryState` más avanzado disponible:

- `NotSent`: no se envió ningún byte de la petición.
- `PossiblySent`: comenzó la transmisión, pero no llegó ningún byte de
  respuesta.
- `ResponseStarted`: llegó al menos un byte de respuesta, pero todavía no
  hay un resultado completo.

Un resultado KMIP completo del servidor se devuelve como resultado del
servidor y no incluye estado local de entrega. El estado de entrega no indica
que sea seguro reintentar. El cliente realiza un solo intercambio por llamada
y nunca reintenta, cambia automáticamente de servidor, hace Poll ni espera.

Los cuerpos de petición y respuesta, credenciales, material de claves
secretas y valores opacos de correlación asíncrona no se registran en logs,
no se formatean dentro de errores y no se conservan en las cadenas de origen
expuestas. El propietario privado de la petición codificada sigue vivo
durante el intercambio síncrono y pone a cero los bytes inicializados al
destruirse. Esta garantía no cubre capacidad libre o no inicializada, copias
del llamador, asignaciones anteriores que no se limpiaran antes de crecer el
buffer ni copias de TLS, del sistema operativo o de bibliotecas externas. La
política aprobada y sus límites exactos están en
[ADR-0012](../../adr/0012-caller-requested-wire-encoding-policy.md).

## Extensiones de mensaje

KMIP §9.13 exige rechazar una extensión Message Extension crítica que no se
reconozca. Una
extensión de respuesta no crítica y desconocida se conserva como TTLV opaco
para inspección explícita; conservarla no interpreta ni valida la semántica
del fabricante. Esta parte no registra ni envía extensiones de fabricantes.
KMIPKIT-0012 es responsable del registro inmutable por cliente y de los
adaptadores tipados validados para extensiones de fabricantes antes de
congelar la API 1.0, según [ADR-0013](../../adr/0013-client-extension-registry-ownership.md).

## Contrato de transporte de bajo nivel

`kmipkit-transport::Transport::exchange` es una API pública documentada para
usuarios directos de Rust, pero el facade superior `kmipkit` no la reexporta y
`kmipkit-client` no acepta transportes implementados por el llamador. El
contrato intercambia de forma síncrona bytes de petición aportados por el
llamador y un límite de bytes de respuesta. No codifica ni valida mensajes
KMIP, y no proporciona la garantía de propiedad de la petición del cliente
tipado. El llamador es dueño de su buffer y responsable de su validez y ciclo
de vida.

En caso de éxito, el usuario directo del transporte construye un
`TransportResponse` con el constructor público
`TransportResponse::new(Vec<u8>)`; `as_bytes()` presta una vista de lectura y `Debug` redacta
el contenido y, al destruir el wrapper, se ponen a cero los bytes
inicializados de su asignación actual. La ruta tipada `Client::execute` decodifica los wrappers válidos y nunca devuelve cuerpos de respuesta sin procesar. Las implementaciones de transporte deben aplicar el límite mientras
leen y
detenerse antes de ampliar el
almacenamiento de respuesta más allá de ese límite. No deben registrar ni
conservar los bytes de petición después del intercambio, reintentar ni dejar
sin limpiar los bytes temporales de petición o respuesta parcial propiedad de
KMIPKit antes de liberarlos. Un adaptador concreto debe impedir que el buffer
de respuesta se reasigne tras almacenar bytes o limpiar las asignaciones
anteriores antes de liberarlas, tanto en éxito como en error. Estas garantías no cubren capacidad libre ni asignaciones anteriores salvo que se limpien antes de crecer. Tampoco cubren copias del llamador ni copias externas de TLS, del sistema operativo o de bibliotecas. Consulta [ADR-0014](../../adr/0014-public-transport-exchange-contract.md)
para conocer el contrato exacto.

Los adaptadores de producción TLS y HTTPS usan la configuración de transporte
validada que se describe en la [guía de transportes](transportes-produccion.md).
La [arquitectura de transporte](../../architecture/transport-security.md)
registra su política de seguridad y el ciclo de vida de las conexiones.

## Guías y decisiones relacionadas

- [Inspeccionar mensajes KMIP](modelo-mensaje.md)
- [Transportes de producción TLS y HTTPS](transportes-produccion.md)
- [Arquitectura de la API pública](../../architecture/public-api.md)
- [Guía rápida de revisión para operaciones asíncronas KMIP 2.1](../../../specs/009-asynchronous-operations/quickstart.md)
- [ADR-0002: alcance KMIP 2.1](../../adr/0002-kmip-21-release-scope.md)
- [ADR-0012: codificación de bytes solicitada por el llamador](../../adr/0012-caller-requested-wire-encoding-policy.md)
- [ADR-0013: propiedad del registro de extensiones del cliente](../../adr/0013-client-extension-registry-ownership.md)
- [ADR-0014: contrato público de transporte de bajo nivel](../../adr/0014-public-transport-exchange-contract.md)
