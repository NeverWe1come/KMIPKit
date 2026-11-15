pub struct ClientRequest;
pub struct Structure;

impl From<Vec<u8>> for ClientRequest {
    fn from(_: Vec<u8>) -> Self {
        Self
    }
}

impl TryFrom<Structure> for ClientRequest {
    type Error = ();

    fn try_from(_: Structure) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
