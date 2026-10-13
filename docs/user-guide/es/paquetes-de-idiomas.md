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
| `public_binding_surface` | blocked | El inventario completo de declaraciones públicas y sus mapeos aún no está reconciliado. |
| `vendor_extension_registry` | declared | Hay mapeos compartidos exactos; falta verificar la paridad y las declaraciones no reconciliadas siguen bloqueadas. |
| `discover_versions_request_item` | declared | Hay mapeos de los constructores del elemento de solicitud en los cuatro lenguajes; la ejecución completa y la paridad de respuesta de Discover Versions siguen bloqueadas. |

Las **56 / 56** filas de operaciones KMIP `client_1_0` tienen actualmente el
estado `blocked` en el manifiesto. No se debe inferir que una operación está
disponible porque haya tipos o código de implementación en otra parte del
repositorio. Esta instantánea no afirma paridad completa de operaciones,
compatibilidad con un perfil ni certificación formal de KMIPKit.

El [manifiesto](../../../specification/api/public-api.json) y el [diseño de
la API pública](../../architecture/public-api.md) contienen los propietarios
de operaciones y mapeos por capacidad. El proyecto todavía no se ha publicado.
