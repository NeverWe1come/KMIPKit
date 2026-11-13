package org.kmipkit.server;

/** KMIP Result Status values used when a server-initiated request fails. */
public enum ServerResultStatus {
    OPERATION_FAILED(1), OPERATION_PENDING(2), OPERATION_UNDONE(3);
    final int nativeValue;
    ServerResultStatus(int nativeValue) { this.nativeValue = nativeValue; }
}
