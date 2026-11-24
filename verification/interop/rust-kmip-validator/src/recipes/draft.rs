use super::{Attempt, Failure};
use crate::driver::Observed;
use crate::{adapters, models::Case};
use kmipkit::ClientRequest;
use kmipkit::protocol::{
    ActivateRequest, AttributeSet, CreateCredentialObject, CreateCredentialRequest,
    CreateGroupRequest, CreateKeyPairRequest, CreateUserRequest, DeactivateRequest,
    DecapsulateRequest, DestroyRequest, EncapsulateRequest, ObliterateRequest,
    PasswordCredentialObject, PasswordCredentialObjectFields, RevocationReason, RevokeRequest,
    SecretText,
};

pub(super) fn supports(operation: &str) -> bool {
    [
        "Create Group",
        "Create User",
        "Create Credential",
        "Deactivate",
        "Obliterate",
        "Encapsulate",
        "Decapsulate",
    ]
    .contains(&operation)
}

impl Attempt<'_> {
    pub(super) fn run_draft(&mut self, case: &Case) -> Result<Observed, Failure> {
        let fluent = self.connection.api_level == "fluent";
        match case.operation.as_str() {
            "Create Group" | "Create User" | "Create Credential" => {
                self.create_draft_object(&case.operation, fluent)
            }
            "Encapsulate" | "Decapsulate" => self.kem_draft(&case.operation, fluent),
            "Deactivate" | "Obliterate" => self.lifecycle_draft(&case.operation, fluent),
            _ => Err(Failure::Prerequisite("draft operation recipe unavailable")),
        }
    }

    fn create_draft_object(&mut self, operation: &str, fluent: bool) -> Result<Observed, Failure> {
        let attributes = AttributeSet::try_new(vec![adapters::fixture_label(&self.name, "3.0")?])
            .map_err(|_| Failure::Assertion("invalid draft label fixture"))?;
        let request = match operation {
            "Create Group" if fluent => self.client.request().create_group(attributes).build(),
            "Create Group" => ClientRequest::create_group(CreateGroupRequest::new(attributes)),
            "Create User" if fluent => self.client.request().create_user(attributes).build(),
            "Create User" => ClientRequest::create_user(CreateUserRequest::new(attributes)),
            _ => {
                let password = PasswordCredentialObject::try_new(PasswordCredentialObjectFields {
                    password: Some(SecretText::new(uuid::Uuid::new_v4().to_string())),
                    ..PasswordCredentialObjectFields::default()
                })
                .map_err(|_| Failure::Assertion("invalid disposable credential fixture"))?;
                let object = CreateCredentialObject::PasswordCredential(password);
                if fluent {
                    self.client
                        .request()
                        .create_credential(attributes, object)
                        .build()
                } else {
                    ClientRequest::create_credential(CreateCredentialRequest::new(
                        attributes, object,
                    ))
                }
            }
        };
        let result = self.exchange(request)?;
        if result.status == 0 {
            self.own(&result);
            if result.identifiers.is_empty() {
                return Err(Failure::Assertion(
                    "draft creation returned no owned object identifier",
                ));
            }
        }
        Ok(result)
    }

    fn lifecycle_draft(&mut self, operation: &str, fluent: bool) -> Result<Observed, Failure> {
        let (id, _) = self.key("AES", 256, true)?;
        if operation == "Deactivate" {
            let request = if fluent {
                self.client.request().deactivate(id.clone()).build()
            } else {
                ClientRequest::deactivate(DeactivateRequest::new(id.clone()))
            };
            let result = self.exchange(request)?;
            if result.status == 0 && result.identifier.as_ref() != Some(&id) {
                return Err(Failure::Assertion(
                    "Deactivate did not return the owned identifier",
                ));
            }
            return Ok(result);
        }
        self.setup(ClientRequest::Revoke(
            RevokeRequest::new(RevocationReason::CESSATION_OF_OPERATION)
                .with_unique_identifier(id.clone()),
        ))?;
        self.setup(ClientRequest::Destroy(DestroyRequest::new(Some(
            id.clone(),
        ))))?;
        let request = if fluent {
            self.client.request().obliterate(id.clone()).build()
        } else {
            ClientRequest::obliterate(ObliterateRequest::new(id.clone()))
        };
        let result = self.exchange(request)?;
        if result.status == 0 {
            self.owned.retain(|candidate| candidate != &id);
        }
        Ok(result)
    }

    fn kem_draft(&mut self, operation: &str, fluent: bool) -> Result<Observed, Failure> {
        let attributes = adapters::attributes(&self.name, 0x3a, 768, "3.0")?;
        let pair = self.setup(ClientRequest::CreateKeyPair(
            CreateKeyPairRequest::new().with_common_attributes(attributes),
        ))?;
        self.own(&pair);
        let [private, public] = pair.identifiers.as_slice() else {
            return Err(Failure::Prerequisite(
                "ML-KEM setup must return an owned key pair",
            ));
        };
        for id in [private, public] {
            self.setup(ClientRequest::Activate(ActivateRequest::new(Some(
                id.clone(),
            ))))?;
        }
        if operation == "Encapsulate" {
            let request = if fluent {
                self.client.request().encapsulate(public.clone()).build()
            } else {
                ClientRequest::encapsulate(EncapsulateRequest::new(public.clone()))
            };
            return self.exchange(request);
        }
        let ciphertext = self
            .setup(ClientRequest::encapsulate(EncapsulateRequest::new(
                public.clone(),
            )))?
            .bytes
            .ok_or(Failure::Prerequisite(
                "Encapsulate setup returned no ciphertext",
            ))?;
        let request = ciphertext.with_bytes(|value| {
            if fluent {
                self.client
                    .request()
                    .decapsulate(private.clone(), value.to_vec())
                    .build()
            } else {
                ClientRequest::decapsulate(DecapsulateRequest::new(private.clone(), value.to_vec()))
            }
        });
        self.exchange(request)
    }
}
