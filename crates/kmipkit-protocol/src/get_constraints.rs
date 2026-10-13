//! Typed OASIS KMIP v2.1 §6.1.22, Tables 229–231 policy payloads.
use kmipkit_ttlv::{Structure,StructureView,ValueView};
use crate::{KmipOperationResult,ProtocolError,PolicyControlsError,ResponseBatchItemView};
use crate::policy_controls as common;
/// Empty Get Constraints request, Table 229.
#[derive(Debug,Default)]
pub struct GetConstraintsRequest;
impl GetConstraintsRequest {
 /// Creates the empty request.
 #[must_use]
 pub const fn new()->Self{Self}
 /// Builds the empty payload.
 /// # Errors
 /// Kept fallible for consistency with typed operation encoding.
 pub fn to_ttlv_payload(&self)->Result<Structure,ProtocolError>{Ok(Structure::new())}
}

/// Typed operation result; unknown generic fields remain in the response tree.
pub struct GetConstraintsResponse{result:KmipOperationResult,constraints:Option<Structure>}
impl GetConstraintsResponse{
 /// Converts a validated response item using the exact success schema.
 /// # Errors
 /// Returns a payload-free error for wrong operation, result or required fields.
 pub fn try_from_response_item(item:ResponseBatchItemView<'_>)->Result<Self,PolicyControlsError>{
  let result=common::result(item,56)?;
  let constraints=if result.status().raw()==0{
 Some(item.with_response_payload(|p|common::required(&p,0x420168).and_then(|f|f.with_value(|v|match v{ValueView::Structure(v)=>v.try_clone().ok(),_=>None}))).flatten().ok_or(PolicyControlsError::MalformedSuccessPayload)?)
 }else{None};
  Ok(Self{result,constraints})
 }
 /// Returns the complete KMIP result.
 #[must_use]
 pub const fn result(&self)->&KmipOperationResult{&self.result}
 /// Lends the required ordered Constraints on Success.
 pub fn with_constraints<R>(&self,callback:impl for<'a> FnOnce(StructureView<'a>)->R)->Option<R>{self.constraints.as_ref().map(|v|callback(v.view()))}
}
impl std::fmt::Debug for GetConstraintsResponse{
 fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.debug_struct("GetConstraintsResponse").field("result",&self.result).finish_non_exhaustive()}
}
