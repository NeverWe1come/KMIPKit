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

## Evidencia de desarrollo: payload de Get Request

- Red: `d1ccec5e2` exige ejecutar los casos normativos Get Request de KMIP 2.0
  y 2.1 en las seis combinaciones de codificación/transporte y falló cuando
  ninguna celda tenía una ruta ejecutable.
- Green: `10d86288c` asocia los dos casos con la receta tipada Get y el peer
  controlado, y habilita el flujo Get en el driver C.
- Referencias: OASIS KMIP 2.0 §6.1.19/Table 205 y KMIP 2.1 §6.1.19/Table 220.
- Verificación: la prueba focal pasó y la matriz completa registró 38.601 de
  66.728 celdas ejecutadas, cero fallidas y 28.127 pendientes. Las 552 celdas
  asociadas a los dos casos ya no están pendientes; 62 se ejecutaron en esa
  corrida. La tarea de recetas exhaustivas sigue abierta y esta PR permanece WIP.

## Evidencia de desarrollo: payload de Get Attribute List

- Red: `b6b8845dc` añadió la regresión de las rutas para los casos Get
  Attribute List de ambas versiones; falló sin celdas ejecutadas. `a38831798`
  añadió la comprobación de presencia del identificador y observó que ambas
  rutas todavía lo omitían.
- Green: `5ff230c63` cubre los dos casos con los constructores C directo y
  request tipado independiente. El peer verifica el Unique Identifier textual
  presente; las rutas externas conservan la omisión normativa para resolver el
  ID Placeholder dentro del lote. El test del paquete instalado comprueba las
  formas opcionales presente y omitida en ambos constructores.
- Referencias: OASIS KMIP 2.0 §6.1.21/Table 211 y KMIP 2.1 §6.1.21/Table 226.
- Verificación: las 24 comprobaciones controladas pasaron; CTest pasó 7/7;
  ambos casos quedaron sin celdas pendientes. La matriz global registra
  38.729/66.728 celdas ejecutadas, cero fallidas y 27.999 pendientes. No se
  afirma compatibilidad con servidores externos; la cobertura C completa y
  los checks de CI exactos siguen pendientes.

## Evidencia de desarrollo: rutas de solicitud Add/Modify Attribute

- Red: `815520cf3` exige ejecutar New Attribute de Add Attribute con las dos
  formas públicas; `af28e51be` exige Current Attribute presente/omitido y New
  Attribute a través de los constructores directo y de solicitud tipada, en
  ambas versiones y las seis combinaciones de codificación/transporte. La
  prueba de Modify falló inicialmente sin celdas completadas.
- Green: `5e9eca027` implementa Add Attribute y `86c36646a` ejecuta Modify
  Attribute después de sembrar el objeto con un Comment existente. Las
  comprobaciones verifican el efecto de estado del peer, los campos de
  solicitud, los IDs de respuesta y la igualdad de los árboles normalizados
  entre las dos rutas para cada codificación y transporte.
- Referencias: OASIS KMIP 2.0 §6.1.2/Table 152 y §6.1.33/Tables 247–248;
  KMIP 2.1 §6.1.2/Table 167 y §6.1.34/Tables 265–266. Los wrappers Current
  Attribute y New Attribute siguen §5.6–§5.7 (Tables 147–148 en 2.0 y
  162–163 en 2.1): cada uno contiene un elemento de atributo directo; el
  fixture usa el tag Comment, no el contenedor Attribute.
- Refactor: `f876dcc07` comparte la construcción TTLV del atributo Comment
  directo entre Add, Set y Modify.
- Verificación: 24 comprobaciones controladas de Add y 48 de Modify pasaron;
  CTest pasó 7/7 tras la refactorización. La matriz anterior a la refactorización
  registró 39.193 celdas invocadas, cero fallidas y 27.543 pendientes
  (66.736 esperadas). La cobertura total sigue abierta: quedan, entre otras,
  las variantes de Unique Identifier de Modify, el rechazo por Current
  Attribute no coincidente, las respuestas negativas y varias rutas TTLV
  genéricas. No se afirma interoperabilidad con KMS externos.
