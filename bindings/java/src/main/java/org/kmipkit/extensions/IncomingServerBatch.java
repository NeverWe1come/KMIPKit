package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.protocol.VersionGate.ProtocolVersion;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Server-to-Client batch request received from a KMIP server. */
public final class IncomingServerBatch implements AutoCloseable {
    private final NativeHandle handle;

    private IncomingServerBatch(long nativeHandle) {
        this.handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.INCOMING_SERVER_BATCH);
    }

    public static IncomingServerBatch fromNativeHandle(long nativeHandle) {
        return new IncomingServerBatch(nativeHandle);
    }

    public static IncomingServerBatch decode(byte[] data, WireEncoding encoding, CodecLimits limits) {
        Objects.requireNonNull(data, "data");
        Objects.requireNonNull(encoding, "encoding");
        Objects.requireNonNull(limits, "limits");
        NativeExtensionRegistry.ensureLoaded();
        long[] nativeLimits = NativeExtensionRegistry.codecLimitValues(limits);
        long nativeHandle = NativeExtensionRegistry.incomingServerBatchDecode(
                data, encoding.nativeValue(), nativeLimits);
        return new IncomingServerBatch(nativeHandle);
    }

    public static IncomingServerBatch decodeTtlv(byte[] data, CodecLimits limits) {
        return decode(data, WireEncoding.TTLV, limits);
    }

    public ProtocolVersion protocolVersion() {
        int[] v = handle.withValue(NativeExtensionRegistry::incomingServerBatchProtocolVersion);
        return new ProtocolVersion(v[0], v[1]);
    }

    public int itemCount() {
        return handle.withValue(NativeExtensionRegistry::incomingServerBatchItemCount);
    }

    public long itemOperation(int index) {
        return handle.withValue(raw -> NativeExtensionRegistry.incomingServerBatchItemOperation(raw, index));
    }

    public byte[] itemUniqueBatchItemId(int index) {
        return handle.withValue(raw -> NativeExtensionRegistry.incomingServerBatchItemUniqueBatchItemId(raw, index));
    }

    public List<ProtocolVersion> itemDiscoverVersions(int index) {
        int[] versions = handle.withValue(raw -> NativeExtensionRegistry.incomingServerBatchItemDiscoverVersions(raw, index));
        List<ProtocolVersion> list = new ArrayList<>();
        if (versions != null) {
            for (int i = 0; i < versions.length; i += 2) {
                list.add(new ProtocolVersion(versions[i], versions[i + 1]));
            }
        }
        return list;
    }

    public byte[] itemNotifyUid(int index) {
        return handle.withValue(raw -> NativeExtensionRegistry.incomingServerBatchItemNotify(raw, index));
    }

    public record PutItemDetails(
            long putFunction,
            byte[] uniqueIdentifier,
            byte[] replacedUniqueIdentifier,
            TtlvItem objectItem,
            TtlvStructure attributes) {}

    public PutItemDetails itemPutV2(int index) {
        return handle.withValue(raw -> {
            Object[] res = NativeExtensionRegistry.incomingServerBatchItemPutV2(raw, index);
            long putFn = ((Number) res[0]).longValue();
            byte[] uid = (byte[]) res[1];
            byte[] repUid = (byte[]) res[2];
            long itemHandle = ((Number) res[3]).longValue();
            long structHandle = ((Number) res[4]).longValue();
            TtlvItem item = itemHandle != 0 ? TtlvItem.fromNativeHandle(itemHandle) : null;
            TtlvStructure struct = structHandle != 0 ? TtlvStructure.fromNative(structHandle, null) : null;
            return new PutItemDetails(putFn, uid, repUid, item, struct);
        });
    }

    public long itemQueryFlags(int index) {
        return handle.withValue(raw -> NativeExtensionRegistry.incomingServerBatchItemQuery(raw, index));
    }

    public long itemSetEndpointRole(int index) {
        return handle.withValue(raw -> NativeExtensionRegistry.incomingServerBatchItemSetEndpointRole(raw, index));
    }

    public NativeHandle nativeHandleOwner() { return handle; }

    @Override
    public void close() { handle.close(); }
}
