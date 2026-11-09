use super::*;

fn vendor_payload() -> Structure {
    let mut payload = Structure::new();
    for (raw_tag, value) in [
        (0x0054_0010, Value::integer(17)),
        (0x0042_0001, Value::integer(29)),
    ] {
        let item = Item::new(
            RawTag::new(raw_tag)
                .expect("fixture tag is within TTLV bounds")
                .try_checked()
                .expect("fixture tag is allocated"),
            value,
        )
        .expect("fixture item is valid");
        payload
            .try_push(item)
            .expect("fixture is within TTLV limits");
    }
    payload
}

#[test]
fn all_seven_draft_routes_preserve_caller_order_and_generic_values() {
    let routes = [
        ClientBatchItem::create_group_draft(vendor_payload()),
        ClientBatchItem::obliterate_draft(vendor_payload()),
        ClientBatchItem::create_user_draft(vendor_payload()),
        ClientBatchItem::create_credential_draft(vendor_payload()),
        ClientBatchItem::deactivate_draft(vendor_payload()),
        ClientBatchItem::encapsulate_draft(vendor_payload()),
        ClientBatchItem::decapsulate_draft(vendor_payload()),
    ];

    for item in routes {
        let payload = item
            .request
            .payload()
            .expect("generic draft payload is retained");
        let observed = payload
            .view()
            .children()
            .iter()
            .map(|field| {
                let value = field.with_value(|value| match value {
                    ValueView::Integer(value) => *value,
                    _ => panic!("integer fixture retains its generic value"),
                });
                (field.tag().raw(), value)
            })
            .collect::<Vec<_>>();

        assert_eq!(observed, [(0x0054_0010, 17), (0x0042_0001, 29)]);
    }
}
