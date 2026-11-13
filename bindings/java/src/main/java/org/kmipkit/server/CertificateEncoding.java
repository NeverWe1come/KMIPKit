package org.kmipkit.server;

/** Encoding used for listener certificates, private keys, and trust material. */
public enum CertificateEncoding {
    PEM(1),
    DER(2);

    final int nativeValue;

    CertificateEncoding(int nativeValue) { this.nativeValue = nativeValue; }
}
