//! Build the same single-part KMIP 2.x inputs through the typed and Fluent APIs.

use super::{Attempt, Failure, parameters, plain};
use crate::driver::Observed;
use crate::models::Case;
use kmipkit::protocol::{
    CryptographicParametersBuilder, DecryptRequest, EncryptRequest, HashRequest, MacRequest,
    MacVerifyRequest, OperationData, SecretBytes, SignRequest, SignatureVerifyRequest,
    UniqueIdentifier,
};
use kmipkit::{Client, ClientRequest};
use sha2::{Digest, Sha256};

pub(crate) const OPERATIONS: &[&str] = &[
    "Encrypt",
    "Decrypt",
    "Hash",
    "MAC",
    "MAC Verify",
    "Sign",
    "Signature Verify",
];

impl Attempt<'_> {
    pub(super) fn hash(&mut self) -> Result<Observed, Failure> {
        let input = b"abc";
        let request = self.hash_request(input, None, None)?;
        let response = self.exchange(request)?;
        verify_digest(&response, input)?;
        Ok(response)
    }

    pub(super) fn multipart_hash(&mut self) -> Result<Observed, Failure> {
        const FIRST: &[u8] = b"KMIP ";
        const UPDATE: &[u8] = b"Hash";
        const LAST: &[u8] = b"!";
        let initial = self.hash_request(FIRST, Some(false), None)?;
        let observed = self.exchange(initial)?;
        if observed.status != 0 {
            return Ok(observed);
        }
        let correlation = observed
            .correlation
            .filter(|value| value.with_bytes(|bytes| !bytes.is_empty()))
            .ok_or(Failure::Assertion(
                "multipart Hash did not return a correlation value",
            ))?;
        if observed.bytes.is_some() {
            return Err(Failure::Assertion(
                "multipart Hash returned a digest before Final",
            ));
        }
        let update = self.hash_request(UPDATE, None, Some(&correlation))?;
        let observed = self.exchange(update)?;
        if observed.status != 0 {
            return Ok(observed);
        }
        if observed.bytes.is_some() || observed.correlation.is_some() {
            return Err(Failure::Assertion(
                "multipart Hash returned unexpected Update fields",
            ));
        }
        let final_request = self.hash_request(LAST, Some(true), Some(&correlation))?;
        let response = self.exchange(final_request)?;
        verify_digest(&response, b"KMIP Hash!")?;
        Ok(response)
    }

    fn hash_request(
        &mut self,
        input: &[u8],
        final_indicator: Option<bool>,
        correlation: Option<&SecretBytes>,
    ) -> Result<ClientRequest, Failure> {
        let parameters = parameters(9)?;
        if self.connection.api_level == "fluent" {
            let mut request = self
                .client
                .request()
                .hash()
                .cryptographic_parameters_structure(parameters)
                .data(input.to_vec());
            match final_indicator {
                Some(false) => request = request.init_indicator(true),
                Some(true) => request = request.final_indicator(true),
                None => {}
            }
            if let Some(correlation) = correlation {
                request = request.correlation_value(correlation.with_bytes(<[u8]>::to_vec));
            }
            return request
                .build()
                .map(ClientRequest::Hash)
                .map_err(|_| Failure::Assertion("Fluent Hash rejected the case inputs"));
        }
        let mut request = HashRequest::new(parameters)
            .with_data(OperationData::ByteString(SecretBytes::new(input.to_vec())));
        match final_indicator {
            Some(false) => request = request.with_init_indicator(true),
            Some(true) => request = request.with_final_indicator(true),
            None => {}
        }
        if let Some(correlation) = correlation {
            request = request
                .with_correlation_value(SecretBytes::new(correlation.with_bytes(<[u8]>::to_vec)));
        }
        Ok(ClientRequest::Hash(request))
    }

    pub(super) fn crypto(&mut self, case: &Case) -> Result<Observed, Failure> {
        if case.requires["block_cipher_mode"] == "GCM" {
            return self.authenticated_encryption(case);
        }
        let default_alg = match case.operation.as_str() {
            "MAC" | "MAC Verify" => "HMAC_SHA256",
            "Sign" | "Signature Verify" => "RSA",
            _ => "AES",
        };
        let algorithm = case.requires["algorithm"].as_str().unwrap_or(default_alg);
        let bits = case.requires["key_length"]
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .unwrap_or(256);
        let (private, public) = self.key(
            algorithm,
            bits,
            case.case_id != "test-neg-encrypt-preactive-key",
        )?;
        let algorithm = match algorithm {
            "RSA" => 4,
            "ECDSA" => 6,
            "HMAC_SHA256" => 9,
            _ => 3,
        };
        let operation = case.operation.as_str();
        let mut data = plain();
        let mut proof = None;
        let target = match operation {
            "Decrypt" | "MAC Verify" | "Signature Verify" => {
                let preparation = match operation {
                    "Decrypt" => "Encrypt",
                    "MAC Verify" => "MAC",
                    _ => "Sign",
                };
                let request =
                    self.crypto_request(preparation, private.clone(), plain(), algorithm, None)?;
                let bytes = self.setup(request)?.bytes.ok_or(Failure::Prerequisite(
                    "setup operation did not return cryptographic data",
                ))?;
                if operation == "Decrypt" {
                    data = OperationData::ByteString(bytes);
                } else {
                    proof = Some(bytes);
                }
                if operation == "Signature Verify" {
                    public
                } else {
                    private
                }
            }
            _ => private,
        };
        let request = self.crypto_request(operation, target.clone(), data, algorithm, proof)?;
        let response = self.exchange(request)?;
        if response.status == 0 {
            check_response(operation, &target, &response)?;
        }
        Ok(response)
    }

    fn authenticated_encryption(&mut self, case: &Case) -> Result<Observed, Failure> {
        let (id, _) = self.key("AES", 256, true)?;
        let request = self.gcm_request(&id, plain(), None)?;
        let response = if case.operation == "Encrypt" {
            self.exchange(request)?
        } else {
            self.setup(request)?
        };
        if response.status != 0 {
            return Ok(response);
        }
        check_response("Encrypt", &id, &response)?;
        let mut tag = response
            .authentication_tag
            .as_ref()
            .filter(|value| value.with_bytes(|bytes| bytes.len() == 16))
            .ok_or(Failure::Assertion(
                "Encrypt omitted the 16-byte AES-GCM tag",
            ))?
            .with_bytes(<[u8]>::to_vec);
        if case.operation == "Encrypt" {
            return Ok(response);
        }
        let ciphertext = response
            .bytes
            .ok_or(Failure::Prerequisite("Encrypt omitted ciphertext"))?;
        if case.requires["expected_status"] == "OperationFailed" {
            tag[0] ^= 1;
        }
        let request = self.gcm_request(&id, OperationData::ByteString(ciphertext), Some(tag))?;
        let response = self.exchange(request)?;
        if response.status == 0 {
            check_response("Decrypt", &id, &response)?;
        }
        Ok(response)
    }

    fn gcm_request(
        &mut self,
        identifier: &UniqueIdentifier,
        data: OperationData,
        tag: Option<Vec<u8>>,
    ) -> Result<ClientRequest, Failure> {
        const AAD: &[u8] = b"kmipkit authenticated data";
        let parameters = CryptographicParametersBuilder::new()
            .with_block_cipher_mode(9)
            .with_padding_method(1)
            .with_iv_length(96)
            .with_tag_length(16)
            .build()
            .map_err(|_| Failure::Assertion("AES-GCM parameters are invalid"))?
            .into_ttlv();
        let iv = vec![0; 12];
        if self.connection.api_level == "fluent" {
            let request = self.client.request();
            let invalid = |_| Failure::Assertion("Fluent AES-GCM rejected the case inputs");
            return if let Some(tag) = tag {
                request
                    .decrypt()
                    .unique_identifier(identifier.clone())
                    .cryptographic_parameters_structure(parameters)
                    .data_value(data)
                    .iv_counter_nonce(iv)
                    .authenticated_encryption_additional_data(AAD.to_vec())
                    .authenticated_encryption_tag(tag)
                    .build()
                    .map(ClientRequest::Decrypt)
                    .map_err(invalid)
            } else {
                request
                    .encrypt()
                    .unique_identifier(identifier.clone())
                    .cryptographic_parameters_structure(parameters)
                    .data_value(data)
                    .iv_counter_nonce(iv)
                    .authenticated_encryption_additional_data(AAD.to_vec())
                    .build()
                    .map(ClientRequest::Encrypt)
                    .map_err(invalid)
            };
        }
        Ok(if let Some(tag) = tag {
            ClientRequest::Decrypt(
                DecryptRequest::new(Some(identifier.clone()), Some(data))
                    .with_cryptographic_parameters(parameters)
                    .with_iv_counter_nonce(SecretBytes::new(iv))
                    .with_authenticated_encryption_additional_data(SecretBytes::new(AAD.to_vec()))
                    .with_authenticated_encryption_tag(SecretBytes::new(tag)),
            )
        } else {
            ClientRequest::Encrypt(
                EncryptRequest::new(Some(identifier.clone()), Some(data))
                    .with_cryptographic_parameters(parameters)
                    .with_iv_counter_nonce(SecretBytes::new(iv))
                    .with_authenticated_encryption_additional_data(SecretBytes::new(AAD.to_vec())),
            )
        })
    }

    fn crypto_request(
        &mut self,
        operation: &str,
        identifier: UniqueIdentifier,
        data: OperationData,
        algorithm: u32,
        proof: Option<SecretBytes>,
    ) -> Result<ClientRequest, Failure> {
        let parameters = parameters(algorithm)?;
        if self.connection.api_level == "fluent" {
            fluent(self.client, operation, identifier, data, parameters, proof)
        } else {
            typed(operation, identifier, data, parameters, proof)
        }
    }
}

fn verify_digest(response: &Observed, input: &[u8]) -> Result<(), Failure> {
    let digest = Sha256::digest(input);
    if response.status == 0
        && !response
            .bytes
            .as_ref()
            .is_some_and(|bytes| bytes.with_bytes(|value| value == &digest[..]))
    {
        return Err(Failure::Assertion(
            "Hash did not return the SHA-256 known answer",
        ));
    }
    Ok(())
}

fn required_proof(proof: Option<SecretBytes>) -> Result<SecretBytes, Failure> {
    proof.ok_or(Failure::Prerequisite("verification proof is missing"))
}

fn typed(
    operation: &str,
    identifier: UniqueIdentifier,
    data: OperationData,
    parameters: kmipkit::ttlv::Structure,
    proof: Option<SecretBytes>,
) -> Result<ClientRequest, Failure> {
    macro_rules! common {
        ($request:ty) => {
            <$request>::new()
                .with_unique_identifier(identifier)
                .with_data(data)
                .with_cryptographic_parameters(parameters)
        };
    }
    Ok(match operation {
        "Encrypt" => ClientRequest::Encrypt(
            EncryptRequest::new(Some(identifier), Some(data))
                .with_cryptographic_parameters(parameters)
                .with_iv_counter_nonce(SecretBytes::new(vec![0; 16])),
        ),
        "Decrypt" => ClientRequest::Decrypt(
            DecryptRequest::new(Some(identifier), Some(data))
                .with_cryptographic_parameters(parameters)
                .with_iv_counter_nonce(SecretBytes::new(vec![0; 16])),
        ),
        "MAC" => ClientRequest::Mac(common!(MacRequest)),
        "MAC Verify" => ClientRequest::MacVerify(
            common!(MacVerifyRequest).with_mac_data(required_proof(proof)?),
        ),
        "Sign" => ClientRequest::Sign(common!(SignRequest)),
        "Signature Verify" => ClientRequest::SignatureVerify(
            common!(SignatureVerifyRequest).with_signature_data(required_proof(proof)?),
        ),
        _ => return Err(Failure::Assertion("unknown single-part cryptographic case")),
    })
}

fn fluent(
    client: &mut Client,
    operation: &str,
    identifier: UniqueIdentifier,
    data: OperationData,
    parameters: kmipkit::ttlv::Structure,
    proof: Option<SecretBytes>,
) -> Result<ClientRequest, Failure> {
    let request = client.request();
    let invalid = |_| Failure::Assertion("Fluent cryptography rejected the case inputs");
    macro_rules! common {
        ($method:ident) => {
            request
                .$method()
                .unique_identifier(identifier)
                .data_value(data)
                .cryptographic_parameters_structure(parameters)
        };
    }
    Ok(match operation {
        "Encrypt" => ClientRequest::Encrypt(
            common!(encrypt)
                .iv_counter_nonce(vec![0; 16])
                .build()
                .map_err(invalid)?,
        ),
        "Decrypt" => ClientRequest::Decrypt(
            common!(decrypt)
                .iv_counter_nonce(vec![0; 16])
                .build()
                .map_err(invalid)?,
        ),
        "MAC" => ClientRequest::Mac(common!(mac).build().map_err(invalid)?),
        "MAC Verify" => ClientRequest::MacVerify(
            common!(mac_verify)
                .mac_data(required_proof(proof)?.with_bytes(<[u8]>::to_vec))
                .build()
                .map_err(invalid)?,
        ),
        "Sign" => ClientRequest::Sign(common!(sign).build().map_err(invalid)?),
        "Signature Verify" => ClientRequest::SignatureVerify(
            common!(signature_verify)
                .signature_data(required_proof(proof)?.with_bytes(<[u8]>::to_vec))
                .build()
                .map_err(invalid)?,
        ),
        _ => return Err(Failure::Assertion("unknown Fluent cryptographic case")),
    })
}

fn check_response(
    operation: &str,
    identifier: &UniqueIdentifier,
    response: &Observed,
) -> Result<(), Failure> {
    if response.identifier.as_ref() != Some(identifier) {
        return Err(Failure::Assertion(
            "cryptographic result changed the key identifier",
        ));
    }
    let matches_bytes = |check: fn(&[u8]) -> bool| {
        response
            .bytes
            .as_ref()
            .is_some_and(|bytes| bytes.with_bytes(check))
    };
    let valid = match operation {
        "Encrypt" => matches_bytes(|bytes| bytes.len() == 16 && bytes != [0x41; 16]),
        "Decrypt" => matches_bytes(|bytes| bytes == [0x41; 16]),
        "MAC" => matches_bytes(|bytes| bytes.len() == 32),
        "Sign" => matches_bytes(|bytes| !bytes.is_empty()),
        "MAC Verify" | "Signature Verify" => response.validity == Some(1),
        _ => false,
    };
    if !valid {
        return Err(Failure::Assertion(
            "cryptographic response did not match the case",
        ));
    }
    Ok(())
}
