//! Unit tests for multi-version client configuration, override, and VersionGate enforcement (KMIPKIT-0069).

use kmipkit_protocol::{ProtocolVersion, VersionGate};
use kmipkit_transport::RequestOptions;

use crate::extension_registry::{client_extension_registry, ClientConfiguration};
use crate::{Client, ClientBatch, ClientBatchItem, ClientRequest};

#[test]
fn test_client_configuration_default_and_version_methods() {
    let registry = client_extension_registry(Vec::new(), kmipkit_protocol::extension::defaults())
        .expect("empty registry is valid");
    let config = ClientConfiguration::new(registry);

    // Default version is KMIP 2.1 (Zero-Breakage principle)
    assert_eq!(config.protocol_version(), ProtocolVersion::V2_1);

    // with_protocol_version changes the version
    let config_2_0 = config.with_protocol_version(ProtocolVersion::V2_0);
    assert_eq!(config_2_0.protocol_version(), ProtocolVersion::V2_0);

    // with_kmip_version alias works
    let config_3_0 = config_2_0.with_kmip_version(ProtocolVersion::V3_0);
    assert_eq!(config_3_0.protocol_version(), ProtocolVersion::V3_0);
}

#[test]
fn test_request_options_protocol_version_overrides() {
    let options = RequestOptions::default();
    assert_eq!(options.protocol_version(), None);

    let opt_2_0 = options.with_protocol_version(ProtocolVersion::V2_0);
    assert_eq!(opt_2_0.protocol_version(), Some(ProtocolVersion::V2_0));

    let opt_forced = options.force_kmip_version(ProtocolVersion::V3_0);
    assert_eq!(opt_forced.protocol_version(), Some(ProtocolVersion::V3_0));
}

#[test]
fn test_client_batch_protocol_version_overrides() {
    let batch = ClientBatch::empty();
    assert_eq!(batch.protocol_version(), None);

    let batch_2_0 = batch.with_protocol_version(ProtocolVersion::V2_0);
    assert_eq!(batch_2_0.protocol_version(), Some(ProtocolVersion::V2_0));

    let batch_forced = batch_2_0.force_kmip_version(ProtocolVersion::V3_0);
    assert_eq!(batch_forced.protocol_version(), Some(ProtocolVersion::V3_0));
}
