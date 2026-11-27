//! Exercise the Get selectors and verify the corresponding typed Key Block.

use super::{Attempt, Failure};
use crate::driver::{Observed, item};
use crate::key_block::{self, KeyBlockShape};
use kmipkit::ClientRequest;
use kmipkit::protocol::{
    ActivateRequest, AttributeSet, CreateKeyPairRequest, CreateRequest, GetRequest, KeyFormatType,
    ObjectType, UniqueIdentifier,
};
use kmipkit::ttlv::Value;

impl Attempt<'_> {
    fn extractable_key(&mut self, usage_mask: i32) -> Result<UniqueIdentifier, Failure> {
        let attributes = AttributeSet::try_new(vec![
            item(0x0042_0028, Value::enumeration(3))?,
            item(0x0042_002a, Value::integer(256))?,
            item(0x0042_002c, Value::integer(usage_mask))?,
            item(0x0042_0122, Value::boolean(true))?,
        ])
        .map_err(|_| Failure::Assertion("invalid extractable AES attributes"))?;
        let response = self.setup(ClientRequest::Create(CreateRequest::new(
            ObjectType::from_raw(2),
            attributes,
        )))?;
        self.own(&response);
        response.identifier.ok_or(Failure::Prerequisite(
            "Create omitted the owned AES key identifier",
        ))
    }

    pub(super) fn get_asymmetric_key(&mut self, format: u32) -> Result<Observed, Failure> {
        let [common, private, public] = key_block::pair_attributes(format)?;
        let attributes = |items| {
            AttributeSet::try_new(items)
                .map_err(|_| Failure::Assertion("invalid key pair fixture attributes"))
        };
        let request = CreateKeyPairRequest::new()
            .with_common_attributes(attributes(common)?)
            .with_private_key_attributes(attributes(private)?)
            .with_public_key_attributes(attributes(public)?);
        let created = self.setup(ClientRequest::CreateKeyPair(request))?;
        self.own(&created);
        let [private_key, public_key] = created.identifiers.as_slice() else {
            return Err(Failure::Prerequisite(
                "Create Key Pair omitted an owned identifier",
            ));
        };
        if private_key == public_key {
            return Err(Failure::Prerequisite(
                "Create Key Pair returned duplicate identifiers",
            ));
        }
        let identifier = if matches!(format, 11 | 21) {
            public_key
        } else {
            private_key
        };
        let request = GetRequest::new()
            .with_unique_identifier(identifier.clone())
            .with_key_format_type(KeyFormatType::from_raw(format));
        let request = ClientRequest::Get(request);
        let request = if self.connection.api_level == "fluent" {
            crate::fluent::build(self.client, request).map_err(Failure::Assertion)?
        } else {
            request
        };
        let response = self.exchange(request)?;
        if response.status == 0
            && (response.identifier.as_ref() != Some(identifier)
                || response.key_block_shape != Some(KeyBlockShape::Transparent(format)))
        {
            return Err(Failure::Assertion(
                "Get did not return the selected transparent key material",
            ));
        }
        Ok(response)
    }

    pub(super) fn get_key_block(&mut self, wrapped: bool) -> Result<Observed, Failure> {
        let identifier = self.extractable_key(12)?;
        let mut request = GetRequest::new().with_unique_identifier(identifier.clone());
        let expected = if wrapped {
            let wrapping_key = self.extractable_key(48)?;
            let active = self.setup(ClientRequest::Activate(ActivateRequest::new(Some(
                wrapping_key.clone(),
            ))))?;
            if active.identifier.as_ref() != Some(&wrapping_key) {
                return Err(Failure::Prerequisite(
                    "Activate did not confirm the wrapping key",
                ));
            }
            let UniqueIdentifier::TextString(text) = wrapping_key else {
                return Err(Failure::Prerequisite(
                    "wrapping key has no TTLV Text String identifier",
                ));
            };
            request =
                request.with_key_wrapping_specification(key_block::wrapping_specification(&text)?);
            KeyBlockShape::Wrapped(text)
        } else {
            request = request.with_key_format_type(KeyFormatType::from_raw(7));
            KeyBlockShape::Transparent(7)
        };
        let request = ClientRequest::Get(request);
        let request = if self.connection.api_level == "fluent" {
            crate::fluent::build(self.client, request).map_err(Failure::Assertion)?
        } else {
            request
        };
        let response = self.exchange(request)?;
        if response.status == 0
            && (response.key_block_shape.as_ref() != Some(&expected)
                || response.identifier.as_ref() != Some(&identifier))
        {
            return Err(Failure::Assertion(
                "Get did not return the expected Key Block",
            ));
        }
        Ok(response)
    }
}
