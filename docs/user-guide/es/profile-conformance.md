# Evidencia de perfiles cliente KMIP 2.1

Esta guía explica cómo leer la evidencia de los perfiles cliente de KMIPKit.
No describe una API de validación de perfiles que pueda utilizarse actualmente.
El registro tipado de reglas de perfil está vacío, por lo que ningún perfil
está disponible para la validación en tiempo de ejecución.

## Estado actual

KMIP Specification v2.1 §14.1 exige que un cliente cumpla uno o más perfiles
cliente. KMIPKit registra Baseline Client como objetivo inicial de 1.0 para el
trabajo futuro de evidencia. Esa selección es una decisión de planificación;
no demuestra que el perfil esté implementado ni satisface el requisito del
estándar.

El estado de Baseline Client en el catálogo es `evidence_incomplete`. Sus 16
registros de fixtures oficiales obligatorios no están disponibles en el
repositorio fijado. Siguen siendo brechas de evidencia y no cuentan como
pruebas superadas. KMIPKit no reconstruye ni deduce el contenido de esos
fixtures ausentes.

## Cómo leer el estado de un perfil

El [informe de cobertura generado](../../../specification/catalog/coverage-report.md)
muestra por separado la aplicabilidad del perfil, su selección como objetivo,
la cobertura de reglas para tiempo de ejecución, el estado de evidencia del
catálogo, las pruebas enlazadas y la disponibilidad de fixtures:

- **Aplicabilidad** describe el ámbito del perfil en KMIPKit 1.0. Un perfil
  fuera de `client_1_0` puede ser exclusivo del servidor, condicional o estar
  aplazado.
- **Selección como objetivo** identifica un perfil cuya evidencia el proyecto
  pretende establecer. No lo convierte en una opción de tiempo de ejecución.
- **Validación en tiempo de ejecución** requiere una regla tipada completa y
  una prueba ejecutable y adecuada a la categoría de cada requisito aplicable.
  Si falta una regla o está incompleta, la selección sigue deshabilitada.
- **Estado de evidencia** refleja la evidencia registrada en el catálogo. Un
  informe local no puede mejorarlo; los datos obsoletos `evidence_complete`
  se tratan como incompletos.
- **Afirmación pública** es una decisión de publicación independiente. Esta
  funcionalidad no publica una afirmación de conformidad.

Usa los identificadores enlazados de cláusulas y requisitos para consultar las
obligaciones de las fuentes. Los resúmenes del catálogo son metadatos
informativos; nunca se interpretan como reglas ejecutables. Una respuesta del
servidor a Query o Discover Versions tampoco demuestra soporte de perfiles.

## Preparación de una afirmación

Un perfil solo puede ser elegible para una afirmación futura cuando todos sus
requisitos aplicables y pruebas oficiales obligatorias tengan evidencia
actual. Los fixtures no disponibles, los enlaces de implementación o
verificación ausentes, las reglas tipadas incompletas, las condiciones no
satisfechas y las dependencias pendientes mantienen el perfil como no
apto. Aunque la evidencia llegue a ser elegible, eso no publica una afirmación;
la publicación pertenece al proceso de versión revisado por separado.

Consulta la [estrategia de conformidad](../../compliance/conformance.md), el
[inventario KMIP 2.1](../../../specification/catalog/README.md) y la
[especificación KMIPKIT-0010](../../../specs/010-profile-conformance/spec.md)
para conocer el modelo de evidencia y su alcance.
