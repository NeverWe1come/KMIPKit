package org.kmipkit.fluent;

import org.kmipkit.credentials.SecretBytes;
import org.kmipkit.credentials.SecretText;
import org.kmipkit.ttlv.TtlvItem;

/** One of the five KMIP 3.0 Create Credential object alternatives. */
public final class CreateCredentialObject implements AutoCloseable {
    private final Kmip30OperationHandle.CreateCredentialObject value;

    private CreateCredentialObject(
            Kmip30OperationHandle.CreateCredentialObject value) {
        this.value = value;
    }

    public static CreateCredentialObject certificate(TtlvItem certificate) {
        return new CreateCredentialObject(
                Kmip30OperationHandle.CreateCredentialObject
                        .certificate(certificate));
    }

    public static CreateCredentialObject password(
            SecretText password, SecretBytes passwordSalt, Long saltAlgorithm,
            SecretBytes saltedPassword, Integer iterationCount) {
        return new CreateCredentialObject(
                Kmip30OperationHandle.CreateCredentialObject.password(
                        password, passwordSalt, saltAlgorithm, saltedPassword, iterationCount));
    }

    public static CreateCredentialObject device(
            String serial, String identifier, String network, String machine, String media) {
        return new CreateCredentialObject(
                Kmip30OperationHandle.CreateCredentialObject
                        .device(serial, identifier, network, machine, media));
    }

    public static CreateCredentialObject hashedPassword(
            SecretBytes hashedUsernamePassword, SecretBytes hashedPasswordUsername,
            Long hashingAlgorithm) {
        return new CreateCredentialObject(
                Kmip30OperationHandle.CreateCredentialObject
                        .hashedPassword(hashedUsernamePassword, hashedPasswordUsername,
                                hashingAlgorithm));
    }

    public static CreateCredentialObject oneTimePassword(
            long algorithm, Long digest, String serial, SecretBytes seed,
            Long interval, Integer digits) {
        return new CreateCredentialObject(
                Kmip30OperationHandle.CreateCredentialObject
                        .oneTimePassword(algorithm, digest, serial, seed, interval, digits));
    }

    Kmip30OperationHandle.CreateCredentialObject nativeObject() {
        return value;
    }

    @Override public void close() { value.close(); }
    @Override public String toString() { return "CreateCredentialObject[redacted]"; }
}
