package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;
import java.util.OptionalLong;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class CreationResultValueTest {
    @Test
    void failureResultMessageMatchesTheSharedFixture() throws Exception {
        Path fixture = Path.of("..").resolve("..").resolve("tests/fixtures/api/shared-capability-parity.json");
        String json = Files.readString(fixture.normalize());
        assertTrue(json.contains("\"id\": \"creation.result-message-on-failure.v1\""));
        assertTrue(json.contains("\"resultMessage\": \"création refusée ✓\""));
        byte[] message = "création refusée ✓".getBytes(StandardCharsets.UTF_8);
        ClientBatchItemMetadataV2 metadata = new ClientBatchItemMetadataV2(
                new long[] {1, 1, 1, 1, 5, 1}, message);
        assertEquals(1, metadata.operation());
        assertEquals(1, metadata.resultStatus());
        assertEquals(OptionalLong.of(5), metadata.resultReason());
        assertEquals(Optional.of("création refusée ✓"), metadata.resultMessage());
        assertFalse(metadata.toString().contains("création refusée"));
    }

    @Test
    void metadataRetainsUnknownValuesAndDistinguishesEmptyFromAbsentMessage() {
        byte[] message = "metadata marker: création ✓".getBytes(StandardCharsets.UTF_8);
        ClientBatchItemMetadataV2 metadata = new ClientBatchItemMetadataV2(
                new long[] {0xffff_ffffL, 99, 0xffff_fffeL, 1, 0xffff_fffdL, 1}, message);

        assertEquals(0xffff_ffffL, metadata.operation());
        assertEquals(99, metadata.outcomeKind().raw());
        assertEquals(0xffff_fffeL, metadata.resultStatus());
        assertEquals(OptionalLong.of(0xffff_fffdL), metadata.resultReason());
        assertEquals(Optional.of("metadata marker: création ✓"), metadata.resultMessage());
        assertFalse(metadata.toString().contains("metadata marker"));

        ClientBatchItemMetadataV2 empty = new ClientBatchItemMetadataV2(
                new long[] {40, 1, 1, 1, 5, 1}, new byte[0]);
        ClientBatchItemMetadataV2 absent = new ClientBatchItemMetadataV2(
                new long[] {40, 1, 1, 1, 5, 0}, null);
        assertEquals(Optional.of(""), empty.resultMessage());
        assertEquals(Optional.empty(), absent.resultMessage());
    }

    @Test
    void metadataRejectsMalformedShapePresenceAndUtf8() {
        assertThrows(InvalidInputException.class, () -> new ClientBatchItemMetadataV2(
                new long[] {-1, 1, 0, 0, 0, 0}, null));
        assertThrows(InvalidInputException.class, () -> new ClientBatchItemMetadataV2(
                new long[] {40, 1, 1, 1, 5, 1}, null));
        assertThrows(InvalidInputException.class, () -> new ClientBatchItemMetadataV2(
                new long[] {40, 1, 1, 1, 5, 1}, new byte[] {(byte) 0xff}));
    }

    @Test
    void uniqueIdentifierPreservesEachWireTypeAndRedactsItsText() {
        UniqueIdentifier text = UniqueIdentifier.fromNative(
                1, "public-key-α".getBytes(StandardCharsets.UTF_8), 0, 0);
        UniqueIdentifier enumeration = UniqueIdentifier.fromNative(2, null, 0xffff_ffffL, 0);
        UniqueIdentifier integer = UniqueIdentifier.fromNative(3, null, 0, -7);

        assertEquals(UniqueIdentifier.Kind.TEXT_STRING, text.kind());
        assertEquals("public-key-α", text.text());
        assertFalse(text.toString().contains("public-key"));
        assertEquals(0xffff_ffffL, enumeration.number());
        assertEquals(-7, integer.number());
    }

    @Test
    void uniqueIdentifierFactoriesAndAccessorsRejectInvalidValuesAndKinds() {
        assertThrows(InvalidInputException.class, () -> UniqueIdentifier.textString("\uD800"));
        assertThrows(InvalidInputException.class, () -> UniqueIdentifier.enumeration(-1));
        assertThrows(InvalidInputException.class, () -> UniqueIdentifier.enumeration(0x1_0000_0000L));

        UniqueIdentifier text = UniqueIdentifier.textString("key-id");
        UniqueIdentifier enumeration = UniqueIdentifier.enumeration(7);
        UniqueIdentifier integer = UniqueIdentifier.integer(-7);
        assertThrows(InvalidInputException.class, text::number);
        assertThrows(InvalidInputException.class, enumeration::text);
        assertThrows(InvalidInputException.class, integer::text);
    }

    @Test
    void uniqueIdentifierRejectsMalformedNativeUtf8WithoutExposingBytes() {
        InvalidInputException failure = assertThrows(InvalidInputException.class,
                () -> UniqueIdentifier.fromNative(1, new byte[] {(byte) 0xff}, 0, 0));

        assertEquals("native Unique Identifier is not valid UTF-8 text", failure.getMessage());
        assertThrows(InvalidInputException.class,
                () -> UniqueIdentifier.fromNative(4, null, 0, 0));
    }

    @Test
    void batchItemParsersRejectMalformedMetadataAndCreateTuples() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseMetadataV2(null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMetadataV2(new Object[] {new long[6]}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMetadataV2(new Object[] {"metadata", null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMetadataV2(new Object[] {new long[6], "message"}));

        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseCreateResponse(null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateResponse(new Object[] {new long[5]}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateResponse(new Object[] {"fields", null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateResponse(new Object[] {new long[4], null}));
    }

    @Test
    void createResponseParserValidatesPresenceAndIdentifierFields() {
        assertEquals(Optional.empty(), ClientBatchItemResult.parseCreateResponse(
                new Object[] {new long[] {0, 0, 0, 0, 0}, "ignored when absent"}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateResponse(
                        new Object[] {new long[] {2, 0, 0, 0, 0}, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateResponse(
                        new Object[] {new long[] {1, 0, 1, 0, 0}, "not bytes"}));

        ClientBatchItemResult.CreateResponse response = ClientBatchItemResult.parseCreateResponse(
                new Object[] {new long[] {1, 2, 3, 0, -7}, null}).orElseThrow();
        assertEquals(2, response.objectType());
        assertEquals(UniqueIdentifier.Kind.INTEGER, response.uniqueIdentifier().kind());
        assertEquals(-7, response.uniqueIdentifier().number());
    }

    @Test
    void createKeyPairAndSplitKeyParsersValidateNestedIdentifierRecords() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateKeyPairResponse(new Object[] {new long[] {1}, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateKeyPairResponse(
                        new Object[] {new long[] {2}, null, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateKeyPairResponse(
                        new Object[] {new long[] {1}, "not a record", null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateKeyPairResponse(
                        new Object[] {new long[] {1}, new Object[] {new long[] {2, 2, 0, 0}, null}, null}));

        ClientBatchItemResult.CreateKeyPairResponse pair = ClientBatchItemResult.parseCreateKeyPairResponse(
                new Object[] {new long[] {1}, identifierRecord(2, 7), identifierRecord(3, 0, -9)})
                .orElseThrow();
        assertEquals(7, pair.privateKeyUniqueIdentifier().number());
        assertEquals(-9, pair.publicKeyUniqueIdentifier().number());

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateSplitKeyUniqueIdentifiers(null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateSplitKeyUniqueIdentifiers(
                        new Object[] {new long[] {1, 1}}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateSplitKeyUniqueIdentifiers(
                        new Object[] {new long[] {1, 0}, new Object[0]}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCreateSplitKeyUniqueIdentifiers(
                        new Object[] {new long[] {1, 2}, new Object[] {identifierRecord(2, 1)}}));

        List<UniqueIdentifier> identifiers = ClientBatchItemResult.parseCreateSplitKeyUniqueIdentifiers(
                new Object[] {new long[] {1, 2},
                        new Object[] {identifierRecord(2, 1), identifierRecord(2, 2)}})
                .orElseThrow();
        assertEquals(2, identifiers.size());
        assertEquals(1, identifiers.get(0).number());
        assertEquals(2, identifiers.get(1).number());
    }

    @Test
    void supportedVersionParserRejectsOddAndOutOfRangeComponents() {
        assertEquals(Optional.empty(), ClientBatchItemResult.parseSupportedVersions(null));
        assertEquals(Optional.of(List.of()), ClientBatchItemResult.parseSupportedVersions(new long[0]));
        assertEquals(Optional.of(List.of(new ProtocolVersion(2, 1), new ProtocolVersion(1, 4))),
                ClientBatchItemResult.parseSupportedVersions(new long[] {2, 1, 1, 4}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseSupportedVersions(new long[] {2}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseSupportedVersions(new long[] {Integer.MAX_VALUE + 1L, 0}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseSupportedVersions(new long[] {0, Integer.MIN_VALUE - 1L}));
    }

    @Test
    void metadataParserRejectsInvalidNativeTupleLength() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseMetadata(null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMetadata(new long[] {0, 0, 0}));
        assertArrayEquals(new long[] {1, 2, 3, 4},
                ClientBatchItemResult.parseMetadata(new long[] {1, 2, 3, 4}));
    }

    private static Object[] identifierRecord(long kind, long number) {
        return identifierRecord(kind, number, 0);
    }

    private static Object[] identifierRecord(long kind, long enumeration, long integer) {
        return new Object[] {new long[] {1, kind, enumeration, integer}, null};
    }
}
