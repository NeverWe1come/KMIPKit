# Plan de implementación

1. Reemplazar el contrato mock por configuración compartida, `plan`,
   `preflight`, informes saneados, autenticación desde variables de entorno y
   guardas de entorno/política.
2. Crear un instalador reproducible que compile la biblioteca FFI y copie el
   header público desde un archivo limpio del commit de release fijado.
3. Implementar adaptadores C sobre la API tipada pública y pruebas TTLV
   genéricas; clasificar todas las filas del catálogo por versión y ruta de API.
4. Añadir TLS controlado, respuestas deterministas y comparaciones
   estructurales efímeras para las rutas disponibles.
5. Ejecutar verificación C11, provenance, catálogo, informes y fixtures; abrir
   un PR `[WIP]` solo tras publicar la rama y CI.
