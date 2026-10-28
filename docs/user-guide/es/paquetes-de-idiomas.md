# Paquetes para Java y Python

## Versión y entornos de ejecución

Los metadatos de los paquetes Java y Python siguen el campo `releaseLine` del
[manifiesto de API pública](../../../specification/api/public-api.json), que
ahora indica `1.0.0`. Las coordenadas Java son
`org.kmipkit:kmipkit-java`; el nombre de la distribución Python es `kmipkit`.
Gradle, Maven y los metadatos de Python usan la misma versión. Es una versión
de metadatos aún no publicada; no significa que el artefacto 1.0.0 ya se haya
distribuido.

Los entornos mínimos son Java 17 y CPython 3.12. La matriz actual de destinos
Java y Python del manifiesto es:

| ID de destino | Sistema operativo y arquitectura |
|---|---|
| `windows-x86_64-msvc` | Windows, x86-64 (MSVC) |
| `linux-glibc-x86_64` | Linux con glibc, x86-64 |
| `linux-glibc-aarch64` | Linux con glibc, AArch64 |
| `macos-x86_64` | macOS, x86-64 |
| `macos-aarch64` | macOS, Apple Silicon |

Estos son los destinos de paquete registrados en el manifiesto. No afirman
que todos los artefactos estén publicados ni que hayan terminado las pruebas
de todas las plataformas.

## Carga de la biblioteca nativa

Ambas familias de paquetes incluyen su biblioteca nativa y no descargan
código ejecutable durante la ejecución.

Java selecciona el recurso del JAR nativo correspondiente al sistema
operativo y arquitectura actuales, verifica su archivo SHA-256, lo extrae de
forma atómica en un directorio temporal privado y comprueba la biblioteca
cargada mediante `kmipkit_version_get`. Requiere ABI 1. La propiedad explícita
de administrador `-Dkmipkit.nativeLibrary=<ruta>` sustituye la selección del
recurso incluido. Se considera código ejecutable confiable; si la ruta no es
válida, falla sin volver al recurso empaquetado. El cargador no busca en
`java.library.path` ni descarga bibliotecas. Consulta [la carga nativa de
Java](../../../bindings/java/docs/native-library-loading.md) para conocer
los detalles de permisos y verificación del sistema de archivos.

Python carga el único archivo regular `.dll`, `.dylib` o `.so` situado junto
a las declaraciones CFFI generadas del paquete instalado. Rechaza enlaces
simbólicos y bibliotecas adyacentes ausentes o ambiguas. La única ruta
alternativa de administrador es `KMIPKIT_NATIVE_LIBRARY`; si está definida,
sustituye la selección incluida y, si no es válida o compatible, falla sin
fallback. Antes de exponer CFFI, el paquete llama a `kmipkit_version_get` y
requiere ABI 1. Python no busca en otros directorios ni descarga bibliotecas.

## Alcance actual de capacidades en el manifiesto

Las etiquetas de estado de esta tabla se copian del manifiesto actual.
`declared` significa que el manifiesto contiene un mapeo; no demuestra que se
haya verificado la paridad completa entre lenguajes ni que haya pasado la
validación de publicación. `blocked` significa que no se debe anunciar la
capacidad como disponible. El manifiesto es la fuente para actualizar esta
tabla.

| ID de capacidad | Estado en el manifiesto | Alcance actual |
|---|---|---|
| `generic_ttlv_model` | declared | Hay un mapeo para cuatro lenguajes; falta verificar toda la paridad. |
| `ttlv_wire_codec` | declared | Hay un mapeo para cuatro lenguajes; falta verificar toda la paridad. |
| `ffi_runtime_metadata` | declared | Los metadatos de versión y consulta de capacidades están mapeados en Rust, C, Java y Python. |
| `rust_transport_exchange` | declared | Intercambio de transporte exclusivo de Rust según ADR-0014 aceptada; no es una operación de cliente para otros lenguajes. |
| `public_binding_surface` | blocked | T011 confirma que cada declaración observada tiene un registro en el manifiesto o una exclusión aprobada exacta. Los mapeos de la capacidad siguen incompletos, por lo que continúa bloqueada. |
| `vendor_extension_registry` | declared | Los mapeos exactos incluyen Rust drop, C release, Java close y Python close para los handles nativos; falta verificar la paridad. |
| `discover_versions_request_item` | declared | Hay mapeos de los constructores del elemento de solicitud en los cuatro lenguajes; la conformidad completa de Discover Versions sigue pendiente. |
| `client_batch_model` | declared | El modelo de lotes tipados y ordenados está mapeado en cuatro lenguajes; aún no cubre todas las operaciones. |
| `client_execution` | declared | Los mapeos de solicitud y respuesta de Discover Versions están presentes en Rust, C, Java y Python; falta completar la conformidad de operaciones 1.0. |
| `crypto_response_ttlv` | declared | El acceso TTLV genérico se limita a errores aprobados de forma del identificador único en respuestas correctas de MAC, MAC Verify, Sign y Signature Verify. |
| `production_transport` | declared | Rust y los tres adaptadores exponen la configuración aprobada de transporte cliente TLS 1.3; falta evidencia de interoperabilidad. |
| `api_level_discovery` | declared | Rust, C, Java y Python exponen la taxonomía local de niveles de API sin acceso a la red. |
| `opaque_register_context` | declared | El contexto explícito de Opaque Object y la validación de la forma de solicitud Register están mapeados en cuatro lenguajes; esta capacidad distinta sigue en estado `declared`. La fila de la operación Register está `verified`. |
| `server_generated_creation` | declared | Los identificadores generados por el servidor, los metadatos del resultado y las representaciones de identificadores únicos están mapeados en cuatro lenguajes; la conformidad completa de las operaciones sigue pendiente. |
| `usage_allocation_guard` | declared | Un contexto local de Usage Allocation, propiedad del llamador, protege la ejecución de Encrypt, MAC y Sign en cuatro lenguajes; no aplica políticas del servidor. Véase [controles de uso](controles-de-uso.md). |

De las **56** filas de operaciones KMIP `client_1_0`, **4 / 56** siguen en
estado `blocked`: Cancel, Poll, Process y Query Asynchronous Requests. Sus
mapeos de API para cuatro lenguajes quedan sujetos a la aprobación de la
especificación propietaria KMIPKIT-0009; la respuesta discutida de Query sigue
como TTLV genérico mientras `KMIPKIT-DISC-039` permanezca abierta.
**8** filas están en estado `declared`: Create, Create Key Pair, Create Split
Key, Hash, MAC, MAC Verify, Sign y Signature Verify. Sus mapeos de solicitud
y respuesta para cuatro lenguajes están registrados; `declared` no demuestra
la conformidad completa de las operaciones. No se debe inferir que una
operación está disponible porque haya tipos o código en otra parte del
repositorio.
**44** mapeos de operación están en estado `verified`:
Activate, Add Attribute, Adjust Attribute, Archive, Certify, Check, Decrypt,
Delegated Login, Delete Attribute, Derive Key, Destroy, Discover Versions,
Encrypt, Export, Get, Get Attribute List, Get Attributes, Get Constraints,
Get Usage Allocation, Import, Interop, Join Split Key, Locate, Log, Login,
Logout, Modify Attribute, Obtain Lease, PKCS#11, Ping, Query, Recover,
Register, Revoke, Re-certify, Re-key, Re-key Key Pair, Re-Provision,
RNG Retrieve, RNG Seed, Set Attribute, Set Constraints, Set Defaults y
Validate. Son mapeos de API; no afirman conformidad completa de las
operaciones ni interoperabilidad con servidores. Logout envía
el Ticket proporcionado por el llamador; no añade credenciales a peticiones
posteriores ni cierra la sesión automáticamente. Véase [sesiones de credenciales](sesiones-credenciales.md). El mapeo de Derive
Key rechaza una respuesta correcta que no incluya el Unique Identifier
obligatorio y conserva el TTLV completo y
ordenado en los cuatro lenguajes. Ping mapea la petición y la respuesta correcta
con payload vacío, y no implica que el servidor esté sano en general. Query
conserva el orden de las funciones, los Object Groups y los Items desconocidos;
acepta ambas formas descritas en §6.1.40 mientras `KMIPKIT-DISC-047` siga sin
resolver. Son mapeos de API, no afirmaciones de conformidad del servidor. El
mapeo de Get Constraints envía la petición
vacía de §6.1.22 y expone la estructura Constraints obligatoria y ordenada,
además del TTLV completo de la respuesta correcta en Rust, C, Java y Python. C
devuelve una copia propiedad del llamador; Java y Python liberan su respuesta
con `close()`. Pending y Failure conservan los metadatos comunes del resultado
y de entrega, sin exponer TTLV de respuesta correcta ni reintentar. El mapeo de
Export conserva el identificador opcional, los selectores de Table 217, la
especificación de wrapping y los recursos de resultado propios en Rust, C, Java
y Python. Get conserva el Unique Identifier opcional y los selectores de Table
220 y expone el Any Object correcto como respuesta propia en los cuatro
lenguajes. Locate conserva los filtros opcionales, el orden del servidor, los
Unique Identifier repetidos y la diferencia entre Located Items omitido y
vacío. El mapeo de Get Usage Allocation incluye su petición con identificador
opcional y contador con signo, la respuesta con identificador resuelto y los
resultados comunes del lote en Rust, C, Java y Python. Esta evidencia de mapeo
no afirma la conformidad completa de las operaciones. Re-certify mapea los
campos de petición de la Tabla 300: Unique Identifier y Certificate Request
Unique Identifier opcionales, Certificate Request Type (obligatorio cuando
está presente Certificate Request Value), Certificate Request Value, Offset,
Attributes y Protection Storage Masks. Las respuestas correctas exponen el
identificador de reemplazo y el TTLV completo y ordenado en Rust, C, Java y
Python. Re-key mapea los campos opcionales de la Tabla 305: Unique Identifier,
Offset, Attributes y Protection Storage Masks; el llamador debería enviarlo
como máximo una vez por clave. Las respuestas correctas de la Tabla 306 exponen
el Unique Identifier de reemplazo y el TTLV completo y ordenado, incluidos
campos desconocidos. C devuelve una copia TTLV propiedad del llamador; Java y
Python devuelven un resultado propio que el llamador debe cerrar. Failure y
Pending conservan los metadatos comunes del resultado y de entrega sin campos
de éxito, cuerpos de error sin procesar ni reintentos automáticos. Es evidencia
de mapeo de API, no una afirmación de conformidad. Re-key Key Pair mapea §6.1.47,
Tablas 308–312: la petición puede incluir Private Key Unique Identifier,
Offset, tres grupos diferenciados de Attributes y tres grupos diferenciados
de Protection Storage Masks. Java usa `ClientBatchItem.rekeyKeyPair(...)`;
Python usa `extensions.client_batch_item_rekey_key_pair(...)`. C usa
`kmipkit_client_batch_item_rekey_key_pair` y copia cada estructura recibida
antes de volver. Si se omite el identificador, se conserva el ID Placeholder
del servidor y no se inventa localmente. Una respuesta Success expone ambos
identificadores de sustitución y el TTLV completo y ordenado, incluidos los
campos desconocidos. Los llamadores de Java y Python cierran la respuesta
propia; en C, los spans de texto son prestados hasta liberar el elemento de
respuesta, mientras que `kmipkit_client_batch_item_response_rekey_key_pair_ttlv`
devuelve una copia independiente propiedad del llamador. Failure y Pending
conservan el estado, el motivo, la correlación y los metadatos de entrega
comunes, sin campos de éxito del par ni seguimientos automáticos. Este mapeo
no afirma conformidad, interoperabilidad, compatibilidad con perfiles ni
certificación. Re-Provision mapea §6.1.48, Tablas 313–315. Java usa
`ClientBatchItem.reprovision(...)`; Python usa
`extensions.client_batch_item_reprovision(...)`; y C usa
`kmipkit_client_batch_item_reprovision`. Certificate Request y Certificate
son opcionales y mutuamente excluyentes; si se omiten ambos, el servidor crea
la credencial. Una respuesta correcta expone el Unique Identifier opcional y
el TTLV completo y ordenado. Los llamadores de Java y Python cierran la
respuesta propia; en C, el span del identificador es prestado hasta liberar el
elemento de respuesta y el accessor TTLV devuelve una copia propiedad del
llamador. Failure y Pending exponen solo los metadatos comunes, sin campos de
éxito, cuerpos sin procesar ni seguimientos automáticos. Es evidencia del
mapeo de API, no una afirmación de conformidad. Log conserva el mensaje
de solicitud y expone el TTLV completo de respuesta mediante el resultado tipado
en los cuatro lenguajes. Login mapea los campos opcionales de la Tabla 253 y
expone el Ticket Type raw, el Ticket Value y el TTLV genérico ordenado de una
respuesta correcta. En C, `kmipkit_client_batch_item_response_login` devuelve
un span del Ticket Value prestado hasta liberar el elemento de respuesta;
`kmipkit_client_batch_item_response_login_ttlv` devuelve una estructura
independiente que se libera con `kmipkit_ttlv_structure_release`. Java usa
`ClientBatchItem.login(...)` y `ClientBatchItemResult.LoginResponse`;
`ticketValue()` devuelve un `byte[]` propiedad del llamador que debe protegerse
y limpiarse, y `close()` limpia el valor propiedad de KMIPKit y cierra la copia
TTLV del Ticket. Python usa `extensions.client_batch_item_login(...)` y
`client.LoginResponse`; `ticket_value` devuelve `bytes` inmutables propiedad
del llamador, mientras que `close()` pone a cero el almacenamiento mutable
propiedad de KMIPKit y cierra su copia TTLV del Ticket. Los llamadores y
runtimes de Python no pueden borrar determinísticamente las copias inmutables
devueltas. Java y Python redactan el Ticket Value en la salida formateada. En C,
el span del Ticket Value es prestado y no debe registrarse. Esta instantánea no
afirma paridad completa de operaciones, compatibilidad con un perfil ni
certificación formal de KMIPKit.

Obtain Lease mapea los campos de su respuesta correcta y conserva el TTLV
completo y ordenado en los cuatro lenguajes. C devuelve una estructura TTLV
propia; Java expone `ObtainLeaseResponse.responseTtlv()` y Python,
`ObtainLeaseResponse.response_ttlv`; el llamador cierra el resultado cuando
termina. KMIPKit no controla la expiración del lease ni inicia temporizadores.

Encrypt y Decrypt también están mapeados en los cuatro lenguajes. En Java se
crean elementos con `ClientBatchItem.encrypt(...)` y
`ClientBatchItem.decrypt(...)`; en Python se usan
`extensions.client_batch_item_encrypt(...)` y
`extensions.client_batch_item_decrypt(...)`. Para un resultado correcto, Java
expone `encryptResponse()` o `decryptResponse()`, y Python los campos
`encrypt_response` o `decrypt_response`. Encrypt conserva el Unique Identifier
obligatorio y cada valor presente de Data, IV/Counter/Nonce, Correlation Value
y Authenticated Encryption Tag. Decrypt conserva el identificador obligatorio
y los valores presentes de Data y Correlation Value. Estas operaciones envían
el trabajo criptográfico al servidor KMIP; KMIPKit no cifra ni descifra
localmente. Los accesores Java devuelven copias `byte[]` propiedad del llamador,
que debe protegerlas y limpiarlas. Python devuelve `bytes` inmutables; ni el
llamador ni el runtime de Python pueden borrarlas de forma determinista. KMIPKit
redacta estos valores en `toString()` de Java y `repr()` de Python. Tras usarlos,
cierra cada `EncryptResponse` o `DecryptResponse` de Java y llama a `close()` en
cada respuesta Python para poner a cero los bytes mutables que conserva KMIPKit;
cerrar el elemento del lote no cierra la respuesta tipada devuelta por separado.

El [manifiesto](../../../specification/api/public-api.json) y el [diseño de
la API pública](../../architecture/public-api.md) contienen los propietarios
de operaciones y mapeos por capacidad. El proyecto todavía no se ha publicado.
