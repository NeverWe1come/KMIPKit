# Recorridos de la API fluida

Los consumidores de los cuatro paquetes ejecutan los mismos cinco recorridos
en Rust, C, Java y Python, usando un fixture loopback con TLS 1.3 y
autenticación mutua:

1. Crear una clave simétrica, obtenerla con el identificador devuelto y
   destruirla.
2. Crear una clave AES-256, cifrar y descifrar con parámetros AES-GCM
   explícitos y destruir la clave.
3. Crear un par ECDSA P-256, firmar datos conocidos y verificar la firma con el
   identificador público devuelto.
4. Añadir a Create el atributo catalogado Object Group como Text String; un
   valor Integer negativo se rechaza localmente antes del transporte.
5. Crear una clave simétrica y obtenerla en el siguiente elemento del mismo
   batch ordenado mediante ID Placeholder.

El último recorrido sigue KMIP Specification v2.1 §6.1 y §11.58 Tabla 489.
El batch establece Batch Order Option a `true`. Create no incluye un Unique
Identifier; Get envía la enumeración `1`, correspondiente a ID Placeholder.
Cada elemento incluye un Unique Batch Item ID distinto y el peer lo repite en
la respuesta correspondiente. El peer controlado asigna el identificador en
la respuesta de Create y devuelve ese mismo identificador en Get. El consumidor
lo lee de la respuesta de Create solo para comprobar el resultado; nunca lo
adivina ni lo copia en la petición Get.

Desde la raíz del repositorio, ejecuta todos los consumidores en Linux o macOS
con:

```sh
sh tests/consumers/0093-journeys/verify.sh
```

En Windows, ejecuta `tests/consumers/0093-journeys/verify.ps1`. El script
compila cada consumidor contra los paquetes del checkout y exige que terminen
las 52 conexiones TLS ordenadas. El fixture valida los campos de las peticiones
decodificadas y devuelve respuestas KMIP codificadas. Esto verifica el flujo
del cliente y el intercambio controlado; no demuestra interoperabilidad con un
KMS externo.

Implementaciones de los consumidores:

- Rust: `tests/consumers/0093-journeys/rust/src/main.rs`
- C: `tests/consumers/0093-journeys/c/main.c`
- Java: `tests/consumers/0093-journeys/java/src/main/java/Consumer.java`
- Python: `tests/consumers/0093-journeys/python/main.py`
- Fixture TLS 1.3/mTLS: `tests/consumers/0093-journeys/local_journey_responder.py`

## Operaciones del borrador KMIP 3.0

El cliente fluido también ofrece builders tipados para las siete operaciones
del borrador KMIP 3.0 asignadas por KMIPKIT-0116: Create Credential, Create
Group, Create User, Deactivate, Decapsulate, Encapsulate y Obliterate. Selecciona
explícitamente la versión 3.0 en el cliente o en las opciones de la petición
antes de ejecutarla; 2.1 continúa siendo la versión predeterminada. Esta
superficie acotada no implica conformidad general con KMIP 3.0.

| Operación | Entradas del builder |
| --- | --- |
| Create Credential | Atributos ordenados y una alternativa de credencial tipada: Certificate, Password, Device, One Time Password o Hashed Password. La alternativa determina Credential Type. |
| Create Group / Create User | Valores `AttributeSet` ordenados. |
| Deactivate | Unique Identifier y, opcionalmente, Deactivation Date y Deactivation Reason. |
| Decapsulate | Unique Identifier, texto cifrado y parámetros criptográficos tipados. La respuesta no devuelve el secreto compartido que gestiona el servidor. |
| Encapsulate | Unique Identifier y parámetros criptográficos tipados. El resultado puede exponer su Unique Identifier y los datos públicos (`Data`). |
| Obliterate | Unique Identifier. |

Como ejemplos, Create Group empieza con `client.request().create_group(attributes)`
en Rust, `new FluentRequest(client).createGroup(attributes)` en Java,
`Request(client).create_group(attributes)` en Python y
`kmipkit_client_create_group_v1(...)` en C. Las demás operaciones usan
snake_case en Rust y Python, y camelCase en Java. C ofrece funciones
`kmipkit_client_*_v1` con códigos de estado de ancho fijo para las tareas
directas y el manejador opaco `kmipkit_kmip30_operation_builder_t` para opciones
y campos avanzados. Cada builder valida localmente, puede crear un elemento de
batch y se ejecuta mediante un intercambio tipado. No reintenta ni envía Poll
automáticamente.

Las pruebas ejecutables de rutas están junto a cada binding:

- Rust: `crates/kmipkit-client/tests/unit/fluent_request_tests.rs`
- C: `bindings/c/tests/fluent_kmip30_consumer.c`
- Java: `bindings/java/src/test/java/org/kmipkit/Kmip30FluentExecutionTest.java`
- Python: `bindings/python/tests/test_fluent_kmip30.py`
