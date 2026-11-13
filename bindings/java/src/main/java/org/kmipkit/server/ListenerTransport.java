package org.kmipkit.server;

/** Transport accepted by a server listener. */
public enum ListenerTransport {
    RAW_TLS(1),
    HTTPS(2);

    final int nativeValue;

    ListenerTransport(int nativeValue) { this.nativeValue = nativeValue; }
}
