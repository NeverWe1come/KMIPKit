# Evidencia de perfiles cliente KMIP 2.1

Esta guía explica cómo leer la evidencia de los perfiles cliente de KMIPKit y
usar su punto de entrada explícito de validación. La selección en tiempo de
ejecución falla de forma cerrada por ahora: el registro tipado incluye una
regla para Wrapped Secret Data según §2.7, pero Baseline aún no tiene la
cobertura completa de reglas y pruebas aplicables para completar un intercambio
validado.

## Selección explícita por intercambio

Pasa un cliente configurado, un lote tipado y los límites del códec a una
función que selecciona un perfil para un intercambio. Esta función Rust
completa compila como ejemplo de rustdoc y devuelve al llamador el resultado o
el error estructurado:

```rust,no_run
use kmipkit::{
    Client, ClientBatch, ClientBatchResponse, ClientError, ProfileValidationSelection,
};
use kmipkit_ttlv::codec::CodecLimits;

fn execute_for_profile(
    client: &mut Client,
    batch: ClientBatch,
    limits: &CodecLimits,
) -> Result<ClientBatchResponse, ClientError> {
    let selection = ProfileValidationSelection::new(["KMIPKIT-PROFILE-BASELINE-CLIENT"]);
    client.execute_with_profiles(batch, &selection, limits)
}
```

El método valida aplicabilidad, dependencias y cobertura tipada completa antes
de escribir en el transporte. Con el catálogo y registro de reglas actuales,
Baseline Client falla de forma cerrada y el error devuelve identificadores
estables del perfil, requisito y cláusula de origen, junto con el estado de
envío `NotSent`. El error estructurado se consulta con
`ClientError::profile_validation_error()`. `Client::execute` sigue sin perfil y
no selecciona un valor predeterminado oculto. Un intercambio perfilado que
termine correctamente estará disponible cuando se registren las reglas
tipadas aplicables y sus pruebas de verificación.

## Estado actual

KMIP Specification v2.1 §14.1 exige que un cliente cumpla uno o más perfiles
cliente. KMIPKit registra Baseline Client como objetivo inicial de 1.0 para el
trabajo futuro de evidencia. Esa selección es una decisión de planificación;
no demuestra que el perfil esté implementado ni satisface el requisito del
estándar.

El estado de Baseline Client en el catálogo es `evidence_incomplete`. Los 16
fixtures oficiales obligatorios están fijados en el repositorio. T036 registra
13 reproducciones deterministas completas del cliente y 3 bloqueos previos al
envío causados por la validación tipada. Las reproducciones no demuestran
interoperabilidad con servidores ni conformidad del perfil; esos bloqueos y
las demás brechas de evidencia siguen abiertos. Otros 77 casos vinculados a
perfiles no tienen fixture local. KMIPKit no reconstruye ni deduce el contenido
de fixtures ausentes.

El mapping tipado para recuperar resultados archivados de Locate solo registra
un flujo dirigido por la aplicación. El preflight del perfil comprueba un
intercambio cada vez; no impone ni ejecuta la secuencia Recover y luego Get de
KMIP Specification v2.1 §6.1.28. La aplicación debe solicitar Recover cuando
Locate termine correctamente y después Get cuando Recover tenga éxito.

Los mappings de Recover también registran seguimientos opcionales dirigidos
por la aplicación: esta puede solicitar Poll con el valor de correlación si
Recover devuelve Pending y Get después de un Recover exitoso (KMIP
Specification v2.1 §6.1.42). El preflight del perfil no hace Poll, no lo
repite ni solicita Get automáticamente, y no garantiza que el servidor
complete la operación.

El mapping Baseline Client de Derive Key registra la obligación de la
aplicación de elegir un Managed Object fuente cuyo Cryptographic Usage Mask
incluya el bit Derive Key (KMIP Specification v2.1 §6.1.14). Esa máscara
pertenece al estado del objeto en el servidor. El preflight del perfil no la
consulta ni la certifica, y no vuelve elegible por sí solo una petición
Derive Key.

KMIP Specification v2.1 §§6.1.45–6.1.47 recomienda ejecutar Re-certify,
Re-key y Re-key Key Pair como máximo una vez para el mismo certificado, clave o
par de claves existente. KMIPKit no conserva historial de objetos entre
llamadas; la aplicación debe evitar repetir esas operaciones de sustitución.
Cuando la aplicación necesite credenciales nuevas, debe invocar explícitamente
Re-Provision conforme a §6.1.48. KMIPKit no deduce la caducidad de las
credenciales ni invoca Re-Provision automáticamente. Estos mapeos a cargo de
la aplicación no demuestran aceptación del servidor ni conformidad del perfil.

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
