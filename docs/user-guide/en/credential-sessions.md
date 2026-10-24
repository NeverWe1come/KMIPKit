# Credential sessions

This guide covers the KMIP 2.1 client operations Delegated Login, Login, and
Logout: OASIS KMIP Specification v2.1 §§6.1.12, 6.1.30, and 6.1.31. These
operations exchange caller-controlled session tickets with the server. KMIPKit
does not install a returned Ticket as implicit client state, attach it to later
requests, or perform an automatic Logout.

## Login

All Login request fields are optional. The caller may send an empty request or
select Lease Time, Request Count, and Usage Limits explicitly. The following
example builds a typed request batch but does not send it.

```rust,kmipkit-test
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::LoginRequest;

fn main() {
    let request = LoginRequest::new().with_lease_time_interval(300);
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::login(request)));
    assert_eq!(batch.items().len(), 1);
}
```

A successful Login response returns a `Ticket`. Keep it as an explicit value;
KMIPKit never applies it to a later request. The current response accessor
borrows the Ticket, while Logout takes ownership. If the caller needs an owned
Ticket for Logout, it can deliberately create a caller-owned copy through the
callback accessor, as shown below. The sample ticket is synthetic and must not
be used for authentication.

## Passing a Ticket to Logout

Ticket Value is secret data. `Ticket` and the Logout request redact it from
`Debug`; read it only through the callback accessor. The copy below is explicit
and remains caller-controlled. Do not log or persist the sample bytes.

```rust,kmipkit-test
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::{
    LoginRequest, LogoutRequest, SecretBytes, Ticket, TicketType, TicketValidationError,
};

fn owned_ticket(ticket: &Ticket) -> Result<Ticket, TicketValidationError> {
    let value = ticket.with_ticket_value(|bytes| bytes.to_vec());
    Ticket::new(ticket.ticket_type(), SecretBytes::new(value))
}

fn main() {
    let login_batch = ClientBatch::new(ClientBatchItem::new(
        ClientRequest::login(LoginRequest::new()),
    ));
    assert_eq!(login_batch.items().len(), 1);

    // Synthetic placeholder for a Ticket returned by a server.
    let returned_ticket = Ticket::new(
        TicketType::LOGIN,
        SecretBytes::new(b"synthetic-ticket-value".to_vec()),
    )
    .expect("the synthetic Ticket is structurally valid");
    assert!(!format!("{returned_ticket:?}").contains("synthetic-ticket-value"));

    let logout_ticket = owned_ticket(&returned_ticket)
        .expect("the parsed Ticket remains a valid Ticket");
    let logout = LogoutRequest::new(logout_ticket);
    let logout_batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::logout(logout)));
    assert_eq!(logout_batch.items().len(), 1);
}
```

Creating the second owned `Ticket` copies its bytes under the caller's control.
KMIPKit zeroizes its own `SecretBytes` allocations when dropped; language
runtimes and any copies made by the application have their own memory-lifetime
limitations. Logout sends exactly the Ticket supplied by the caller. A successful
Logout has an empty operation payload; Failure and Pending preserve their KMIP
result metadata. KMIPKit does not retry, poll, or silently reuse the Ticket.

## Delegated Login

Delegated Login requires a caller-supplied Rights structure and returns a
Ticket on success. Lease Time, Request Count, and Usage Limits remain optional.
Use `DelegatedLoginRequest` when the application has a rights structure defined
for its KMIP server. The library preserves that TTLV structure; it does not
interpret rights or claim the server granted them.

The typed client entry points are available in all four 1.0 languages:

| Language | Request mapping | Typed response mapping |
| --- | --- | --- |
| Rust | `ClientRequest::delegated_login(DelegatedLoginRequest::new(rights))` | `DelegatedLoginResponse::ticket()` and `Ticket` accessors |
| C | `kmipkit_client_batch_item_delegated_login(...)` | `kmipkit_client_batch_item_response_delegated_login(...)` |
| Java | `ClientBatchItem.delegatedLogin(...)` | `ClientBatchItemResult.delegatedLoginResponse()` |
| Python | `client_batch_item_delegated_login(...)` | `BatchItemResult.delegated_login_response` |

Each binding also exposes the complete ordered Ticket TTLV structure, including
unknown children, while preserving unknown Ticket Type values. Treat both the
Ticket Value and the generic Ticket structure as secret data. C's Ticket Value
span is borrowed from the response item and expires when that item is released.
Java's `ticketValue()` returns a caller-owned `byte[]` copy that the application
must clear; KMIPKit wipes its internal Java buffers and the response's owned
copy when closed. Python's `ticket_value` returns immutable `bytes`, which the
Python runtime cannot deterministically clear; KMIPKit wipes its mutable owned
buffer and the response's generic TTLV copy when `close()` is called. Rust
exposes Ticket Value through a callback and owns it in a zeroizing secret type.
Never log a Ticket, its generic TTLV, or any Ticket Value copy.

## Scope and limitations

The code examples compile against the Rust client/protocol APIs. Cross-language
Delegated Login mappings and parity evidence are recorded in the approved
KMIPKIT-0043 manifest and its traceability record. Project tests cover the
pinned Login CN01 §2.48 fixture; source-derived tests for the other listed
cases are not official fixture passes. This guide makes no claim of server
interoperability or formal certification.
