// Generated from specification/api/public-api.json and tests/fixtures/extensions/cases.json. API contract sha256: db9618f0b89ec0c9c6842c3be1749aea88202d426890f19ca5e747dd5ab575f1. Do not edit.
use kmipkit_ttlv::ItemType;

#[derive(Clone, Copy)]
#[rustfmt::skip]
pub struct Item { pub tag: u32, pub kind: ItemType, pub text: &'static str, pub number: i64, pub children: &'static [Item] }
#[derive(Clone, Copy)]
#[rustfmt::skip]
pub struct ChildRule { pub tag: u32, pub schema: Schema }
#[derive(Clone, Copy)]
#[rustfmt::skip]
pub struct Schema { pub kind: ItemType, pub has_range: bool, pub minimum: i64, pub maximum: i64, pub children: &'static [ChildRule] }
#[rustfmt::skip]
pub struct Definition { pub id: &'static str, pub vendor: &'static str, pub name: &'static str, pub version: &'static str, pub path: &'static [u32], pub discriminator_type: ItemType, pub discriminator_text: &'static str, pub discriminator_number: i64, pub schema: Schema }
#[rustfmt::skip]
pub struct Case { pub id: &'static str, pub vendor: &'static str, pub critical: bool, pub payload: &'static [Item], pub outcome: &'static str, pub matched_ids: &'static [&'static str], pub typed: bool }
#[derive(Clone, Copy)]
#[rustfmt::skip]
pub struct OutboundAttachment { pub fixture_id: &'static str, pub criticality_indicator: bool }
#[derive(Clone, Copy)]
#[rustfmt::skip]
pub struct OutboundRequest { pub fixture_id: &'static str, pub outcome: &'static str, pub attachments: &'static [OutboundAttachment] }

#[rustfmt::skip]
pub static DEFINITIONS: &[Definition] = &[
    Definition { id: "ambiguous.beta", vendor: "example.vendor", name: "ambiguous-beta", version: "1", path: &[0x0054_0010, 0x0054_0012], discriminator_type: ItemType::TextString, discriminator_text: "route-beta", discriminator_number: 0, schema: Schema { kind: ItemType::Structure, has_range: false, minimum: 0, maximum: 0, children: &[ChildRule { tag: 0x0054_0010, schema: Schema { kind: ItemType::Structure, has_range: false, minimum: 0, maximum: 0, children: &[ChildRule { tag: 0x0054_0011, schema: Schema { kind: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }, ChildRule { tag: 0x0054_0012, schema: Schema { kind: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }, ChildRule { tag: 0x0054_0013, schema: Schema { kind: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }] } }] } },
    Definition { id: "known.alpha", vendor: "example.vendor", name: "alpha", version: "1", path: &[0x0042_0001], discriminator_type: ItemType::TextString, discriminator_text: "alpha-v1", discriminator_number: 0, schema: Schema { kind: ItemType::Structure, has_range: false, minimum: 0, maximum: 0, children: &[ChildRule { tag: 0x0042_0001, schema: Schema { kind: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }, ChildRule { tag: 0x0042_0004, schema: Schema { kind: ItemType::LongInteger, has_range: true, minimum: 0, maximum: 99, children: &[] } }] } },
    Definition { id: "ambiguous.alpha", vendor: "example.vendor", name: "ambiguous-alpha", version: "1", path: &[0x0054_0010, 0x0054_0011], discriminator_type: ItemType::TextString, discriminator_text: "route-alpha", discriminator_number: 0, schema: Schema { kind: ItemType::Structure, has_range: false, minimum: 0, maximum: 0, children: &[ChildRule { tag: 0x0054_0010, schema: Schema { kind: ItemType::Structure, has_range: false, minimum: 0, maximum: 0, children: &[ChildRule { tag: 0x0054_0011, schema: Schema { kind: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }, ChildRule { tag: 0x0054_0012, schema: Schema { kind: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }, ChildRule { tag: 0x0054_0013, schema: Schema { kind: ItemType::TextString, has_range: false, minimum: 0, maximum: 0, children: &[] } }] } }] } },
];

#[rustfmt::skip]
pub static CASES: &[Case] = &[
    Case { id: "valid-recognized", vendor: "example.vendor", critical: false, payload: &[Item { tag: 0x0042_0001, kind: ItemType::TextString, text: "alpha-v1", number: 0, children: &[] }, Item { tag: 0x0042_0002, kind: ItemType::Enumeration, text: "", number: 4_294_967_295, children: &[] }, Item { tag: 0x0042_0004, kind: ItemType::LongInteger, text: "", number: 42, children: &[] }, Item { tag: 0x0042_0006, kind: ItemType::TextString, text: "preserve-this-child-order", number: 0, children: &[] }, Item { tag: 0x0054_0001, kind: ItemType::TextString, text: "preserve-vendor-range-tag", number: 0, children: &[] }], outcome: "recognized", matched_ids: &["known.alpha"], typed: true },
    Case { id: "invalid-schema", vendor: "example.vendor", critical: false, payload: &[Item { tag: 0x0042_0001, kind: ItemType::TextString, text: "alpha-v1", number: 0, children: &[] }, Item { tag: 0x0042_0004, kind: ItemType::LongInteger, text: "", number: 100, children: &[] }], outcome: "unrecognized.schema_invalid", matched_ids: &["known.alpha"], typed: false },
    Case { id: "multiply-matching", vendor: "example.vendor", critical: false, payload: &[Item { tag: 0x0054_0010, kind: ItemType::Structure, text: "", number: 0, children: &[Item { tag: 0x0054_0011, kind: ItemType::TextString, text: "route-alpha", number: 0, children: &[] }, Item { tag: 0x0054_0012, kind: ItemType::TextString, text: "route-beta", number: 0, children: &[] }, Item { tag: 0x0054_0013, kind: ItemType::TextString, text: "shared", number: 0, children: &[] }] }, Item { tag: 0x0054_0014, kind: ItemType::Enumeration, text: "", number: 314_159, children: &[] }, Item { tag: 0x0054_0015, kind: ItemType::TextString, text: "preserve-this-child-order", number: 0, children: &[] }], outcome: "unrecognized.ambiguous", matched_ids: &["ambiguous.alpha", "ambiguous.beta"], typed: false },
    Case { id: "unknown-preserved", vendor: "example.vendor", critical: false, payload: &[Item { tag: 0x0054_0021, kind: ItemType::Enumeration, text: "", number: 8_675_309, children: &[] }, Item { tag: 0x0054_0022, kind: ItemType::TextString, text: "unknown-extension-content", number: 0, children: &[] }], outcome: "unrecognized.no_match", matched_ids: &[], typed: false },
    Case { id: "synthetic-secret-bearing", vendor: "example.vendor", critical: false, payload: &[Item { tag: 0x0042_0001, kind: ItemType::TextString, text: "alpha-v1", number: 0, children: &[] }, Item { tag: 0x0042_0004, kind: ItemType::LongInteger, text: "", number: 42, children: &[] }, Item { tag: 0x0042_0005, kind: ItemType::TextString, text: "SYNTHETIC-ONLY-EXTENSION-SECRET-7C4A", number: 0, children: &[] }], outcome: "recognized", matched_ids: &["known.alpha"], typed: true },
];

#[rustfmt::skip]
pub static OUTBOUND_REQUESTS: &[OutboundRequest] = &[
    OutboundRequest { fixture_id: "synthetic-secret-bearing", outcome: "outbound.validated", attachments: &[OutboundAttachment { fixture_id: "synthetic-secret-bearing", criticality_indicator: false }, OutboundAttachment { fixture_id: "valid-recognized", criticality_indicator: true }] },
];
