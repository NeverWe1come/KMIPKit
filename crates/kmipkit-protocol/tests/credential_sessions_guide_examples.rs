//! The marked English and Spanish credential-session examples must compile and
//! keep Ticket values explicit and redacted.

const ENGLISH_GUIDE: &str = include_str!("../../../docs/user-guide/en/credential-sessions.md");
const SPANISH_GUIDE: &str = include_str!("../../../docs/user-guide/es/sesiones-credenciales.md");

use kmipkit_protocol::{LoginRequest, LogoutRequest, SecretBytes, Ticket, TicketType};

#[test]
fn both_guides_document_login_ticket_ownership_and_logout_without_claiming_server_state() {
    for guide in [ENGLISH_GUIDE, SPANISH_GUIDE] {
        assert!(guide.contains("```rust,kmipkit-test"));
        assert!(guide.contains("LoginRequest"));
        assert!(guide.contains("LogoutRequest"));
        assert!(guide.contains("Ticket"));
        assert!(guide.contains("Delegated Login"));
        assert!(guide.contains("KMIPKIT-0043"));
        assert!(guide.contains("synthetic-ticket-value"));
    }
}

#[test]
fn example_ticket_is_explicitly_passed_to_logout_and_stays_redacted() {
    const SECRET: &[u8] = b"synthetic-ticket-value";
    let ticket = Ticket::new(TicketType::LOGIN, SecretBytes::new(SECRET.to_vec()))
        .expect("the synthetic Ticket is structurally valid");
    let debug = format!("{ticket:?}");
    assert!(!debug.contains("synthetic-ticket-value"));

    let logout = LogoutRequest::new(ticket);
    let debug = format!("{logout:?}");
    assert!(!debug.contains("synthetic-ticket-value"));

    let login_payload = LoginRequest::new()
        .into_ttlv_payload()
        .expect("an empty Login request is valid");
    assert!(login_payload.view().children().is_empty());
}
