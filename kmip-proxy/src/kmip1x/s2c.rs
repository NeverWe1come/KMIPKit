//! Complete Server-to-Client (S2C) translation bridge:
//! `KMIP 2.0/2.1/3.0 Server -> Kmip1xS2cBridge -> S2cClientTransport -> Legacy KMIP 1.X Client`
//!
//! Supports all KMIP 1.X S2C operations: `Notify` (0x1B), `Put` (0x1C),
//! `Discover Versions` (0x1E), and `Query` (0x18).
#![forbid(unsafe_code)]

use std::sync::Arc;

use async_trait::async_trait;
use kmipkit_ttlv::Item;

use crate::kmip1x::mapping::UpstreamKmipVersion;
use crate::kmip1x::tags::*;
use crate::kmip1x::translator::{
    extract_protocol_version, translate_server_to_client_request_2x_to_1x,
    translate_server_to_client_response_1x_to_2x,
};
use crate::kmip1x::ttlv_helpers::{
    build_protocol_version, build_response_message, build_success_batch_item, extract_byte_string,
    extract_enumeration, find_child, find_children,
};

/// Async transport abstraction for delivering translated S2C KMIP 1.X requests to a legacy client.
#[async_trait]
pub trait S2cClientTransport: Send + Sync + 'static {
    async fn send_to_client(&self, request_1x: &Item) -> Result<Item, String>;
}

/// Complete Server-to-Client bridge translating KMIP 2.0/2.1/3.0 server-initiated requests
/// into KMIP 1.X requests for a legacy client, and translating the client's 1.X responses back.
pub struct Kmip1xS2cBridge {
    client_transport: Arc<dyn S2cClientTransport>,
    client_version_1x: (i32, i32),
    upstream_version: UpstreamKmipVersion,
}

impl Kmip1xS2cBridge {
    #[must_use]
    pub fn new(
        client_transport: Arc<dyn S2cClientTransport>,
        client_minor_1x: i32,
        upstream_version: UpstreamKmipVersion,
    ) -> Self {
        Self {
            client_transport,
            client_version_1x: (1, client_minor_1x.clamp(0, 4)),
            upstream_version,
        }
    }

    /// Handles an incoming Server-to-Client request initiated by a KMIP 2.0 / 2.1 / 3.0 server.
    pub async fn handle_server_request(&self, server_req_2x: &Item) -> Result<Item, String> {
        let (srv_major, srv_minor) = extract_protocol_version(server_req_2x)
            .unwrap_or_else(|| self.upstream_version.major_minor());

        // If the server already speaks KMIP 1.X, pass through directly
        if srv_major == 1 {
            return self.client_transport.send_to_client(server_req_2x).await;
        }

        let batch_items = find_children(server_req_2x, TAG_BATCH_ITEM);
        if batch_items.len() == 1 {
            let op = find_child(&batch_items[0], TAG_OPERATION)
                .and_then(|i| extract_enumeration(&i))
                .unwrap_or(0);
            if op == OP_DISCOVER_VERSIONS {
                // Intercept S2C Discover Versions locally so the 2.0/2.1/3.0 server sees its own version
                let bid = find_child(&batch_items[0], TAG_UNIQUE_BATCH_ITEM_ID)
                    .and_then(|i| extract_byte_string(&i));
                let (up_maj, up_min) = self.upstream_version.major_minor();
                let bi = build_success_batch_item(
                    OP_DISCOVER_VERSIONS,
                    bid.as_deref(),
                    vec![build_protocol_version(up_maj, up_min)?],
                )?;
                return build_response_message(srv_major, srv_minor, vec![bi]);
            }
        }

        let (c_maj, c_min) = self.client_version_1x;
        let req_1x = translate_server_to_client_request_2x_to_1x(server_req_2x, c_maj, c_min)?;
        let resp_1x = self.client_transport.send_to_client(&req_1x).await?;
        translate_server_to_client_response_1x_to_2x(&resp_1x, self.upstream_version)
    }
}
