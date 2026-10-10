# Cifrar y descifrar con un cliente tipado

Las peticiones tipadas Encrypt y Decrypt envían al servidor KMIP los valores
elegidos por el llamador; KMIPKit no cifra localmente ni elige parámetros
criptográficos. Estos ejemplos corresponden a OASIS KMIP Specification v2.1
§6.1.17 Tablas 214–216 (Encrypt) y §6.1.11 Tablas 196–198 (Decrypt). La
colocación de campos multipart sigue §6.1 y §§7.3, 7.4, 7.8, 7.14 y 7.17.

El código muestra la construcción de peticiones con valores del llamador,
operaciones de una parte y multipart, el tratamiento de resultados completados
y Pending, y un Recover explícito antes de Encrypt cuando el llamador ya sabe
que el objeto está archivado. Cada llamada a `execute` hace un único
intercambio. Un resultado Pending detiene este flujo para que la aplicación
continúe con su flujo asíncrono explícito; el cliente no ejecuta Poll ni
reintenta automáticamente.

```rust,kmipkit-test
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientBatchResponse, ClientError,
    ClientRequest,
};
use kmipkit_protocol::{
    DecryptRequest, EncryptRequest, OperationData, RecoverRequest, SecretBytes, UniqueIdentifier,
};
use kmipkit_ttlv::{Structure, codec::CodecLimits};

fn observe_encrypt(response: &ClientBatchResponse) {
    match response.get(0).map(|item| item.outcome()) {
        Some(ClientBatchOutcome::Pending(pending)) => {
            let _opaque_value_length = pending.asynchronous_correlation_value().len();
        }
        Some(ClientBatchOutcome::Encrypt(encrypted))
            if encrypted.result().status().known_name() == Some("Success") =>
        {
            let _ciphertext_length = encrypted
                .data()
                .map(|value| value.with_bytes(|bytes| bytes.len()));
            let _multipart_value_length = encrypted
                .correlation_value()
                .map(|value| value.with_bytes(|bytes| bytes.len()));
        }
        Some(ClientBatchOutcome::Encrypt(_)) => {
            // Preserve and handle the typed server result in application code.
        }
        _ => {}
    }
}

fn observe_decrypt(response: &ClientBatchResponse) {
    match response.get(0).map(|item| item.outcome()) {
        Some(ClientBatchOutcome::Pending(pending)) => {
            let _opaque_value_length = pending.asynchronous_correlation_value().len();
        }
        Some(ClientBatchOutcome::Decrypt(decrypted))
            if decrypted.result().status().known_name() == Some("Success") =>
        {
            let _plaintext_length = decrypted
                .data()
                .map(|value| value.with_bytes(|bytes| bytes.len()));
            let _multipart_value_length = decrypted
                .correlation_value()
                .map(|value| value.with_bytes(|bytes| bytes.len()));
        }
        Some(ClientBatchOutcome::Decrypt(_)) => {
            // Preserve and handle the typed server result in application code.
        }
        _ => {}
    }
}

pub fn encrypt_one_part(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: UniqueIdentifier,
    parameters_selected_by_caller: Option<Structure>,
    iv_selected_by_caller: Option<SecretBytes>,
    plaintext: SecretBytes,
) -> Result<ClientBatchResponse, ClientError> {
    let mut request = EncryptRequest::new(Some(key_id), Some(OperationData::ByteString(plaintext)));
    if let Some(parameters) = parameters_selected_by_caller {
        request = request.with_cryptographic_parameters(parameters);
    }
    if let Some(iv) = iv_selected_by_caller {
        request = request.with_iv_counter_nonce(iv);
    }

    let response = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Encrypt(request)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    observe_encrypt(&response);
    Ok(response)
}

pub fn decrypt_one_part(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: UniqueIdentifier,
    parameters_selected_by_caller: Option<Structure>,
    iv_selected_by_caller: Option<SecretBytes>,
    aad_selected_by_caller: Option<SecretBytes>,
    tag_selected_by_caller: Option<SecretBytes>,
    ciphertext: SecretBytes,
) -> Result<ClientBatchResponse, ClientError> {
    let mut request =
        DecryptRequest::new(Some(key_id), Some(OperationData::ByteString(ciphertext)));
    if let Some(parameters) = parameters_selected_by_caller {
        request = request.with_cryptographic_parameters(parameters);
    }
    if let Some(iv) = iv_selected_by_caller {
        request = request.with_iv_counter_nonce(iv);
    }
    if let Some(aad) = aad_selected_by_caller {
        request = request.with_authenticated_encryption_additional_data(aad);
    }
    if let Some(tag) = tag_selected_by_caller {
        request = request.with_authenticated_encryption_tag(tag);
    }

    let response = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Decrypt(request)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    observe_decrypt(&response);
    Ok(response)
}

pub fn encrypt_in_two_parts(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: UniqueIdentifier,
    initial_parameters: Option<Structure>,
    final_parameters: Option<Structure>,
    initial_plaintext: SecretBytes,
    final_plaintext: SecretBytes,
    initial_aad: SecretBytes,
) -> Result<Vec<ClientBatchResponse>, ClientError> {
    let mut initial = EncryptRequest::new(
        Some(key_id.clone()),
        Some(OperationData::ByteString(initial_plaintext)),
    )
    .with_init_indicator(true)
    .with_authenticated_encryption_additional_data(initial_aad);
    if let Some(parameters) = initial_parameters {
        initial = initial.with_cryptographic_parameters(parameters);
    }

    let first = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Encrypt(initial)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    let correlation = match first.get(0).map(|item| item.outcome()) {
        Some(ClientBatchOutcome::Pending(pending)) => {
            let _opaque_value_length = pending.asynchronous_correlation_value().len();
            return Ok(vec![first]);
        }
        Some(ClientBatchOutcome::Encrypt(encrypted))
            if encrypted.result().status().known_name() == Some("Success") =>
        {
            let Some(value) = encrypted.correlation_value() else {
                return Ok(vec![first]);
            };
            value.with_bytes(|bytes| SecretBytes::new(bytes.to_vec()))
        }
        _ => return Ok(vec![first]),
    };

    let mut final_part = EncryptRequest::new(
        Some(key_id),
        Some(OperationData::ByteString(final_plaintext)),
    )
    .with_correlation_value(correlation)
    .with_final_indicator(true);
    if let Some(parameters) = final_parameters {
        final_part = final_part.with_cryptographic_parameters(parameters);
    }
    let last = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Encrypt(final_part)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    observe_encrypt(&last);
    Ok(vec![first, last])
}

pub fn decrypt_in_two_parts(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: UniqueIdentifier,
    initial_parameters: Option<Structure>,
    final_parameters: Option<Structure>,
    initial_iv: Option<SecretBytes>,
    initial_ciphertext: SecretBytes,
    final_ciphertext: SecretBytes,
    initial_aad: SecretBytes,
    initial_tag: SecretBytes,
) -> Result<Vec<ClientBatchResponse>, ClientError> {
    let mut initial = DecryptRequest::new(
        Some(key_id.clone()),
        Some(OperationData::ByteString(initial_ciphertext)),
    )
    .with_init_indicator(true)
    .with_authenticated_encryption_additional_data(initial_aad)
    .with_authenticated_encryption_tag(initial_tag);
    if let Some(parameters) = initial_parameters {
        initial = initial.with_cryptographic_parameters(parameters);
    }
    if let Some(iv) = initial_iv {
        initial = initial.with_iv_counter_nonce(iv);
    }

    let first = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Decrypt(initial)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    let correlation = match first.get(0).map(|item| item.outcome()) {
        Some(ClientBatchOutcome::Pending(pending)) => {
            let _opaque_value_length = pending.asynchronous_correlation_value().len();
            return Ok(vec![first]);
        }
        Some(ClientBatchOutcome::Decrypt(decrypted))
            if decrypted.result().status().known_name() == Some("Success") =>
        {
            let Some(value) = decrypted.correlation_value() else {
                return Ok(vec![first]);
            };
            value.with_bytes(|bytes| SecretBytes::new(bytes.to_vec()))
        }
        _ => return Ok(vec![first]),
    };

    let mut final_part = DecryptRequest::new(
        Some(key_id),
        Some(OperationData::ByteString(final_ciphertext)),
    )
    .with_correlation_value(correlation)
    .with_final_indicator(true);
    if let Some(parameters) = final_parameters {
        final_part = final_part.with_cryptographic_parameters(parameters);
    }
    let last = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Decrypt(final_part)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    observe_decrypt(&last);
    Ok(vec![first, last])
}

pub fn recover_then_encrypt(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: UniqueIdentifier,
    parameters_selected_by_caller: Option<Structure>,
    plaintext: SecretBytes,
) -> Result<Vec<ClientBatchResponse>, ClientError> {
    let recovered = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Recover(
            RecoverRequest::new(Some(key_id.clone())),
        )))
        .with_asynchronous_indicator(1),
        limits,
    )?;
    match recovered.get(0).map(|item| item.outcome()) {
        Some(ClientBatchOutcome::Pending(pending)) => {
            let _opaque_value_length = pending.asynchronous_correlation_value().len();
            return Ok(vec![recovered]);
        }
        Some(ClientBatchOutcome::Recover(response))
            if response.result().status().known_name() == Some("Success") =>
        {
            let _recovered_identifier = response.unique_identifier();
        }
        _ => return Ok(vec![recovered]),
    }

    let encrypted = encrypt_one_part(
        client,
        limits,
        key_id,
        parameters_selected_by_caller,
        None,
        plaintext,
    )?;
    Ok(vec![recovered, encrypted])
}

fn main() {}
```

Las funciones de una parte devuelven la respuesta tipada completa para que el
llamador pueda inspeccionar Success, Failure o Pending. Las funciones
multipart devuelven las respuestas recibidas hasta el momento; si una
respuesta inicial es Pending o no es exitosa, se detienen antes de enviar la
siguiente parte. Tras un resultado exitoso completado, solo trasladan el valor
exacto Correlation Value del servidor a la siguiente petición construida por
el llamador; la copia se guarda en `SecretBytes`. En este ejemplo, AAD y el tag
Decrypt aparecen solo en la petición Decrypt inicial. No se mueve ningún
campo automáticamente. La aplicación puede inspeccionar y conservar cada
resultado KMIP tipado según su política; que la llamada de Rust tenga éxito no
significa que la operación KMIP haya tenido éxito.

La secuencia Recover también queda bajo el control del llamador: envía un
Recover explícito, continúa a Encrypt solo tras un resultado Success completado
y se detiene ante Pending u otro resultado, devolviendo la respuesta para que
el llamador la gestione. Esta es la precondición para un
objeto que el llamador sabe que está archivado, indicada por KMIP v2.1 §6.1;
KMIPKit no deduce el estado del objeto ni ejecuta Recover automáticamente.
