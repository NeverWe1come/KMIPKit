package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;

/** KMIP 2.1 wire encoding used by a client exchange. */
public enum WireEncoding {
    /** Binary Tag-Type-Length-Value encoding and the default. */
    TTLV(0),
    /** JSON encoding from OASIS KMIP Profiles v2.1 §5.5. */
    JSON(1),
    /** XML encoding from OASIS KMIP Profiles v2.1 §5.4. */
    XML(2);

    private final int nativeValue;

    WireEncoding(int nativeValue) {
        this.nativeValue = nativeValue;
    }

    int nativeValue() {
        return nativeValue;
    }

    static WireEncoding fromNative(int value) {
        return switch (value) {
            case 0 -> TTLV;
            case 1 -> JSON;
            case 2 -> XML;
            default -> throw new InvalidInputException("native wire encoding is invalid");
        };
    }
}
