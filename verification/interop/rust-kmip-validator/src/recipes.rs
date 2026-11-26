//! Per-case prerequisite, target and teardown recipes using installed public APIs.
use crate::models::{Case, Connection};
use crate::{adapters, driver, fixtures};
use driver::Observed;
use kmipkit::protocol::{
    ActivateRequest, ArchiveRequest, CreateKeyPairRequest, CreateRequest, CreateSplitKeyRequest,
    CryptographicParameters, CryptographicParametersBuilder, DecryptRequest, DestroyRequest,
    EncryptRequest, JoinSplitKeyRequest, LoginRequest, LogoutRequest, ObjectType, OperationData,
    RevocationReason, RevokeRequest, SecretBytes, SignRequest, SplitKeyMethod, SplitKeyReference,
    UniqueIdentifier,
};
use kmipkit::{Client, ClientRequest};

mod crypto;
mod draft;
mod key_block;
pub(crate) use crypto::OPERATIONS as CRYPTO_OPERATIONS;

pub(crate) enum Failure {
    LocalRejection(&'static str),
    Prerequisite(&'static str),
    Library(kmipkit::ClientError),
    Assertion(&'static str),
}
impl From<kmipkit::ClientError> for Failure {
    fn from(error: kmipkit::ClientError) -> Self {
        Self::Library(error)
    }
}
impl From<&'static str> for Failure {
    fn from(error: &'static str) -> Self {
        Self::Prerequisite(error)
    }
}

struct Attempt<'a> {
    client: &'a mut Client,
    connection: &'a Connection,
    owned: Vec<UniqueIdentifier>,
    pending: Vec<SecretBytes>,
    name: String,
}
impl Attempt<'_> {
    fn exchange(&mut self, request: ClientRequest) -> std::result::Result<Observed, Failure> {
        Ok(driver::execute(self.client, self.connection, request)?)
    }
    fn setup(&mut self, request: ClientRequest) -> std::result::Result<Observed, Failure> {
        let response = self.exchange(request)?;
        if response.status != 0 {
            return Err(Failure::Prerequisite(
                "server rejected a required setup operation; target not sent",
            ));
        }
        Ok(response)
    }
    fn own(&mut self, response: &Observed) {
        for id in &response.identifiers {
            if !self.owned.contains(id) {
                self.owned.push(id.clone());
            }
        }
    }
    fn key(
        &mut self,
        algorithm: &str,
        bits: i32,
        active: bool,
    ) -> std::result::Result<(UniqueIdentifier, UniqueIdentifier), Failure> {
        let raw = match algorithm {
            "RSA" => 4,
            "ECDSA" => 6,
            "HMAC_SHA256" => 9,
            _ => 3,
        };
        let attrs = adapters::attributes(&self.name, raw, bits, &self.connection.protocol_version)?;
        let request = if [4, 6].contains(&raw) {
            ClientRequest::CreateKeyPair(CreateKeyPairRequest::new().with_common_attributes(attrs))
        } else {
            ClientRequest::Create(CreateRequest::new(ObjectType::from_raw(2), attrs))
        };
        let response = self.setup(request)?;
        self.own(&response);
        let private = response
            .identifiers
            .first()
            .cloned()
            .ok_or(Failure::Prerequisite(
                "server did not supply an owned identifier",
            ))?;
        let public = response
            .identifiers
            .get(1)
            .cloned()
            .unwrap_or_else(|| private.clone());
        if active {
            self.setup(ClientRequest::Activate(ActivateRequest::new(Some(
                private.clone(),
            ))))?;
            if public != private {
                self.setup(ClientRequest::Activate(ActivateRequest::new(Some(
                    public.clone(),
                ))))?;
            }
        }
        Ok((private, public))
    }
    fn cleanup(&mut self) -> bool {
        let mut complete = true;
        for correlation in &self.pending {
            match correlation.with_bytes(|value| {
                driver::asynchronous(self.client, self.connection, "Cancel", value)
            }) {
                Ok(response) if response.status == 0 || response.reason == Some(1) => {}
                _ => complete = false,
            }
        }
        for id in &self.owned {
            // Separate, explicit teardown operations. A failed operation is never repeated.
            let _ = driver::execute(
                self.client,
                self.connection,
                ClientRequest::Revoke(
                    RevokeRequest::new(RevocationReason::CESSATION_OF_OPERATION)
                        .with_unique_identifier(id.clone()),
                ),
            );
            match driver::execute(
                self.client,
                self.connection,
                ClientRequest::Destroy(DestroyRequest::new(Some(id.clone()))),
            ) {
                Ok(response) if response.status == 0 || response.reason == Some(1) => {}
                _ => complete = false,
            }
        }
        complete
    }
    #[allow(clippy::too_many_lines)] // Prerequisite ordering remains adjacent to the corresponding target.
    fn run(&mut self, case: &Case) -> std::result::Result<Observed, Failure> {
        match case.requires["workflow_type"].as_str() {
            Some("async_poll") => return self.asynchronous("Poll"),
            Some("batch_id_placeholder") => return self.batch(case),
            Some("multipart_streaming") => return self.multipart(case),
            _ => {}
        }
        if case.operation == "Get" && case.requires["key_block_shape"].is_string() {
            return self.get_key_block(case.requires["key_block_shape"] == "wrapped");
        }
        if draft::supports(&case.operation) {
            return self.run_draft(case);
        }
        match case.operation.as_str() {
            "Poll" | "Process" | "Cancel" => return self.asynchronous(&case.operation),
            "Query Asynchronous Requests" => {
                return Ok(driver::asynchronous(
                    self.client,
                    self.connection,
                    &case.operation,
                    &[],
                )?);
            }
            "Logout" => {
                let response = self.setup(ClientRequest::Login(
                    LoginRequest::new().with_request_count(2),
                ))?;
                let ticket = response
                    .ticket
                    .ok_or(Failure::Prerequisite("Login did not return a ticket"))?;
                return self.exchange(ClientRequest::Logout(LogoutRequest::new(ticket)));
            }
            "Join Split Key" => return self.join(),
            "Hash" => return self.hash(),
            operation if CRYPTO_OPERATIONS.contains(&operation) => {
                return self.crypto(case);
            }
            "Login" | "Delegated Login" => {
                let request =
                    adapters::request(case, None, &self.name, &self.connection.protocol_version)?;
                let mut response = self.exchange(request)?;
                if let Some(ticket) = response.ticket.take() {
                    self.setup(ClientRequest::Logout(LogoutRequest::new(ticket)))?;
                }
                return Ok(response);
            }
            _ => {}
        }
        if case.operation == "Query" && case.requires["query_function"].is_string() {
            let tag = case.requires["query_result_tag"]
                .as_str()
                .and_then(|value| value.parse::<u32>().ok())
                .ok_or(Failure::Assertion("invalid Query field fixture"))?;
            let request =
                adapters::request(case, None, &self.name, &self.connection.protocol_version)?;
            let request = if self.connection.api_level == "fluent" {
                crate::fluent::build(self.client, request).map_err(Failure::Assertion)?
            } else {
                request
            };
            let response = self.exchange(request)?;
            if response.status == 0 && !response.nonempty_structure_tags.contains(&tag) {
                return Err(Failure::Assertion(
                    "Query did not return the requested nonempty structure",
                ));
            }
            return Ok(response);
        }
        if case.operation == "Locate" && case.requires["rotate_attribute_ttlv"].is_string() {
            let request =
                adapters::request(case, None, &self.name, &self.connection.protocol_version)?;
            let request = if self.connection.api_level == "fluent" {
                crate::fluent::build(self.client, request).map_err(Failure::Assertion)?
            } else {
                request
            };
            let response = self.exchange(request)?;
            if response.status == 0 && response.identifiers.len() > 1 {
                return Err(Failure::Assertion(
                    "Locate returned more identifiers than requested",
                ));
            }
            return Ok(response);
        }
        let mut identifier = None;
        let needs_object = ![
            "Discover Versions",
            "Ping",
            "Query",
            "Interop",
            "Log",
            "Set Endpoint Role",
            "Create",
            "Create Key Pair",
            "Create Split Key",
            "Register",
            "Import",
            "Get Constraints",
            "Set Constraints",
            "Set Defaults",
            "Hash",
            "RNG Retrieve",
            "RNG Seed",
            "Certify",
            "Re-Provision",
            "Validate",
            "PKCS#11",
        ]
        .contains(&case.operation.as_str());
        if needs_object {
            if case.requires["attribute"]
                .as_str()
                .is_some_and(|v| v.contains("Certificate") || v.starts_with("X.509"))
                || case.operation == "Re-certify"
            {
                let response = self.setup(fixtures::register(
                    "Certificate",
                    &self.connection.protocol_version,
                )?)?;
                self.own(&response);
                identifier = response.identifier;
            } else if case.operation == "Re-key Key Pair" {
                identifier = Some(self.key("RSA", 2048, false)?.0);
            } else {
                identifier = Some(self.key("AES", 256, false)?.0);
            }
            if case.operation == "Recover" {
                self.setup(ClientRequest::Archive(ArchiveRequest::new(
                    identifier.clone(),
                )))?;
            }
            if case.case_id == "test-neg-destroy-active-without-revoke" {
                self.setup(ClientRequest::Activate(ActivateRequest::new(
                    identifier.clone(),
                )))?;
            }
        }
        let target_identifier = identifier.clone();
        let request = adapters::request(
            case,
            identifier.as_ref(),
            &self.name,
            &self.connection.protocol_version,
        )?;
        if case.case_id == "test-neg-modify-readonly-attribute"
            && !request.validation_report().is_valid()
        {
            return Err(Failure::LocalRejection(
                "library_preflight_rejection: State modification is rejected locally under OASIS KMIP 2.1 §4.57; server negative probe cannot be sent",
            ));
        }
        let request = if self.connection.api_level == "fluent" {
            crate::fluent::build(self.client, request).map_err(Failure::Assertion)?
        } else {
            request
        };
        let response = self.exchange(request)?;
        if case.operation == "Destroy" && response.status == 0 {
            self.owned
                .retain(|id| Some(id) != target_identifier.as_ref());
        }
        if [
            "Create",
            "Create Key Pair",
            "Create Split Key",
            "Register",
            "Import",
            "Derive Key",
            "Re-key",
            "Re-key Key Pair",
            "Certify",
            "Re-certify",
        ]
        .contains(&case.operation.as_str())
            && response.status == 0
        {
            self.own(&response);
        }
        Ok(response)
    }
    fn asynchronous(&mut self, operation: &str) -> std::result::Result<Observed, Failure> {
        if self
            .connection
            .authentication
            .as_ref()
            .and_then(|a| a.username_password.as_ref())
            .is_some()
        {
            return Err(Failure::Prerequisite(
                "library_api_unavailable: public asynchronous follow-up methods cannot attach Request Header Authentication",
            ));
        }
        let request = adapters::create(&self.name, &self.connection.protocol_version)?;
        let mut response = driver::batch(self.client, self.connection, vec![request], true)?
            .into_iter()
            .next()
            .ok_or(Failure::Prerequisite(
                "asynchronous setup returned no result",
            ))?;
        self.own(&response);
        let mut correlation = response.correlation.take().ok_or(Failure::Prerequisite(
            "server did not return the required Pending correlation; asynchronous target not sent",
        ))?;
        let mut exchange = |operation: &str, token: &[u8]| {
            if self.connection.api_level == "generic-ttlv" {
                crate::generic::asynchronous(self.connection, operation, token)
                    .map_err(Failure::Assertion)
            } else {
                driver::asynchronous(self.client, self.connection, operation, token)
                    .map_err(Failure::Library)
            }
        };
        if operation != "Poll" {
            if operation != "Cancel" {
                self.pending
                    .push(correlation.with_bytes(|v| SecretBytes::new(v.to_vec())));
            }
            return correlation.with_bytes(|value| exchange(operation, value));
        }
        for _ in 0..16 {
            self.pending.clear();
            self.pending
                .push(correlation.with_bytes(|v| SecretBytes::new(v.to_vec())));
            let mut observed = correlation.with_bytes(|value| exchange("Poll", value))?;
            if observed.status != 2 {
                if observed.status == 0 {
                    if observed.identifiers.len() != 1 {
                        return Err(Failure::Assertion(
                            "completed Poll did not return the owned Create identifier",
                        ));
                    }
                    self.pending.clear();
                    self.own(&observed);
                }
                return Ok(observed);
            }
            correlation = observed.correlation.take().ok_or(Failure::Assertion(
                "Pending Poll response omitted its correlation value",
            ))?;
        }
        self.pending.clear();
        self.pending.push(correlation);
        Err(Failure::Prerequisite(
            "server remained Pending after the bounded Poll sequence",
        ))
    }
    fn join(&mut self) -> std::result::Result<Observed, Failure> {
        let request = ClientRequest::CreateSplitKey(CreateSplitKeyRequest::new(
            ObjectType::from_raw(2),
            2,
            2,
            SplitKeyMethod::XOR,
            adapters::attributes(&self.name, 3, 256, &self.connection.protocol_version)?,
        ));
        let response = self.setup(request)?;
        self.own(&response);
        if response.identifiers.len() != 2 {
            return Err(Failure::Prerequisite(
                "Create Split Key did not supply both shares",
            ));
        }
        let refs = response
            .identifiers
            .into_iter()
            .map(|id| SplitKeyReference::new(id).with_split_key_threshold(2))
            .collect();
        let result = self.exchange(ClientRequest::JoinSplitKey(
            JoinSplitKeyRequest::new(ObjectType::from_raw(2), refs).with_attributes(
                adapters::attributes(&self.name, 3, 256, &self.connection.protocol_version)?,
            ),
        ))?;
        if result.status == 0 {
            self.own(&result);
        }
        Ok(result)
    }
    fn batch(&mut self, case: &Case) -> std::result::Result<Observed, Failure> {
        let requests = if case.case_id == "test-batch-create-activate-destroy" {
            vec![
                adapters::create(&self.name, &self.connection.protocol_version)?,
                ClientRequest::Activate(ActivateRequest::new(None)),
                ClientRequest::Revoke(RevokeRequest::new(RevocationReason::CESSATION_OF_OPERATION)),
                ClientRequest::Destroy(DestroyRequest::new(None)),
            ]
        } else {
            vec![
                fixtures::register("Symmetric Key", &self.connection.protocol_version)?,
                ClientRequest::Activate(ActivateRequest::new(None)),
                ClientRequest::Encrypt(
                    EncryptRequest::new(None, Some(plain()))
                        .with_cryptographic_parameters(parameters(3)?)
                        .with_iv_counter_nonce(SecretBytes::new(vec![0; 16])),
                ),
                ClientRequest::Decrypt(
                    DecryptRequest::new(
                        None,
                        Some(OperationData::ByteString(SecretBytes::new(
                            include_bytes!("../tests/fixtures/aes256-cbc.bin").to_vec(),
                        ))),
                    )
                    .with_cryptographic_parameters(parameters(3)?)
                    .with_iv_counter_nonce(SecretBytes::new(vec![0; 16])),
                ),
                ClientRequest::Revoke(RevokeRequest::new(RevocationReason::CESSATION_OF_OPERATION)),
                ClientRequest::Destroy(DestroyRequest::new(None)),
            ]
        };
        let results = driver::batch(self.client, self.connection, requests, false)?;
        for result in &results {
            self.own(result);
        }
        let failed = results.iter().position(|v| v.status != 0);
        if let Some(index) = failed {
            return results
                .into_iter()
                .nth(index)
                .ok_or(Failure::Assertion("batch result missing"));
        }
        if case.case_id == "test-batch-register-encrypt-decrypt-destroy" {
            let bytes = results
                .get(3)
                .and_then(|r| r.bytes.as_ref())
                .ok_or(Failure::Assertion("Decrypt returned no data"))?;
            if !bytes.with_bytes(|v| v == [0x41; 16]) {
                return Err(Failure::Assertion(
                    "batch decrypt did not recover fixture plaintext",
                ));
            }
        }
        self.owned.clear();
        results
            .into_iter()
            .last()
            .ok_or(Failure::Assertion("batch returned no results"))
    }
    fn multipart(&mut self, case: &Case) -> std::result::Result<Observed, Failure> {
        if case.operation == "Hash" {
            return self.multipart_hash();
        }
        let algorithm = case.requires["algorithm"].as_str().unwrap_or("AES");
        let bits = case.requires["key_length"]
            .as_i64()
            .and_then(|v| i32::try_from(v).ok())
            .unwrap_or(128);
        let id = Some(self.key(algorithm, bits, true)?.0);
        let raw = if algorithm == "RSA" { 4 } else { 3 };
        let initial = match case.operation.as_str() {
            "Sign" => ClientRequest::Sign(
                SignRequest::new()
                    .with_unique_identifier(
                        id.clone()
                            .ok_or(Failure::Prerequisite("missing private key"))?,
                    )
                    .with_cryptographic_parameters(parameters(raw)?)
                    .with_data(plain())
                    .with_init_indicator(true)
                    .with_final_indicator(false),
            ),
            "Decrypt" => ClientRequest::Decrypt(
                DecryptRequest::new(id.clone(), Some(plain()))
                    .with_cryptographic_parameters(parameters(raw)?)
                    .with_iv_counter_nonce(SecretBytes::new(vec![0; 16]))
                    .with_init_indicator(true)
                    .with_final_indicator(false),
            ),
            _ => ClientRequest::Encrypt(
                EncryptRequest::new(id.clone(), Some(plain()))
                    .with_cryptographic_parameters(parameters(raw)?)
                    .with_iv_counter_nonce(SecretBytes::new(vec![0; 16]))
                    .with_init_indicator(true)
                    .with_final_indicator(false),
            ),
        };
        let response = self.setup(initial)?;
        let correlation = response.correlation.ok_or(Failure::Prerequisite(
            "server did not return a multipart correlation",
        ))?;
        let final_request = match case.operation.as_str() {
            "Sign" => ClientRequest::Sign(
                SignRequest::new()
                    .with_unique_identifier(
                        id.clone()
                            .ok_or(Failure::Prerequisite("missing private key"))?,
                    )
                    .with_data(plain())
                    .with_correlation_value(correlation)
                    .with_init_indicator(false)
                    .with_final_indicator(true),
            ),
            "Decrypt" => ClientRequest::Decrypt(
                DecryptRequest::new(id.clone(), Some(plain()))
                    .with_correlation_value(correlation)
                    .with_init_indicator(false)
                    .with_final_indicator(true),
            ),
            _ => ClientRequest::Encrypt(
                EncryptRequest::new(id.clone(), Some(plain()))
                    .with_correlation_value(correlation)
                    .with_init_indicator(false)
                    .with_final_indicator(true),
            ),
        };
        self.exchange(final_request)
    }
}

pub(crate) fn execute(
    case: &Case,
    connection: &Connection,
    client: &mut Client,
) -> (std::result::Result<Observed, Failure>, bool) {
    let mut attempt = Attempt {
        client,
        connection,
        owned: Vec::new(),
        pending: Vec::new(),
        name: format!("kmipkit-rust-{}", uuid::Uuid::new_v4()),
    };
    let result = attempt.run(case);
    let clean = attempt.cleanup();
    (result, clean)
}
fn plain() -> OperationData {
    OperationData::ByteString(SecretBytes::new(vec![0x41; 16]))
}
fn parameters(algorithm: u32) -> std::result::Result<kmipkit::ttlv::Structure, Failure> {
    let builder = CryptographicParametersBuilder::new();
    let builder = match algorithm {
        3 => builder
            .with_cryptographic_algorithm(3)
            .with_block_cipher_mode(1)
            .with_padding_method(1),
        4 => builder
            .with_cryptographic_algorithm(4)
            .with_hashing_algorithm(6)
            .with_padding_method(8)
            .with_digital_signature_algorithm(5),
        6 => builder
            .with_cryptographic_algorithm(6)
            .with_hashing_algorithm(6)
            .with_digital_signature_algorithm(14),
        _ => builder.with_hashing_algorithm(6),
    };
    builder
        .build()
        .map(CryptographicParameters::into_ttlv)
        .map_err(|_| Failure::Assertion("explicit cryptographic parameters rejected"))
}
