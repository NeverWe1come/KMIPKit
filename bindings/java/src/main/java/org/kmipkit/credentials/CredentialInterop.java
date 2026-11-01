package org.kmipkit.credentials;

import java.util.function.LongFunction;

import org.kmipkit.InvalidInputException;
import org.kmipkit.KmipException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Internal status handling shared by credential envelope conversions. */
final class CredentialInterop {
    static final int SHARED_OWNER_STATUS = 11;

    private CredentialInterop() {}

    static void throwIfFailed(int status, String modelName) {
        switch (status) {
            case 0 -> { return; }
            case 4 -> {
                int kind = CredentialValidationException.lastKindFromNative();
                if (kind != 0) throw new CredentialValidationException(kind);
                throw new InvalidInputException(modelName + " structure is invalid");
            }
            case SHARED_OWNER_STATUS -> throw new InvalidInputException(
                    modelName + " structure is still owned by a scoped view");
            case 6 -> throw new ResourceLimitException("configured resource limit was exceeded");
            default -> throw new KmipException("native " + modelName + " conversion failed");
        }
    }

    static TtlvStructure intoTtlv(NativeHandle owner, LongFunction<Long> conversion) {
        return owner.withValue(value -> {
            long structure = conversion.apply(value);
            owner.transfer();
            return TtlvStructure.fromNative(structure, owner);
        });
    }
}
