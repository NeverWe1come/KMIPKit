package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

final class PublicWrapperLifecycleCoverageTest {
    private static final int PRIVATE_TAG = 0x420001;
    private static final int AFTER_TAG = 0x420002;
    private static final CodecLimits CODEC_LIMITS = CodecLimits.defaults();

    @Test
    void closesTtlvWrappersAndReadsOwnedAndBorrowedValues() {
        CodecLimits limits = CodecLimits.create(1_024, 8, 32);
        assertEquals(1_024, limits.maxMessageBytes());
        assertEquals(8, limits.maxStructureDepth());
        assertEquals(32, limits.maxElements());
        assertThrows(InvalidInputException.class, () -> CodecLimits.create(-1, 1, 1));
        assertThrows(ResourceLimitException.class, () -> CodecLimits.create(1, 65, 1));
        assertThrows(InvalidInputException.class, () -> TtlvValue.ByteString(null, limits));
        assertThrows(InvalidInputException.class, () -> TtlvValue.ByteString(new byte[] {1}, null));
        assertThrows(ResourceLimitException.class,
                () -> TtlvValue.ByteString(new byte[1_025], limits));

        try (RawTag rawTag = RawTag.fromRaw(PRIVATE_TAG);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.Integer(17);
                TtlvValueView valueView = value.view()) {
            assertEquals(PRIVATE_TAG, rawTag.raw());
            assertEquals(PRIVATE_TAG, tag.raw());
            assertEquals(TtlvItemType.Integer, valueView.itemType());
            assertEquals(17, valueView.integer());
        }

        try (TtlvValue value = TtlvValue.ByteString(new byte[] {1, 127}, limits);
                TtlvValueView valueView = value.view()) {
            assertTrue(value.handle() != 0);
            assertEquals(2, valueView.byteLength());
            assertEquals(127, valueView.byteAt(1));
        }

        TtlvValue closedValue = TtlvValue.Integer(31);
        closedValue.close();
        assertThrows(InvalidInputException.class, closedValue::handle);
        closedValue.close();

        try (RawTag rawTag = RawTag.fromRaw(PRIVATE_TAG);
                Tag tag = rawTag.checked();
                TtlvStructure empty = TtlvStructure.create();
                TtlvValue itemValue = TtlvValue.Integer(29);
                TtlvItem item = TtlvItem.create(tag, itemValue, limits)) {
            TtlvStructure populated = TtlvStructure.withItem(empty, item, limits);
            try (populated;
                    TtlvStructureView structureView = populated.view();
                    TtlvItemView itemView = structureView.itemAt(0);
                    Tag copiedTag = itemView.tag();
                    TtlvValueView copiedValue = itemView.value()) {
                assertEquals(1, structureView.itemCount());
                assertEquals(PRIVATE_TAG, copiedTag.raw());
                assertEquals(TtlvItemType.Integer, itemView.itemType());
                assertEquals(29, copiedValue.integer());
            }
        }
    }

    @Test
    void closesExtensionRegistryGraphAndExercisesPublicMetadata() {
        ExtensionRegistryLimits limits = ExtensionRegistryLimits.defaults();
        assertEquals(256, limits.maxDefinitions());
        assertEquals(16_384, limits.maxSchemaNodes());
        assertEquals(256, limits.maxChildRulesPerStructure());
        assertEquals(4_096, limits.maxTextBytesPerField());
        assertEquals(1_048_576, limits.maxRegistryTextBytes());
        assertEquals(4_096, limits.maxDiscriminatorScalarBytes());
        assertEquals(1_048_576, limits.maxTotalDiscriminatorScalarBytes());
        assertEquals(256, limits.maxConstraintMembersPerRule());
        assertEquals(16_384, limits.maxTotalConstraintMembers());
        assertEquals(200_000, limits.maxPayloadIndexRecords());
        assertEquals(1_048_576, limits.maxLookupComparisons());
        assertEquals(64, limits.maxDepth());

        try (RawTag rawTag = RawTag.fromRaw(PRIVATE_TAG);
                RawTag rawAfterTag = RawTag.fromRaw(AFTER_TAG);
                Tag tag = rawTag.checked();
                Tag afterTag = rawAfterTag.checked();
                ExtensionIdentity identity = ExtensionIdentity.create("example.vendor", "policy", "1.0");
                Compatibility compatibility = Compatibility.create(2, 1, 2, 1, "1.0.0", "1.0.0");
                TtlvPath discriminatorPath = TtlvPath.create(tag);
                TtlvValue discriminatorValue = TtlvValue.TextString(
                        "marker".getBytes(StandardCharsets.UTF_8), CODEC_LIMITS);
                ExtensionSchema scalarSchema = ExtensionSchema.scalar(TtlvItemType.Integer);
                ExtensionSchema markerSchema = ExtensionSchema.scalar(TtlvItemType.TextString);
                ExtensionChildRule childRule = ExtensionChildRule.required(tag, markerSchema);
                ExtensionChildRule optionalRule = ExtensionChildRule.optional(afterTag, scalarSchema);
                ExtensionOrderConstraint order = ExtensionOrderConstraint.create(tag, afterTag);
                ExtensionSchema schema = ExtensionSchema.structure(
                        List.of(childRule, optionalRule), List.of(order), true);
                ExtensionInformation sourceInformation = ExtensionInformation.create("policy metadata");
                ExtensionInformation information = ExtensionInformation.with_tag(sourceInformation, PRIVATE_TAG);
                Discriminator discriminator = Discriminator.create(discriminatorPath, discriminatorValue)) {
            assertEquals("example.vendor", identity.vendorIdentifier());
            assertEquals("policy", identity.name());
            assertEquals("1.0", identity.version());
            assertThrows(InvalidInputException.class, () -> ExtensionInformation.create(""));

            ExtensionDefinition original = ExtensionDefinition.create(
                    identity, compatibility, discriminator, schema);
            ExtensionDefinition definition = ExtensionDefinition.withInformation(original, information);
            try (original;
                    definition;
                    TtlvStructure informationTtlv = ExtensionInformation.toTtlv(information);
                    TtlvStructureView informationView = informationTtlv.view();
                    TtlvStructure payload = markerPayload(tag.raw(), CODEC_LIMITS)) {
                assertSame(identity, ExtensionDefinition.identity(definition));
                assertSame(information, ExtensionDefinition.information(definition).orElseThrow());
                assertTrue(informationView.itemCount() > 0);

                try (ValidatedExtensionValue validated = ExtensionDefinition.validate(
                        definition, payload, CODEC_LIMITS);
                        TtlvStructureView genericValue = ValidatedExtensionValue.genericValue(validated)) {
                    assertSame(identity, ValidatedExtensionValue.identity(validated));
                    assertEquals(1, genericValue.itemCount());
                }

                try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(List.of(definition), limits)) {
                    assertEquals(1, registry.definitionCount());
                    assertSame(definition, registry.definitionAt(0).orElseThrow());
                    assertTrue(ClientExtensionRegistry.definitionForIdentity(registry, identity).isPresent());
                    assertFalse(registry.definitionAt(-1).isPresent());

                    try (ExtensionRecognition recognition = registry.inspect(
                            "unregistered.vendor", payload, CODEC_LIMITS);
                            TtlvStructureView generic = ExtensionRecognition.genericValue(recognition)) {
                        assertFalse(ExtensionRecognition.isRecognized(recognition));
                        assertTrue(ExtensionRecognition.validatedValue(recognition).isEmpty());
                        assertEquals(1, generic.itemCount());
                        assertTrue(recognition.toString().contains("REDACTED"));
                    }

                    try (ExtensionRecognition recognition = registry.inspect(
                            identity.vendorIdentifier(), payload, CODEC_LIMITS)) {
                        assertTrue(ExtensionRecognition.isRecognized(recognition));
                        try (ValidatedExtensionValue value = ExtensionRecognition.validatedValue(recognition)
                                .orElseThrow()) {
                            assertEquals(identity, ValidatedExtensionValue.identity(value));
                        }
                    }

                    try (RegisteredExtensionValue registered = registry.validateExtensionValue(
                            identity, payload, CODEC_LIMITS)) {
                        assertTrue(registered.toString().contains("REDACTED"));
                        ClientRequestMessageExtension requestExtension =
                                ClientRequestMessageExtension.create(registered, true);
                        try (requestExtension;
                                ClientBatchItem baseItem = ClientBatchItem.discoverVersions()) {
                            ClientBatchItem itemWithExtension = baseItem.withExtension(requestExtension);
                            try (itemWithExtension;
                                    ExtensionIdentity copiedIdentity = itemWithExtension.extensionIdentityAt(0)) {
                                assertEquals(1, itemWithExtension.extensionCount());
                                assertTrue(itemWithExtension.extensionCriticalityIndicatorAt(0));
                                assertEquals(identity, copiedIdentity);
                            }
                        }
                    }

                    ClientConfiguration configuration = ClientConfiguration.create(registry);
                    try (configuration; ClientExtensionRegistry restored = configuration.extensionRegistry()) {
                        assertEquals(1, restored.definitionCount());
                        assertEquals(limits.maxDefinitions(), restored.limits().maxDefinitions());
                    }
                }
            }
        }
    }

    @Test
    void rejectsOversizedAndMalformedExtensionIdentityText() {
        assertThrows(InvalidInputException.class,
                () -> ExtensionIdentity.create(
                        "example.vendor", String.valueOf((char) 0xdc00), "1.0"));
        assertThrows(ResourceLimitException.class,
                () -> ExtensionIdentity.create("example.vendor", "x".repeat(4_097), "1.0"));
    }

    @Test
    void closesTransportAndBatchWrappersAndChecksRawOutcomeValues() {
        BatchOutcomeKind completed = new BatchOutcomeKind(BatchOutcomeKind.COMPLETED_VALUE);
        BatchOutcomeKind pending = new BatchOutcomeKind(BatchOutcomeKind.PENDING_VALUE);
        BatchOutcomeKind future = new BatchOutcomeKind(0xffff_ffffL);
        assertEquals(BatchOutcomeKind.COMPLETED_VALUE, completed.raw());
        assertTrue(completed.isCompleted());
        assertFalse(completed.isPending());
        assertEquals(BatchOutcomeKind.PENDING_VALUE, pending.raw());
        assertTrue(pending.isPending());
        assertFalse(pending.isCompleted());
        assertFalse(future.isCompleted());
        assertFalse(future.isPending());
        assertThrows(InvalidInputException.class, () -> new BatchOutcomeKind(-1));
        assertThrows(InvalidInputException.class, () -> new BatchOutcomeKind(0x1_0000_0000L));

        try (ClientBatch batch = ClientBatch.create()) {
            assertEquals(0, batch.itemCount());
            assertThrows(InvalidInputException.class, () -> batch.add(null));
            assertThrows(InvalidInputException.class, () -> batch.asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class,
                    () -> batch.asynchronousIndicator(0x1_0000_0000L));
            batch.asynchronousIndicator(BatchOutcomeKind.PENDING_VALUE);
            try (ClientBatchItem item = ClientBatchItem.discoverVersions()) {
                assertEquals(0, item.extensionCount());
                batch.add(item);
                item.close();
                assertEquals(1, batch.itemCount());
            }
        }

        TransportConfiguration.Builder transportBuilder = TransportConfiguration.builder();
        assertSame(transportBuilder, transportBuilder.maxRequestBytes(4_096));
        assertThrows(InvalidInputException.class, () -> transportBuilder.maxRequestBytes(0));
    }

    private static TtlvStructure markerPayload(int tagValue, CodecLimits limits) {
        try (RawTag rawTag = RawTag.fromRaw(tagValue);
                Tag tag = rawTag.checked();
                TtlvStructure empty = TtlvStructure.create();
                TtlvValue value = TtlvValue.TextString("marker".getBytes(StandardCharsets.UTF_8), limits);
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(empty, item, limits);
        }
    }
}
