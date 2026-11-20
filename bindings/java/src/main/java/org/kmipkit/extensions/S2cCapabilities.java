package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import org.jspecify.annotations.Nullable;
import org.kmipkit.InvalidInputException;

/** Server-to-Client capabilities advertised during Query negotiations. */
public final class S2cCapabilities {
    private static final int MAX_CAPABILITY_ITEMS = 1024;
    private static final long MAX_UINT32 = 0xFFFF_FFFFL;

    private List<Long> operations = new ArrayList<>();
    private List<Long> objectTypes = new ArrayList<>();
    private @Nullable String vendorIdentification;
    private List<Long> extensionTags = new ArrayList<>();

    public S2cCapabilities() {}

    public S2cCapabilities(
            List<Long> operations,
            List<Long> objectTypes,
            @Nullable String vendorIdentification,
            List<Long> extensionTags) {
        this.operations = validateAndCopy(operations, "operations");
        this.objectTypes = validateAndCopy(objectTypes, "objectTypes");
        this.vendorIdentification = vendorIdentification;
        this.extensionTags = validateAndCopy(extensionTags, "extensionTags");
    }

    private static List<Long> validateAndCopy(List<Long> values, String fieldName) {
        if (values == null) {
            return new ArrayList<>();
        }
        if (values.size() > MAX_CAPABILITY_ITEMS) {
            throw new InvalidInputException(fieldName + " exceeds maximum allowed items (" + MAX_CAPABILITY_ITEMS + ")");
        }
        List<Long> copy = new ArrayList<>(values.size());
        for (Long v : values) {
            if (v == null) {
                throw new InvalidInputException(fieldName + " contains null element");
            }
            if (v < 0L || v > MAX_UINT32) {
                throw new InvalidInputException(fieldName + " element out of uint32 range [0..0xFFFFFFFF]: " + v);
            }
            copy.add(v);
        }
        return copy;
    }

    public List<Long> operations() { return Collections.unmodifiableList(operations); }
    public S2cCapabilities operations(List<Long> ops) {
        this.operations = validateAndCopy(ops, "operations");
        return this;
    }

    public List<Long> objectTypes() { return Collections.unmodifiableList(objectTypes); }
    public S2cCapabilities objectTypes(List<Long> types) {
        this.objectTypes = validateAndCopy(types, "objectTypes");
        return this;
    }

    public @Nullable String vendorIdentification() { return vendorIdentification; }
    public S2cCapabilities vendorIdentification(@Nullable String vendorId) {
        this.vendorIdentification = vendorId;
        return this;
    }

    public List<Long> extensionTags() { return Collections.unmodifiableList(extensionTags); }
    public S2cCapabilities extensionTags(List<Long> tags) {
        this.extensionTags = validateAndCopy(tags, "extensionTags");
        return this;
    }
}
