//! OASIS KMIP 2.1 §§3.4–3.12, Tables 18–26: transparent key material.

use kmipkit_protocol::{
    TransparentDhPrivateKey, TransparentDhPublicKey, TransparentDsaPrivateKey,
    TransparentDsaPublicKey, TransparentEcPrivateKey, TransparentEcPublicKey,
    TransparentRsaPrivateKey, TransparentRsaPublicKey, TransparentSymmetricKey,
};
use kmipkit_ttlv::{ItemType, ValueView};

fn field_tags(item: &kmipkit_ttlv::Item) -> Vec<(u32, ItemType)> {
    item.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .map(|field| (field.tag().raw(), field.item_type()))
            .collect(),
        _ => Vec::new(),
    })
}

#[test]
fn transparent_symmetric_key_contains_required_key_bytes() {
    let item = TransparentSymmetricKey::try_new(vec![0x11, 0x22])
        .expect("valid symmetric key material")
        .into_item();
    assert_eq!(field_tags(&item), [(0x0042_003F, ItemType::ByteString)]);
}

#[test]
fn transparent_dsa_private_and_public_fields_follow_tables_19_and_20() {
    let private = TransparentDsaPrivateKey::try_new(vec![1], vec![2], vec![3], vec![4])
        .expect("required DSA private fields")
        .into_item();
    let public = TransparentDsaPublicKey::try_new(vec![1], vec![2], vec![3], vec![4])
        .expect("required DSA public fields")
        .into_item();
    assert_eq!(
        field_tags(&private),
        [
            (0x0042_005E, ItemType::BigInteger),
            (0x0042_0071, ItemType::BigInteger),
            (0x0042_0037, ItemType::BigInteger),
            (0x0042_009F, ItemType::BigInteger),
        ]
    );
    assert_eq!(
        field_tags(&public),
        [
            (0x0042_005E, ItemType::BigInteger),
            (0x0042_0071, ItemType::BigInteger),
            (0x0042_0037, ItemType::BigInteger),
            (0x0042_00A0, ItemType::BigInteger),
        ]
    );
}

#[test]
fn transparent_rsa_private_supports_only_source_defined_alternatives() {
    let private = TransparentRsaPrivateKey::try_from_components(
        vec![1],
        Some(vec![2]),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("Modulus plus Private Exponent is a §3.7 alternative")
    .into_item();
    assert_eq!(
        field_tags(&private),
        [
            (0x0042_0052, ItemType::BigInteger),
            (0x0042_0063, ItemType::BigInteger)
        ]
    );
    let malformed = TransparentRsaPrivateKey::try_from_components(
        vec![1],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(malformed.is_err());

    let factor_form = TransparentRsaPrivateKey::try_from_components(
        vec![1],
        None,
        None,
        Some(vec![2]),
        Some(vec![3]),
        None,
        None,
        None,
    )
    .expect("P and Q are the second §3.7 alternative")
    .into_item();
    let exponent_form = TransparentRsaPrivateKey::try_from_components(
        vec![1],
        None,
        None,
        None,
        None,
        Some(vec![4]),
        Some(vec![5]),
        None,
    )
    .expect("Prime Exponent P and Q are the third §3.7 alternative")
    .into_item();
    assert_eq!(
        field_tags(&factor_form),
        [
            (0x0042_0052, ItemType::BigInteger),
            (0x0042_005E, ItemType::BigInteger),
            (0x0042_0071, ItemType::BigInteger),
        ]
    );
    assert_eq!(
        field_tags(&exponent_form),
        [
            (0x0042_0052, ItemType::BigInteger),
            (0x0042_0060, ItemType::BigInteger),
            (0x0042_0061, ItemType::BigInteger),
        ]
    );
}

#[test]
fn transparent_rsa_public_requires_modulus_and_public_exponent() {
    let item = TransparentRsaPublicKey::try_new(vec![1], vec![2])
        .expect("required RSA public fields")
        .into_item();
    assert_eq!(
        field_tags(&item),
        [
            (0x0042_0052, ItemType::BigInteger),
            (0x0042_006C, ItemType::BigInteger),
        ]
    );
}

#[test]
fn transparent_dh_private_and_public_preserve_optional_q_and_j() {
    let private = TransparentDhPrivateKey::try_new(vec![1], vec![2], vec![3], None, None)
        .expect("required DH private fields")
        .into_item();
    let public = TransparentDhPublicKey::try_new(vec![1], vec![2], vec![3], None, None)
        .expect("required DH public fields")
        .into_item();
    assert_eq!(
        field_tags(&private),
        [
            (0x0042_005E, ItemType::BigInteger),
            (0x0042_0037, ItemType::BigInteger),
            (0x0042_009F, ItemType::BigInteger),
        ]
    );
    assert_eq!(
        field_tags(&public),
        [
            (0x0042_005E, ItemType::BigInteger),
            (0x0042_0037, ItemType::BigInteger),
            (0x0042_00A0, ItemType::BigInteger),
        ]
    );
    let private_with_optional =
        TransparentDhPrivateKey::try_new(vec![1], vec![2], vec![3], Some(vec![4]), Some(vec![5]))
            .expect("optional DH Q and J are retained")
            .into_item();
    let public_with_optional =
        TransparentDhPublicKey::try_new(vec![1], vec![2], vec![3], Some(vec![4]), Some(vec![5]))
            .expect("optional DH Q and J are retained")
            .into_item();
    assert_eq!(
        field_tags(&private_with_optional),
        [
            (0x0042_005E, ItemType::BigInteger),
            (0x0042_0071, ItemType::BigInteger),
            (0x0042_0037, ItemType::BigInteger),
            (0x0042_003E, ItemType::BigInteger),
            (0x0042_009F, ItemType::BigInteger),
        ]
    );
    assert_eq!(
        field_tags(&public_with_optional),
        [
            (0x0042_005E, ItemType::BigInteger),
            (0x0042_0071, ItemType::BigInteger),
            (0x0042_0037, ItemType::BigInteger),
            (0x0042_003E, ItemType::BigInteger),
            (0x0042_00A0, ItemType::BigInteger),
        ]
    );
}

#[test]
fn transparent_ec_private_and_public_preserve_curve_and_material() {
    let private = TransparentEcPrivateKey::try_new(0x8000_0001, vec![1, 2])
        .expect("future curve value is retained")
        .into_item();
    let public = TransparentEcPublicKey::try_new(0x8000_0001, vec![3, 4])
        .expect("future curve value is retained")
        .into_item();
    assert_eq!(
        field_tags(&private),
        [
            (0x0042_0075, ItemType::Enumeration),
            (0x0042_002E, ItemType::BigInteger),
        ]
    );
    assert_eq!(
        field_tags(&public),
        [
            (0x0042_0075, ItemType::Enumeration),
            (0x0042_0072, ItemType::ByteString),
        ]
    );
}
