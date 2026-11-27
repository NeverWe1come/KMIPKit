//! Fluent builders for KMIP 2.1 session authentication and endpoint role operations.

use kmipkit_protocol::{
    DelegatedLoginRequest, DelegatedLoginResponse, EndpointRole, LoginRequest, LoginResponse,
    LogoutRequest, LogoutResponse, SetEndpointRoleRequest, SetEndpointRoleResponse, Ticket,
};
use kmipkit_ttlv::Structure;
use kmipkit_ttlv::codec::CodecLimits;

use crate::{
    Client, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome, ClientError,
    ClientRequest,
};

use super::execution::{
    FluentBuildError, FluentBuilderExecution, fluent_client_validation_error,
};

/// A fluent builder for KMIP Login requests (§6.1.30, Table 253).
pub struct LoginBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    request: LoginRequest,
}

impl<'a> LoginBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            request: LoginRequest::new(),
        }
    }

    /// Sets Lease Time using the Interval form (in seconds).
    #[must_use]
    pub fn lease_time_interval(mut self, seconds: u32) -> Self {
        self.request = self.request.with_lease_time_interval(seconds);
        self
    }

    /// Sets Lease Time using the Date Time form (Unix seconds).
    #[must_use]
    pub fn lease_time_date_time(mut self, unix_seconds: i64) -> Self {
        self.request = self.request.with_lease_time_date_time(unix_seconds);
        self
    }

    /// Sets the optional Request Count integer.
    #[must_use]
    pub fn request_count(mut self, count: i32) -> Self {
        self.request = self.request.with_request_count(count);
        self
    }

    /// Sets the optional generic Usage Limits structure.
    #[must_use]
    pub fn usage_limits(mut self, usage_limits: Structure) -> Self {
        self.request = self.request.with_usage_limits(usage_limits);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates the typed Login request locally (always valid).
    ///
    /// # Errors
    ///
    /// This method is infallible and always returns `Ok(())`.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        Ok(())
    }

    /// Builds the typed [`LoginRequest`] without sending it.
    #[must_use]
    pub fn build(self) -> LoginRequest {
        self.request
    }

    /// Converts Login to a typed [`ClientBatchItem`].
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::Login(self.request))
    }

    /// Executes one Login exchange and returns the unwrapped [`LoginResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<LoginResponse, ClientError> {
        self.execution.run_typed(
            ClientRequest::Login(self.request),
            |outcome| match outcome {
                ClientBatchOutcome::Login(resp) => Some(resp),
                _ => None,
            },
            LoginResponse::result,
        )
    }

    /// Executes one Login exchange and returns the raw [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution.run_outcome(ClientRequest::Login(self.request))
    }

    /// Executes one Login exchange with explicit [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution.run(ClientRequest::Login(self.request), limits)
    }
}

/// A fluent builder for KMIP Logout requests (§6.1.31, Table 256).
pub struct LogoutBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    ticket: Ticket,
}

impl<'a> LogoutBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, ticket: Ticket) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            ticket,
        }
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates the typed Logout request locally (always valid).
    ///
    /// # Errors
    ///
    /// This method is infallible and always returns `Ok(())`.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        Ok(())
    }

    /// Builds the typed [`LogoutRequest`] without sending it.
    #[must_use]
    pub fn build(self) -> LogoutRequest {
        LogoutRequest::new(self.ticket)
    }

    /// Converts Logout to a typed [`ClientBatchItem`].
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::Logout(self.build()))
    }

    /// Executes one Logout exchange and returns the unwrapped [`LogoutResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<LogoutResponse, ClientError> {
        self.execution.run_typed(
            ClientRequest::Logout(LogoutRequest::new(self.ticket)),
            |outcome| match outcome {
                ClientBatchOutcome::Logout(resp) => Some(resp),
                _ => None,
            },
            LogoutResponse::result,
        )
    }

    /// Executes one Logout exchange and returns the raw [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution
            .run_outcome(ClientRequest::Logout(LogoutRequest::new(self.ticket)))
    }

    /// Executes one Logout exchange with explicit [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution
            .run(ClientRequest::Logout(LogoutRequest::new(self.ticket)), limits)
    }
}

/// A fluent builder for KMIP Delegated Login requests (§6.1.12, Table 199).
pub struct DelegatedLoginBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    rights: Option<Structure>,
    lease_time_interval: Option<u32>,
    lease_time_date_time: Option<i64>,
    request_count: Option<i32>,
    usage_limits: Option<Structure>,
}

impl<'a> DelegatedLoginBuilder<'a> {
    pub(super) fn new(client: &'a mut Client) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            rights: None,
            lease_time_interval: None,
            lease_time_date_time: None,
            request_count: None,
            usage_limits: None,
        }
    }

    /// Sets the required Rights Structure for delegated login.
    #[must_use]
    pub fn rights(mut self, rights: Structure) -> Self {
        self.rights = Some(rights);
        self
    }

    /// Sets Lease Time using the Interval form (in seconds).
    #[must_use]
    pub const fn lease_time_interval(mut self, seconds: u32) -> Self {
        self.lease_time_interval = Some(seconds);
        self.lease_time_date_time = None;
        self
    }

    /// Sets Lease Time using the Date Time form (Unix seconds).
    #[must_use]
    pub const fn lease_time_date_time(mut self, unix_seconds: i64) -> Self {
        self.lease_time_date_time = Some(unix_seconds);
        self.lease_time_interval = None;
        self
    }

    /// Sets the optional Request Count integer.
    #[must_use]
    pub const fn request_count(mut self, count: i32) -> Self {
        self.request_count = Some(count);
        self
    }

    /// Sets the optional generic Usage Limits structure.
    #[must_use]
    pub fn usage_limits(mut self, usage_limits: Structure) -> Self {
        self.usage_limits = Some(usage_limits);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates required builder fields without consuming `self`.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::MissingDelegatedLoginRights`] if Rights is unset.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        if self.rights.is_none() {
            return Err(FluentBuildError::MissingDelegatedLoginRights);
        }
        Ok(())
    }

    fn into_request_and_execution(
        self,
    ) -> Result<(DelegatedLoginRequest, FluentBuilderExecution<'a>), FluentBuildError> {
        let rights = self
            .rights
            .ok_or(FluentBuildError::MissingDelegatedLoginRights)?;
        let mut request = DelegatedLoginRequest::new(rights);
        if let Some(seconds) = self.lease_time_interval {
            request = request.with_lease_time_interval(seconds);
        } else if let Some(unix_seconds) = self.lease_time_date_time {
            request = request.with_lease_time_date_time(unix_seconds);
        }
        if let Some(count) = self.request_count {
            request = request.with_request_count(count);
        }
        if let Some(usage_limits) = self.usage_limits {
            request = request.with_usage_limits(usage_limits);
        }
        Ok((request, self.execution))
    }

    /// Builds the typed [`DelegatedLoginRequest`] without sending it.
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError::MissingDelegatedLoginRights`] if Rights is unset.
    pub fn build(self) -> Result<DelegatedLoginRequest, FluentBuildError> {
        self.into_request_and_execution().map(|(req, _)| req)
    }

    /// Converts Delegated Login to a typed [`ClientBatchItem`].
    ///
    /// # Errors
    ///
    /// Returns [`FluentBuildError`] if required fields are missing.
    pub fn to_batch_item(self) -> Result<ClientBatchItem, FluentBuildError> {
        self.build()
            .map(|req| ClientBatchItem::new(ClientRequest::DelegatedLogin(req)))
    }

    /// Executes one Delegated Login exchange and returns [`DelegatedLoginResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<DelegatedLoginResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_typed(
            ClientRequest::DelegatedLogin(req),
            |outcome| match outcome {
                ClientBatchOutcome::DelegatedLogin(resp) => Some(resp),
                _ => None,
            },
            DelegatedLoginResponse::result,
        )
    }

    /// Executes one Delegated Login exchange and returns [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run_outcome(ClientRequest::DelegatedLogin(req))
    }

    /// Executes one Delegated Login exchange with explicit [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns validation, exchange, or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        let (req, execution) = self
            .into_request_and_execution()
            .map_err(fluent_client_validation_error)?;
        execution.run(ClientRequest::DelegatedLogin(req), limits)
    }
}

/// A fluent builder for KMIP Set Endpoint Role requests (§6.1.54, Table 331).
pub struct SetEndpointRoleBuilder<'a> {
    execution: FluentBuilderExecution<'a>,
    request: SetEndpointRoleRequest,
}

impl<'a> SetEndpointRoleBuilder<'a> {
    pub(super) fn new(client: &'a mut Client, role: EndpointRole) -> Self {
        Self {
            execution: FluentBuilderExecution::new(client),
            request: SetEndpointRoleRequest::new(role),
        }
    }

    /// Sets the optional Vendor Identification text string.
    #[must_use]
    pub fn vendor_identification(mut self, vendor_id: &str) -> Self {
        self.request = self.request.with_vendor_identification(vendor_id);
        self
    }

    fluent_option_field_setters!(
        "Sets exchange-local options without changing client policy.";
        execution.options;
        execution.asynchronous_indicator;
        const
    );

    /// Validates the typed Set Endpoint Role request locally (always valid).
    ///
    /// # Errors
    ///
    /// This method is infallible and always returns `Ok(())`.
    pub const fn validate(&self) -> Result<(), FluentBuildError> {
        Ok(())
    }

    /// Builds the typed [`SetEndpointRoleRequest`] without sending it.
    #[must_use]
    pub fn build(self) -> SetEndpointRoleRequest {
        self.request
    }

    /// Converts Set Endpoint Role to a typed [`ClientBatchItem`].
    #[must_use]
    pub fn to_batch_item(self) -> ClientBatchItem {
        ClientBatchItem::new(ClientRequest::SetEndpointRole(self.request))
    }

    /// Executes one Set Endpoint Role exchange and returns [`SetEndpointRoleResponse`].
    ///
    /// # Errors
    ///
    /// Returns a validation, protocol, transport, `ServerResult`, or `Pending` error.
    pub fn execute(self) -> Result<SetEndpointRoleResponse, ClientError> {
        self.execution.run_typed(
            ClientRequest::SetEndpointRole(self.request),
            |outcome| match outcome {
                ClientBatchOutcome::SetEndpointRole(resp) => Some(resp),
                _ => None,
            },
            SetEndpointRoleResponse::result,
        )
    }

    /// Executes one Set Endpoint Role exchange and returns [`ClientBatchItemResponse`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution
            .run_outcome(ClientRequest::SetEndpointRole(self.request))
    }

    /// Executes one Set Endpoint Role exchange with explicit [`CodecLimits`].
    ///
    /// # Errors
    ///
    /// Returns exchange or decoding failures.
    pub fn execute_outcome_with_limits(
        self,
        limits: &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> {
        self.execution
            .run(ClientRequest::SetEndpointRole(self.request), limits)
    }
}
