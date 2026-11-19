package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.Arrays;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.TtlvStructure;

final class CredentialInputValidationCoverageTest {
    @Test
    void secretInputsRejectNullAndMalformedUnicodeWithoutAllocatingNativeSecrets() {
        assertThrows(InvalidInputException.class, () -> SecretBytes.fromBytes(null));
        assertThrows(InvalidInputException.class, () -> SecretText.fromString(null));
        assertThrows(InvalidInputException.class, () -> SecretText.fromString("bad-\ud800-text"));

        try (SecretBytes bytes = SecretBytes.fromBytes(new byte[] {1});
                SecretText text = SecretText.fromString("secret")) {
            assertThrows(InvalidInputException.class, () -> bytes.withBytes(null));
            assertThrows(InvalidInputException.class, () -> text.withStr(null));
        }
    }

    @Test
    void credentialFactoriesRejectMissingStructuresAndOutOfRangeTypesBeforeNativeCalls() {
        assertThrows(InvalidInputException.class, () -> Nonce.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> AttestationCredential.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> DeviceCredential.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> UsernameAndPasswordCredential.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> HashedPasswordCredential.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> Authentication.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> CredentialValue.fromTtlv(-1, null));
        assertThrows(InvalidInputException.class, () -> CredentialValue.fromTtlv(0x1_0000_0000L, null));
        assertThrows(InvalidInputException.class, () -> CredentialValue.fromTtlv(1, null));
        assertThrows(InvalidInputException.class, () -> Authentication.create(null));
        assertThrows(InvalidInputException.class,
                () -> Authentication.create(Arrays.asList((Credential) null)));
        assertThrows(ResourceLimitException.class,
                () -> Authentication.create(java.util.Collections.nCopies(100_001, (Credential) null)));
        assertThrows(InvalidInputException.class, () -> HashedPasswordTimestampSequence.create(null));
    }
}
