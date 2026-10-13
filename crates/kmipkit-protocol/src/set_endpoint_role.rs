//! Typed OASIS KMIP v2.1 §6.1.54, Tables 331–333 policy payloads.
use kmipkit_ttlv::{Structure,ValueView,Value};
use crate::{KmipOperationResult,ProtocolError,PolicyControlsError,ResponseBatchItemView};
use crate::policy_controls as common;
/// Client-to-server role request, Table 331 and §11.39.
#[derive(Debug)]
pub struct SetEndpointRoleRequest{endpoint_role:u32}
impl SetEndpointRoleRequest{
 /// Preserves an open Endpoint Role Enumeration.
 #[must_use]
 pub const fn new(endpoint_role:u32)->Self{Self{endpoint_role}}
 /// Returns the explicit raw role.
 #[must_use]
 pub const fn endpoint_role(&self)->u32{self.endpoint_role}
 /// Encodes the required Enumeration.
 /// # Errors
 /// Returns a sanitized TTLV model error.
 pub fn to_ttlv_payload(&self)->Result<Structure,ProtocolError>{let mut p=Structure::new();common::push(&mut p,0x420151,Value::enumeration(self.endpoint_role))?;Ok(p)}
}

/// Typed operation result; unknown generic fields remain in the response tree.
pub struct SetEndpointRoleResponse{result:KmipOperationResult,endpoint_role:Option<u32>}
impl SetEndpointRoleResponse{
 /// Converts a validated response item using the exact success schema.
 /// # Errors
 /// Returns a payload-free error for wrong operation, result or required fields.
 pub fn try_from_response_item(item:ResponseBatchItemView<'_>)->Result<Self,PolicyControlsError>{
  let result=common::result(item,50)?;
  let endpoint_role=if result.status().raw()==0{
 Some(item.with_response_payload(|p|common::required(&p,0x420151).and_then(|f|f.with_value(|v|match v{ValueView::Enumeration(v)=>Some(*v),_=>None}))).flatten().ok_or(PolicyControlsError::MalformedSuccessPayload)?)
 }else{None};
  Ok(Self{result,endpoint_role})
 }
 /// Returns the complete KMIP result.
 #[must_use]
 pub const fn result(&self)->&KmipOperationResult{&self.result}
 /// Returns the accepted open role on Success.
 #[must_use]
 pub const fn endpoint_role(&self)->Option<u32>{self.endpoint_role}
}
impl std::fmt::Debug for SetEndpointRoleResponse{
 fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.debug_struct("SetEndpointRoleResponse").field("result",&self.result).finish_non_exhaustive()}
}
