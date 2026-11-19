package org.kmipkit.ttlv;

import java.util.List;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Shared owned TTLV lifecycle for typed managed object and key wrappers. */
abstract class ManagedTtlvValue implements AutoCloseable {
    private final NativeHandle handle;
    private final int ownerKind;
    private final int contextRaw;

    ManagedTtlvValue(long nativeHandle, int ownerKind, int contextRaw) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_ITEM);
        this.ownerKind = ownerKind;
        this.contextRaw = contextRaw;
    }

    static long withFields(List<TtlvItem> fields, Function<long[], Long> operation) {
        if (fields == null || fields.size() > 100_000 || operation == null) {
            throw new org.kmipkit.InvalidInputException("managed field inputs are invalid");
        }
        NativeHandle[] owners = new NativeHandle[fields.size()];
        for (int index = 0; index < fields.size(); index++) {
            TtlvItem item = fields.get(index);
            if (item == null) {
                throw new org.kmipkit.InvalidInputException("managed field inputs are invalid");
            }
            owners[index] = item.nativeHandleOwner();
        }
        return NativeHandle.withValues(owners, values -> operation.apply(values));
    }

    private long readScalar(int wantedTag, int readKind) {
        return handle.withValue(value -> NativeExtensionRegistry.managedValueFieldRead(
                ownerKind, contextRaw, value, wantedTag, readKind, null, null));
    }

    /** Returns an independently owned raw child Item when its raw tag is present. */
    public final TtlvItem fieldItem(int wantedTag) {
        long nested = handle.withValue(value -> NativeExtensionRegistry.managedValueFieldItemRead(
                ownerKind, contextRaw, value, wantedTag));
        return nested == 0 ? null : TtlvItem.fromNative(nested);
    }

    /** Reads one direct child by raw tag using the selected typed value kind. */
    public final Object readField(int rawTag, int readKind) {
        return switch (readKind) {
            case 1 -> hasField(rawTag);
            case 2 -> TtlvItemType.fromCode((int) readScalar(rawTag, readKind));
            case 3 -> readScalar(rawTag, readKind);
            case 4 -> (int) readScalar(rawTag, readKind);
            case 5 -> readBytesField(rawTag);
            default -> throw new InvalidInputException("managed field read kind is invalid");
        };
    }

    public final boolean hasField(int wantedTag) {
        return readScalar(wantedTag, 1) != 0;
    }

    public final TtlvItemType fieldType(int wantedTag) {
        return (TtlvItemType) readField(wantedTag, 2);
    }

    final long readEnumerationField(int wantedTag) {
        return (long) readField(wantedTag, 3);
    }

    final int readIntegerField(int wantedTag) {
        return (int) readField(wantedTag, 4);
    }

    final byte[] readBytesField(int wantedTag) {
        return handle.withValue(value -> {
            long[] requiredLength = new long[1];
            long length = NativeExtensionRegistry.managedValueFieldRead(
                    ownerKind, contextRaw, value, wantedTag, 5, null, requiredLength);
            if (length > Integer.MAX_VALUE) {
                throw new org.kmipkit.ResourceLimitException("managed field is too large");
            }
            byte[] result = new byte[(int) length];
            long[] copiedLength = new long[1];
            NativeExtensionRegistry.managedValueFieldRead(
                    ownerKind, contextRaw, value, wantedTag, 5, result, copiedLength);
            if (copiedLength[0] != length) {
                throw new InvalidInputException("managed field length changed during read");
            }
            return result;
        });
    }

    /** Lends the complete TTLV node through a view retaining this owner. */
    public final TtlvItemView view() {
        return handle.withValue(value -> TtlvItemView.fromNative(
                NativeTtlvCodec.itemViewFromItem(value), this));
    }

    /**
     * Consumes this typed wrapper and transfers its complete TTLV node to a generic item.
     *
     * <p>After a successful transfer this wrapper is closed, and the returned item owns the same
     * TTLV node, including its original tag and nested fields.</p>
     */
    public final TtlvItem intoItem() {
        return TtlvItem.fromNative(handle.transfer());
    }

    @Override
    public final void close() {
        handle.close();
    }

    @Override
    public final String toString() {
        return getClass().getSimpleName() + "([REDACTED])";
    }
}
