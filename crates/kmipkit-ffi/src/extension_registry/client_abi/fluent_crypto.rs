//! Semantic C builders for the KMIP 2.1 cryptographic client operations.

use super::*;
use kmipkit_protocol::ValidateKmipRequest;
use kmipkit_ttlv::ValueView;

#[derive(Clone, Copy)]
enum CryptoByteField {
    IvCounterNonce,
    AuthenticatedEncryptionAdditionalData,
    AuthenticatedEncryptionTag,
    DigestedData,
    SignatureData,
    MacData,
    CorrelationValue,
}

struct CryptoRequestParts {
    unique_identifier: Option<UniqueIdentifier>,
    data: Option<OperationData>,
    cryptographic_parameters: Option<Structure>,
    correlation_value: Option<SecretBytes>,
    init_indicator: Option<bool>,
    final_indicator: Option<bool>,
}

fn create_crypto_builder(
    operation: CryptoOperationKind,
    out_builder: *mut *mut kmipkit_crypto_operation_builder_t,
) -> FfiResult<()> {
    if out_builder.is_null() || !out_builder.is_aligned() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: the output slot is non-null, aligned, and writable for this synchronous call.
    let out = unsafe { &mut *out_builder };
    *out = ptr::null_mut();
    *out = try_make_handle(
        Kind::CryptoOperationBuilder,
        HandleValue::CryptoOperationBuilder(Mutex::new(CryptoOperationBuilder::new(operation))),
    )?;
    Ok(())
}

fn update_crypto_builder(
    builder: *mut kmipkit_crypto_operation_builder_t,
    update: impl FnOnce(&mut CryptoOperationBuilder) -> FfiResult<()>,
) -> FfiResult<()> {
    let handle = reference_handle(builder, Kind::CryptoOperationBuilder)?;
    let HandleValue::CryptoOperationBuilder(builder) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let mut builder = builder.lock().map_err(|_| ERROR_INTERNAL)?;
    update(&mut builder)
}

fn copy_builder_span(span: *const kmipkit_byte_span_v1_t) -> FfiResult<Zeroizing<Vec<u8>>> {
    if span.is_null() || !span.is_aligned() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: a non-null aligned span pointer is borrowed only for this call.
    let span = unsafe { span.read() };
    let input = input_span(span.data, span.length, MAX_FFI_INPUT_BYTES)?;
    let mut owned = Zeroizing::new(Vec::new());
    owned
        .try_reserve_exact(input.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    owned.extend_from_slice(input);
    Ok(owned)
}

fn copy_builder_data(value: *mut kmipkit_ttlv_value_t) -> FfiResult<CryptoBuilderData> {
    let handle = reference_handle(value, Kind::Value)?;
    let HandleValue::Value(value) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    value.with_value(|view| match view {
        ValueView::ByteString(bytes) => {
            let mut owned = Zeroizing::new(Vec::new());
            owned
                .try_reserve_exact(bytes.len())
                .map_err(|_| ERROR_RESOURCE_LIMIT)?;
            owned.extend_from_slice(bytes);
            Ok(CryptoBuilderData::ByteString(owned))
        }
        ValueView::Enumeration(value) => Ok(CryptoBuilderData::Enumeration(*value)),
        ValueView::Integer(value) => Ok(CryptoBuilderData::Integer(*value)),
        _ => Err(ERROR_INVALID_INPUT),
    })
}

fn secret_copy(bytes: &Zeroizing<Vec<u8>>) -> FfiResult<SecretBytes> {
    let mut copy = Vec::new();
    copy.try_reserve_exact(bytes.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    copy.extend_from_slice(bytes);
    Ok(SecretBytes::new(copy))
}

impl CryptoOperationBuilder {
    fn build_request(&self) -> FfiResult<ClientRequest> {
        let parts = self.request_parts()?;
        let request = match self.operation {
            CryptoOperationKind::Encrypt => self.build_encrypt_request(parts)?,
            CryptoOperationKind::Decrypt => self.build_decrypt_request(parts)?,
            CryptoOperationKind::Sign => self.build_sign_request(parts)?,
            CryptoOperationKind::SignatureVerify => self.build_signature_verify_request(parts)?,
            CryptoOperationKind::Mac => Self::build_mac_request(parts),
            CryptoOperationKind::MacVerify => self.build_mac_verify_request(parts)?,
            CryptoOperationKind::Hash => Self::build_hash_request(parts)?,
        };
        validate_crypto_request(&request)?;
        Ok(request)
    }

    fn request_parts(&self) -> FfiResult<CryptoRequestParts> {
        let data = self
            .data
            .as_ref()
            .map(|value| match value {
                CryptoBuilderData::ByteString(bytes) => {
                    secret_copy(bytes).map(OperationData::ByteString)
                }
                CryptoBuilderData::Enumeration(value) => Ok(OperationData::Enumeration(*value)),
                CryptoBuilderData::Integer(value) => Ok(OperationData::Integer(*value)),
            })
            .transpose()?;
        let cryptographic_parameters = self
            .cryptographic_parameters
            .as_ref()
            .map(|value| value.view().try_clone().map_err(model_status))
            .transpose()?;
        let correlation_value = self
            .correlation_value
            .as_ref()
            .map(secret_copy)
            .transpose()?;
        Ok(CryptoRequestParts {
            unique_identifier: self.unique_identifier.clone(),
            data,
            cryptographic_parameters,
            correlation_value,
            init_indicator: self.init_indicator,
            final_indicator: self.final_indicator,
        })
    }

    fn build_encrypt_request(&self, parts: CryptoRequestParts) -> FfiResult<ClientRequest> {
        let mut request = EncryptRequest::new(parts.unique_identifier, parts.data);
        if let Some(parameters) = parts.cryptographic_parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(value) = self.iv_counter_nonce.as_ref() {
            request = request.with_iv_counter_nonce(secret_copy(value)?);
        }
        if let Some(value) = parts.correlation_value {
            request = request.with_correlation_value(value);
        }
        if let Some(value) = parts.init_indicator {
            request = request.with_init_indicator(value);
        }
        if let Some(value) = parts.final_indicator {
            request = request.with_final_indicator(value);
        }
        if let Some(value) = self.authenticated_encryption_additional_data.as_ref() {
            request = request.with_authenticated_encryption_additional_data(secret_copy(value)?);
        }
        Ok(ClientRequest::Encrypt(request))
    }

    fn build_decrypt_request(&self, parts: CryptoRequestParts) -> FfiResult<ClientRequest> {
        let mut request = DecryptRequest::new(parts.unique_identifier, parts.data);
        if let Some(parameters) = parts.cryptographic_parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(value) = self.iv_counter_nonce.as_ref() {
            request = request.with_iv_counter_nonce(secret_copy(value)?);
        }
        if let Some(value) = self.authenticated_encryption_tag.as_ref() {
            request = request.with_authenticated_encryption_tag(secret_copy(value)?);
        }
        if let Some(value) = parts.correlation_value {
            request = request.with_correlation_value(value);
        }
        if let Some(value) = parts.init_indicator {
            request = request.with_init_indicator(value);
        }
        if let Some(value) = parts.final_indicator {
            request = request.with_final_indicator(value);
        }
        if let Some(value) = self.authenticated_encryption_additional_data.as_ref() {
            request = request.with_authenticated_encryption_additional_data(secret_copy(value)?);
        }
        Ok(ClientRequest::Decrypt(request))
    }

    fn build_sign_request(&self, parts: CryptoRequestParts) -> FfiResult<ClientRequest> {
        let mut request = SignRequest::new();
        if let Some(identifier) = parts.unique_identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(parameters) = parts.cryptographic_parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(data) = parts.data {
            request = request.with_data(data);
        }
        if let Some(value) = self.digested_data.as_ref() {
            request = request.with_digested_data(secret_copy(value)?);
        }
        request = with_cryptographic_framing(
            request,
            parts.correlation_value,
            parts.init_indicator,
            parts.final_indicator,
        );
        Ok(ClientRequest::Sign(request))
    }

    fn build_signature_verify_request(
        &self,
        parts: CryptoRequestParts,
    ) -> FfiResult<ClientRequest> {
        let mut request = SignatureVerifyRequest::new();
        if let Some(identifier) = parts.unique_identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(parameters) = parts.cryptographic_parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(data) = parts.data {
            request = request.with_data(data);
        }
        if let Some(value) = self.digested_data.as_ref() {
            request = request.with_digested_data(secret_copy(value)?);
        }
        if let Some(value) = self.signature_data.as_ref() {
            request = request.with_signature_data(secret_copy(value)?);
        }
        request = with_cryptographic_framing(
            request,
            parts.correlation_value,
            parts.init_indicator,
            parts.final_indicator,
        );
        Ok(ClientRequest::SignatureVerify(request))
    }

    fn build_mac_request(parts: CryptoRequestParts) -> ClientRequest {
        let mut request = MacRequest::new();
        if let Some(identifier) = parts.unique_identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(parameters) = parts.cryptographic_parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(data) = parts.data {
            request = request.with_data(data);
        }
        ClientRequest::Mac(with_cryptographic_framing(
            request,
            parts.correlation_value,
            parts.init_indicator,
            parts.final_indicator,
        ))
    }

    fn build_mac_verify_request(&self, parts: CryptoRequestParts) -> FfiResult<ClientRequest> {
        let mut request = MacVerifyRequest::new();
        if let Some(identifier) = parts.unique_identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(parameters) = parts.cryptographic_parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(data) = parts.data {
            request = request.with_data(data);
        }
        if let Some(value) = self.mac_data.as_ref() {
            request = request.with_mac_data(secret_copy(value)?);
        }
        request = with_cryptographic_framing(
            request,
            parts.correlation_value,
            parts.init_indicator,
            parts.final_indicator,
        );
        Ok(ClientRequest::MacVerify(request))
    }

    fn build_hash_request(parts: CryptoRequestParts) -> FfiResult<ClientRequest> {
        let parameters = parts.cryptographic_parameters.ok_or(ERROR_INVALID_INPUT)?;
        let mut request = HashRequest::new(parameters);
        if let Some(data) = parts.data {
            request = request.with_data(data);
        }
        request = with_cryptographic_framing(
            request,
            parts.correlation_value,
            parts.init_indicator,
            parts.final_indicator,
        );
        Ok(ClientRequest::Hash(request))
    }
}

fn validate_crypto_request(request: &ClientRequest) -> FfiResult<()> {
    let valid = match request {
        ClientRequest::Encrypt(request) => request.validate().is_valid(),
        ClientRequest::Decrypt(request) => request.validate().is_valid(),
        ClientRequest::Sign(request) => request.validate().is_valid(),
        ClientRequest::SignatureVerify(request) => request.validate().is_valid(),
        ClientRequest::Mac(request) => request.validate().is_valid(),
        ClientRequest::MacVerify(request) => request.validate().is_valid(),
        ClientRequest::Hash(request) => request.validate().is_valid(),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(ERROR_INVALID_INPUT)
    }
}

fn set_crypto_bytes(
    builder: *mut kmipkit_crypto_operation_builder_t,
    field: CryptoByteField,
    span: *const kmipkit_byte_span_v1_t,
) -> FfiResult<()> {
    let bytes = copy_builder_span(span)?;
    update_crypto_builder(builder, |builder| {
        let allowed = match field {
            CryptoByteField::IvCounterNonce
            | CryptoByteField::AuthenticatedEncryptionAdditionalData => matches!(
                builder.operation,
                CryptoOperationKind::Encrypt | CryptoOperationKind::Decrypt
            ),
            CryptoByteField::AuthenticatedEncryptionTag => {
                builder.operation == CryptoOperationKind::Decrypt
            }
            CryptoByteField::DigestedData => matches!(
                builder.operation,
                CryptoOperationKind::Sign | CryptoOperationKind::SignatureVerify
            ),
            CryptoByteField::SignatureData => {
                builder.operation == CryptoOperationKind::SignatureVerify
            }
            CryptoByteField::MacData => builder.operation == CryptoOperationKind::MacVerify,
            CryptoByteField::CorrelationValue => true,
        };
        if !allowed {
            return Err(ERROR_INVALID_INPUT);
        }
        match field {
            CryptoByteField::IvCounterNonce => builder.iv_counter_nonce = Some(bytes),
            CryptoByteField::AuthenticatedEncryptionAdditionalData => {
                builder.authenticated_encryption_additional_data = Some(bytes);
            }
            CryptoByteField::AuthenticatedEncryptionTag => {
                builder.authenticated_encryption_tag = Some(bytes);
            }
            CryptoByteField::DigestedData => builder.digested_data = Some(bytes),
            CryptoByteField::SignatureData => builder.signature_data = Some(bytes),
            CryptoByteField::MacData => builder.mac_data = Some(bytes),
            CryptoByteField::CorrelationValue => builder.correlation_value = Some(bytes),
        }
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_encrypt_builder_create(
    out_builder: *mut *mut kmipkit_crypto_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(create_crypto_builder(
            CryptoOperationKind::Encrypt,
            out_builder
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_decrypt_builder_create(
    out_builder: *mut *mut kmipkit_crypto_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(create_crypto_builder(
            CryptoOperationKind::Decrypt,
            out_builder
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_sign_builder_create(
    out_builder: *mut *mut kmipkit_crypto_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(create_crypto_builder(
            CryptoOperationKind::Sign,
            out_builder
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_signature_verify_builder_create(
    out_builder: *mut *mut kmipkit_crypto_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(create_crypto_builder(
            CryptoOperationKind::SignatureVerify,
            out_builder,
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_mac_builder_create(
    out_builder: *mut *mut kmipkit_crypto_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(create_crypto_builder(CryptoOperationKind::Mac, out_builder));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_mac_verify_builder_create(
    out_builder: *mut *mut kmipkit_crypto_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(create_crypto_builder(
            CryptoOperationKind::MacVerify,
            out_builder
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_hash_builder_create(
    out_builder: *mut *mut kmipkit_crypto_operation_builder_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(create_crypto_builder(
            CryptoOperationKind::Hash,
            out_builder
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_unique_identifier(
    builder: *mut kmipkit_crypto_operation_builder_t,
    identifier: *const kmipkit_unique_identifier_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let identifier = try_ffi!(copy_optional_identifier(identifier));
        let Some(identifier) = identifier else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        try_ffi!(update_crypto_builder(builder, |builder| {
            if builder.operation == CryptoOperationKind::Hash {
                return Err(ERROR_INVALID_INPUT);
            }
            builder.unique_identifier = Some(identifier);
            Ok(())
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_data(
    builder: *mut kmipkit_crypto_operation_builder_t,
    data: *const kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        let bytes = try_ffi!(copy_builder_span(data));
        try_ffi!(update_crypto_builder(builder, |builder| {
            builder.data = Some(CryptoBuilderData::ByteString(bytes));
            Ok(())
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_data_value(
    builder: *mut kmipkit_crypto_operation_builder_t,
    data: *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let data = try_ffi!(copy_builder_data(data));
        try_ffi!(update_crypto_builder(builder, |builder| {
            builder.data = Some(data);
            Ok(())
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_cryptographic_parameters(
    builder: *mut kmipkit_crypto_operation_builder_t,
    parameters: *mut kmipkit_cryptographic_parameters_t,
) -> i32 {
    ffi_entrypoint(|| {
        let handle = try_ffi!(reference_handle(parameters, Kind::CryptographicParameters));
        let HandleValue::CryptographicParameters(parameters) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let structure =
            try_ffi!(parameters.with_ttlv(|view| view.try_clone().map_err(model_status)));
        try_ffi!(update_crypto_builder(builder, |builder| {
            builder.cryptographic_parameters = Some(structure);
            Ok(())
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_cryptographic_parameters_structure(
    builder: *mut kmipkit_crypto_operation_builder_t,
    parameters: *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let structure = try_ffi!(copy_required_structure_handle(parameters));
        let parameters = try_ffi!(
            CryptographicParameters::try_from_ttlv(structure).map_err(|_| ERROR_INVALID_INPUT)
        );
        let structure = parameters.into_ttlv();
        try_ffi!(update_crypto_builder(builder, |builder| {
            builder.cryptographic_parameters = Some(structure);
            Ok(())
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_iv_counter_nonce(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: *const kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(set_crypto_bytes(
            builder,
            CryptoByteField::IvCounterNonce,
            value
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_authenticated_encryption_additional_data(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: *const kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(set_crypto_bytes(
            builder,
            CryptoByteField::AuthenticatedEncryptionAdditionalData,
            value,
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_authenticated_encryption_tag(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: *const kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(set_crypto_bytes(
            builder,
            CryptoByteField::AuthenticatedEncryptionTag,
            value,
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_digested_data(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: *const kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(set_crypto_bytes(
            builder,
            CryptoByteField::DigestedData,
            value
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_signature_data(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: *const kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(set_crypto_bytes(
            builder,
            CryptoByteField::SignatureData,
            value
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_mac_data(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: *const kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(set_crypto_bytes(builder, CryptoByteField::MacData, value));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_correlation_value(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: *const kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(set_crypto_bytes(
            builder,
            CryptoByteField::CorrelationValue,
            value
        ));
        SUCCESS
    })
}

fn set_crypto_indicator(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: u8,
    is_init: bool,
) -> FfiResult<()> {
    let value = match value {
        0 => None,
        1 => Some(false),
        2 => Some(true),
        _ => return Err(ERROR_INVALID_INPUT),
    };
    update_crypto_builder(builder, |builder| {
        if is_init {
            builder.init_indicator = value;
        } else {
            builder.final_indicator = value;
        }
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_init_indicator(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: u8,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(set_crypto_indicator(builder, value, true));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_final_indicator(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: u8,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(set_crypto_indicator(builder, value, false));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_set_asynchronous_indicator(
    builder: *mut kmipkit_crypto_operation_builder_t,
    value: u32,
) -> i32 {
    ffi_entrypoint(|| {
        const ENUMERATION_EXTENSION_MIN: u32 = 0x8000_0000;
        const ENUMERATION_EXTENSION_MAX: u32 = 0x8fff_ffff;
        const ASYNCHRONOUS_INDICATOR_MIN: u32 =
            kmipkit_protocol::named_values_generated::values::enumeration::asynchronous_indicator::MANDATORY;
        const ASYNCHRONOUS_INDICATOR_MAX: u32 =
            kmipkit_protocol::named_values_generated::values::enumeration::asynchronous_indicator::PROHIBITED;
        let assigned = (ASYNCHRONOUS_INDICATOR_MIN..=ASYNCHRONOUS_INDICATOR_MAX).contains(&value);
        let extension = (ENUMERATION_EXTENSION_MIN..=ENUMERATION_EXTENSION_MAX).contains(&value);
        if !assigned && !extension {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        try_ffi!(update_crypto_builder(builder, |builder| {
            builder.asynchronous_indicator = Some(value);
            Ok(())
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_build_batch_item(
    builder: *mut kmipkit_crypto_operation_builder_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let handle = read_handle!(builder, CryptoOperationBuilder);
        let HandleValue::CryptoOperationBuilder(builder) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(builder) = builder.lock() else {
            return record_ffi_error(ERROR_INTERNAL);
        };
        let request = match builder.build_request() {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::new(request))),
        ));
        SUCCESS
    })
}

fn execute_crypto_builder(
    builder: *mut kmipkit_crypto_operation_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    options: Option<&kmipkit_request_options_v1_t>,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    let handle = read_handle!(builder, CryptoOperationBuilder);
    let HandleValue::CryptoOperationBuilder(builder) = &handle.value else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let Ok(builder) = builder.lock() else {
        return record_ffi_error(ERROR_INTERNAL);
    };
    let request = match builder.build_request() {
        Ok(request) => request,
        Err(status) => return record_ffi_error(status),
    };
    let asynchronous_indicator = builder.asynchronous_indicator;
    let options = match options {
        Some(options) => match request_options_from_v1(options) {
            Ok(options) => options,
            Err(status) => return record_ffi_error(status),
        },
        None => kmipkit_client::RequestOptions::default(),
    };
    fluent::execute_fluent_item(
        client,
        request,
        limits,
        &options,
        asynchronous_indicator,
        out_response,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_execute_v1(
    builder: *mut kmipkit_crypto_operation_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = out_slot!(out_response);
        execute_crypto_builder(builder, client, limits, None, out_response)
    })
}

/// Executes one typed C fluent cryptographic request with exchange-local options.
///
/// # Safety
///
/// `options` must be null or point to a readable v1 options record. The
/// builder, client, limits, and output slot must be valid opaque handles or
/// null where their existing ABI contracts permit it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_crypto_operation_builder_execute_with_options_v1(
    builder: *mut kmipkit_crypto_operation_builder_t,
    client: *mut kmipkit_client_t,
    limits: *mut kmipkit_codec_limits_t,
    options: *const kmipkit_request_options_v1_t,
    out_response: *mut *mut kmipkit_client_batch_response_t,
) -> i32 {
    ffi_entrypoint(|| {
        let _out = out_slot!(out_response);
        if options.is_null() || !options.is_aligned() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the non-null aligned record is borrowed for this synchronous execution call.
        let options = unsafe { &*options };
        execute_crypto_builder(builder, client, limits, Some(options), out_response)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_crypto_operation_builder_release(
    builder: *mut kmipkit_crypto_operation_builder_t,
) {
    ffi_entrypoint(|| release(builder, Kind::CryptoOperationBuilder));
}
