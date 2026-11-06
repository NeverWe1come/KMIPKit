package org.kmipkit.typed;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.List;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.OptionalLong;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.credentials.Authentication;
import org.kmipkit.credentials.Credential;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;

final class ParametersAuthenticationParityTest {
    private static final int[] TABLE_59_TAGS = {
        0x00420011, 0x0042005f, 0x00420038, 0x00420083, 0x004200ae,
        0x00420028, 0x004200c5, 0x004200cd, 0x004200ce, 0x004200cf,
        0x004200d2, 0x004200d0, 0x004200d1, 0x00420100, 0x00420101,
        0x00420102, 0x00420103, 0x00420104
    };

    @Test
    void buildsAllTable59FieldsInNormativeOrderAndPreservesFutureEnums() {
        try (CryptographicParameters parameters = CryptographicParameters.builder()
                     .trailerField(1).pSource(new byte[] {0x51, 0, (byte) 0xff})
                     .maskGeneratorHashingAlgorithm(0x80000003L).maskGenerator(0x80000002L)
                     .saltLength(16).initialCounterValue(1).counterLength(32)
                     .invocationFieldLength(4).fixedFieldLength(2).tagLength(16).ivLength(8)
                     .randomIv(true).cryptographicAlgorithm(0xe1230001L)
                     .digitalSignatureAlgorithm(0x80005678L).keyRoleType(0x80001234L)
                     .hashingAlgorithm(0xf1234567L).paddingMethod(0x80000001L)
                     .blockCipherMode(0xe1234567L).build();
                TtlvStructure structure = parameters.intoTtlv();
                TtlvStructureView view = structure.view()) {
            assertEquals(18L, view.itemCount());
            for (int index = 0; index < TABLE_59_TAGS.length; index++) {
                try (TtlvItemView item = view.itemAt(index); Tag tag = item.tag()) {
                    assertEquals(TABLE_59_TAGS[index], tag.raw());
                }
            }
        }
    }

    @Test
    void readsEveryTypedTable59FieldWithoutConsumingTheModel() {
        try (CryptographicParameters parameters = CryptographicParameters.builder()
                     .blockCipherMode(0xe1234567L).paddingMethod(0x80000001L)
                     .hashingAlgorithm(0xf1234567L).keyRoleType(0x80001234L)
                     .digitalSignatureAlgorithm(0x80005678L).cryptographicAlgorithm(0xe1230001L)
                     .randomIv(false).ivLength(8).tagLength(16).fixedFieldLength(2)
                     .invocationFieldLength(4).counterLength(32).initialCounterValue(1)
                     .saltLength(16).maskGenerator(0x80000002L)
                     .maskGeneratorHashingAlgorithm(0x80000003L)
                     .pSource(new byte[] {0x51, 0, (byte) 0xff}).trailerField(1).build()) {
            assertEquals(OptionalLong.of(0xe1234567L), parameters.blockCipherMode());
            assertEquals(OptionalLong.of(0x80000001L), parameters.paddingMethod());
            assertEquals(OptionalLong.of(0xf1234567L), parameters.hashingAlgorithm());
            assertEquals(OptionalLong.of(0x80001234L), parameters.keyRoleType());
            assertEquals(OptionalLong.of(0x80005678L), parameters.digitalSignatureAlgorithm());
            assertEquals(OptionalLong.of(0xe1230001L), parameters.cryptographicAlgorithm());
            assertEquals(Optional.of(false), parameters.randomIv());
            assertEquals(OptionalInt.of(8), parameters.ivLength());
            assertEquals(OptionalInt.of(16), parameters.tagLength());
            assertEquals(OptionalInt.of(2), parameters.fixedFieldLength());
            assertEquals(OptionalInt.of(4), parameters.invocationFieldLength());
            assertEquals(OptionalInt.of(32), parameters.counterLength());
            assertEquals(OptionalInt.of(1), parameters.initialCounterValue());
            assertEquals(OptionalInt.of(16), parameters.saltLength());
            assertEquals(OptionalLong.of(0x80000002L), parameters.maskGenerator());
            assertEquals(OptionalLong.of(0x80000003L), parameters.maskGeneratorHashingAlgorithm());
            assertArrayEquals(new byte[] {0x51, 0, (byte) 0xff}, parameters.pSource().orElseThrow());
            assertEquals(OptionalInt.of(1), parameters.trailerField());
        }
    }

    @Test
    void distinguishesAbsentAndPresentEmptyPSource() {
        try (CryptographicParameters absent = CryptographicParameters.builder()
                     .randomIv(false).build();
                CryptographicParameters presentEmpty = CryptographicParameters.builder()
                        .randomIv(false).pSource(new byte[0]).build()) {
            assertEquals(Optional.empty(), absent.pSource());
            assertEquals(0, presentEmpty.pSource().orElseThrow().length);
        }
    }

    @Test
    void authenticationIsAttachedToTheRequestBatchAndConsumedOnlyOnSuccess() {
        Credential credential = Credential.fromTtlv(credentialStructure());
        Authentication authentication = Authentication.create(List.of(credential));
        try (ClientBatch batch = ClientBatch.create()) {
            batch.setAuthentication(authentication)
                    .add(ClientBatchItem.ping())
                    .add(ClientBatchItem.ping());
            assertEquals(2L, batch.itemCount());
            assertThrows(InvalidInputException.class, authentication::credentials);
        }
    }

    @Test
    void failedAuthenticationAttachmentKeepsTheCallerOwnerLive() {
        Credential credential = Credential.fromTtlv(credentialStructure());
        Authentication authentication = Authentication.create(List.of(credential));
        ClientBatch closedBatch = ClientBatch.create();
        closedBatch.close();
        try {
            assertThrows(InvalidInputException.class, () -> closedBatch.setAuthentication(authentication));
            assertEquals(1, authentication.credentials().size());
        } finally {
            authentication.close();
        }
    }

    private static TtlvStructure credentialStructure() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure credential = TtlvStructure.create();
        try (RawTag raw = RawTag.fromRaw(0x00420024);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Enumeration(0x80000001L)) {
            credential = TtlvStructure.withItem(credential, TtlvItem.create(tag, value, limits), limits);
        }
        TtlvStructure payload = TtlvStructure.create();
        try (RawTag raw = RawTag.fromRaw(0x0042002a);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Integer(1)) {
            payload = TtlvStructure.withItem(payload, TtlvItem.create(tag, value, limits), limits);
        }
        try (RawTag raw = RawTag.fromRaw(0x00420025);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.structure(payload, limits)) {
            credential = TtlvStructure.withItem(
                    credential, TtlvItem.create(tag, value, limits), limits);
        }
        return credential;
    }
}
