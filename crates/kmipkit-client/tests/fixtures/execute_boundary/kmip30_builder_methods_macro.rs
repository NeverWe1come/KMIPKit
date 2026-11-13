macro_rules! impl_kmip30_builder_methods {
    ($builder:ident) => {
        impl<'a> $builder<'a> {
            /// Sets one-exchange options, including a caller-selected protocol version.
            #[must_use]
            pub fn with_options(mut self, options: RequestOptions) -> Self {
                self.execution().options = Some(options);
                self
            }

            /// Sets the Request Header Asynchronous Indicator for this exchange.
            #[must_use]
            pub fn with_asynchronous_indicator(mut self, value: u32) -> Self {
                self.execution().asynchronous_indicator = Some(value);
                self
            }

            /// Validates the semantic inputs held by the builder without sending.
            /// The selected protocol version and typed request are checked again before transport.
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

            /// Executes exactly one typed exchange.
            ///
            /// # Errors
            /// Returns the existing local validation, version, transport, or response error.
            pub fn execute(
                self,
                limits: &kmipkit_ttlv::codec::CodecLimits,
            ) -> Result<ClientBatchItemResponse, ClientError> {
                let (execution, request) = self.into_parts();
                execution.execute(request, limits)
            }
        }
    };
}

