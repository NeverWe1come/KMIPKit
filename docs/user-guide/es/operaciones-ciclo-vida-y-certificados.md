# Operaciones de ciclo de vida y certificados

Esta guía cubre las nueve operaciones de cliente de KMIP 2.1 incluidas en la
familia de ciclo de vida y certificados de la versión 1.0: Certify, Check,
Obtain Lease, Re-certify, Re-key, Re-key Key Pair, Re-Provision, Register y
Revoke. Cada ejemplo recibe un `Client` ya configurado; la configuración de la
conexión y TLS se describe en la [guía de transportes de producción](transportes-produccion.md).

KMIPKit envía cada petición una vez y devuelve el resultado del servidor. No
crea certificados ni claves localmente, no simula el estado remoto, no
reintenta, no ejecuta Poll ni realiza llamadas Get posteriores. Activate,
Archive, Destroy y Recover se explican en la [guía de ciclo de vida de objetos](operaciones-ciclo-vida.md).

## Certify

Certify reenvía una petición de certificado al servidor. Proporciona el
identificador de la clave pública, el tipo de petición y los bytes opacos; KMIPKit
no analiza ni genera la petición de certificado.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{CertifyRequest, SecretBytes, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn certify(
    client: &mut Client,
    limits: &CodecLimits,
    public_key_id: UniqueIdentifier,
    request_bytes: SecretBytes,
) -> Result<(), kmipkit_client::ClientError> {
    let request = CertifyRequest::new()
        .with_unique_identifier(public_key_id)
        .with_certificate_request_type(1)
        .with_certificate_request_value(request_bytes);
    let item = client.certify(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Certify);
    let response = outcome.response().certify().expect("respuesta Certify tipada");
    let _status = response.result().status().raw();
    let _certificate_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = certify; }
```

## Check

Check pregunta al servidor si permite el uso solicitado. Los valores de
política pueden acompañar a un resultado fallido; consúltalos con independencia
del estado y del identificador.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{CheckRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn check(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = CheckRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("object-id".to_owned()))
        .with_usage_limits_count(1)
        .with_cryptographic_usage_mask(2)
        .with_lease_time(60);
    let item = client.check(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Check);
    let response = outcome.response().check().expect("respuesta Check tipada");
    let _status = response.result().status().raw();
    let _usage_count = response.usage_limits_count();
    let _usage_mask = response.cryptographic_usage_mask();
    let _lease_time = response.lease_time();
    Ok(())
}

fn main() { let _example = check; }
```

## Obtain Lease

Obtain Lease devuelve el intervalo concedido por el servidor y la fecha del
último cambio. KMIPKit no inicia un temporizador ni controla la expiración del
alquiler.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{ObtainLeaseRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn obtain_lease(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = ObtainLeaseRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("object-id".to_owned()));
    let item = client.obtain_lease(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::ObtainLease);
    let response = outcome.response().obtain_lease().expect("respuesta Obtain Lease tipada");
    let _status = response.result().status().raw();
    let _lease_seconds = response.lease_time();
    let _last_change = response.last_change_date();
    Ok(())
}

fn main() { let _example = obtain_lease; }
```

## Re-certify

Re-certify solicita al servidor la renovación de un solo certificado existente.
Si no se proporciona una nueva petición ni su tipo, el servidor utiliza el
tipo del certificado actual.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RecertifyRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn recertify(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = RecertifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("certificate-id".to_owned()))
        .with_offset(86_400);
    let item = client.recertify(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Recertify);
    let response = outcome.response().recertify().expect("respuesta Re-certify tipada");
    let _status = response.result().status().raw();
    let _certificate_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = recertify; }
```

## Re-key

Re-key solicita al servidor una clave simétrica de sustitución. El servidor
genera el material y mantiene los enlaces de sustitución; KMIPKit no realiza
ninguna de esas acciones localmente.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RekeyRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn rekey(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = RekeyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("key-id".to_owned()))
        .with_offset(86_400);
    let item = client.rekey(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Rekey);
    let response = outcome.response().rekey().expect("respuesta Re-key tipada");
    let _status = response.result().status().raw();
    let _replacement_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = rekey; }
```

## Re-key Key Pair

Re-key Key Pair solicita un par de sustitución generado por el servidor. Los
atributos públicos y privados pueden proporcionarse por separado, y la
respuesta expone ambos identificadores.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RekeyKeyPairRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn rekey_key_pair(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = RekeyKeyPairRequest::new()
        .with_private_key_unique_identifier(UniqueIdentifier::TextString("private-key-id".to_owned()))
        .with_offset(86_400);
    let item = client.rekey_key_pair(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::RekeyKeyPair);
    let response = outcome.response().rekey_key_pair().expect("respuesta Re-key Key Pair tipada");
    let _status = response.result().status().raw();
    let _private_id = response.private_key_unique_identifier();
    let _public_id = response.public_key_unique_identifier();
    Ok(())
}

fn main() { let _example = rekey_key_pair; }
```

## Re-Provision

Re-Provision puede enviar una petición de certificado, enviar un certificado o
no incluir ninguno para que el servidor cree credenciales. Este ejemplo usa la
variante sin parámetros; cualquier Get posterior es una acción separada del
llamador.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::ReProvisionRequest;
use kmipkit_ttlv::codec::CodecLimits;

fn reprovision(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let item = client.reprovision(ReProvisionRequest::new(), limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::ReProvision);
    let response = outcome.response().reprovision().expect("respuesta Re-Provision tipada");
    let _status = response.result().status().raw();
    let _credential_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = reprovision; }
```

## Register

Register envía el objeto y los atributos suministrados por el llamador. Este
debe proporcionar un objeto de certificado válido y sus atributos requeridos,
incluido Certificate Length.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{AttributeSet, ObjectType, RegisterRequest};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::Item;

fn register_certificate(
    client: &mut Client,
    limits: &CodecLimits,
    attributes: AttributeSet,
    certificate: Item,
) -> Result<(), kmipkit_client::ClientError> {
    let request = RegisterRequest::new(ObjectType::from_raw(1), attributes, certificate);
    let item = client.register(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Register);
    let response = outcome.response().register().expect("respuesta Register tipada");
    let _status = response.result().status().raw();
    let _registered_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = register_certificate; }
```

## Revoke

Revoke envía un motivo explícito. El mensaje opcional se reserva para el
registro de auditoría; la fecha del compromiso solo se permite para compromiso
de clave o de CA.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RevocationReason, RevokeRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn revoke(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = RevokeRequest::new(RevocationReason::KEY_COMPROMISE)
        .with_unique_identifier(UniqueIdentifier::TextString("certificate-id".to_owned()))
        .with_revocation_message("retirada aprobada".to_owned())
        .with_compromise_occurrence_date(1_700_000_000);
    let item = client.revoke(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Revoke);
    let response = outcome.response().revoke().expect("respuesta Revoke tipada");
    let _status = response.result().status().raw();
    let _revoked_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = revoke; }
```

Los ejemplos marcados se compilan con `python scripts/test_user_guide_examples.py`.
Solo demuestran la forma de la API: no afirman que un servidor KMIP externo
acepte la petición ni conceda la operación.
