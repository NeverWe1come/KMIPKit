//! OASIS KMIP 2.1 Server-to-Client operations (§6.2, Tables 343–351).

pub mod discover_versions;
pub mod notify;
pub mod put;
pub mod query;
pub mod response_builder;

pub use discover_versions::{
    DISCOVER_VERSIONS_OPERATION, ServerDiscoverVersionsError, ServerDiscoverVersionsRequest,
    ServerDiscoverVersionsResponse,
};
pub use notify::{
    DeletedAttributeReference, NOTIFY_OPERATION, ServerNotifyError, ServerNotifyRequest,
    ServerNotifyResponse,
};
pub use put::{PUT_OPERATION, PutFunction, ServerPutError, ServerPutRequest, ServerPutResponse};
pub use query::{
    QUERY_OPERATION, ServerCapabilities, ServerQueryError, ServerQueryRequest, ServerQueryResponse,
};
pub use response_builder::ServerResponseBuilder;

/// Retains the wire type alongside the text used by the original callback API.
#[derive(Debug)]
struct ServerIdentifier {
    value: crate::UniqueIdentifier,
    numeric_text: Option<String>,
}

impl ServerIdentifier {
    fn new(value: crate::UniqueIdentifier) -> Self {
        let numeric_text = match &value {
            crate::UniqueIdentifier::Integer(n) => Some(n.to_string()),
            crate::UniqueIdentifier::Enumeration(n) => Some(n.to_string()),
            _ => None,
        };
        Self {
            value,
            numeric_text,
        }
    }

    fn text(&self) -> &str {
        self.value
            .as_str()
            .or(self.numeric_text.as_deref())
            .expect("every identifier has a display form")
    }
}
