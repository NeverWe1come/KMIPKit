# Interop, Log y Validate de cadenas de certificados

El cliente Rust ofrece `Client::interop`, `Client::log` y `Client::validate`.
Cada llamada realiza un intercambio explícito y devuelve un resultado tipado.
Para aceptar Pending, incluya `ClientRequest::{Interop,Log,Validate}` en un
`ClientBatch` con el indicador asíncrono explícito. El
[contrato común de ejecución](ejecucion-cliente.md) conserva la correlación,
Result Status, Result Reason y el Result Message permitido. La aplicación
decide si realiza un Poll o una nueva solicitud.

Interop sigue OASIS KMIP v2.1 §6.1.26, Tablas 241–243. Interop Function e
Interop Identifier son obligatorios. La función conserva su Enumeration raw:
Begin es 1, End es 2 y Reset es 3 (§11.22); los valores de proveedor y futuros
se mantienen. La respuesta de éxito está vacía. La prohibición de ofrecer
Interop en producción corresponde al servidor. Implementar la operación del
cliente no demuestra interoperabilidad.

El mapeo aprobado en KMIPKIT-0042 también está disponible en las fachadas de
cliente C, Java y Python. C crea el elemento de batch con
`kmipkit_client_batch_item_interop` y obtiene la respuesta válida completa con
`kmipkit_client_batch_item_response_interop_ttlv`; libere la estructura
devuelta con `kmipkit_ttlv_structure_release`. Java usa
`ClientBatchItem.interop` y `ClientBatchItemResult.interopResponse`; cierre el
`InteropResponse` devuelto. Python usa `client_batch_item_interop` y
`BatchItemResult.interop_response`; cierre el `InteropResponse` devuelto.
Function conserva una Enumeration abierta de 32 bits sin signo e Identifier
debe ser UTF-8. El acceso a la respuesta está disponible para resultados
válidos Success, Failure y Pending; no expone cuerpos de respuesta de errores
locales de transporte, decodificación o validación. Estos accesores transfieren
al llamante una estructura TTLV genérica propia. Los runtimes de Java y Python
pueden conservar copias adicionales de los datos del identificador o la
respuesta.

```rust,kmipkit-test
use kmipkit_protocol::{InteropRequest, SecretText};

fn main() -> Result<(), kmipkit_protocol::ProtocolError> {
    let request = InteropRequest::new(
        1,
        SecretText::new("caller-selected-test-id".to_owned()),
    );
    let payload = request.into_ttlv_payload()?;
    assert_eq!(payload.view().children().len(), 2);
    Ok(())
}
```

Log sigue §6.1.29, Tablas 250–252. La decisión aprobada DISC-017 asigna la
Tabla 252 a Log. Log Message es obligatorio y el payload de éxito está vacío.
El usuario no debe enviar secretos al log del servidor. KMIPKit conserva el
texto en `SecretText`, lo oculta en los diagnósticos y lo mueve al payload
TTLV con borrado al liberar la memoria.

```rust,kmipkit-test
use kmipkit_protocol::{LogRequest, SecretText};

fn main() -> Result<(), kmipkit_protocol::ProtocolError> {
    let request = LogRequest::new(SecretText::new("test completed".to_owned()));
    let payload = request.into_ttlv_payload()?;
    assert_eq!(payload.view().children()[0].tag().raw(), 0x420141);
    Ok(())
}
```

Validate sigue §6.1.57, Tablas 340–342. Añada estructuras Certificate,
identificadores de certificados gestionados o ambos a dos listas que componen
una sola cadena mediante `with_certificate` y `with_unique_identifier`. Cada
Certificate requiere un Certificate Type Enumeration y un Certificate Value
Byte String (§2.1, Tabla 2), en ese orden. La fila Structure de la tabla sin
número de §10.1.2 exige el orden de la Tabla 340: todos los certificados,
todos los identificadores y la fecha opcional. Se conserva el orden interno
de cada lista incluso al combinar los constructores. El contenido DER permanece opaco. Se conservan los
campos desconocidos y valores futuros. El cliente no construye ni valida
cadenas, selecciona anclas de confianza ni deduce la política del servidor.

```rust,kmipkit-test
use kmipkit_protocol::{UniqueIdentifier, ValidateRequest};

fn main() -> Result<(), kmipkit_protocol::ProtocolError> {
    let request = ValidateRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("leaf-certificate".to_owned()))
        .with_unique_identifier(UniqueIdentifier::TextString("issuer-certificate".to_owned()))
        .with_validity_date(1_700_000_000);
    let payload = request.into_ttlv_payload()?;
    assert_eq!(payload.view().children().len(), 3);
    Ok(())
}
```

Omita `with_validity_date` para conservar Validity Date ausente; el servidor
asume su fecha actual. No se crea una fecha local. El éxito expone
`ValidateResponse::validity_indicator` con el valor del servidor: Valid 1,
Invalid 2, Unknown 3 (§11.61), conservando valores desconocidos. Failure y
Pending no inventan un indicador. Cada respuesta permite inspeccionar su
estructura genérica, incluidos los campos desconocidos, mediante `with_ttlv`.

El texto Log, los certificados, la correlación, Result Message y los payloads
se ocultan en el formato normal. Los callbacks de inspección pueden crear
copias del usuario fuera de la garantía de borrado de KMIPKit. Java y Python
también pueden conservar copias externas en sus runtimes. Consulte el
[contrato de propiedad](../../architecture/public-api.md).

Los ejemplos se ejecutan en `diagnostics_guide_examples.rs` y se compilan
directamente con `python scripts/test_user_guide_examples.py --guide
docs/user-guide/es/diagnostico-e-interop.md`. La
[trazabilidad](../../../specs/028-diagnostics-interop/traceability.md) y las
evidencias de KMIPKIT-0043 enlazan las pruebas derivadas del estándar y las
fachadas. El mapeo Interop está implementado en los cuatro lenguajes; quedan
pendientes los controles finales de la PR y CI. Estos ejemplos no demuestran
perfiles, certificación, fixtures oficiales de OASIS ni interoperabilidad
con dos servidores.


`with_extension(Item)` conserva los campos desconocidos del payload y su
orden. Los campos definidos por las tablas usan los constructores tipados.
Los Message Extension del batch siguen el contrato común de extensiones.
