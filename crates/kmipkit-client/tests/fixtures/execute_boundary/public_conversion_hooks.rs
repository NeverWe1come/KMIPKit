pub struct ClientRequest;
pub trait CallerDefinedConversion {}
pub struct Client;

impl Client {
    pub fn execute<T: Into<ClientRequest>>(&mut self, request: T) {}

    pub fn execute_with_custom_conversion<T: CallerDefinedConversion>(
        &mut self,
        request: T,
    ) {
    }
}
