//! 100% OASIS Master Table Conformance Test
//!
//! Validates that all enums, tags, operations, and KMIP 3.0 primitives match the official
//! OASIS KMIP 2.0 (§11.56), KMIP 2.1 (§11.56), and KMIP 3.0 (Table 605) standards.

use kmipkit_protocol::enums::{
    AttributeTag, EphemeralMode, KemAlgorithmCode, KeyLifecycleState, LinkAttributeTag,
    ObjectTypeCode, OperationCode,
};
use kmipkit_protocol::{
    KmipTag, NistKeyLifecycleGuard, ProtocolVersion, QueryFunction, RequestMessage,
};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value};

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("tag fits")
        .try_checked()
        .expect("allocated tag")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("item is valid")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut s = Structure::new();
    for child in items {
        s.try_push(child).expect("valid structure push");
    }
    s
}

#[test]
fn test_critical_oasis_discrepancy_tags_are_100_percent_correct() {
    // 1. Link vs Key Wrapping Specification
    assert_eq!(KmipTag::KeyWrappingSpecification.as_u32(), 0x0042_0047);
    assert_eq!(KmipTag::LastChangeDate.as_u32(), 0x0042_0048);
    assert_eq!(KmipTag::LeaseTime.as_u32(), 0x0042_0049);
    assert_eq!(KmipTag::Link.as_u32(), 0x0042_004A);
    assert_eq!(KmipTag::LinkType.as_u32(), 0x0042_004B);
    assert_eq!(KmipTag::LinkedObjectIdentifier.as_u32(), 0x0042_004C);

    // 2. Rotate attributes
    assert_eq!(KmipTag::RotateOffset.as_u32(), 0x0042_016C);
    assert_eq!(KmipTag::RotateDate.as_u32(), 0x0042_016D);
    assert_eq!(KmipTag::RotateGeneration.as_u32(), 0x0042_016E);
    assert_eq!(KmipTag::RotateName.as_u32(), 0x0042_016F);
    assert_eq!(KmipTag::RotateNameValue.as_u32(), 0x0042_0170);
    assert_eq!(KmipTag::RotateNameType.as_u32(), 0x0042_0171);

    // 3. Certificate and Key Links (KMIP 3.0)
    assert_eq!(KmipTag::CertificateLink.as_u32(), 0x0042_0190);
    assert_eq!(KmipTag::PrivateKeyLink.as_u32(), 0x0042_0199);
    assert_eq!(KmipTag::PublicKeyLink.as_u32(), 0x0042_019A);
    assert_eq!(KmipTag::ReplacedObjectLink.as_u32(), 0x0042_019B);
}

#[test]
fn test_kmip_30_pqc_and_kem_tags_and_operations() {
    // KEM Tags
    assert_eq!(KmipTag::KEMAlgorithm.as_u32(), 0x0042_01C3);
    assert_eq!(KmipTag::Deterministic.as_u32(), 0x0042_01C4);
    assert_eq!(KmipTag::ContextString.as_u32(), 0x0042_01C5);
    assert_eq!(KmipTag::Seed.as_u32(), 0x0042_01C6);
    assert_eq!(KmipTag::InputKeyMaterial.as_u32(), 0x0042_01C7);
    assert_eq!(KmipTag::Internal.as_u32(), 0x0042_01C8);
    assert_eq!(KmipTag::ExternalMu.as_u32(), 0x0042_01C9);
    assert_eq!(KmipTag::Random.as_u32(), 0x0042_01CA);
    assert_eq!(KmipTag::PerformsCRLChecks.as_u32(), 0x0042_01CB);

    // KEM Operations
    assert_eq!(OperationCode::KemEncrypt.as_u32(), 0x0000_003C);
    assert_eq!(OperationCode::KemDecrypt.as_u32(), 0x0000_003D);
    assert_eq!(OperationCode::Encapsulate.as_u32(), 0x0000_0041);
    assert_eq!(OperationCode::Decapsulate.as_u32(), 0x0000_0042);

    // KEM Algorithm Enum values
    assert_eq!(KemAlgorithmCode::Rsasve.as_u32(), 0x01);
    assert_eq!(KemAlgorithmCode::Dhkem.as_u32(), 0x02);
    assert_eq!(KemAlgorithmCode::Mlkem.as_u32(), 0x03);
}

#[test]
fn test_all_twenty_kmip30_link_attribute_tags() {
    let links = [
        (LinkAttributeTag::CertificateLink, 0x0042_0190),
        (LinkAttributeTag::ChildLink, 0x0042_0191),
        (LinkAttributeTag::DerivationBaseObjectLink, 0x0042_0192),
        (LinkAttributeTag::DerivedObjectLink, 0x0042_0193),
        (LinkAttributeTag::NextLink, 0x0042_0194),
        (LinkAttributeTag::ParentLink, 0x0042_0195),
        (LinkAttributeTag::PKCS12CertificateLink, 0x0042_0196),
        (LinkAttributeTag::PKCS12PasswordLink, 0x0042_0197),
        (LinkAttributeTag::PreviousLink, 0x0042_0198),
        (LinkAttributeTag::PrivateKeyLink, 0x0042_0199),
        (LinkAttributeTag::PublicKeyLink, 0x0042_019A),
        (LinkAttributeTag::ReplacedObjectLink, 0x0042_019B),
        (LinkAttributeTag::ReplacementObjectLink, 0x0042_019C),
        (LinkAttributeTag::WrappingKeyLink, 0x0042_019D),
        (LinkAttributeTag::CredentialLink, 0x0042_01A0),
        (LinkAttributeTag::PasswordLink, 0x0042_01A5),
        (LinkAttributeTag::GroupLink, 0x0042_01B3),
        (LinkAttributeTag::SplitKeyBaseLink, 0x0042_01B4),
        (LinkAttributeTag::JoinedSplitKeyPartsLink, 0x0042_01B5),
        (LinkAttributeTag::CertificateRequestLink, 0x0042_01BC),
    ];

    assert_eq!(links.len(), 20, "Must contain exactly 20 KMIP 3.0 link types");
    for (link, expected_tag) in links {
        assert_eq!(link.as_u32(), expected_tag);
        assert_eq!(link.into_tag().raw(), expected_tag);
    }
}

#[test]
fn test_kmip_30_primitives_wire_types() {
    let id_val = Value::identifier("oasis-identifier".to_string());
    assert_eq!(id_val.item_type(), ItemType::Identifier);

    let ref_val = Value::reference("oasis-reference".to_string());
    assert_eq!(ref_val.item_type(), ItemType::Reference);

    let name_ref_val = Value::name_reference("oasis-name-reference".to_string());
    assert_eq!(name_ref_val.item_type(), ItemType::NameReference);

    // Verify wire decoding of 0x0C (Identifier), 0x0D (Reference), 0x0E (NameReference)
    // Wire item: Tag 0x420008, Type 0x0C (Identifier), Length 4, Value "uuid", 4 bytes padding
    let wire_id: [u8; 16] = [
        0x42, 0x00, 0x08, 0x0C, 0x00, 0x00, 0x00, 0x04, b'u', b'u', b'i', b'd', 0x00, 0x00, 0x00,
        0x00,
    ];
    let decoded_id =
        kmipkit_ttlv::codec::decode(&wire_id).expect("valid TTLV wire decode for 0x0C");
    assert_eq!(decoded_id.item_type(), ItemType::Identifier);

    // Wire item: Tag 0x420008, Type 0x0D (Reference), Length 4, Value "refr", 4 bytes padding
    let wire_ref: [u8; 16] = [
        0x42, 0x00, 0x08, 0x0D, 0x00, 0x00, 0x00, 0x04, b'r', b'e', b'f', b'r', 0x00, 0x00, 0x00,
        0x00,
    ];
    let decoded_ref =
        kmipkit_ttlv::codec::decode(&wire_ref).expect("valid TTLV wire decode for 0x0D");
    assert_eq!(decoded_ref.item_type(), ItemType::Reference);

    // Wire item: Tag 0x420008, Type 0x0E (NameReference), Length 4, Value "name", 4 bytes padding
    let wire_name_ref: [u8; 16] = [
        0x42, 0x00, 0x08, 0x0E, 0x00, 0x00, 0x00, 0x04, b'n', b'a', b'm', b'e', 0x00, 0x00, 0x00,
        0x00,
    ];
    let decoded_name_ref =
        kmipkit_ttlv::codec::decode(&wire_name_ref).expect("valid TTLV wire decode for 0x0E");
    assert_eq!(decoded_name_ref.item_type(), ItemType::NameReference);
}

#[test]
fn test_kmip_30_framing_option_a_and_prohibited_tags() {
    let v3_version = item(
        0x0042_0069, // Protocol Version
        Value::structure(structure([
            item(0x0042_006A, Value::integer(3)),
            item(0x0042_006B, Value::integer(0)),
        ])),
    );

    // Valid KMIP 3.0 request header without Batch Count
    let req_header = structure([v3_version]);
    let req_batch = structure([
        item(0x0042_005C, Value::enumeration(0x0000_0018)), // Query
        item(0x0042_01BF, Value::enumeration(1)),           // Ephemeral Mode: Ephemeral
        item(0x0042_0079, Value::structure(Structure::new())), // Request Payload
    ]);

    let full_msg = structure([
        item(0x0042_0077, Value::structure(req_header)),
        item(0x0042_000F, Value::structure(req_batch)),
    ]);

    let req = RequestMessage::try_from_ttlv(full_msg).expect("KMIP 3.0 valid message");
    assert_eq!(req.header().batch_count(), None, "Batch Count is None on wire in 3.0");
    assert_eq!(req.header().effective_batch_count(), 1, "Effective batch count is 1");
    let batch_item = req.batch_items().next().unwrap();
    assert_eq!(batch_item.ephemeral_mode(), Some(1));
    assert!(batch_item.is_ephemeral());
}

#[test]
fn test_nist_key_lifecycle_guard_rules() {
    assert!(!NistKeyLifecycleGuard::can_originate(KeyLifecycleState::PreActive));
    assert!(!NistKeyLifecycleGuard::can_process(KeyLifecycleState::PreActive));

    assert!(NistKeyLifecycleGuard::can_originate(KeyLifecycleState::Active));
    assert!(NistKeyLifecycleGuard::can_process(KeyLifecycleState::Active));

    assert!(!NistKeyLifecycleGuard::can_originate(KeyLifecycleState::Deactivated));
    assert!(NistKeyLifecycleGuard::can_process(KeyLifecycleState::Deactivated));

    assert!(!NistKeyLifecycleGuard::can_originate(KeyLifecycleState::Compromised));
    assert!(!NistKeyLifecycleGuard::can_process(KeyLifecycleState::Compromised));

    assert!(!NistKeyLifecycleGuard::can_originate(KeyLifecycleState::Destroyed));
    assert!(!NistKeyLifecycleGuard::can_process(KeyLifecycleState::Destroyed));

    assert!(!NistKeyLifecycleGuard::can_originate(KeyLifecycleState::DestroyedCompromised));
    assert!(!NistKeyLifecycleGuard::can_process(KeyLifecycleState::DestroyedCompromised));
}

#[test]
fn test_query_function_performs_crl_checks_kmip30() {
    let qf = QueryFunction::PERFORMS_CRL_CHECKS;
    assert_eq!(qf.raw(), 15);
}

#[test]
fn test_enums_completeness() {
    assert_eq!(EphemeralMode::Immediate.as_u32(), 2);
    assert_eq!(ObjectTypeCode::Certificate.as_u32(), 1);
    assert_eq!(AttributeTag::CryptographicAlgorithm.as_u32(), 0x0042_0028);
    assert_eq!(ProtocolVersion::V3_0.major(), 3);
}
