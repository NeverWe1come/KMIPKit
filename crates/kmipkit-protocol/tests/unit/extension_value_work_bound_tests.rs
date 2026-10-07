use super::validate_with_metrics;
    use crate::extension;
    use kmipkit_ttlv::codec::CodecLimits;
    use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value};

    const TAG_BASE: u32 = 0x0054_0000;
    const DISCRIMINATOR_OFFSET: u32 = 1;
    const DISCRIMINATOR: &str = "work-bound-fixture-v1";

    fn tag(offset: u32) -> Tag {
        RawTag::new(TAG_BASE + offset)
            .expect("work-bound fixture tag fits the KMIP Tag width")
            .try_checked()
            .expect("work-bound fixture tag uses the KMIP extension allocation")
    }

    fn item(offset: u32, value: Value) -> Item {
        Item::new(tag(offset), value).expect("checked work-bound tag forms an Item")
    }

    fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
        let mut result = Structure::new();
        for item in items {
            result
                .try_push(item)
                .expect("work-bound fixture stays within the TTLV model depth");
        }
        result
    }

    fn definition(
        payload_rules: Vec<extension::ExtensionChildRule>,
        order: Vec<extension::ExtensionOrderConstraint>,
    ) -> extension::ExtensionDefinition {
        let mut children = vec![
            extension::required(
                tag(DISCRIMINATOR_OFFSET),
                extension::scalar(ItemType::TextString).expect("Text String is supported"),
            )
            .expect("discriminator schema child is valid"),
        ];
        children.extend(payload_rules);
        let schema = extension::structure(children, order, false)
            .expect("work-bound fixture schema is structurally valid");
        let identity = extension::extension_identity("example.vendor", "work-bound", "1")
            .expect("work-bound identity is valid");
        let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
            .expect("work-bound fixture supports the workspace version");
        let path = extension::ttlv_path(tag(DISCRIMINATOR_OFFSET))
            .expect("discriminator path is non-empty");
        let discriminator =
            extension::discriminator(path, Value::text_string(DISCRIMINATOR.into()))
                .expect("discriminator scalar is valid");
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("discriminator resolves through the fixture schema")
    }

    fn extension_value(payload: impl IntoIterator<Item = Item>) -> Structure {
        let mut items = vec![item(
            DISCRIMINATOR_OFFSET,
            Value::text_string(DISCRIMINATOR.into()),
        )];
        items.extend(payload);
        structure(items)
    }

    #[test]
    fn wide_schema_uses_at_most_thirteen_tag_comparisons_per_input_item() {
        const MAX_RULES: u32 = 4_096;
        const REPEATED_ITEMS: usize = 1_024;

        let rule_capacity =
            usize::try_from(MAX_RULES).expect("the bounded fixture rule count fits usize") - 1;
        let mut rules = Vec::with_capacity(rule_capacity);
        for offset in 2..MAX_RULES {
            rules.push(
                extension::optional(
                    tag(offset),
                    extension::scalar(ItemType::Integer).expect("Integer is supported"),
                )
                .expect("wide optional child rule is valid"),
            );
        }
        rules.push(
            extension::repeated(
                tag(MAX_RULES),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("last indexed child rule is valid"),
        );
        let definition = definition(rules, Vec::new());
        let payload = (0..REPEATED_ITEMS).map(|value| {
            item(
                MAX_RULES,
                Value::integer(i32::try_from(value).expect("fixture value fits i32")),
            )
        });
        let (_, metrics) = validate_with_metrics(
            &definition,
            extension_value(payload),
            &CodecLimits::defaults(),
        )
        .expect("the repeated last-tag field satisfies its schema");

        assert_eq!(
            metrics.schema_tag_comparisons_per_item.len(),
            REPEATED_ITEMS + 1
        );
        assert!(
            metrics
                .schema_tag_comparisons_per_item
                .iter()
                .all(|comparisons| *comparisons <= 13)
        );
    }

    #[test]
    fn repeated_worst_case_enum_lookups_use_at_most_thirteen_comparisons_each() {
        const ENUM_VALUES: u32 = 4_096;
        const REPEATED_ITEMS: usize = 4_096;
        const PAYLOAD_OFFSET: u32 = 2;

        let mut schema =
            extension::scalar(ItemType::Enumeration).expect("Enumeration is a supported scalar");
        for value in (0..ENUM_VALUES).rev() {
            schema = extension::with_allowed_enumeration(schema, value)
                .expect("the enumeration reaches its hard member count");
        }
        let rule = extension::repeated(tag(PAYLOAD_OFFSET), schema)
            .expect("repeated Enumeration rule is valid");
        let definition = definition(vec![rule], Vec::new());
        let payload =
            (0..REPEATED_ITEMS).map(|_| item(PAYLOAD_OFFSET, Value::enumeration(ENUM_VALUES - 1)));
        let (_, metrics) = validate_with_metrics(
            &definition,
            extension_value(payload),
            &CodecLimits::defaults(),
        )
        .expect("every repeated enum value is declared");

        assert_eq!(metrics.enum_comparisons_per_item.len(), REPEATED_ITEMS);
        assert!(
            metrics
                .enum_comparisons_per_item
                .iter()
                .all(|comparisons| *comparisons <= 13)
        );
    }

    #[test]
    fn repeated_ordered_fields_check_each_declared_edge_exactly_once() {
        const EDGE_COUNT: u32 = 32;
        const OCCURRENCES_PER_FIELD: usize = 32;
        const FIRST_OFFSET: u32 = 100;

        let rules = (0..=EDGE_COUNT)
            .map(|index| {
                extension::repeated(
                    tag(FIRST_OFFSET + index),
                    extension::scalar(ItemType::Integer).expect("Integer is supported"),
                )
                .expect("repeated ordered field rule is valid")
            })
            .collect();
        let edges = (0..EDGE_COUNT)
            .map(|index| {
                extension::extension_order_constraint(
                    tag(FIRST_OFFSET + index),
                    tag(FIRST_OFFSET + index + 1),
                )
                .expect("adjacent tags form a directed order edge")
            })
            .collect();
        let definition = definition(rules, edges);
        let payload = (0..=EDGE_COUNT).flat_map(|index| {
            (0..OCCURRENCES_PER_FIELD).map(move |occurrence| {
                item(
                    FIRST_OFFSET + index,
                    Value::integer(i32::try_from(occurrence).expect("fixture value fits i32")),
                )
            })
        });
        let (_, metrics) = validate_with_metrics(
            &definition,
            extension_value(payload),
            &CodecLimits::defaults(),
        )
        .expect("all repeated fields occur in the declared edge order");

        assert_eq!(
            metrics.order_edge_checks_per_structure,
            [usize::try_from(EDGE_COUNT).expect("fixture edge count fits usize")]
        );
        assert_eq!(
            metrics.order_edge_work_per_structure,
            [usize::try_from(EDGE_COUNT).expect("fixture edge count fits usize")],
            "the dense ordered-field case scans the declared edge list exactly once"
        );
    }
