//! Public deterministic test inputs and explicitly disposable test-key material.
use crate::driver::{item, structure};
use crate::models::Result;
use kmipkit::protocol::{
    AttributeSet, Certificate, CertificateRequest, KeyBlock, KeyBlockValue, KeyFormatType,
    KeyMaterial, KeyValue, ObjectType, OpaqueObject, OpaqueObjectContext, PgpKey, RegisterRequest,
    SecretData, SymmetricKey,
};
use kmipkit::ttlv::{Item, Value};

pub(crate) const CERTIFICATE: &[u8] = include_bytes!("../tests/fixtures/certificate.der");
pub(crate) const CSR: &[u8] = include_bytes!("../tests/fixtures/certificate-request.der");

pub(crate) fn key_block(secret_data: bool) -> Result<KeyBlock> {
    let material =
        KeyMaterial::byte_string(1, vec![0x41; 32]).map_err(|_| "invalid fixture material")?;
    let key_value = KeyValue::try_new(material, None).map_err(|_| "invalid fixture key value")?;
    let block = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .map_err(|_| "invalid fixture key format")?
        .with_key_value(KeyBlockValue::Plaintext(key_value));
    Ok(if secret_data {
        block
    } else {
        block.with_cryptographic_properties(Some(3), Some(256))
    })
}

pub(crate) fn object(name: &str) -> Result<(ObjectType, Item, AttributeSet)> {
    let attrs = || {
        AttributeSet::try_new(vec![item(0x0042_002c, Value::integer(0x0c))?])
            .map_err(|_| "invalid fixture usage")
    };
    match name {
        "Certificate" => {
            let object = Certificate::try_new(1, CERTIFICATE.to_vec())
                .map_err(|_| "invalid certificate fixture")?
                .into_item();
            let attrs = AttributeSet::try_new(vec![
                item(0x0042_002c, Value::integer(0x20))?,
                item(0x0042_0028, Value::enumeration(6))?,
                item(0x0042_002a, Value::integer(256))?,
                item(
                    0x0042_00ad,
                    Value::integer(
                        i32::try_from(CERTIFICATE.len() * 8).map_err(|_| "fixture size")?,
                    ),
                )?,
            ])
            .map_err(|_| "invalid certificate fixture attributes")?;
            Ok((ObjectType::from_raw(1), object, attrs))
        }
        "Certificate Request" => Ok((
            ObjectType::from_raw(10),
            CertificateRequest::try_new(1, CSR.to_vec())
                .map_err(|_| "invalid CSR fixture")?
                .into_item(),
            crate::adapters::attributes("kmipkit-csr-fixture", 6, 256)?,
        )),
        "Opaque Object" => Ok((
            ObjectType::from_raw(8),
            OpaqueObject::try_new(1, b"dedicated fixture".to_vec())
                .map_err(|_| "invalid opaque fixture")?
                .into_item(),
            AttributeSet::new(),
        )),
        "PGP Key" => Ok((
            ObjectType::from_raw(9),
            PgpKey::try_new(4, key_block(false)?)
                .map_err(|_| "invalid PGP fixture")?
                .into_item(),
            crate::adapters::attributes("kmipkit-pgp-fixture", 3, 256)?,
        )),
        "Secret Data" => Ok((
            ObjectType::from_raw(7),
            SecretData::try_new(1, key_block(true)?)
                .map_err(|_| "invalid secret-data fixture")?
                .into_item(),
            attrs()?,
        )),
        _ => Ok((
            ObjectType::from_raw(2),
            SymmetricKey::try_new(key_block(false)?)
                .map_err(|_| "invalid key fixture")?
                .into_item(),
            crate::adapters::attributes("kmipkit-key-fixture", 3, 256)?,
        )),
    }
}

pub(crate) fn register(name: &str) -> Result<kmipkit::ClientRequest> {
    let (kind, object, attributes) = object(name)?;
    let request = RegisterRequest::new(kind, attributes, object);
    Ok(kmipkit::ClientRequest::Register(
        if name == "Opaque Object" {
            request.with_opaque_object_context(OpaqueObjectContext::Generic)
        } else {
            request
        },
    ))
}

pub(crate) fn certificate_structure() -> Result<kmipkit::ttlv::Structure> {
    structure(
        0x0042_0013,
        vec![
            item(0x0042_001d, Value::enumeration(1))?,
            item(0x0042_001e, Value::byte_string(CERTIFICATE.to_vec()))?,
        ],
    )
}
pub(crate) fn csr_structure() -> Result<kmipkit::ttlv::Structure> {
    structure(
        0x0042_0018,
        vec![
            item(0x0042_0019, Value::enumeration(1))?,
            item(0x0042_0140, Value::byte_string(CSR.to_vec()))?,
        ],
    )
}
