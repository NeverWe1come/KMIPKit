use std::sync::Arc;

use rustls::pki_types::ServerName;
use rustls::{ClientConnection, RootCertStore};

use crate::tls_policy::{apply_client_safety_policy, client_config_builder};

#[test]
fn applied_policy_disables_key_logging_and_early_data() {
    let mut config = client_config_builder()
        .expect("the TLS 1.3 AWS-LC builder is valid")
        .with_root_certificates(RootCertStore::empty())
        .with_no_client_auth();
    apply_client_safety_policy(&mut config);

    assert!(!config.key_log.will_log("CLIENT_HANDSHAKE_TRAFFIC_SECRET"));
    assert!(!config.key_log.will_log("CLIENT_EARLY_TRAFFIC_SECRET"));
    assert!(!config.enable_early_data);

    let mut connection = ClientConnection::new(
        Arc::new(config),
        ServerName::try_from("server.kmipkit.test")
            .expect("the fixture hostname is a valid server name"),
    )
    .expect("the policy config creates a client connection");
    assert!(connection.early_data().is_none());
}
