use std::sync::Arc;
use std::time::Duration;

use rustls::NamedGroup;
use rustls::client::{ClientSessionStore, Tls12ClientSessionValue};
use rustls::pki_types::ServerName;

use super::{MonotonicClock, TlsSessionStore};

struct FixedClock;

impl MonotonicClock for FixedClock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}

#[test]
fn session_store_keeps_tls12_disabled_and_redacts_its_debug_output() {
    let store = TlsSessionStore::new(Arc::new(FixedClock));
    let server_name = ServerName::try_from("server.kmipkit.test".to_owned())
        .expect("the fixture hostname is a valid server name");

    store.set_kx_hint(server_name.clone(), NamedGroup::X25519);
    assert_eq!(store.kx_hint(&server_name), None);
    store.set_tls12_session(server_name.clone(), Tls12ClientSessionValue {});
    assert!(store.tls12_session(&server_name).is_none());
    store.remove_tls12_session(&server_name);

    let debug = format!("{store:?}");
    assert!(debug.contains("ticket_count: 0"));
    assert!(debug.contains("[REDACTED]"));
}
