package org.kmipkit.extensions;

import java.util.Optional;
import java.util.OptionalLong;
import org.kmipkit.ttlv.TtlvItemView;

/** One Rust-classified Query response field at its original wire-order position. */
public final class QueryResponseField {
    private final QueryResponse response;
    private final int index;
    private final int tag;
    private final QueryResponseFieldKind kind;
    private final long enumeration;

    QueryResponseField(
            QueryResponse response, int index, QueryResponse.FieldData field) {
        this.response = response;
        this.index = index;
        this.tag = field.tag;
        this.kind = field.kind;
        this.enumeration = field.enumeration;
    }

    /** Returns the field's raw 24-bit TTLV tag. */
    public int tag() { return tag; }

    /** Returns the category selected by the Rust Query response model. */
    public QueryResponseFieldKind kind() { return kind; }

    /** Returns the raw Enumeration value when this field has Enumeration type. */
    public OptionalLong enumeration() {
        return kind == QueryResponseFieldKind.ENUMERATION
                ? OptionalLong.of(enumeration) : OptionalLong.empty();
    }

    /** Returns this non-enumeration field as a caller-owned view of the original TTLV node. */
    public Optional<TtlvItemView> item() {
        return kind == QueryResponseFieldKind.ENUMERATION
                ? Optional.empty() : Optional.of(response.itemAt(index));
    }
}
