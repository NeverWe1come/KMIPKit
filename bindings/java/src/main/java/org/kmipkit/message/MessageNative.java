package org.kmipkit.message;

import org.kmipkit.KmipException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.internal.NativeHandle;

final class MessageNative {
    private static final int ERROR_PROTOCOL = 8;

    private MessageNative() { }

    static long parse(TtlvStructure structure, boolean request) {
        if (structure == null) {
            throw new org.kmipkit.InvalidInputException("KMIP message input is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long rawStructure = NativeHandle.transferAll(structure.nativeHandleOwner())[0];
        long[] result = request
                ? NativeExtensionRegistry.requestMessageFromTtlv(rawStructure)
                : NativeExtensionRegistry.responseMessageFromTtlv(rawStructure);
        if (result == null || result.length != 5) {
            throw new KmipException("KMIP message parsing failed; diagnostic details are redacted");
        }
        int status = Math.toIntExact(result[4]);
        if (status == 0 && result[0] != 0) {
            return result[0];
        }
        if (status == ERROR_PROTOCOL) {
            throw new MessageValidationException(result[1], result[2], result[3]);
        }
        throw new KmipException("KMIP message parsing failed; diagnostic details are redacted");
    }

    static long rawHandle(long[] value) {
        if (value == null || value.length != 2) {
            throw new KmipException("KMIP message access failed; diagnostic details are redacted");
        }
        if (value[0] == 0) {
            throw new org.kmipkit.InvalidInputException("KMIP message field is unavailable");
        }
        return value[1];
    }

    static java.util.OptionalLong optionalRaw(long[] value) {
        if (value == null || value.length != 2) {
            throw new KmipException("KMIP message access failed; diagnostic details are redacted");
        }
        return value[0] == 0 ? java.util.OptionalLong.empty()
                : java.util.OptionalLong.of(value[1]);
    }
}
