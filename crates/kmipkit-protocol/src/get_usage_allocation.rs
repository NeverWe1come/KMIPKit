//! Typed OASIS KMIP v2.1 §6.1.23, Tables 232–234 policy payloads.
use kmipkit_ttlv::{Structure,Value};
use crate::{KmipOperationResult,ProtocolError,PolicyControlsError,ResponseBatchItemView,UniqueIdentifier};
use crate::policy_controls as common;
/// Allocation request, Table 232; the caller knows the protection window.
pub struct GetUsageAllocationRequest{unique_identifier:Option<UniqueIdentifier>,usage_limits_count:i32}
impl GetUsageAllocationRequest{
 /// Preserves the explicit count and optional identifier without fetching metadata.
 #[must_use]
 pub const fn new(unique_identifier:Option<UniqueIdentifier>,usage_limits_count:i32)->Self{Self{unique_identifier,usage_limits_count}}
 /// Returns the exact requested count.
 #[must_use]
 pub const fn usage_limits_count(&self)->i32{self.usage_limits_count}
 /// Returns the optional object identifier.
 #[must_use]
 pub const fn unique_identifier(&self)->Option<&UniqueIdentifier>{self.unique_identifier.as_ref()}
 /// Encodes Table 232 in order with KMIP Integer count.
 /// # Errors
 /// Returns a sanitized TTLV model error.
 pub fn to_ttlv_payload(&self)->Result<Structure,ProtocolError>{
  let mut p=crate::lifecycle::request_payload(self.unique_identifier.as_ref())?;
  common::push(&mut p,0x420096,Value::integer(self.usage_limits_count))?;
  Ok(p)
 }
}
impl std::fmt::Debug for GetUsageAllocationRequest{
 fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("GetUsageAllocationRequest([REDACTED])")}
}

/// Typed operation result; unknown generic fields remain in the response tree.
pub struct GetUsageAllocationResponse{result:KmipOperationResult,unique_identifier:Option<UniqueIdentifier>}
impl GetUsageAllocationResponse{
 /// Converts a validated response item using the exact success schema.
 /// # Errors
 /// Returns a payload-free error for wrong operation, result or required fields.
 pub fn try_from_response_item(item:ResponseBatchItemView<'_>)->Result<Self,PolicyControlsError>{
  let result=common::result(item,17)?;
  let unique_identifier=if result.status().raw()==0{
 Some(item.with_response_payload(|p|crate::lifecycle::successful_response_identifier(&p)).flatten().ok_or(PolicyControlsError::MalformedSuccessPayload)?)
 }else{None};
  Ok(Self{result,unique_identifier})
 }
 /// Returns the complete KMIP result.
 #[must_use]
 pub const fn result(&self)->&KmipOperationResult{&self.result}
 /// Returns the required successful resolved identifier.
 #[must_use]
 pub const fn unique_identifier(&self)->Option<&UniqueIdentifier>{self.unique_identifier.as_ref()}
}
impl std::fmt::Debug for GetUsageAllocationResponse{
 fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.debug_struct("GetUsageAllocationResponse").field("result",&self.result).finish_non_exhaustive()}
}
