//! Explicit caller-owned local accounting for KMIP Usage Allocation.
//!
//! OASIS KMIP Specification v2.1 §6.1.23 defines allocation and protection
//! obligations. This context tracks only reservations explicitly routed
//! through this instance; it does not observe or claim remote consumption.

use std::error::Error;
use std::fmt;

use kmipkit_protocol::{GetUsageAllocationResponse, UniqueIdentifier};
use kmipkit_transport::RequestDeliveryState;

const SUCCESS: u32 = 0;
const ID_PLACEHOLDER: u32 = 1;

pub(crate) const fn is_id_placeholder(identifier: &UniqueIdentifier) -> bool {
    matches!(identifier, UniqueIdentifier::Enumeration(ID_PLACEHOLDER))
}

/// Caller-owned local accounting for one successful Get Usage Allocation.
///
/// Construct this value with the exact requested allocation count, the
/// caller-known open Usage Limits Unit, and the server-resolved identifier
/// from a successful response. It is deliberately not `Clone`: duplicating
/// local allowance would permit the combined contexts to exceed the count.
///
/// The context only tracks guarded requests made through this instance in the
/// current process. It does not know remote consumption or uses from ordinary
/// client calls, other contexts, or external clients.
pub struct UsageAllocationContext {
    unique_identifier: UniqueIdentifier,
    usage_limits_unit: u32,
    allocated_units: i64,
    reserved_units: i64,
}

impl UsageAllocationContext {
    /// Creates a context from a complete successful allocation response and
    /// caller-known request and object metadata.
    ///
    /// `requested_units` must be the exact positive Usage Limits Count sent in
    /// the request. The response does not echo that count or the object's
    /// Usage Limits Unit, so the caller supplies both explicitly. The returned
    /// response identifier binds the context to the server-resolved object.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error if the response is not Success, has no
    /// resolved identifier, or the supplied allocation count is not positive.
    pub fn from_successful_allocation(
        response: &GetUsageAllocationResponse,
        requested_units: i64,
        usage_limits_unit: u32,
    ) -> Result<Self, UsageAllocationContextError> {
        if response.result().status().raw() != SUCCESS {
            return Err(UsageAllocationContextError::AllocationNotSuccessful);
        }
        if requested_units <= 0 {
            return Err(UsageAllocationContextError::InvalidAllocationCount);
        }
        let unique_identifier = response
            .unique_identifier()
            .cloned()
            .ok_or(UsageAllocationContextError::MissingResolvedIdentifier)?;
        if is_id_placeholder(&unique_identifier) {
            return Err(UsageAllocationContextError::MissingResolvedIdentifier);
        }

        Ok(Self {
            unique_identifier,
            usage_limits_unit,
            allocated_units: requested_units,
            reserved_units: 0,
        })
    }

    /// Returns the concrete server-resolved object identifier.
    #[must_use]
    pub const fn unique_identifier(&self) -> &UniqueIdentifier {
        &self.unique_identifier
    }

    /// Returns the caller-supplied open Usage Limits Unit Enumeration value.
    #[must_use]
    pub const fn usage_limits_unit(&self) -> u32 {
        self.usage_limits_unit
    }

    /// Returns the exact caller-supplied allocation count.
    #[must_use]
    pub const fn allocated_units(&self) -> i64 {
        self.allocated_units
    }

    /// Returns allocation units not currently reserved through this context.
    #[must_use]
    pub const fn available_units(&self) -> i64 {
        self.allocated_units - self.reserved_units
    }

    /// Returns units reserved locally through this context.
    ///
    /// A reservation is not evidence that the server consumed those units.
    #[must_use]
    pub const fn reserved_units(&self) -> i64 {
        self.reserved_units
    }

    pub(crate) fn reserve(&mut self, cost: i64) -> Result<(), UsageAllocationContextError> {
        if cost <= 0 {
            return Err(UsageAllocationContextError::InvalidUsageCost);
        }
        let next_reserved = self
            .reserved_units
            .checked_add(cost)
            .ok_or(UsageAllocationContextError::ReservationOverflow)?;
        if next_reserved > self.allocated_units {
            return Err(UsageAllocationContextError::CostExceedsAvailable);
        }
        self.reserved_units = next_reserved;
        Ok(())
    }

    /// Releases only when the client has positive evidence that no request was sent.
    pub(crate) fn settle_after_error(
        &mut self,
        cost: i64,
        delivery_state: Option<RequestDeliveryState>,
    ) {
        if delivery_state == Some(RequestDeliveryState::NotSent) {
            self.release_not_sent(cost);
        }
    }

    fn release_not_sent(&mut self, cost: i64) {
        if let Some(remaining) = self.reserved_units.checked_sub(cost) {
            self.reserved_units = remaining;
        }
    }
}

impl fmt::Debug for UsageAllocationContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UsageAllocationContext")
            .field("has_unique_identifier", &true)
            .field("usage_limits_unit", &self.usage_limits_unit)
            .field("allocated_units", &self.allocated_units)
            .field("reserved_units", &self.reserved_units)
            .finish_non_exhaustive()
    }
}

/// Safe failure categories for constructing or reserving a usage context.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageAllocationContextError {
    /// The allocation response did not have KMIP Result Status Success.
    AllocationNotSuccessful,
    /// The supplied Usage Limits Count was zero or negative.
    InvalidAllocationCount,
    /// The successful response did not contain a resolved Unique Identifier.
    MissingResolvedIdentifier,
    /// A protected operation cost was zero or negative.
    InvalidUsageCost,
    /// The requested cost is greater than this context's available local units.
    CostExceedsAvailable,
    /// Adding a reservation would overflow the local integer representation.
    ReservationOverflow,
    /// The typed operation is outside the guarded Encrypt/MAC/Sign scope.
    UnsupportedOperation,
    /// A guarded request omitted Unique Identifier.
    MissingUniqueIdentifier,
    /// The guarded request identifier does not exactly match this context.
    MismatchedUniqueIdentifier,
}

impl fmt::Display for UsageAllocationContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocationNotSuccessful => {
                formatter.write_str("usage allocation response is not successful")
            }
            Self::InvalidAllocationCount => {
                formatter.write_str("usage allocation count must be positive")
            }
            Self::MissingResolvedIdentifier => {
                formatter.write_str("usage allocation response has no resolved identifier")
            }
            Self::InvalidUsageCost => formatter.write_str("usage cost must be positive"),
            Self::CostExceedsAvailable => {
                formatter.write_str("usage cost exceeds locally available allocation")
            }
            Self::ReservationOverflow => {
                formatter.write_str("usage reservation exceeds local integer bounds")
            }
            Self::UnsupportedOperation => {
                formatter.write_str("operation is outside the guarded protection scope")
            }
            Self::MissingUniqueIdentifier => {
                formatter.write_str("guarded protection requires an explicit object identifier")
            }
            Self::MismatchedUniqueIdentifier => {
                formatter.write_str("guarded protection object does not match the allocation")
            }
        }
    }
}

impl Error for UsageAllocationContextError {}

/// Caller-owned local guard for a leased KMIP object obtained via `ObtainLease`.
///
/// Encapsulates the lease interval (in seconds) or absolute expiry timestamp,
/// ensuring that client-side operations verify lease validity before sending requests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseContext {
    unique_identifier: UniqueIdentifier,
    obtained_at_unix_secs: i64,
    lease_time_secs: i64,
}

impl LeaseContext {
    /// Creates a new `LeaseContext` from a unique identifier, the Unix timestamp (in seconds) when the lease was granted,
    /// and the lease duration (in seconds).
    #[must_use]
    pub const fn new(
        unique_identifier: UniqueIdentifier,
        obtained_at_unix_secs: i64,
        lease_time_secs: i64,
    ) -> Self {
        Self {
            unique_identifier,
            obtained_at_unix_secs,
            lease_time_secs,
        }
    }

    /// Returns the unique identifier of the leased object.
    #[must_use]
    pub const fn unique_identifier(&self) -> &UniqueIdentifier {
        &self.unique_identifier
    }

    /// Returns the absolute expiry Unix timestamp in seconds.
    #[must_use]
    pub const fn expires_at_unix_secs(&self) -> i64 {
        self.obtained_at_unix_secs
            .saturating_add(self.lease_time_secs)
    }

    /// Checks if the lease has expired given current time.
    #[must_use]
    pub const fn is_expired(&self, current_time_unix_secs: i64) -> bool {
        current_time_unix_secs >= self.expires_at_unix_secs()
    }

    /// Validates lease validity at `current_time_unix_secs`.
    ///
    /// # Errors
    /// Returns `LeaseExpired` error if the lease has elapsed.
    pub const fn check_valid(&self, current_time_unix_secs: i64) -> Result<(), LeaseContextError> {
        if self.is_expired(current_time_unix_secs) {
            Err(LeaseContextError::LeaseExpired)
        } else {
            Ok(())
        }
    }
}

/// Errors emitted by local lease validation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LeaseContextError {
    /// The lease duration has elapsed.
    LeaseExpired,
}

impl fmt::Display for LeaseContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LeaseExpired => formatter.write_str("object lease has expired"),
        }
    }
}

impl Error for LeaseContextError {}
