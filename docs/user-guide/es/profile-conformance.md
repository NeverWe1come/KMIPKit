# Evidencia de perfiles cliente KMIP 2.1

Esta guía explica cómo leer la evidencia de los perfiles cliente de KMIPKit y
usar su punto de entrada explícito de validación. La selección runtime de
Baseline Client está disponible para sus 294 requisitos mapeados, incluidas las
reglas que dependen del estado del llamador en KMIP Specification v2.1 §§4.38
y 4.59. Esto habilita la validación de reglas tipadas, pero no demuestra la
evidencia oficial de conformidad ni la aceptación del servidor.

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
de escribir en el transporte. Esta entrada, sin contexto, no aporta el estado
remoto del objeto; las peticiones que necesiten esos datos fallan de forma
cerrada. Para ellas usa la entrada que acepta contexto descrita abajo. Cualquier
petición aún puede incumplir una regla antes del transporte; consulta los
errores estructurados con `ClientError::profile_validation_error()`.
`Client::execute` sigue sin perfil y no selecciona un valor predeterminado
oculto. Superar las comprobaciones locales no demuestra interoperabilidad con
servidores, éxito en las pruebas oficiales ni conformidad del perfil.

## Estado del objeto declarado por el llamador

Para las reglas que dependen del estado remoto, proporciona un contexto separado
para ese intercambio. Vincula cada entrada al índice de lote (empezando por
cero) y al Unique Identifier de tipo Text String exacto de la petición:

```rust,no_run
use kmipkit::{
    Client, ClientBatch, ClientBatchResponse, ClientError, ProfileObjectState,
    ProfileValidationContext, ProfileValidationSelection,
};
use kmipkit_ttlv::codec::CodecLimits;

fn execute_with_object_state(
    client: &mut Client,
    batch: ClientBatch,
    object_id: &str,
    limits: &CodecLimits,
) -> Result<ClientBatchResponse, ClientError> {
    let selection = ProfileValidationSelection::new(["KMIPKIT-PROFILE-BASELINE-CLIENT"]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(object_id)
            .with_original_creation_date_present(Some(true)),
    );
    client.execute_with_profiles_and_context(batch, &selection, &context, limits)
}
```

Cada hecho usa `Option`: `Some(true)` y `Some(false)` son afirmaciones del
llamador; `None` significa desconocido. Las entradas ausentes, desconocidas,
duplicadas o que no coincidan fallan antes de escribir en el transporte.
KMIPKit no consulta ni almacena el estado, no lo deduce de otro elemento del
lote ni verifica que la afirmación del llamador sea cierta. Los identificadores
y valores de estado se ocultan en errores y en la salida de depuración.

Para Original Creation Date, KMIP Specification v2.1 §4.38 permite Add si el
llamador afirma que el atributo no existe. La Tabla 106 indica que el cliente
no puede modificarlo ni eliminarlo, por lo que Modify, Delete y Set se rechazan
incluso si el llamador afirma que está ausente; una vez establecido, §4.38
también prohíbe actualizarlo o eliminarlo. Para Usage Limits, §4.59 y la
Tabla 149 prohíben cambiar Total cuando ya empezó el uso de protección o
después de Get Usage Allocation, según DEC-024. Un Set que cambie Total exige
Usage Limits presente y conocido, los valores actuales de Total y Count,
conserva Count y rechaza una estructura mal formada que omita un miembro
obligatorio de §7.40 Tabla 392. Sigue vigente la restricción independiente de
KMIPKIT-0016 para Add/Modify Usage Limits. La aplicación es responsable de
aportar un estado fiable.

## Estado actual

KMIP Specification v2.1 §14.1 exige que un cliente cumpla uno o más perfiles
cliente. KMIPKit registra Baseline Client como objetivo inicial de 1.0 para el
trabajo de evidencia. Esa selección es una decisión de planificación. La
validación runtime está disponible para los 294 requisitos mapeados de Baseline
Client. Esto habilita comprobaciones locales de reglas, pero no demuestra el
soporte del perfil ni satisface por sí solo el requisito de conformidad del
estándar.

El estado de Baseline Client en el catálogo es `evidence_incomplete`. Los 16
fixtures oficiales obligatorios están fijados en el repositorio. T036 registra
13 reproducciones deterministas completas del cliente y 3 bloqueos previos al
envío causados por la validación tipada. Otros 77 casos vinculados a perfiles
no tienen fixture local. Estas carencias de fixtures y reproducciones son
independientes de la validación runtime local. KMIPKit no reconstruye ni deduce
el contenido de fixtures ausentes.

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

Otros mapeos de Baseline registran responsabilidades de la aplicación que
dependen del estado remoto del objeto o del tiempo: Activation Date (§4.1),
Deactivation Date (§4.18), Process Start Date (§4.40), Protect Stop Date
(§4.41), el Cryptographic Usage Mask efectivo (§4.17), la caducidad del lease
(§§4.30 y 6.1.35) y la asignación de uso con su ventana de fechas (§6.1.23).
KMIP Specification v2.1 §4.34 también recomienda un solo
uso criptográfico por objeto e indica que el servidor no aplica esa
recomendación. KMIPKit la registra como una recomendación del cliente a cargo
de la aplicación: esta debería elegir y conservar el uso único previsto. KMIPKit
no consulta el estado remoto ni aplica el límite de usos. El requisito
obligatorio de monotonicidad de Hashed Password Timestamp (§9.11) está mapeado
a una función de secuencia a cargo de la aplicación según la decisión resuelta
OD-003 de KMIPKIT-0008. Exige marcas de tiempo que aumenten estrictamente por
Username y productor lógico de credenciales; la aplicación mantiene la
persistencia entre reinicios. La función no calcula hashes ni envía
credenciales, y el mapeo no establece el orden del cable ni la conformidad
completa del perfil. La capacidad OPTIONAL de fechas relativas de §9.20 es
independiente: si la aplicación la usa, proporciona el temporizador de cuenta
atrás y resta los valores Time Stamp y Date-Time expuestos. El preflight no
consulta el estado remoto, no lo conserva entre llamadas ni aplica estas
condiciones a peticiones normales o TTLV genéricas. Los §§4.40 y 4.41 describen
cuándo puede usarse un objeto para procesar información protegida, cuándo debe
dejar de aplicarse protección y la relación permitida con Activation y
Deactivation Date. Los párrafos no asignan qué extremo realiza el procesamiento
de información protegida; las tablas de reglas del atributo permiten que lo
establezca inicialmente el servidor o el cliente y describen la mutabilidad
por parte del cliente. KMIPKit registra estas reglas como responsabilidades
del llamador cuando las fechas y el Cryptographic Usage Mask sean pertinentes;
dependen de la hora actual, del uso criptográfico local y del estado del objeto.
KMIPKit no realiza esas operaciones criptográficas, no lee el reloj ni recupera
atributos remotos, por lo que el mapeo no las aplica automáticamente.
KMIP Specification v2.1 §4.57, párrafo 2, prohíbe usar un objeto Pre-Active
con cualquier fin criptográfico, pero no identifica un actor ni una dirección
de mensaje. KMIPKIT-DEC-020 interpreta esta regla como exclusiva del servidor
para el cliente KMIPKit 1.0: §4.57 dice que el servidor conoce el State y es el
único que lo modifica, y §6.1.17 define Encrypt como una petición del cliente
para que el servidor realice el cifrado. La cláusula fuente permanece en el
cierre de cláusulas de los perfiles cliente, pero el requisito se excluye de
su cierre de requisitos y de las reglas de ejecución. KMIPKit no consulta el
State remoto, no realiza criptografía local ni afirma aplicar esta restricción
del servidor.
Para las excepciones de Deactivation Date, la aplicación también debe comprobar
que existen circunstancias extraordinarias y permiso especial. Estos mapeos
estáticos registran obligaciones del llamador; no demuestran aceptación del
servidor, un caso oficial aprobado ni conformidad del perfil.

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
  Baseline Client ya cumple este control runtime; su estado de evidencia
  oficial sigue incompleto.
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
