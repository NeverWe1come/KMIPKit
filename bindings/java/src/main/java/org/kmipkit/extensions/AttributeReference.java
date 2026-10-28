package org.kmipkit.extensions;

import java.util.Objects;
import java.util.Optional;
import java.util.OptionalLong;

import org.kmipkit.InvalidInputException;

/** Lossless KMIP Attribute Reference in tag or exact name form. */
public final class AttributeReference {
    private final long tagValue;
    private final String vendorIdentification;
    private final String attributeName;

    private AttributeReference(long tagValue, String vendorIdentification, String attributeName) {
        this.tagValue = tagValue;
        this.vendorIdentification = vendorIdentification;
        this.attributeName = attributeName;
    }

    /** Creates a tag-form reference while preserving every raw uint32 value. */
    public static AttributeReference tag(long rawTag) {
        if (rawTag < 0 || rawTag > 0xffff_ffffL) {
            throw new InvalidInputException("Attribute Reference tag is outside uint32");
        }
        return new AttributeReference(rawTag, null, null);
    }

    /** Creates a name-form reference preserving the exact source text. */
    public static AttributeReference name(String vendor, String name) {
        Objects.requireNonNull(vendor, "vendorIdentification");
        Objects.requireNonNull(name, "attributeName");
        ExtensionText.encodeUtf8(vendor, UniqueIdentifier.MAX_NATIVE_TEXT_BYTES,
                "Vendor Identification");
        ExtensionText.encodeUtf8(name, UniqueIdentifier.MAX_NATIVE_TEXT_BYTES, "Attribute Name");
        if (!vendor.chars().allMatch(value -> value >= 'A' && value <= 'Z'
                || value >= 'a' && value <= 'z' || value >= '0' && value <= '9'
                || value == '_' || value == '.')) {
            throw new InvalidInputException("Vendor Identification contains an invalid character");
        }
        return new AttributeReference(0, vendor, name);
    }

    /** Returns the raw tag value when this reference uses tag form. */
    public OptionalLong tagValue() {
        return vendorIdentification == null ? OptionalLong.of(tagValue) : OptionalLong.empty();
    }

    /** Returns the exact Vendor Identification for name form. */
    public Optional<String> vendorIdentification() {
        return Optional.ofNullable(vendorIdentification);
    }

    /** Returns the exact Attribute Name for name form. */
    public Optional<String> attributeName() {
        return Optional.ofNullable(attributeName);
    }

    @Override
    public boolean equals(Object other) {
        if (!(other instanceof AttributeReference value)) return false;
        return tagValue == value.tagValue
                && Objects.equals(vendorIdentification, value.vendorIdentification)
                && Objects.equals(attributeName, value.attributeName);
    }

    @Override
    public int hashCode() {
        return Objects.hash(tagValue, vendorIdentification, attributeName);
    }

    @Override
    public String toString() {
        return vendorIdentification == null
                ? "AttributeReference[tag=0x" + Long.toHexString(tagValue) + "]"
                : "AttributeReference[name=" + vendorIdentification + "/" + attributeName + "]";
    }
}
