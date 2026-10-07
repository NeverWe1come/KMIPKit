use std::error::Error;

use kmipkit_client::extension_registry as client_extension;
use kmipkit_client::{ClientBatchItem, extension_registry};
use kmipkit_protocol::extension;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

const NESTED_TAG: u32 = 0x0054_0010;
const DISCRIMINATOR_TAG: u32 = 0x0054_0011;
const POLICY_TAG: u32 = 0x0054_0012;

fn main() -> Result<(), Box<dyn Error>> {
    let vendor_identifier = "com.example";
    let identity = extension::extension_identity(vendor_identifier, "key-policy", "1")?;
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.1.0", "0.1.0")?;

    let discriminator_tag = tag(DISCRIMINATOR_TAG)?;
    let nested_tag = tag(NESTED_TAG)?;
    let discriminator_path =
        extension::with_child_tag(extension::ttlv_path(nested_tag)?, discriminator_tag)?;
    let discriminator = extension::discriminator(
        discriminator_path,
        Value::text_string("key-policy-v1".to_owned()),
    )?;

    let label_rule =
        extension::required(discriminator_tag, extension::scalar(ItemType::TextString)?)?;
    let policy_rule = extension::optional(
        tag(POLICY_TAG)?,
        extension::with_signed_range(
            extension::scalar(ItemType::Integer)?,
            0,
            i64::from(i32::MAX),
        )?,
    )?;
    let nested_schema = extension::structure(vec![label_rule, policy_rule], Vec::new(), false)?;
    let root_schema = extension::structure(
        vec![extension::required(nested_tag, nested_schema)?],
        Vec::new(),
        false,
    )?;
    let definition = extension::extension_definition(
        identity.clone(),
        compatibility,
        discriminator,
        root_schema,
    )?;
    let information = extension::extension_information("Key policy extension")?;
    let definition = extension::with_information(definition, information)?;

    let registry =
        extension_registry::client_extension_registry(vec![definition], extension::defaults())?;
    let configuration = client_extension::ClientConfiguration::new(registry);
    let codec_limits = CodecLimits::defaults();

    let first_value = client_extension::validate_extension_value(
        configuration.extension_registry(),
        identity.clone(),
        payload("key-policy-v1", 7)?,
        &codec_limits,
    )?;
    let second_value = client_extension::validate_extension_value(
        configuration.extension_registry(),
        identity.clone(),
        payload("key-policy-v1", 9)?,
        &codec_limits,
    )?;

    // Vendor payloads may contain secrets. Do not log or debug-format them.
    // KMIPKit-owned TTLV values zeroize on drop; copies made before transfer or
    // copied from borrowed views remain the caller's responsibility. In the
    // client receive path, KMIPKIT-0007 rejects unrecognized critical response
    // extensions and preserves unrecognized non-critical ones generically;
    // this example inspects local TTLV and does not exercise that transport path.
    let recognized = client_extension::inspect(
        configuration.extension_registry(),
        vendor_identifier,
        payload("key-policy-v1", 7)?,
        &codec_limits,
    )?;
    assert!(client_extension::is_recognized(&recognized));
    assert!(client_extension::validated_value(&recognized).is_some());
    assert!(has_key_policy_payload(
        client_extension::generic_value(&recognized),
        nested_tag,
        discriminator_tag,
        tag(POLICY_TAG)?,
        7,
    ));

    let schema_invalid = client_extension::inspect(
        configuration.extension_registry(),
        vendor_identifier,
        payload("key-policy-v1", -1)?,
        &codec_limits,
    )?;
    assert!(!client_extension::is_recognized(&schema_invalid));
    assert!(client_extension::validated_value(&schema_invalid).is_none());
    assert!(has_key_policy_payload(
        client_extension::generic_value(&schema_invalid),
        nested_tag,
        discriminator_tag,
        tag(POLICY_TAG)?,
        -1,
    ));

    let first_use = client_extension::client_request_message_extension(first_value, true)?;
    let second_use = client_extension::client_request_message_extension(second_value, false)?;
    let batch_item = ClientBatchItem::discover_versions()
        .with_extension(first_use)
        .with_extension(second_use);

    assert_eq!(
        client_extension::definition_count(configuration.extension_registry()),
        1
    );
    assert_eq!(batch_item.extension_count(), 2);
    assert_eq!(batch_item.extension_identity_at(0), Some(identity.clone()));
    assert_eq!(batch_item.extension_identity_at(1), Some(identity));
    assert_eq!(batch_item.extension_criticality_indicator_at(0), Some(true));
    assert_eq!(
        batch_item.extension_criticality_indicator_at(1),
        Some(false)
    );

    println!(
        "recognized a valid inbound extension, retained a schema-invalid payload generically, and attached two outbound uses in caller order"
    );
    Ok(())
}

fn has_key_policy_payload(
    value: &Structure,
    nested_tag: Tag,
    discriminator_tag: Tag,
    policy_tag: Tag,
    expected_policy: i32,
) -> bool {
    let root = value.view();
    let Some(nested_item) = root.children().first() else {
        return false;
    };
    if root.children().len() != 1 || nested_item.tag() != nested_tag {
        return false;
    }

    nested_item.with_value(|value| {
        let ValueView::Structure(nested) = value else {
            return false;
        };
        let children = nested.children();
        children.len() == 2
            && children[0].tag() == discriminator_tag
            && children[0].with_value(
                |value| matches!(value, ValueView::TextString(actual) if actual == "key-policy-v1"),
            )
            && children[1].tag() == policy_tag
            && children[1].with_value(
                |value| matches!(value, ValueView::Integer(actual) if *actual == expected_policy),
            )
    })
}

fn payload(discriminator: &str, policy: i32) -> Result<Structure, Box<dyn Error>> {
    let mut nested = Structure::new();
    nested.try_push(Item::new(
        tag(DISCRIMINATOR_TAG)?,
        Value::text_string(discriminator.to_owned()),
    )?)?;
    nested.try_push(Item::new(tag(POLICY_TAG)?, Value::integer(policy))?)?;

    let mut root = Structure::new();
    root.try_push(Item::new(tag(NESTED_TAG)?, Value::structure(nested))?)?;
    Ok(root)
}

fn tag(raw: u32) -> Result<Tag, Box<dyn Error>> {
    Ok(RawTag::new(raw)?.try_checked()?)
}
