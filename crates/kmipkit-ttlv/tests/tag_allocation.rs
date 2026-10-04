//! OASIS KMIP Specification v2.1, §10.1.1 (Tag), Chapter 11 introduction, and §11.56 (Tag
//! Enumeration). Traceability: KMIPKIT-0004-FR-002, KMIPKIT-0004-FR-003,
//! KMIPKIT-0004-NR-001, and KMIPKIT-0004-NR-005. Exact-over-range precedence
//! is the accepted KMIPKit policy in ADR-0010, not an OASIS interpretation.

#[path = "../src/generated/tag_allocations.rs"]
mod tag_allocations;

use kmipkit_ttlv::{RawTag, Tag};
use tag_allocations::{EXACT_TAG_ALLOCATIONS, TAG_ALLOCATION_RANGES, TagAllocationKind};

fn catalog_kind(raw: u32) -> Option<TagAllocationKind> {
    EXACT_TAG_ALLOCATIONS
        .iter()
        .find(|(tag, _)| *tag == raw)
        .map(|(_, kind)| *kind)
        .or_else(|| {
            TAG_ALLOCATION_RANGES
                .iter()
                .find(|(start, end, _)| *start <= raw && raw <= *end)
                .map(|(_, _, kind)| *kind)
        })
}

fn assert_public_check_matches_catalog(raw: u32) {
    let raw_tag = RawTag::new(raw).expect("catalog and range values fit the 24-bit field");

    match catalog_kind(raw) {
        Some(TagAllocationKind::Assigned | TagAllocationKind::Extension) => {
            let checked = RawTag::try_checked(&raw_tag)
                .unwrap_or_else(|_| panic!("catalog-accepted tag 0x{raw:06X} was rejected"));
            assert_eq!(checked.raw(), raw);
        }
        Some(TagAllocationKind::Reserved | TagAllocationKind::Unused) | None => {
            assert!(
                RawTag::try_checked(&raw_tag).is_err(),
                "catalog-rejected tag 0x{raw:06X} was accepted"
            );
        }
    }
}

fn raw_tag(raw: u32) -> RawTag {
    RawTag::new(raw).expect("the test value must fit in 24 bits")
}

fn checked_tag(raw: u32) -> Tag {
    let raw_tag = raw_tag(raw);
    RawTag::try_checked(&raw_tag).expect("the test value must be an allocated KMIP tag")
}

#[test]
fn raw_tag_accepts_24_bit_boundaries_and_rejects_wider_values() {
    for raw in [0x0000_0000, 0x00FF_FFFF] {
        let tag = RawTag::new(raw).expect("the 24-bit boundary must be representable");
        assert_eq!(tag.raw(), raw);
    }

    assert!(RawTag::new(0x0100_0000).is_err());
}

#[test]
fn every_exact_catalog_record_matches_public_allocation_check() {
    let mut assigned = 0;
    let mut reserved = 0;

    for &(raw, kind) in EXACT_TAG_ALLOCATIONS {
        match kind {
            TagAllocationKind::Assigned => assigned += 1,
            TagAllocationKind::Reserved => reserved += 1,
            TagAllocationKind::Unused | TagAllocationKind::Extension => {
                panic!("exact tag record 0x{raw:06X} has a range-only allocation kind")
            }
        }

        assert_public_check_matches_catalog(raw);
    }

    assert_eq!(EXACT_TAG_ALLOCATIONS.len(), 374);
    assert_eq!(assigned, 354);
    assert_eq!(reserved, 20);
}

#[test]
fn every_allocation_range_boundary_and_adjacent_value_matches_catalog() {
    const MAX_RAW_TAG: u32 = 0x00FF_FFFF;

    for &(start, end, _) in TAG_ALLOCATION_RANGES {
        assert!(start <= end);

        let mut candidates = vec![start, end];
        if start < end {
            candidates.extend([start + 1, end - 1, start + (end - start) / 2]);
        }
        if let Some(before) = start.checked_sub(1) {
            candidates.push(before);
        }
        if end < MAX_RAW_TAG {
            candidates.push(end + 1);
        }

        for raw in candidates {
            assert_public_check_matches_catalog(raw);
        }
    }
}

#[test]
fn every_extension_range_tag_is_accepted() {
    let (start, end, kind) = TAG_ALLOCATION_RANGES
        .iter()
        .find(|(_, _, kind)| *kind == TagAllocationKind::Extension)
        .copied()
        .expect("the generated catalog must include the Extensions range");

    assert_eq!(kind, TagAllocationKind::Extension);
    assert_eq!((start, end), (0x0054_0000, 0x0054_FFFF));
    for raw in start..=end {
        assert_public_check_matches_catalog(raw);
    }
}

#[test]
fn exact_project_tags_take_precedence_over_the_aggregate_reserved_range() {
    for raw in [0x0042_0173, 0x0042_0174, 0x0042_0175, 0x0042_0176] {
        let tag = checked_tag(raw);
        assert_eq!(tag.raw(), raw);
    }
}

#[test]
fn individually_and_residually_reserved_tags_are_rejected() {
    for raw in [0x0042_0000, 0x0042_0009, 0x0042_0177, 0x0042_FFFF] {
        let raw_tag = raw_tag(raw);
        assert!(RawTag::try_checked(&raw_tag).is_err());
    }
}

#[test]
fn extension_range_endpoints_and_middle_value_are_accepted() {
    for raw in [0x0054_0000, 0x0054_2BCD, 0x0054_FFFF] {
        let tag = checked_tag(raw);
        assert_eq!(tag.raw(), raw);
    }
}

#[test]
fn unused_values_are_rejected_while_remaining_raw_and_failed_checks_borrow() {
    for raw in [0x0000_0000, 0x0043_0000, 0x0055_0000, 0x00FF_FFFF] {
        let tag = raw_tag(raw);
        assert!(RawTag::try_checked(&tag).is_err());
        assert_eq!(tag.raw(), raw);
    }

    let reserved = raw_tag(0x0042_0009);
    assert!(RawTag::try_checked(&reserved).is_err());
    assert_eq!(reserved.raw(), 0x0042_0009);
}
