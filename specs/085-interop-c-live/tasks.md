# Tareas

- [x] Instalar y verificar la biblioteca C desde el commit exacto de release.
- [x] Parsear y validar configuración, perfiles, catálogo y el esquema de resultados.
- [x] Implementar plan y preflight offline, con salida saneada.
- [x] Sustituir el driver simulado por el cliente TLS público del binding.
- [ ] Añadir recetas tipadas y cobertura exhaustiva para ambas versiones.
- [x] Ejecutar Batch Error Continuation Option Stop y validar el resultado parcial de Get fallido en KMIP 2.0/2.1.
- [x] Ejecutar Logout con el Ticket retornado por Login en KMIP 2.1; clasificar la limitación C de Logout en KMIP 2.0.
- [x] Ejecutar Interop con Begin, End y Reset ordenados en KMIP 2.0 y 2.1, incluyendo IDs de lote únicos y respuestas TTLV tipadas.
- [x] Verificar construcción/navegación TTLV y registrar la indisponibilidad de envío TTLV genérico.
- [x] Añadir un peer TLS local con respuestas deterministas para Discover Versions, Ping, Query y Log; verificar autenticación Username/Password.
- [x] Asegurar que la CI exige razón saneada para cada caso no ejecutado o restringido.
- [x] Actualizar documentación de uso, límites y evidencia reproducible.

## Evidencia de desarrollo: operación Interop

- Red: `4337c0254`, `abaface48`, `150afe86e` y `f885a79b2` añadieron
  regresiones para receta, secuencia Begin/End/Reset, efecto de estado y IDs
  únicos de lote. Las pruebas de receta ausente e ID ausente fallaron antes
  de sus implementaciones.
- Green: `c4cf549c3` construye las tres peticiones mediante la API C instalada,
  usa un identificador de lote distinto para cada una y valida la respuesta
  TTLV completa de cada resultado. El peer controlado aplica la transición de
  prueba sin conservar el identificador en claro.
- Referencias: OASIS KMIP 2.0 §6.1.25/Table 223 y KMIP 2.1 §6.1.26/Tables
  241–243. Interop es control de estado de prueba y requiere el gate de mutación
  del catálogo común; el cambio de clasificación está en la PR #180.
- Refactor/documentación: README y `implementation-evidence.md` describen el
  flujo, sus límites y las pruebas sin incluir IDs ni cuerpos KMIP.
- Verificación: CMake build pasó; 3 pruebas enfocadas de Interop y las 12
  combinaciones controladas (dos versiones, tres codificaciones y dos
  transportes) pasaron; CTest pasó 7/7. La matriz completa C sigue incompleta,
  por lo que esta PR permanece WIP.

## Evidencia de desarrollo: campo Ephemeral y parser tipado

- Red: `85eaf12cf` y `dc8a0103c` añadieron pruebas que fallaron al faltar una
  petición KMIP válida para ambas ramas Boolean y al no incluir la función
  pública de parser tipado en las rutas locales del coordinador.
- Green: `3c05d47a7` genera un Request Message Get para KMIP 2.0 y 2.1, construye
  y decodifica TTLV, y llama `kmipkit_request_message_from_ttlv`; verifica el
  árbol de TTLV original sin guardar el cuerpo en el informe.
- Referencias: OASIS KMIP 2.0 §8.3/Table 358 y KMIP 2.1 §8.3/Table 396 para el
  campo; §10.1.2 para el valor Boolean de ocho octetos.
- Verificación: prueba focal pasada, `test_route_matrix.py` pasó 123/123 y
  CTest pasó 7/7. La matriz completa quedó en 38.539/66.728 celdas ejecutadas,
  cero fallidas y 28.189 pendientes. La tarea de cobertura exhaustiva sigue
  abierta y esta PR permanece WIP.
