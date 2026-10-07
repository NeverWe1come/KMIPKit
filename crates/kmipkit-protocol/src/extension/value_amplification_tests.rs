use super::validate_with_metrics;
use crate::extension;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value};

const TAG_BASE: u32 = 0x0054_0000;
const DISCRIMINATOR_TAG_OFFSET: u32 = 1;
const NESTED_STRUCTURE_TAG_OFFSET: u32 = 300;
const FIRST_NESTED_RULE_TAG_OFFSET: u32 = 400;
const CHILD_RULE_COUNT: u32 = 256;
const NESTED_STRUCTURE_COUNT: usize = 10_000;
const DISCRIMINATOR: &str = "empty-nested-structure-amplification-v1";

fn tag(offset: u32) -> Tag {
    RawTag::new(TAG_BASE + offset)
        .expect("test tag fits the KMIP Tag width")
        .try_checked()
        .expect("test tag uses the KMIP extension allocation")
}

fn item(offset: u32, value: Value) -> Item {
    Item::new(tag(offset), value).expect("checked test tag forms a TTLV Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut result = Structure::new();
    for item in items {
        result
            .try_push(item)
            .expect("bounded test value stays within the TTLV model depth");
    }
    result
}

#[test]
fn many_empty_nested_structures_do_not_repeat_schema_width_work() {
    let children = (0..CHILD_RULE_COUNT)
        .map(|index| {
            extension::optional(
                tag(FIRST_NESTED_RULE_TAG_OFFSET + index),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("optional child rule is valid")
        })
        .collect();
    let order_edges = (0..CHILD_RULE_COUNT - 1)
        .map(|index| {
            extension::extension_order_constraint(
                tag(FIRST_NESTED_RULE_TAG_OFFSET + index),
                tag(FIRST_NESTED_RULE_TAG_OFFSET + index + 1),
            )
            .expect("adjacent child tags form a valid order edge")
        })
        .collect();
    let nested_schema = extension::structure(children, order_edges, false)
        .expect("default schema width and constraint limits accept this nested schema");
    let repeated_nested = extension::repeated(tag(NESTED_STRUCTURE_TAG_OFFSET), nested_schema)
        .expect("repeated nested Structure rule is valid");
    let mut root_children = vec![
        extension::required(
            tag(DISCRIMINATOR_TAG_OFFSET),
            extension::scalar(ItemType::TextString).expect("Text String is supported"),
        )
        .expect("discriminator rule is valid"),
    ];
    root_children.push(repeated_nested);
    let root_schema =
        extension::structure(root_children, Vec::new(), false).expect("root schema is valid");
    let identity = extension::extension_identity("example.vendor", "amplification", "1")
        .expect("extension identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("extension compatibility includes the client");
    let path =
        extension::ttlv_path(tag(DISCRIMINATOR_TAG_OFFSET)).expect("discriminator path is valid");
    let discriminator = extension::discriminator(path, Value::text_string(DISCRIMINATOR.into()))
        .expect("discriminator value is valid");
    let definition =
        extension::extension_definition(identity, compatibility, discriminator, root_schema)
            .expect("extension definition is valid");

    let nested_structures = (0..NESTED_STRUCTURE_COUNT).map(|_| {
        item(
            NESTED_STRUCTURE_TAG_OFFSET,
            Value::structure(Structure::new()),
        )
    });
    let mut root_items = vec![item(
        DISCRIMINATOR_TAG_OFFSET,
        Value::text_string(DISCRIMINATOR.into()),
    )];
    root_items.extend(nested_structures);
    let value = structure(root_items);

    let (_, metrics) = validate_with_metrics(&definition, value, &CodecLimits::defaults())
        .expect("empty optional nested Structures satisfy the schema");

    assert_eq!(
        metrics
            .order_edge_checks_per_structure
            .iter()
            .sum::<usize>(),
        0,
        "no declared order edge can be violated when all nested children are absent"
    );
    assert_eq!(
        metrics
            .occurrence_entries_per_structure
            .iter()
            .sum::<usize>(),
        2,
        "only the two present root fields need occurrence tracking"
    );
}

#[test]
fn sparse_nonempty_structures_bound_order_lookup_work_by_edge_count() {
    let children = (0..CHILD_RULE_COUNT)
        .map(|index| {
            extension::optional(
                tag(FIRST_NESTED_RULE_TAG_OFFSET + index),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("optional child rule is valid")
        })
        .collect();
    let order_edges = (0..CHILD_RULE_COUNT - 1)
        .map(|index| {
            extension::extension_order_constraint(
                tag(FIRST_NESTED_RULE_TAG_OFFSET + index),
                tag(FIRST_NESTED_RULE_TAG_OFFSET + index + 1),
            )
            .expect("adjacent child tags form a valid order edge")
        })
        .collect();
    let nested_schema = extension::structure(children, order_edges, false)
        .expect("default schema width and constraint limits accept this nested schema");
    let root_schema = extension::structure(
        vec![
            extension::required(
                tag(DISCRIMINATOR_TAG_OFFSET),
                extension::scalar(ItemType::TextString).expect("Text String is supported"),
            )
            .expect("discriminator rule is valid"),
            extension::repeated(tag(NESTED_STRUCTURE_TAG_OFFSET), nested_schema)
                .expect("repeated nested Structure rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("root schema is valid");
    let identity = extension::extension_identity("example.vendor", "sparse-work", "1")
        .expect("extension identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("extension compatibility includes the client");
    let path =
        extension::ttlv_path(tag(DISCRIMINATOR_TAG_OFFSET)).expect("discriminator path is valid");
    let discriminator = extension::discriminator(path, Value::text_string(DISCRIMINATOR.into()))
        .expect("discriminator value is valid");
    let definition =
        extension::extension_definition(identity, compatibility, discriminator, root_schema)
            .expect("extension definition is valid");

    let mut nested_items = (0..32)
        .step_by(2)
        .map(|index| item(FIRST_NESTED_RULE_TAG_OFFSET + index, Value::integer(1)));
    let nested_value = Value::structure(structure(nested_items.by_ref()));
    let value = structure([
        item(
            DISCRIMINATOR_TAG_OFFSET,
            Value::text_string(DISCRIMINATOR.into()),
        ),
        item(NESTED_STRUCTURE_TAG_OFFSET, nested_value),
    ]);

    let (_, metrics) = validate_with_metrics(&definition, value, &CodecLimits::defaults())
        .expect("sparse declared children satisfy the schema");

    assert!(
        metrics
            .order_edge_work_per_structure
            .iter()
            .all(|work| *work <= (CHILD_RULE_COUNT - 1) as usize),
        "order lookup work should stay within one declared-edge pass per Structure"
    );
}
