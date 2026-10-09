# Quickstart: Get and Locate

**Feature**: KMIPKIT-0017<br>
**Prerequisite**: Approved and merged KMIPKIT-0017 specification and implementation. These examples use the existing typed-batch execution API and must compile in the implementation PR.

## Build a Locate request<br>

    use kmipkit_protocol::{AttributeSet, LocateRequest};

    let query = LocateRequest::new(AttributeSet::new())
        .with_offset_items(0);

    assert!(query.attributes().is_empty());
    assert_eq!(query.offset_items(), Some(0));

An empty Attributes Structure requests a match against all server objects. Offset Items zero remains explicitly present even though OASIS treats it as equivalent to omission.

## Build a Get request<br>

    use kmipkit_protocol::{GetRequest, UniqueIdentifier};

    let request = GetRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("object-123".to_owned()));

    assert_eq!(
        request.unique_identifier(),
        Some(&UniqueIdentifier::TextString("object-123".to_owned()))
    );

Format and wrapping options are explicit request values. KMIPKit forwards them; it does not perform key conversion or wrapping.

## Execute Get and consume its result<br>

    use kmipkit_client::{
        Client, ClientBatch, ClientBatchItem, ClientError, ClientRequest,
    };
    use kmipkit_protocol::{GetRequest, UniqueIdentifier};
    use kmipkit_ttlv::codec::CodecLimits;

    // `client` was constructed with the existing TLS 1.3 mutual-TLS configuration.
    fn retrieve(client: &mut Client, limits: &CodecLimits) -> Result<(), ClientError> {
        let request = GetRequest::new()
            .with_unique_identifier(UniqueIdentifier::TextString("object-123".to_owned()));
        let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Get(request)));
        let response = client.execute(batch, limits)?;

        if let Some(item) = response.get(0) {
            if let Some(get) = item.outcome().get_response() {
                if let Some(object) = get.object() {
                    // Inspect the object tag/shape without formatting its payload.
                    let object_tag = object.tag();
                    let _ = object_tag;
                }
            }
        }
        Ok(())
    }

For Locate, build `ClientBatch::new(ClientBatchItem::new(ClientRequest::Locate(query)))`, call `client.execute(batch, limits)`, then read `response.get(0)?.outcome().locate_response()?.unique_identifiers()`. Preserve server order and repeated identifiers. Archived-object contents require the separate Recover then Get workflow; Recover is outside this feature.

The implementation PR must compile these snippets as documentation tests and add the Spanish user guide. This proposed quickstart does not claim server interoperability or profile conformance.
