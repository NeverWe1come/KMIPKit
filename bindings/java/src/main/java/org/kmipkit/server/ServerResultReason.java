package org.kmipkit.server;

/** Common KMIP Result Reason values for explicit listener failures. */
public enum ServerResultReason {
    ITEM_NOT_FOUND(1), OPERATION_NOT_SUPPORTED(5), ILLEGAL_OPERATION(11),
    PERMISSION_DENIED(12), CRYPTOGRAPHIC_FAILURE(10);
    final int nativeValue;
    ServerResultReason(int nativeValue) { this.nativeValue = nativeValue; }
}
