# Ejecución tipada del cliente

Esta guía describe la base de ejecución actual de KMIPKIT-0007. No es una guía
para conectarse a un servidor: esta funcionalidad no proporciona un
constructor de producción para `Client` ni un backend TCP/TLS o HTTPS. La
única implementación de transporte es un doble determinista para pruebas.

## Límite de peticiones tipadas

La API síncrona `kmipkit_client::Client::execute` acepta un `ClientBatch` que
contiene únicamente variantes del conjunto cerrado `ClientRequest`. Esta
primera parte incluye una operación: una petición Discover Versions explícita
del cliente al servidor. Solo anuncia el par de versión KMIP 2.1 (2, 1),
conforme a OASIS KMIP Specification v2.1 §6.1.16, Tablas 211–213. La API no admite valores TTLV
genéricos `Item` o `Structure`, bytes de mensajes codificados ni hooks de
conversión definidos por el llamador.

Discover Versions es una operación normal que el llamador debe solicitar. El
cliente no la ejecuta como paso previo oculto ni la usa para negociar otra
operación. La respuesta comunica versiones, pero no demuestra compatibilidad
con otras operaciones. Una lista vacía o el resultado `Operation Not
Supported` del servidor se devuelve como un resultado tipado normal.

KMIPKit 1.0 se limita a KMIP 2.1. Esta parte emite y acepta únicamente la
versión de protocolo 2.1 conforme a la decisión de producto registrada para
[`KMIPKIT-DISC-022`](../../../specs/007-client-execution/spec.md); esto no afirma
compatibilidad retroactiva entre versiones principales conforme a KMIP §9.16.
Consulta [ADR-0002](../../adr/0002-kmip-21-release-scope.md) para conocer el
alcance de la versión.

Aunque `Client::execute` y los tipos de petición están documentados como API
de Rust, las aplicaciones no pueden construir un `Client` de producción con
esta funcionalidad. El doble interno solo se usa en pruebas deterministas de
ejecución. Aquí no se ofrece un adaptador TLS/HTTPS ni disponibilidad para
servidores reales.

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
al destruir el valor Pending. Poll, Cancel, procesamiento del resultado,
espera automática y tareas en segundo plano no forman parte de esta
funcionalidad.

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
recursos, no campos de la cabecera KMIP.

Discover Versions no se clasifica como una respuesta probablemente grande,
por lo que esta API no incluye el campo Maximum Response Size visible para el
servidor. Ese campo es distinto del límite local de bytes. Las futuras
especificaciones de operaciones deben evaluar por separado el tamaño de sus
respuestas. Esta funcionalidad tampoco ofrece límites de conexión, lectura,
escritura o plazo total de red porque no hay backend de producción.

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

Una funcionalidad TLS/HTTPS aprobada por separado será responsable de crear
clientes a partir de configuración de transporte validada y no deberá
exponer la inyección arbitraria de transportes. La
[arquitectura de transporte](../../architecture/transport-security.md)
describe el perfil TLS y HTTPS previsto; no confirma que exista un adaptador
disponible actualmente.

## Guías y decisiones relacionadas

- [Inspeccionar mensajes KMIP](modelo-mensaje.md)
- [Arquitectura de la API pública](../../architecture/public-api.md)
- [ADR-0002: alcance KMIP 2.1](../../adr/0002-kmip-21-release-scope.md)
- [ADR-0012: codificación de bytes solicitada por el llamador](../../adr/0012-caller-requested-wire-encoding-policy.md)
- [ADR-0013: propiedad del registro de extensiones del cliente](../../adr/0013-client-extension-registry-ownership.md)
- [ADR-0014: contrato público de transporte de bajo nivel](../../adr/0014-public-transport-exchange-contract.md)