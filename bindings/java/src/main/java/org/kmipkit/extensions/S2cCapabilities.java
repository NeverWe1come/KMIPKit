package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import org.jspecify.annotations.Nullable;

/** Server-to-Client capabilities advertised during Query negotiations. */
public final class S2cCapabilities {
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
        if (operations != null) this.operations = new ArrayList<>(operations);
        if (objectTypes != null) this.objectTypes = new ArrayList<>(objectTypes);
        this.vendorIdentification = vendorIdentification;
        if (extensionTags != null) this.extensionTags = new ArrayList<>(extensionTags);
    }

    public List<Long> operations() { return Collections.unmodifiableList(operations); }
    public S2cCapabilities operations(List<Long> ops) {
        this.operations = ops == null ? new ArrayList<>() : new ArrayList<>(ops);
        return this;
    }

    public List<Long> objectTypes() { return Collections.unmodifiableList(objectTypes); }
    public S2cCapabilities objectTypes(List<Long> types) {
        this.objectTypes = types == null ? new ArrayList<>() : new ArrayList<>(types);
        return this;
    }

    public @Nullable String vendorIdentification() { return vendorIdentification; }
    public S2cCapabilities vendorIdentification(@Nullable String vendorId) {
        this.vendorIdentification = vendorId;
        return this;
    }

    public List<Long> extensionTags() { return Collections.unmodifiableList(extensionTags); }
    public S2cCapabilities extensionTags(List<Long> tags) {
        this.extensionTags = tags == null ? new ArrayList<>() : new ArrayList<>(tags);
        return this;
    }
}
