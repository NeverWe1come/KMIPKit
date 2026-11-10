# Especificación: consumidor C de interoperabilidad KMIP

**Rama**: `feature/KMIPKIT-0085-interop-c-live`  
**Estado**: aprobado para implementación por autorización explícita del plan KMIPKIT de interoperabilidad  
**Identificador**: KMIPKIT-0085  
**Base**: release `9692057c9ade474fb0c4e0b7130539da368eb794`

## Alcance

El proyecto C independiente consumirá la biblioteca C pública instalada desde
un artefacto creado en un archivo limpio de un commit exacto de
`release/1.0.0`. Se prueban KMIP 2.0 y 2.1, sin extensiones de fabricante,
KMIP 1.x ni KMIP 3.0. El catálogo y el esquema comunes son la fuente de casos
y resultados.

La API pública C expone construcción tipada de operaciones y lectura,
construcción y navegación TTLV genérica. No expone un cliente de alto nivel ni
la emisión directa de un request TTLV genérico; esos dos límites se registran
como indisponibilidad de ruta pública y no como incompatibilidad del servidor.

## Requisitos y aceptación

| ID | Requisito |
|---|---|
| KMIPKIT-0085-FR-001 | `plan` y `preflight` validarán configuración y catálogo localmente, sin DNS ni sockets. |
| KMIPKIT-0085-FR-002 | `run` enlazará una biblioteca C instalada desde el commit pinneado y verificará su commit y SHA-256. No enlazará `target/debug` del checkout anfitrión. |
| KMIPKIT-0085-FR-003 | Cada caso aplicable del catálogo de KMIP 2.0/2.1 tendrá una receta tipada ejecutable o una explicación normativa/limitación pública explícita que no se confunda con éxito. CI fallará ante casos sin clasificación. |
| KMIPKIT-0085-FR-004 | Cada ejecución requerirá entorno dedicado; las mutaciones requieren habilitación explícita y no se reintentan. |
| KMIPKIT-0085-FR-005 | Los resultados cumplirán el esquema compartido 2.0, separarán corrección de biblioteca, compatibilidad observada, política y no ejecutados y no contendrán secretos, cuerpos KMIP ni texto remoto. |
| KMIPKIT-0085-FR-006 | Pruebas locales verificarán construcción tipada, TTLV genérico, batch ordenado, decodificación de respuestas y árboles normalizados efímeros; bytes exactos se exigirán solo para entradas deterministas con la misma codificación. |
| KMIPKIT-0085-FR-007 | CI compilará el consumidor externo con C11, instalará la biblioteca C desde el commit pinneado y ejecutará la suite contra TLS/respuestas controladas. No afirmará compatibilidad con un fabricante. |
| KMIPKIT-0085-FR-008 | La autenticación Username/Password opcional leerá nombres de variables de entorno, construirá credenciales con la API de secretos zeroizing y rechazará credenciales ausentes antes de abrir una conexión; ningún informe ni diagnóstico expondrá sus valores. |

## Seguridad y límites

El preflight solo prueba disponibilidad local. Una ejecución controlada no
demuestra que un KMS externo soporte la operación. No se inventarán respuestas
positivas para casos con precondiciones ausentes ni para operaciones
iniciadas-por-servidor. Los árboles normalizados y los alias de recursos solo
existirán durante la prueba y no se escribirán en los informes.
