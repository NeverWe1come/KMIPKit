macro_rules! impl_kmip30_builder_methods {
    ($builder:ident, $resp:ty, $variant:ident) => {
        impl<'a> $builder<'a> {
            /// Sets one-exchange options, including a caller-selected protocol version.
            #[must_use]
            pub fn with_options(mut self, options: RequestOptions) -> Self {
                self.execution().options = Some(options);
                self
            }

            /// Sets the KMIP protocol version override for this request/exchange.
            #[must_use]
            pub fn with_protocol_version(mut self, version: ProtocolVersion) -> Self {
                let exec = self.execution();
                let options = exec
                    .options
                    .unwrap_or_default()
                    .with_protocol_version(version);
                exec.options = Some(options);
                self
            }

            /// Sets the KMIP protocol version override (alias for [`Self::with_protocol_version`]).
            #[must_use]
            pub fn with_kmip_version(self, version: ProtocolVersion) -> Self {
                self.with_protocol_version(version)
            }

            /// Sets the Request Header Asynchronous Indicator for this exchange.
            #[must_use]
            pub fn with_asynchronous_indicator(mut self, value: u32) -> Self {
                self.execution().asynchronous_indicator = Some(value);
                self
            }

            /// Validates the semantic inputs held by the builder without sending.
            ///
            /// # Errors
            ///
            /// This operation currently has no local failure case because the
            /// builder accepts only required, already typed request fields.
            pub fn validate(&self) -> Result<(), FluentBuildError> {
                Ok(())
            }

            /// Builds the existing typed request variant.
            #[must_use]
            pub fn build(self) -> ClientRequest {
                self.into_parts().1
            }

            /// Converts the request to one typed batch item for explicit batching.
            #[must_use]
            pub fn to_batch_item(self) -> ClientBatchItem {
                let (_, request) = self.into_parts();
                ClientBatchItem::new(request)
            }

            /// Executes exactly one typed exchange using the client's configured [`CodecLimits`]
            /// and returns the unwrapped response, erroring on server failure or `Pending`.
            ///
            /// # Errors
            /// Returns the local validation, version, transport, protocol, `ServerResult`, or `Pending` error.
            pub fn execute(self) -> Result<$resp, ClientError> {
                let item = self.execute_outcome()?;
                unwrap_typed_outcome(
                    item,
                    |outcome| match outcome {
                        ClientBatchOutcome::Kmip30(Kmip30OperationResponse::$variant(resp)) => {
                            Some(resp)
                        }
                        _ => None,
                    },
                    |resp| resp.result(),
                )
            }

            /// Executes exactly one typed exchange using the client's configured [`CodecLimits`]
            /// and returns the raw [`ClientBatchItemResponse`] envelope (`Pending` and failures included).
            ///
            /// # Errors
            /// Returns the local validation, version, transport, or response-decoding error.
            pub fn execute_outcome(self) -> Result<ClientBatchItemResponse, ClientError> {
                let (execution, request) = self.into_parts();
                execution.execute_outcome(request)
            }

            /// Executes exactly one typed exchange with explicit [`CodecLimits`] overrides
            /// and returns the raw [`ClientBatchItemResponse`] envelope.
            ///
            /// # Errors
            /// Returns the local validation, version, transport, or response-decoding error.
            pub fn execute_outcome_with_limits(
                self,
                limits: &CodecLimits,
            ) -> Result<ClientBatchItemResponse, ClientError> {
                let (execution, request) = self.into_parts();
                execution.execute_outcome_with_limits(request, limits)
            }
        }
    };
}
