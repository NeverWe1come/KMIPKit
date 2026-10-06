pub struct ClientBatch;
pub struct CallerInput;

impl Into<ClientBatch> for CallerInput {
    fn into(self) -> ClientBatch {
        ClientBatch
    }
}
