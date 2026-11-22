macro_rules! fluent_option_field_setters {
    ($option_doc:literal; $($options_path:ident).+; $($indicator_path:ident).+; const) => {
        pub fn with_options(mut self, options: RequestOptions) -> Self {
            self.$($options_path).+ = Some(options);
            Client::execute(self)
        }
    };
}

impl FluentBuilder {
    fluent_option_field_setters!("Sets exchange-local options."; options; asynchronous_indicator; const);
}
