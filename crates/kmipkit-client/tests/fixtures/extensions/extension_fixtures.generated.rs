// Generated from specification/api/public-api.json and tests/fixtures/extensions/cases.json. API contract sha256: f54ae735a2ca4d7173b0c493c367c9fde70d17e02b06cd7134572f068c8d8c1d. Do not edit.
use kmipkit_ttlv::ItemType;

#[derive(Clone, Copy)]
#[rustfmt::skip]
pub struct Item { pub tag: u32, pub item_type: ItemType, pub text: &'static str, pub number: i64, pub children: &'static [Item] }
#[derive(Clone, Copy)]
#[rustfmt::skip]
pub struct ChildRule { pub tag: u32, pub schema: Schema }
#[derive(Clone, Copy)]
#[rustfmt::skip]
pub struct Schema { pub item_type: ItemType, pub has_range: bool, pub minimum: i64, pub maximum: i64, pub children: &'static [ChildRule] }
#[rustfmt::skip]
pub struct Definition { pub id: &'static str, pub vendor: &'static str, pub name: &'static str, pub version: &'static str, pub path: &'static [u32], pub discriminator_type: ItemType, pub discriminator_text: &'static str, pub discriminator_number: i64, pub schema: Schema }
#[rustfmt::skip]
pub struct Case { pub id: &'static str, pub vendor: &'static str, pub payload: &'static [Item], pub outcome: &'static str, pub matched_ids: &'static [&'static str], pub typed: bool }

#[rustfmt::skip]
pub static DEFINITIONS: &[Definition] = &[
    Definition { id: "ambiguous.beta", vendor: "example.vendor", name: "ambiguous-beta", version: "1", path: &[0x540010, 0x540012], discriminator_type: ItemType::TextString, discriminator_text: "route-beta", discriminator_number: 0, schema: Schema { item_type: ItemType::Structure, has_range: false, minimum: 0, maximum: 0, children: &[ChildRule { tag: 0x540010, schema: Schema { item_type: ItemType::Structure, has_range: false, minimum: 0, maximum: 0, children: &[ChildRule { tag: 0x540011, schema: Schema { item_type: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }, ChildRule { tag: 0x540012, schema: Schema { item_type: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }, ChildRule { tag: 0x540013, schema: Schema { item_type: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }] } }] } },
    Definition { id: "known.alpha", vendor: "example.vendor", name: "alpha", version: "1", path: &[0x420001], discriminator_type: ItemType::TextString, discriminator_text: "alpha-v1", discriminator_number: 0, schema: Schema { item_type: ItemType::Structure, has_range: false, minimum: 0, maximum: 0, children: &[ChildRule { tag: 0x420001, schema: Schema { item_type: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }, ChildRule { tag: 0x420004, schema: Schema { item_type: ItemType::LongInteger, has_range: true, minimum: 0, maximum: 99, children: &[] } }] } },
    Definition { id: "ambiguous.alpha", vendor: "example.vendor", name: "ambiguous-alpha", version: "1", path: &[0x540010, 0x540011], discriminator_type: ItemType::TextString, discriminator_text: "route-alpha", discriminator_number: 0, schema: Schema { item_type: ItemType::Structure, has_range: false, minimum: 0, maximum: 0, children: &[ChildRule { tag: 0x540010, schema: Schema { item_type: ItemType::Structure, has_range: false, minimum: 0, maximum: 0, children: &[ChildRule { tag: 0x540011, schema: Schema { item_type: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }, ChildRule { tag: 0x540012, schema: Schema { item_type: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }, ChildRule { tag: 0x540013, schema: Schema { item_type: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }] } }] } },
];

#[rustfmt::skip]
pub static CASES: &[Case] = &[
    Case { id: "valid-recognized", vendor: "example.vendor", payload: &[Item { tag: 0x420001, item_type: ItemType::TextString, text: "alpha-v1", number: 0, children: &[] }, Item { tag: 0x420002, item_type: ItemType::Enumeration, text: "", number: 4294967295, children: &[] }, Item { tag: 0x420004, item_type: ItemType::LongInteger, text: "", number: 42, children: &[] }, Item { tag: 0x420006, item_type: ItemType::TextString, text: "preserve-this-child-order", number: 0, children: &[] }, Item { tag: 0x540001, item_type: ItemType::TextString, text: "preserve-vendor-range-tag", number: 0, children: &[] }], outcome: "recognized", matched_ids: &["known.alpha"], typed: true },
    Case { id: "invalid-schema", vendor: "example.vendor", payload: &[Item { tag: 0x420001, item_type: ItemType::TextString, text: "alpha-v1", number: 0, children: &[] }, Item { tag: 0x420004, item_type: ItemType::LongInteger, text: "", number: 100, children: &[] }], outcome: "unrecognized.schema_invalid", matched_ids: &["known.alpha"], typed: false },
    Case { id: "multiply-matching", vendor: "example.vendor", payload: &[Item { tag: 0x540010, item_type: ItemType::Structure, text: "", number: 0, children: &[Item { tag: 0x540011, item_type: ItemType::TextString, text: "route-alpha", number: 0, children: &[] }, Item { tag: 0x540012, item_type: ItemType::TextString, text: "route-beta", number: 0, children: &[] }, Item { tag: 0x540013, item_type: ItemType::TextString, text: "shared", number: 0, children: &[] }] }, Item { tag: 0x540014, item_type: ItemType::Enumeration, text: "", number: 314159, children: &[] }, Item { tag: 0x540015, item_type: ItemType::TextString, text: "preserve-this-child-order", number: 0, children: &[] }], outcome: "unrecognized.ambiguous", matched_ids: &["ambiguous.alpha", "ambiguous.beta"], typed: false },
    Case { id: "unknown-preserved", vendor: "example.vendor", payload: &[Item { tag: 0x540021, item_type: ItemType::Enumeration, text: "", number: 8675309, children: &[] }, Item { tag: 0x540022, item_type: ItemType::TextString, text: "unknown-extension-content", number: 0, children: &[] }], outcome: "unrecognized.no_match", matched_ids: &[], typed: false },
    Case { id: "synthetic-secret-bearing", vendor: "example.vendor", payload: &[Item { tag: 0x420001, item_type: ItemType::TextString, text: "alpha-v1", number: 0, children: &[] }, Item { tag: 0x420004, item_type: ItemType::LongInteger, text: "", number: 42, children: &[] }, Item { tag: 0x420005, item_type: ItemType::TextString, text: "SYNTHETIC-ONLY-EXTENSION-SECRET-7C4A", number: 0, children: &[] }], outcome: "recognized", matched_ids: &["known.alpha"], typed: true },
];
