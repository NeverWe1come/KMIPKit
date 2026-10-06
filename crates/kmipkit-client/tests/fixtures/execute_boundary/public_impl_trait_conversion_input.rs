pub struct ClientRequest;
pub struct Client;

impl Client {
    pub fn execute(&mut self, request: impl Into<ClientRequest>) {}
}
