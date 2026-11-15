package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.KmipException;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class CredentialAdapterValidationCoverageTest {
    @Test
    void factoriesRejectNullAndOutOfRangeInputsBeforeNativeConversion() {
        assertThrows(InvalidInputException.class, () -> Authentication.create(null));
        assertThrows(InvalidInputException.class, () -> Authentication.fromTtlv(null));
        assertThrows(ResourceLimitException.class,
                () -> Authentication.create(Collections.nCopies(100_001, null)));
        assertThrows(InvalidInputException.class,
                () -> Authentication.create(Arrays.asList((Credential) null)));

        assertThrows(InvalidInputException.class, () -> Credential.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> CredentialValue.fromTtlv(-1, null));
        assertThrows(InvalidInputException.class,
                () -> CredentialValue.fromTtlv(0x1_0000_0000L, null));
        assertThrows(InvalidInputException.class,
                () -> CredentialValue.fromTtlv(1, null));
        assertThrows(InvalidInputException.class, () -> AttestationCredential.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> DeviceCredential.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> HashedPasswordCredential.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> Nonce.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> UsernameAndPasswordCredential.fromTtlv(null));
        assertThrows(InvalidInputException.class, () -> SecretBytes.fromBytes(null));
        assertThrows(InvalidInputException.class, () -> SecretText.fromString(null));
        assertThrows(InvalidInputException.class,
                () -> SecretText.fromString(new String(new char[] {'\ud800'})));
        assertThrows(InvalidInputException.class,
                () -> HashedPasswordTimestampSequence.create(null));
    }

    @Test
    void credentialInteropPreservesItsStatusCategories() {
        assertThrows(ResourceLimitException.class,
                () -> CredentialInterop.throwIfFailed(6, "credential"));
        assertThrows(InvalidInputException.class,
                () -> CredentialInterop.throwIfFailed(CredentialInterop.SHARED_OWNER_STATUS, "credential"));
        assertThrows(KmipException.class,
                () -> CredentialInterop.throwIfFailed(99, "credential"));
    }

    @Test
    void credentialOwnersExposeRedactedViewsAndRejectNullCallbacks() {
        Credential credential = Credential.fromTtlv(credentialStructure(1));
        assertThrows(InvalidInputException.class, () -> credential.withTtlv(null));

        try (Authentication authentication = Authentication.create(List.of(credential))) {
            assertEquals("Authentication[redacted]", authentication.toString());
            assertThrows(InvalidInputException.class, () -> authentication.withTtlv(null));

            List<CredentialView> views = authentication.credentials();
            assertEquals(1, views.size());
            try (CredentialView view = views.get(0)) {
                assertEquals("CredentialView[redacted]", view.toString());
                assertThrows(InvalidInputException.class, () -> view.withTtlv(null));
            }
        }
    }

    private static TtlvStructure credentialStructure(long rawType) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        try (RawTag raw = RawTag.fromRaw(0x0042_0024);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Enumeration(rawType)) {
            structure = TtlvStructure.withItem(structure, TtlvItem.create(tag, value, limits), limits);
        }

        TtlvStructure payload = TtlvStructure.create();
        try (RawTag raw = RawTag.fromRaw(0x0042_002a);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Integer(1)) {
            payload = TtlvStructure.withItem(payload, TtlvItem.create(tag, value, limits), limits);
        }
        try (RawTag raw = RawTag.fromRaw(0x0042_0025);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.structure(payload, limits)) {
            structure = TtlvStructure.withItem(structure, TtlvItem.create(tag, value, limits), limits);
        }
        return structure;
    }

}
