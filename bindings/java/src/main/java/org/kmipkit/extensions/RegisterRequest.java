package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Owned KMIP 2.1 Register request with caller-selected Opaque Object context. */
public final class RegisterRequest implements AutoCloseable {
    private final NativeHandle handle;

    private RegisterRequest(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.REGISTER_REQUEST);
    }

    /**
     * Creates a Register request from the raw Object Type and ordered TTLV inputs.
     *
     * <p>The attributes and object handles are consumed by this call.</p>
     */
    public static RegisterRequest create(int objectType, TtlvStructure attributes, TtlvItem object) {
        NativeExtensionRegistry.ensureLoaded();
        if (attributes == null || object == null) {
            throw new InvalidInputException("Register request inputs are invalid");
        }
        long[] consumed = NativeHandle.transferAll(
                attributes.nativeHandleOwner(), object.nativeHandleOwner());
        return new RegisterRequest(NativeExtensionRegistry.registerRequestCreate(
                objectType, consumed[0], consumed[1]));
    }

    /**
     * Selects local attribute handling for this Opaque Object request.
     *
     * <p>This operation consumes this request and returns a new owned request.</p>
     */
    public RegisterRequest withOpaqueObjectContext(OpaqueObjectContext context) {
        NativeExtensionRegistry.ensureLoaded();
        if (context == null) {
            throw new InvalidInputException("Opaque Object context is invalid");
        }
        long request = handle.transfer();
        return new RegisterRequest(NativeExtensionRegistry.registerRequestWithOpaqueObjectContext(
                request, context.nativeValue()));
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    @Override
    public void close() {
        handle.close();
    }
}
