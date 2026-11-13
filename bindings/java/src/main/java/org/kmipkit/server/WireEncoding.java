package org.kmipkit.server;

/** KMIP wire encoding accepted by the listener. */
public enum WireEncoding {
    TTLV(1),
    JSON(2),
    XML(3);

    final int nativeValue;

    WireEncoding(int nativeValue) { this.nativeValue = nativeValue; }
}
