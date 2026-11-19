pub struct Structure;
pub struct ClientBatchItem;

impl TryFrom<Structure> for ClientBatchItem {
    type Error = ();

    fn try_from(_: Structure) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
