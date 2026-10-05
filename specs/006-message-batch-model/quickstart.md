# Quickstart: Build and inspect a KMIP message

The example below is the public usage target. During implementation it MUST be compiled as a doctest against the released `kmipkit-protocol` API. It demonstrates an in-memory message only; it does not encode bytes or contact a server.

```rust,no_run
use kmipkit_protocol::{ProtocolVersion, RequestMessage};
use kmipkit_ttlv::Structure;

fn validate_message(tree: Structure) -> Result<(), Box<dyn std::error::Error>> {
    let message = RequestMessage::try_from_ttlv(tree)?;
    let version: ProtocolVersion = message.header().protocol_version();
    assert_eq!(version.major(), 2);
    assert_eq!(version.minor(), 1);
    assert_eq!(message.batch_items().len(), message.header().batch_count());
    let _tree_for_the_next_layer = message.into_ttlv();
    Ok(())
}
```

A production example in the user guide will build the `Structure` from typed operation payloads after those operation-family specifications land. This feature does not invent an operation payload or send a request.
