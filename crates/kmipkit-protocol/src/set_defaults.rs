//! Typed OASIS KMIP v2.1 §6.1.53, Tables 328–330 policy payloads.
use crate::policy_controls as common;
use crate::{KmipOperationResult, PolicyControlsError, ProtocolError, ResponseBatchItemView};
use kmipkit_ttlv::{Structure, Value};
/// Explicit defaults change, Table 328 and §7.11.
/// Omission requests removal of all Object Defaults.
#[derive(Debug)]
pub struct SetDefaultsRequest {
    defaults_information: Option<Structure>,
}
impl SetDefaultsRequest {
    /// Preserves omission separately from a present empty structure.
    #[must_use]
    pub const fn new(defaults_information: Option<Structure>) -> Self {
        Self {
            defaults_information,
        }
    }
    /// Encodes supplied defaults only when present.
    /// # Errors
    /// Returns a sanitized model or nesting error.
    pub fn to_ttlv_payload(&self) -> Result<Structure, ProtocolError> {
        let mut p = Structure::new();
        if let Some(v) = &self.defaults_information {
            common::push(
                &mut p,
                0x420152,
                Value::structure(v.view().try_clone().map_err(common::model_error)?),
            )?;
        }
        Ok(p)
    }
}

/// Typed operation result; unknown generic fields remain in the response tree.
pub struct SetDefaultsResponse {
    result: KmipOperationResult,
}
impl SetDefaultsResponse {
    /// Converts a validated response item using the exact success schema.
    /// # Errors
    /// Returns a payload-free error for wrong operation, result or required fields.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, PolicyControlsError> {
        let result = common::result(item, 54)?;
        if result.status().raw() == 0
            && item.with_response_payload(|p| p.children().is_empty()) != Some(true)
        {
            return Err(PolicyControlsError::MalformedSuccessPayload);
        }
        Ok(Self { result })
    }
    /// Returns the complete KMIP result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }
}
impl std::fmt::Debug for SetDefaultsResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetDefaultsResponse")
            .field("result", &self.result)
            .finish_non_exhaustive()
    }
}
