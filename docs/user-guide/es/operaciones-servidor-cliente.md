# Operaciones servidor-cliente y cambios de rol

KMIPKit 1.0 incluye las operaciones KMIP 2.1 iniciadas por el servidor y la
gestión del rol de endpoint definidas por KMIPKIT-0066 y ADR-0021. Esto permite
que una aplicación acepte una conexión autenticada del servidor KMIP y
responda a solicitudes originadas por él. Es una función distinta de las
llamadas iniciadas por el cliente mediante `Client`.

## Flujo de escucha

La API Rust ofrece `ClientListener::bind_raw_tls` y
`ClientListener::bind_https`. Construye un `ListenerTransportConfig` con la
identidad del listener, las raíces de confianza del peer, las listas de
revocación configuradas, los límites de codificación y tamaño de solicitud, y
los timeouts. Ambos transportes exigen TLS 1.3 y autenticación mutua mediante
certificados; los certificados del peer se validan con la fuente de confianza
configurada.

Después de enlazar el listener, usa `accept` para obtener una
`ClientListenerConnection` y `recv_batch` para leer un
`IncomingServerBatch` acotado. Inspecciona su versión de protocolo, cantidad
de elementos, identificadores de lote y vistas tipadas de operación. Las
solicitudes iniciadas por el servidor admitidas son Discover Versions, Notify,
Put, Query y Set Endpoint Role. Construye un `ResponseMessage` para el lote
recibido y envíalo con `send_response`. Aplica la política de la aplicación
usando los modelos por operación y la especificación KMIP 2.1; recibir una
solicitud válida no obliga a la aplicación a aceptar sus efectos.

### Java

`org.kmipkit.server` ofrece una fachada `AutoCloseable`. Configura la identidad
del listener y los certificados de confianza del peer, enlaza con
`ServerListener.bindRawTls` o `bindHttps`, acepta con un `Duration` explícito
y recibe con `CodecLimits`. `ServerRequestBatch.items()` devuelve tipos de
solicitud específicos de cada operación. Las fábricas `forRequest` construyen
respuestas de éxito tipadas; `ServerReply.failure` crea un fallo KMIP explícito.
`ServerResponseBuilder` rechaza respuestas asociadas a otra solicitud. Envía
la respuesta de forma explícita por la conexión aceptada.
`NotifyRequest.attributes()` devuelve una copia independiente de tipo
`AttributeSet` con los atributos modificados opcionales; ciérrala al terminar.

```java
try (ServerListenerConfiguration config = ServerListenerConfiguration.builder()
        .bindHost("127.0.0.1").bindPort(5696)
        .listenerCertificateChain(List.of(serverCertificatePem))
        .listenerPrivateKey(serverPrivateKeyPem)
        .trustCertificates(List.of(clientCaPem)).build();
     ServerListener listener = ServerListener.bindRawTls(config);
     ServerConnection connection = listener.accept(Duration.ofSeconds(5));
     ServerRequestBatch batch = connection.receive(CodecLimits.defaults(), Duration.ofSeconds(5));
     ServerResponseMessage response = batch.respond(request -> {
         if (request instanceof DiscoverVersionsRequest discover) {
             return ServerReply.success(DiscoverVersionsResponse.forRequest(
                     discover, List.of(new ProtocolVersion(2, 1))));
         }
         return ServerReply.failure(ServerResultStatus.OPERATION_FAILED,
                 ServerResultReason.OPERATION_NOT_SUPPORTED, "operación no admitida");
     })) {
    connection.send(response, Duration.ofSeconds(5));
}
```

### Python

`kmipkit.server` expone el mismo flujo explícito de recepción, respuesta y
envío mediante gestores de contexto. La configuración acepta certificados en
bytes y sobrescribe la copia temporal de la clave privada al pasarla al código
nativo. `NotifyRequest.attributes` devuelve una copia validada de tipo
`AttributeSet`, que se puede cerrar o usar como gestor de contexto. Python no
permite poner a cero de forma fiable todas las copias que mantiene el runtime.

```python
with ServerListenerConfiguration(
    bind_host="127.0.0.1",
    bind_port=5696,
    listener_certificate_chain=(server_certificate_pem,),
    listener_private_key=server_private_key_pem,
    trust_certificates=(client_ca_pem,),
) as config, ServerListener.bind(config) as listener:
    with listener.accept(timeout=5) as connection:
        connection.serve_once(handler, ttlv.CodecLimits.defaults(), timeout=5)
```

El binding C mantiene la ruta ABI de bajo nivel con handles opacos y códigos
de estado de ancho fijo. Los cuatro bindings reciben y responden solicitudes
tipadas del servidor; ninguno añade un `send()` de cliente a esos tipos.
Consulta la referencia de cada lenguaje para conocer las reglas de propiedad y
liberación.

## Cambio del rol de endpoint

Set Endpoint Role de KMIP está admitido en ambas direcciones. Tras una
respuesta satisfactoria al intercambio Set Endpoint Role iniciado por el
cliente sobre TLS directo, convierte la conexión mediante
`into_role_reversed` y usa la `RoleReversedSession` resultante para recibir
solicitudes del servidor y enviar respuestas del cliente por el mismo flujo
establecido. Un listener HTTPS acepta conexiones HTTPS entrantes; no se puede
convertir en una sesión de cambio de rol sobre TLS directo.

El cambio de rol determina qué endpoint envía el siguiente mensaje KMIP.
Vincula esa transición a la respuesta satisfactoria del protocolo y a la
conexión correspondiente. No es sondeo en segundo plano ni autoriza reconectar
y reproducir solicitudes automáticamente.

## Seguridad y evidencia

La decodificación del listener aplica los límites configurados de tamaño y
estructura. El material secreto recibido en solicitudes Put del servidor
sigue las mismas reglas de puesta a cero y redacción que los demás secretos
KMIP. La aplicación debe autorizar y validar cada solicitud según su propia
política.

El [estado del proyecto](../../project-status.md) resume la implementación y
las evidencias de release. La matriz de servidores mantiene los resultados
externos por separado: la existencia de la API de escucha no demuestra que se
haya probado cada operación, codificación o cambio de rol con un servidor
concreto. Consulta la
[especificación KMIPKIT-0066](../../../specs/066-server-to-client-operations/spec.md)
y sus referencias normativas antes de implementar un handler.
