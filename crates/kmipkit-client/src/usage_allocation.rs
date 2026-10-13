//! Explicit local reservations for OASIS KMIP v2.1 §6.1.23.

use std::fmt;
use kmipkit_protocol::{EncryptRequest, GetUsageAllocationResponse, MacRequest, SignRequest, UniqueIdentifier};
use kmipkit_transport::RequestDeliveryState;
use crate::{ClientCauseCategory, ClientError};

/// One caller-owned successful allocation, bound to one concrete object and unit.
///
/// This context cannot observe remote consumption, ordinary calls or other processes.
/// The caller supplies exact request count, known object unit and per-call cost, and
/// is responsible for requesting allocation during the known protection window.
pub struct UsageAllocationContext {
    unique_identifier: UniqueIdentifier,
    usage_limits_unit: u32,
    allocated_units: i32,
    locally_reserved_units: i32,
}

impl UsageAllocationContext {
    /// Creates a local context from a complete successful allocation.
    ///
    /// `requested_count` must be the exact positive Integer from that request.
    /// The response supplies neither this count nor the known object usage unit.
    ///
    /// # Errors
    /// Returns sanitized NotSent validation for non-success, nonpositive count
    /// or an unresolved placeholder identifier.
    pub fn from_allocation(response: &GetUsageAllocationResponse, requested_count: i32, usage_limits_unit: u32) -> Result<Self, ClientError> {
        if response.result().status().raw() != 0 || requested_count <= 0 {
            return Err(validation_error());
        }
        let identifier=response.unique_identifier().filter(|id| concrete(id)).ok_or_else(validation_error)?;
        Ok(Self { unique_identifier:identifier.clone(), usage_limits_unit, allocated_units:requested_count, locally_reserved_units:0 })
    }

    /// Returns the concrete allocation object identifier.
    #[must_use]
    pub const fn unique_identifier(&self) -> &UniqueIdentifier { &self.unique_identifier }
    /// Returns the caller-known open usage unit without conversion.
    #[must_use]
    pub const fn usage_limits_unit(&self) -> u32 { self.usage_limits_unit }
    /// Returns the exact requested allocation count.
    #[must_use]
    pub const fn allocated_units(&self) -> i32 { self.allocated_units }
    /// Returns conservative local reservations, without claiming remote consumption.
    #[must_use]
    pub const fn locally_reserved_units(&self) -> i32 { self.locally_reserved_units }
    /// Returns the unreserved local allowance.
    #[must_use]
    pub const fn available_units(&self) -> i32 { self.allocated_units - self.locally_reserved_units }

    pub(crate) fn reserve(&mut self, identifier: &UniqueIdentifier, cost: i32) -> Result<(), ClientError> {
        if !concrete(identifier) || identifier != &self.unique_identifier || cost <= 0 || cost > self.available_units() {
            return Err(validation_error());
        }
        self.locally_reserved_units=self.locally_reserved_units.checked_add(cost).ok_or_else(validation_error)?;
        Ok(())
    }

    pub(crate) fn release(&mut self, cost: i32) {
        // Only the executing call's existing positive reservation reaches here.
        self.locally_reserved_units -= cost;
    }
}

impl fmt::Debug for UsageAllocationContext {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
        f.debug_struct("UsageAllocationContext").field("usage_limits_unit",&self.usage_limits_unit).field("allocated_units",&self.allocated_units).field("locally_reserved_units",&self.locally_reserved_units).finish_non_exhaustive()
    }
}

/// The three typed protection operations admitted by the local usage guard.
/// No generic batch or raw byte input is accepted.
#[derive(Debug)]
pub enum GuardedProtectionRequest {
    /// Explicit server-side encryption.
    Encrypt(EncryptRequest),
    /// Explicit server-side MAC generation.
    Mac(MacRequest),
    /// Explicit server-side signature generation.
    Sign(SignRequest),
}

fn concrete(identifier: &UniqueIdentifier) -> bool {
    // §4.58: enumerations and integers reference batch placeholders.
    matches!(identifier,UniqueIdentifier::TextString(value) if !value.is_empty())
}

pub(crate) fn validation_error() -> ClientError {
    ClientError::Validation { cause:ClientCauseCategory::InvalidInput, delivery_state:RequestDeliveryState::NotSent }
}
