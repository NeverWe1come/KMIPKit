//! Typed OASIS KMIP v2.1 §6.1.52, Tables 325–327 policy payloads.
use kmipkit_ttlv::{Structure,Value};
use crate::{KmipOperationResult,ProtocolError,PolicyControlsError,ResponseBatchItemView};
use crate::policy_controls as common;
/// Explicit Constraints structure, Table 325 and §7.7.
#[derive(Debug)]
pub struct SetConstraintsRequest{constraints:Structure}
impl SetConstraintsRequest{
 /// Supplies the required ordered structure.
 #[must_use]
 pub const fn new(constraints:Structure)->Self{Self{constraints}}
 /// Encodes required Constraints without filtering nested members.
 /// # Errors
 /// Returns a sanitized model or nesting error.
 pub fn to_ttlv_payload(&self)->Result<Structure,ProtocolError>{
  let mut p=Structure::new();
  common::push(&mut p,0x420168,Value::structure(self.constraints.view().try_clone().map_err(common::model_error)?))?;
  Ok(p)
 }
}

/// Typed operation result; unknown generic fields remain in the response tree.
pub struct SetConstraintsResponse{result:KmipOperationResult}
impl SetConstraintsResponse{
 /// Converts a validated response item using the exact success schema.
 /// # Errors
 /// Returns a payload-free error for wrong operation, result or required fields.
 pub fn try_from_response_item(item:ResponseBatchItemView<'_>)->Result<Self,PolicyControlsError>{
  let result=common::result(item,55)?;
  if result.status().raw()==0 && item.with_response_payload(|p|p.children().is_empty())!=Some(true){return Err(PolicyControlsError::MalformedSuccessPayload)}
  Ok(Self{result})
 }
 /// Returns the complete KMIP result.
 #[must_use]
 pub const fn result(&self)->&KmipOperationResult{&self.result}
 
}
impl std::fmt::Debug for SetConstraintsResponse{
 fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.debug_struct("SetConstraintsResponse").field("result",&self.result).finish_non_exhaustive()}
}
