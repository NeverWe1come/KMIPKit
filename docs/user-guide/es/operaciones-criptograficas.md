# Operaciones Hash, MAC y firma

KMIPKit envía peticiones Hash, MAC, MAC Verify, Sign y Signature Verify a un
servidor KMIP 2.1. No calcula hashes ni MAC, no firma y no verifica datos de
forma local. Cada llamada explícita a `Client` realiza un intercambio; el
cliente no reintenta, divide datos ni continúa una operación multiparte por su
cuenta. Los campos siguen la especificación OASIS KMIP v2.1 §§6.1.24, 6.1.32,
6.1.33, 6.1.55 y 6.1.56, Tablas 235–237, 259–264 y 334–339.

## Construir y enviar peticiones

Elige las claves y los parámetros criptográficos según la política de la
aplicación y las capacidades del servidor. Los wrappers de enumeración
abiertos conservan el valor KMIP exacto. El ejemplo usa SHA-256 (valor 6 de
Hashing Algorithm en §11.21) para Hash; para MAC, Sign y las verificaciones,
indica estructuras elegidas por el llamador u omítelas solo cuando el servidor
pueda obtenerlas del objeto.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatchItemResponse, ClientError};
use kmipkit_protocol::{
    HashRequest, HashingAlgorithm, MacRequest, MacVerifyRequest, OperationData,
    SecretBytes, SignRequest, SignatureVerifyRequest, UniqueIdentifier,
    VerificationResponseContext,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

fn hash_parameters() -> Structure {
    let tag = RawTag::new(0x0042_0038)
        .expect("Hashing Algorithm es un tag KMIP de 24 bits")
        .try_checked()
        .expect("Hashing Algorithm está asignado");
    let item = Item::new(
        tag,
        Value::enumeration(HashingAlgorithm::from_raw(6).raw()),
    )
    .expect("la enumeración es un valor TTLV válido");
    let mut parameters = Structure::new();
    parameters
        .try_push(item)
        .expect("un elemento cabe en Cryptographic Parameters");
    parameters
}

#[allow(dead_code)]
fn solicitar_hash(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = HashRequest::new(hash_parameters()).with_data(
        OperationData::ByteString(SecretBytes::new(b"hash-input-example".to_vec())),
    );
    client.hash(request, limits)
}

#[allow(dead_code)]
fn solicitar_mac(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: String,
    parameters: Option<Structure>,
) -> Result<ClientBatchItemResponse, ClientError> {
    let mut request = MacRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(key_id))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"mac-input-example".to_vec(),
        )));
    if let Some(parameters) = parameters {
        request = request.with_cryptographic_parameters(parameters);
    }
    client.mac(request, limits)
}

#[allow(dead_code)]
fn solicitar_mac_verify(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: String,
    parameters: Option<Structure>,
) -> Result<ClientBatchItemResponse, ClientError> {
    let mut request = MacVerifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(key_id))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"original-data-example".to_vec(),
        )))
        .with_mac_data(SecretBytes::new(b"mac-data-example".to_vec()));
    if let Some(parameters) = parameters {
        request = request.with_cryptographic_parameters(parameters);
    }
    client.mac_verify(request, limits)
}

#[allow(dead_code)]
fn solicitar_firma(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: String,
    parameters: Option<Structure>,
) -> Result<ClientBatchItemResponse, ClientError> {
    let mut request = SignRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(key_id))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"sign-input-example".to_vec(),
        )));
    if let Some(parameters) = parameters {
        request = request.with_cryptographic_parameters(parameters);
    }
    client.sign(request, limits)
}

#[allow(dead_code)]
fn solicitar_verificacion_de_firma(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: String,
    parameters: Option<Structure>,
) -> Result<ClientBatchItemResponse, ClientError> {
    let mut request = SignatureVerifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(key_id))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"original-data-example".to_vec(),
        )))
        .with_signature_data(SecretBytes::new(b"signature-data-example".to_vec()));
    if let Some(parameters) = parameters {
        request = request.with_cryptographic_parameters(parameters);
    }
    client.signature_verify(request, limits)
}

#[allow(dead_code)]
fn iniciar_hash_multiparte(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    client.hash(
        HashRequest::new(hash_parameters()).with_init_indicator(true),
        limits,
    )
}

#[allow(dead_code)]
fn hash_final_part(
    client: &mut Client,
    limits: &CodecLimits,
    correlation_from_prior_response: SecretBytes,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = HashRequest::new(hash_parameters())
        .with_correlation_value(correlation_from_prior_response)
        .with_final_indicator(true);
    client.hash(request, limits)
}

#[allow(dead_code)]
fn longitud_hash(response: &ClientBatchItemResponse) -> Option<usize> {
    response
        .outcome()
        .response()
        .hash()
        .and_then(|value| value.data())
        .map(|bytes| bytes.with_bytes(<[u8]>::len))
}

#[allow(dead_code)]
fn longitud_mac(response: &ClientBatchItemResponse) -> Option<usize> {
    response
        .outcome()
        .response()
        .mac()
        .and_then(|value| value.mac_data())
        .map(|bytes| bytes.with_bytes(<[u8]>::len))
}

#[allow(dead_code)]
fn longitud_firma(response: &ClientBatchItemResponse) -> Option<usize> {
    response
        .outcome()
        .response()
        .sign()
        .and_then(|value| value.signature_data())
        .map(|bytes| bytes.with_bytes(<[u8]>::len))
}

#[allow(dead_code)]
fn verification_result(response: &ClientBatchItemResponse) -> (u32, Option<u32>) {
    let status = response.outcome().result().status().raw();
    let indicator = response
        .outcome()
        .response()
        .mac_verify()
        .and_then(|value| value.validity_indicator())
        .map(|value| value.raw());
    (status, indicator)
}

#[allow(dead_code)]
fn longitud_de_datos_recuperados(
    response: &ClientBatchItemResponse,
) -> Option<usize> {
    response
        .outcome()
        .response()
        .signature_verify()
        .and_then(|value| value.recovered_data())
        .map(|bytes| bytes.with_bytes(<[u8]>::len))
}

#[allow(dead_code)]
fn contexto_para_conversion_directa(
    request: &SignatureVerifyRequest,
) -> VerificationResponseContext {
    request.verification_response_context()
}

fn main() {}
```

El ejemplo multiparte es una petición Hash final elegida por el llamador.
Pasa el valor opaco Correlation Value obtenido del intercambio anterior;
KMIPKit no lo conserva ni lo reutiliza automáticamente. Los campos Data de las
operaciones multiparte siguen ausentes como exigen sus tablas.

## Leer resultados y errores

Comprueba Result Status de KMIP antes de usar los datos específicos de la
operación. Un fallo del servidor sigue siendo un resultado de operación
completado. Un fallo de transporte, validación local o decodificación devuelve
`ClientError` con la mejor evidencia disponible sobre el envío. Operation
Pending permanece explícito para que el llamador decida cómo continuar; nunca
provoca un Poll automático.

MAC Verify y Signature Verify exponen el valor bruto de Validity Indicator.
`Valid`, `Invalid`, `Unknown` y los valores futuros siguen siendo resultados de
operación y no se convierten en decisiones criptográficas locales. Signature
Verify también puede exponer Data recuperados mediante un callback prestado de
`SecretBytes`.

`KMIPKIT-DISC-048` sigue abierto: las Tablas 263 y 338 omiten Validity
Indicator en toda respuesta multiparte, mientras que §§6.1.33 y 6.1.56 lo
exigen en la respuesta multiparte final. La API tipada lo exige para una sola
parte, lo rechaza en partes no finales y acepta ambas formas en la parte final
sin afirmar conformidad del servidor. Usa `verification_response_context()` o
`try_from_response_item_with_context` al convertir una respuesta fuera del
cliente; el conversor conveniente asume una sola parte.

Los builders redactan su salida Debug y guardan los bytes sensibles en
`SecretBytes`, que los pone a cero al liberarse. `with_bytes` presta los
valores recibidos a un callback; las copias del llamador quedan fuera de la
garantía de puesta a cero de KMIPKit. No registres payloads TTLV en bruto ni
copies secretos a cadenas ordinarias.
