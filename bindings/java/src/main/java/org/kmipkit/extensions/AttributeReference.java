package org.kmipkit.extensions;

import java.util.Objects;
import java.util.Optional;
import java.util.OptionalLong;
import org.kmipkit.InvalidInputException;

/** Lossless KMIP §5.5 Attribute Reference in raw tag or exact name form. */
public final class AttributeReference {
    static final int KIND_TAG = 1;
    static final int KIND_NAME = 2;

    private final int kind;
    private final long tagValue;
    private final String vendorIdentification;
    private final String attributeName;

    private AttributeReference(
            int kind, long tagValue, String vendorIdentification, String attributeName) {
        this.kind = kind;
        this.tagValue = tagValue;
        this.vendorIdentification = vendorIdentification;
        this.attributeName = attributeName;
    }

    /** Creates a tag-form reference while retaining the exact unsigned Enumeration value. */
    public static AttributeReference tag(long rawTag) {
        if (rawTag < 0 || rawTag > 0xffff_ffffL) {
            throw new InvalidInputException("Attribute Reference tag is outside uint32");
        }
        return new AttributeReference(KIND_TAG, rawTag, null, null);
    }

    /** Creates a name-form reference with exact Vendor Identification and Attribute Name text. */
    public static AttributeReference name(String vendorIdentification, String attributeName) {
        Objects.requireNonNull(vendorIdentification, "vendorIdentification");
        Objects.requireNonNull(attributeName, "attributeName");
        byte[] vendorBytes = ExtensionText.encodeUtf8(
                vendorIdentification, UniqueIdentifier.MAX_NATIVE_TEXT_BYTES,
                "Attribute Reference Vendor Identification");
        byte[] nameBytes = ExtensionText.encodeUtf8(
                attributeName, UniqueIdentifier.MAX_NATIVE_TEXT_BYTES,
                "Attribute Reference Attribute Name");
        for (byte value : vendorBytes) {
            int character = Byte.toUnsignedInt(value);
            if (!(character >= 'A' && character <= 'Z'
                    || character >= 'a' && character <= 'z'
                    || character >= '0' && character <= '9'
                    || character == '_' || character == '.')) {
                throw new InvalidInputException("Vendor Identification is invalid");
            }
        }
        return new AttributeReference(KIND_NAME, 0, vendorIdentification, attributeName);
    }

    /** Returns whether this reference uses the raw tag form. */
    public boolean isTag() {
        return kind == KIND_TAG;
    }

    /** Returns the raw unsigned tag value when this reference uses tag form. */
    public OptionalLong tagValue() {
        return isTag() ? OptionalLong.of(tagValue) : OptionalLong.empty();
    }

    /** Returns the exact Vendor Identification for name-form references. */
    public Optional<String> vendorIdentification() {
        return Optional.ofNullable(vendorIdentification);
    }

    /** Returns the exact Attribute Name for name-form references. */
    public Optional<String> attributeName() {
        return Optional.ofNullable(attributeName);
    }

    int nativeKind() {
        return kind;
    }

    long nativeTagValue() {
        return tagValue;
    }

    byte[] nativeVendorIdentification() {
        return vendorIdentification == null ? null : ExtensionText.encodeUtf8(
                vendorIdentification, UniqueIdentifier.MAX_NATIVE_TEXT_BYTES,
                "Attribute Reference Vendor Identification");
    }

    byte[] nativeAttributeName() {
        return attributeName == null ? null : ExtensionText.encodeUtf8(
                attributeName, UniqueIdentifier.MAX_NATIVE_TEXT_BYTES,
                "Attribute Reference Attribute Name");
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) return true;
        if (!(other instanceof AttributeReference reference)) return false;
        return kind == reference.kind
                && tagValue == reference.tagValue
                && Objects.equals(vendorIdentification, reference.vendorIdentification)
                && Objects.equals(attributeName, reference.attributeName);
    }

    @Override
    public int hashCode() {
        return Objects.hash(kind, tagValue, vendorIdentification, attributeName);
    }

    @Override
    public String toString() {
        if (isTag()) {
            return "AttributeReference[tag=0x" + Long.toHexString(tagValue) + "]";
        }
        return "AttributeReference[name=" + vendorIdentification + ":" + attributeName + "]";
    }
}
