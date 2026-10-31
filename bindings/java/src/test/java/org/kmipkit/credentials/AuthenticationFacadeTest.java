package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.concurrent.atomic.AtomicReference;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.junit.jupiter.api.Test;

final class AuthenticationFacadeTest {
    private static final int AUTHENTICATION_CREDENTIAL_TAG = 0x0042_0023;
    private static final int CREDENTIAL_TYPE_TAG = 0x0042_0024;
    private static final int CREDENTIAL_VALUE_TAG = 0x0042_0025;
    private static final int CREDENTIAL_PAYLOAD_TAG = 0x0042_002A;

    @Test
    void authenticationPreservesCredentialOrderAndOwnerBackedViews() {
        long firstType = 0xF123_4567L;
        long secondType = 0x8000_4321L;
        Credential first = Credential.fromTtlv(credentialStructure(firstType));
        Credential second = Credential.fromTtlv(credentialStructure(secondType));

        try (Authentication authentication = Authentication.create(List.of(first, second))) {
            assertThrows(InvalidInputException.class, first::credentialTypeRaw);
            assertThrows(InvalidInputException.class, second::credentialTypeRaw);
            assertEquals(List.of(AUTHENTICATION_CREDENTIAL_TAG, AUTHENTICATION_CREDENTIAL_TAG),
                    authentication.withTtlv(AuthenticationFacadeTest::readTags));

            List<CredentialView> credentials = authentication.credentials();
            assertEquals(List.of(firstType, secondType), credentials.stream()
                    .map(CredentialView::credentialTypeRaw).toList());
            assertEquals(List.of(CREDENTIAL_TYPE_TAG, CREDENTIAL_VALUE_TAG),
                    credentials.get(0).withTtlv(AuthenticationFacadeTest::readTags));
            assertThrows(InvalidInputException.class, authentication::intoTtlv,
                    "a live CredentialView must keep its Authentication owner alive");
            credentials.forEach(CredentialView::close);

            AtomicReference<TtlvStructureView> borrowed = new AtomicReference<>();
            authentication.withTtlv(view -> {
                borrowed.set(view);
                assertThrows(InvalidInputException.class, authentication::intoTtlv);
                assertEquals(2L, view.itemCount());
                return null;
            });
            assertThrows(InvalidInputException.class, borrowed.get()::itemCount);

            TtlvStructure restored = authentication.intoTtlv();
            try (TtlvStructureView view = restored.view()) {
                assertEquals(List.of(AUTHENTICATION_CREDENTIAL_TAG,
                                AUTHENTICATION_CREDENTIAL_TAG), readTags(view));
            }
            try (Authentication parsed = Authentication.fromTtlv(restored)) {
                List<CredentialView> parsedCredentials = parsed.credentials();
                assertEquals(2, parsedCredentials.size());
                parsedCredentials.forEach(CredentialView::close);
            }
            assertThrows(InvalidInputException.class, restored::view);
        }
    }

    @Test
    void authenticationRejectsEmptyCredentials() {
        assertThrows(InvalidInputException.class, () -> Authentication.create(List.of()));
    }

    @Test
    void createPreservesCredentialsRejectedForALiveScopedView() {
        Credential credential = Credential.fromTtlv(credentialStructure(1));
        credential.withTtlv(view -> {
            assertThrows(InvalidInputException.class,
                    () -> Authentication.create(List.of(credential)));
            return null;
        });
        assertEquals(1L, credential.credentialTypeRaw());
        try (Authentication authentication = Authentication.create(List.of(credential))) {
            List<CredentialView> credentials = authentication.credentials();
            assertEquals(1, credentials.size());
            credentials.forEach(CredentialView::close);
        }
    }

    private static List<Integer> readTags(TtlvStructureView view) {
        java.util.ArrayList<Integer> tags = new java.util.ArrayList<>();
        for (long index = 0; index < view.itemCount(); index++) {
            try (TtlvItemView item = view.itemAt(index); Tag tag = item.tag()) {
                tags.add(tag.raw());
            }
        }
        return List.copyOf(tags);
    }

    private static TtlvStructure credentialStructure(long rawType) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure structure = TtlvStructure.create();
        try (RawTag raw = RawTag.fromRaw(CREDENTIAL_TYPE_TAG);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Enumeration(rawType)) {
            TtlvItem item = TtlvItem.create(tag, value, limits);
            structure = TtlvStructure.withItem(structure, item, limits);
        }
        TtlvStructure payload = TtlvStructure.create();
        try (RawTag raw = RawTag.fromRaw(CREDENTIAL_PAYLOAD_TAG);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Integer(0x13579)) {
            TtlvItem item = TtlvItem.create(tag, value, limits);
            payload = TtlvStructure.withItem(payload, item, limits);
        }
        try (RawTag raw = RawTag.fromRaw(CREDENTIAL_VALUE_TAG);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.structure(payload, limits)) {
            TtlvItem item = TtlvItem.create(tag, value, limits);
            structure = TtlvStructure.withItem(structure, item, limits);
        }
        return structure;
    }
}
