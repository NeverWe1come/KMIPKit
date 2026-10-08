# Extensiones de fabricante

Las Message Extensions de KMIP transportan datos definidos por fabricantes
mediante los campos estándar de KMIP 2.1 §9.13, Tabla 418. KMIPKit mantiene
cada definición en un registro inmutable propiedad de una configuración de
cliente. El registro describe datos; no carga código ni concede acceso al
transporte.

Las API del registro cubren la declaración y validación de extensiones, su
asociación a peticiones, la inspección de respuestas y la conservación de TTLV
genérico. Los ejemplos construyen un elemento de batch tipado Discover Versions
y no abren una conexión de red.

## Definir una extensión

Cada definición incluye un Vendor Identification no vacío formado por letras
ASCII, dígitos, `_` o `.`, el nombre y la versión de la extensión, rangos de
compatibilidad KMIP y KMIPKit, un esquema TTLV declarativo y una ruta y valor
exactos para el discriminador. Extension Information es opcional y sirve para
las vistas de metadatos locales.

El discriminador identifica el contenido dentro del espacio de nombres del
fabricante registrado. Vendor Identification por sí solo nunca reconoce un
contenido. KMIPKit valida el esquema completo antes de devolver un valor
registrado sellado. La API de peticiones tipadas no acepta un cuerpo sin
validar, un Item TTLV arbitrario ni una conversión definida por el llamador.

El ejemplo Rust ejecutable crea una definición, valida dos contenidos, elige
un Criticality Indicator para cada uso y los añade en el orden del llamador:

```text
cargo run -p kmipkit-client --example vendor_extension
```

Los ejemplos equivalentes están junto a cada adaptador:

| Adaptador | Ejecución |
|---|---|
| Rust | `cargo run -p kmipkit-client --example vendor_extension` |
| C | Compila `kmipkit-ffi`, configura y compila `bindings/c` con CMake y ejecuta `ctest --test-dir target/c-examples --output-on-failure`. Esto ejecuta `bindings/c/examples/vendor_extension.c`. |
| Java 17 | `mvn -f bindings/java/pom.xml test`; la prueba con JNI ejecuta `bindings/java/examples/VendorExtensionExample.java`. |
| Python 3.12 | En un entorno Python fijado, ejecuta `maturin develop` en `bindings/python` y después `python examples/vendor_extension_registry.py`. La suite también ejecuta la prueba del ejemplo. |

Para C, compila primero la biblioteca nativa para que CMake encuentre la ABI
C generada:

```text
cargo build -p kmipkit-ffi
cmake -S bindings/c -B target/c-examples
cmake --build target/c-examples
ctest --test-dir target/c-examples --output-on-failure
```

## Asociar una extensión a una petición

El llamador elige el Criticality Indicator cada vez que asocia un valor
registrado. KMIPKit no selecciona un valor predeterminado. Las Message
Extensions repetidas conservan el orden de asociación permitido por §8.3,
Tabla 396. Un valor validado con otro registro de cliente no se puede enviar
desde un cliente configurado con otro registro; la ejecución lo rechaza antes
de codificarlo o enviarlo.

La ejecución del cliente codifica Vendor Identification, Criticality
Indicator explícito y Vendor Extension validado en el orden de la Tabla 418.
La capa de extensiones utiliza el escritor privado existente y su propietario
de buffer con zeroize. No añade un escritor de mensajes sin validar ni reintenta
automáticamente peticiones KMIP.

## Inspeccionar extensiones del servidor

Al recibir una extensión, KMIPKit comprueba el fabricante y el discriminador
registrado exacto. Si no coincide ninguna definición o coinciden varios
discriminadores, el contenido queda sin reconocer. Una única coincidencia aún
debe superar el esquema completo registrado. El resultado no depende del orden
de registro.

Los valores reconocidos exponen una vista tipada y conservan su subárbol TTLV
genérico original. Las extensiones no críticas sin reconocer siguen
disponibles como TTLV genérico. Las extensiones críticas desconocidas se
rechazan según KMIPKIT-0007; el registro no debilita esa regla. Los valores
genéricos conservan tags desconocidos, enumeraciones, bits de bitmask, campos
repetidos y orden de hijos aceptados por la política de asignación TTLV de
KMIPKit.

La lista y el mapa locales de Extension Information representan metadatos de
Query Extension List y Query Extension Map (§7.13, Tabla 365; §11.44, Tabla
476). Describen únicamente este registro local. No afirman que un servidor
remoto admita una extensión, y esta función no ejecuta Query.

## Límites y tratamiento de secretos

Los límites TTLV predeterminados son 16 MiB por mensaje, profundidad 64 y
100.000 Items. Los límites predeterminados y máximos del registro son:

| Límite | Predeterminado | Máximo |
|---|---:|---:|
| Definiciones por registro | 256 | 1.024 |
| Nodos de esquema agregados | 16.384 | 100.000 |
| Reglas hijas por Structure | 256 | 4.096 |
| Bytes UTF-8 por campo de identidad/metadatos | 4.096 | 4.096 |
| Bytes agregados de identidad/metadatos | 1 MiB | 16 MiB |
| Bytes por escalar discriminador | 4.096 | 4.096 |
| Bytes agregados de escalares discriminadores | 1 MiB | 16 MiB |
| Miembros de restricciones por regla | 256 | 4.096 |
| Miembros agregados de restricciones | 16.384 | 100.000 |
| Registros del índice de payload | 200.000 | 200.000 |
| Comparaciones de búsqueda del discriminador | 1.048.576 | 4.194.304 |
| Profundidad de esquema/discriminador | 64 | 64 |

Los límites se pueden reducir o elevar hasta el máximo. La construcción del
registro comprueba los contadores agregados antes de copiar o reservar
memoria. Si se agota un presupuesto en tiempo de ejecución, se devuelve un
error redactado de límite de recursos y ningún resultado tipado parcial.
Las cadenas de metadatos de Java deben tener UTF-16 bien formado; si se rechaza
una descripción, el valor original de `ExtensionInformation` sigue siendo
utilizable y conserva su propiedad nativa.

KMIPKit redacta los payloads de extensión en errores y representaciones
predeterminadas. El buffer de petición codificado que posee KMIPKit se
sobrescribe al terminar el envío, tanto si la escritura tiene éxito como si
falla. Los consumidores Java y Python pueden crear copias de texto o bytes
administradas por el runtime; sus runtimes no permiten a KMIPKit sobrescribir
determinísticamente todas esas copias. No incluyas secretos en logs ni en la
salida de depuración.

## Límite de compatibilidad

El registro pertenece a un solo cliente y queda inmutable después de crearlo.
Sus esquemas son declarativos y no ejecutan scripts, callbacks ni plugins
cargados dinámicamente. Esta función no garantiza compatibilidad con las
políticas de todos los servidores; la aceptación depende de la implementación
y configuración del peer.
