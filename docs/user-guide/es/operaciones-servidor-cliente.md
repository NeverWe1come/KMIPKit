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

Los bindings de C, Java y Python exponen las APIs correspondientes de
listener, dispatcher y acceso tipado a solicitudes. Consulta el manifiesto de
API publicado y la referencia de cada lenguaje para conocer las reglas exactas
de propiedad y liberación.

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
