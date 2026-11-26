package org.kmipkit.extensions;

import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;

/**
 * Opt-in interoperability helpers for vendor-specific KMIP attribute extensions
 * (such as Cosmian KMS Attribute Index, Critical, and Object Group Member).
 * These helpers construct explicit TTLV items without relaxing strict KMIP 2.0/2.1 validation.
 */
public final class VendorExtensions {
    /** KMIP 1.x / Cosmian vendor Attribute Index tag (Integer). */
    public static final int TAG_ATTRIBUTE_INDEX = 0x0042_0009;
    /** KMIP Criticality Indicator / Cosmian vendor Critical attribute tag (Boolean). */
    public static final int TAG_CRITICAL = 0x0042_0026;
    /** KMIP Object Group Member tag used as a vendor attribute (Enumeration). */
    public static final int TAG_OBJECT_GROUP_MEMBER = 0x0042_00AC;

    private VendorExtensions() { }

    public static TtlvItem attributeIndex(int index) {
        try (RawTag raw = RawTag.fromRaw(TAG_ATTRIBUTE_INDEX);
             Tag tag = raw.checked();
             TtlvValue value = TtlvValue.Integer(index)) {
            return TtlvItem.create(tag, value, CodecLimits.defaults());
        }
    }

    public static TtlvItem critical(boolean value) {
        try (RawTag raw = RawTag.fromRaw(TAG_CRITICAL);
             Tag tag = raw.checked();
             TtlvValue ttlvValue = TtlvValue.Boolean(value)) {
            return TtlvItem.create(tag, ttlvValue, CodecLimits.defaults());
        }
    }

    public static TtlvItem objectGroupMember(int memberEnumeration) {
        try (RawTag raw = RawTag.fromRaw(TAG_OBJECT_GROUP_MEMBER);
             Tag tag = raw.checked();
             TtlvValue value = TtlvValue.Enumeration(memberEnumeration)) {
            return TtlvItem.create(tag, value, CodecLimits.defaults());
        }
    }
}
