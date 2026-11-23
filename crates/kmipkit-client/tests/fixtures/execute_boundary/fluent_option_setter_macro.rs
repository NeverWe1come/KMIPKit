macro_rules! fluent_option_field_setters {
    ($option_doc:literal; $($options_path:ident).+; $($indicator_path:ident).+; const) => {
        #[doc = $option_doc]
        #[must_use]
        pub fn with_options(mut self, options: RequestOptions) -> Self {
            self.$($options_path).+ = Some(options);
            self
        }

        /// Sets the Request Header Asynchronous Indicator for this exchange.
        #[must_use]
        pub const fn with_asynchronous_indicator(mut self, value: u32) -> Self {
            self.$($indicator_path).+ = Some(value);
            self
        }
    };
    ($option_doc:literal; $($options_path:ident).+; $($indicator_path:ident).+; non_const) => {
        #[doc = $option_doc]
        #[must_use]
        pub fn with_options(mut self, options: RequestOptions) -> Self {
            self.$($options_path).+ = Some(options);
            self
        }

        /// Sets the Request Header Asynchronous Indicator for this exchange.
        #[must_use]
        pub fn with_asynchronous_indicator(mut self, value: u32) -> Self {
            self.$($indicator_path).+ = Some(value);
            self
        }
    };
}

impl FluentBuilder {
    fluent_option_field_setters!("Sets exchange-local options."; options; asynchronous_indicator; const);
    fluent_option_field_setters!("Sets batch-local options."; state.options; state.asynchronous_indicator; non_const);
}
