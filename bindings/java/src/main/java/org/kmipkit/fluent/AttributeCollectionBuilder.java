package org.kmipkit.fluent;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TypedAttribute;

/** Owns ordered attribute copies for Register and Create Split Key. */
abstract class AttributeCollectionBuilder<B extends AttributeCollectionBuilder<B>> extends TypedOperationBuilder<B> {
    private final List<TtlvItem> attributes = new ArrayList<>();
    private final TreeMap<Integer, TtlvItem> positioned = new TreeMap<>();

    AttributeCollectionBuilder(KmipClient client) {
        super(client);
    }

    public final B attribute(TypedAttribute value) {
        try (TtlvItem item = value.asItem()) {
            return attributeItem(item);
        }
    }

    public final B attributeItem(TtlvItem value) {
        ensureOpen();
        validateAttribute(value);
        attributes.add(own(value.copy()));
        return self();
    }

    /** Replaces the ordinary attributes with independent copies in their original order. */
    public final B attributes(AttributeSet values) {
        ensureOpen();
        List<TtlvItem> copies = new ArrayList<>();
        try (TtlvStructure items = values.asItems(); TtlvStructureView view = items.view()) {
            for (long index = 0; index < view.itemCount(); index++) {
                try (TtlvItemView item = view.itemAt(index)) {
                    copies.add(item.copy());
                }
            }
        } catch (RuntimeException error) {
            copies.forEach(TtlvItem::close);
            throw error;
        }
        attributes.forEach(this::discard);
        attributes.clear();
        for (TtlvItem item : copies) attributes.add(own(item));
        return self();
    }

    public final B attributeAt(int position, TtlvItem value) {
        ensureOpen();
        if (position < 0 || positioned.containsKey(position)) {
            throw new InvalidInputException("attribute position is negative or repeated");
        }
        validateAttribute(value);
        positioned.put(position, own(value.copy()));
        return self();
    }

    public final B vendorAttribute(String vendor, String name, TtlvItem value) {
        ensureOpen();
        if (vendor == null || name == null || value == null) {
            throw new InvalidInputException("vendor attribute fields are required");
        }
        TtlvStructure fields = TtlvStructure.create();
        try {
            fields = TtlvStructure.withItem(fields, text(0x42009d, vendor), CodecLimits.defaults());
            fields = TtlvStructure.withItem(fields, text(0x42000a, name), CodecLimits.defaults());
            try (Tag tag = RawTag.fromRaw(0x42000b).checked(); TtlvValue copy = value.valueCopy()) {
                fields = TtlvStructure.withItem(fields, TtlvItem.create(tag, copy, CodecLimits.defaults()), CodecLimits.defaults());
            }
            try (Tag tag = RawTag.fromRaw(0x420008).checked();
                 TtlvValue structure = TtlvValue.structure(fields, CodecLimits.defaults());
                 TtlvItem attribute = TtlvItem.create(tag, structure, CodecLimits.defaults())) {
                return attributeItem(attribute);
            }
        } finally {
            fields.close();
        }
    }

    private static TtlvItem text(int rawTag, String text) {
        if (!StandardCharsets.UTF_8.newEncoder().canEncode(text)) {
            throw new InvalidInputException("attribute text is not valid Unicode");
        }
        try (Tag tag = RawTag.fromRaw(rawTag).checked();
             TtlvValue value = TtlvValue.TextString(text.getBytes(StandardCharsets.UTF_8), CodecLimits.defaults())) {
            return TtlvItem.create(tag, value, CodecLimits.defaults());
        }
    }

    private static void validateAttribute(TtlvItem value) {
        if (value == null) throw new InvalidInputException("attribute is required");
        try (TtlvStructure empty = TtlvStructure.create(); AttributeSet set = AttributeSet.tryNew(empty)) {
            set.tryPush(value);
        }
    }

    protected final TtlvStructure buildAttributes() {
        ensureOpen();
        List<TtlvItem> ordered = new ArrayList<>(attributes);
        positioned.forEach((index, item) -> {
            if (index > ordered.size()) throw new InvalidInputException("attribute position exceeds collection size");
            ordered.add(index, item);
        });
        TtlvStructure items = TtlvStructure.create();
        try {
            for (TtlvItem item : ordered) {
                items = TtlvStructure.withItem(items, item.copy(), CodecLimits.defaults());
            }
            return items;
        } catch (RuntimeException error) {
            items.close();
            throw error;
        }
    }
}
