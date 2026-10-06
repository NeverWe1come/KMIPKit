use std::error::Error;

pub struct ClientCauseCategory;
pub struct RequestDeliveryState;
pub struct ClientError;

impl ClientError {
    pub fn validation<E>(
        cause: ClientCauseCategory,
        delivery_state: RequestDeliveryState,
        source: E,
    ) -> Self
    where
        E: Error + 'static,
    {
        drop(source);
        let _ = (cause, delivery_state);
        Self
    }
}
