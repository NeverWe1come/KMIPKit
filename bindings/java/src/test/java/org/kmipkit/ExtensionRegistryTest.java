package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.ToLongFunction;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ClientRequestMessageExtension;
import org.kmipkit.extensions.Compatibility;
import org.kmipkit.extensions.Discriminator;
import org.kmipkit.extensions.ExtensionChildRule;
import org.kmipkit.extensions.ExtensionDefinition;
import org.kmipkit.extensions.ExtensionIdentity;
import org.kmipkit.extensions.ExtensionInformation;
import org.kmipkit.extensions.ExtensionRecognition;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.ExtensionSchema;
import org.kmipkit.extensions.InvalidExtensionSchemaException;
import org.kmipkit.extensions.RegisteredExtensionValue;
import org.kmipkit.extensions.TtlvPath;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

/** Java 17 registry tests tied to the shared KMIPKIT-0012 fixture corpus. */
final class ExtensionRegistryTest {
    private static final String VENDOR = "example.vendor";
    private static final String ALPHA_NAME = "alpha";
    private static final String ALPHA_VERSION = "1";
    private static final String ALPHA_MARKER = "alpha-v1";
    private static final int MARKER_TAG = 0x420001;
    // KMIP 2.1 §11.56 Extensions range; accepted under ADR-0010's tag allocation policy.
    private static final int VENDOR_RANGE_TAG = 0x540001;
    private static final CodecLimits CODEC_LIMITS = CodecLimits.defaults();

    private record LimitCase(
            String name,
            long defaultValue,
            long hardMaximum,
            ToLongFunction<ExtensionRegistryLimits> readValue) {}

    @Test
    void everySharedInboundFixtureRunsThroughThePublicJavaAdapterInBothOrders() throws Exception {
        for (boolean reversed : List.of(false, true)) {
            List<ExtensionDefinition> definitions = new ArrayList<>();
            for (int offset = 0; offset < SharedExtensionFixtures.DEFINITIONS.size(); offset++) {
                int index = reversed
                        ? SharedExtensionFixtures.DEFINITIONS.size() - offset - 1
                        : offset;
                definitions.add(sharedDefinition(SharedExtensionFixtures.DEFINITIONS.get(index)));
            }
            ClientExtensionRegistry registry = registry(definitions, defaults());
            for (SharedExtensionFixtures.Case fixture : SharedExtensionFixtures.CASES) {
                TtlvStructure payload = sharedPayload(fixture.payload());
                ExtensionRecognition recognition = registry.inspect(fixture.vendor(), payload, CODEC_LIMITS);
                boolean expectedRecognized = fixture.outcome().equals("recognized");
                assertEquals(expectedRecognized, ExtensionRecognition.isRecognized(recognition), fixture.id());
                Optional<?> typed = ExtensionRecognition.validatedValue(recognition);
                assertEquals(fixture.typed(), typed.isPresent(), fixture.id());
                assertEquals(expectedRecognized, typed.isPresent(), fixture.id());
                TtlvStructureView preserved = ExtensionRecognition.genericValue(recognition);
                assertSharedStructure(preserved, fixture.payload(), fixture.id());
            }
            assertSharedOutboundRequests(registry);
        }
    }

    private static void assertSharedOutboundRequests(ClientExtensionRegistry registry) throws Exception {
        for (SharedExtensionFixtures.OutboundRequest request : SharedExtensionFixtures.OUTBOUND_REQUESTS) {
            assertEquals("outbound.validated", request.outcome(), request.fixtureId());
            ClientBatchItem item = ClientBatchItem.discoverVersions();
            for (SharedExtensionFixtures.Attachment attachment : request.attachments()) {
                SharedExtensionFixtures.Case fixture = SharedExtensionFixtures.CASES.stream()
                        .filter(candidate -> candidate.id().equals(attachment.fixtureId()))
                        .findFirst().orElseThrow();
                assertEquals("recognized", fixture.outcome(), fixture.id());
                assertTrue(fixture.typed(), fixture.id());
                assertEquals(1, fixture.matchedIds().size(), fixture.id());
                SharedExtensionFixtures.Definition definition = SharedExtensionFixtures.DEFINITIONS.stream()
                        .filter(candidate -> candidate.id().equals(fixture.matchedIds().get(0)))
                        .findFirst().orElseThrow();
                ExtensionIdentity identity = ExtensionIdentity.create(
                        definition.vendor(), definition.name(), definition.version());
                TtlvStructure payload = sharedPayload(fixture.payload());
                RegisteredExtensionValue value = registry.validateExtensionValue(
                        identity, payload, CODEC_LIMITS);
                item = item.withExtension(ClientRequestMessageExtension.create(
                        value, attachment.criticalityIndicator()));
            }

            assertEquals(request.attachments().size(), item.extensionCount(), request.fixtureId());
            for (int index = 0; index < request.attachments().size(); index++) {
                SharedExtensionFixtures.Attachment attachment = request.attachments().get(index);
                SharedExtensionFixtures.Case fixture = SharedExtensionFixtures.CASES.stream()
                        .filter(candidate -> candidate.id().equals(attachment.fixtureId()))
                        .findFirst().orElseThrow();
                SharedExtensionFixtures.Definition definition = SharedExtensionFixtures.DEFINITIONS.stream()
                        .filter(candidate -> candidate.id().equals(fixture.matchedIds().get(0)))
                        .findFirst().orElseThrow();
                ExtensionIdentity actual = item.extensionIdentityAt(index);
                assertEquals(definition.vendor(), actual.vendorIdentifier(), request.fixtureId());
                assertEquals(definition.name(), actual.name(), request.fixtureId());
                assertEquals(definition.version(), actual.version(), request.fixtureId());
                assertEquals(attachment.criticalityIndicator(),
                        item.extensionCriticalityIndicatorAt(index), request.fixtureId());
            }
        }
    }

    @Test
    void clientConfigurationsOwnImmutableIsolatedRegistrySnapshots() throws Exception {
        List<ExtensionDefinition> callerDefinitions = new ArrayList<>();
        callerDefinitions.add(alphaDefinition());
        ClientExtensionRegistry alphaRegistry = registry(callerDefinitions, defaults());
        callerDefinitions.clear();
        ClientConfiguration alphaConfiguration = ClientConfiguration.create(alphaRegistry);

        ClientExtensionRegistry betaRegistry = registry(
                List.of(definition("beta", "beta-v1", TtlvItemType.TextString)), defaults());
        ClientConfiguration betaConfiguration = ClientConfiguration.create(betaRegistry);

        ClientExtensionRegistry alphaOwnedRegistry = alphaConfiguration.extensionRegistry();
        ClientExtensionRegistry betaOwnedRegistry = betaConfiguration.extensionRegistry();
        assertEquals(1, alphaOwnedRegistry.definitionCount());
        assertEquals(1, betaOwnedRegistry.definitionCount());
        assertTrue(ClientExtensionRegistry.definitionForIdentity(
                alphaOwnedRegistry, identity(ALPHA_NAME)).isPresent());
        assertTrue(ClientExtensionRegistry.definitionForIdentity(
                alphaOwnedRegistry, identity("beta")).isEmpty());
        assertTrue(ClientExtensionRegistry.definitionForIdentity(
                betaOwnedRegistry, identity("beta")).isPresent());
        assertTrue(ClientExtensionRegistry.definitionForIdentity(
                betaOwnedRegistry, identity(ALPHA_NAME)).isEmpty());
    }

    @Test
    void registryListAndIdentityLookupViewsAreStableAcrossRegistrationOrder() throws Exception {
        ExtensionDefinition alpha = alphaDefinition();
        ExtensionDefinition beta = definition("beta", "beta-v1", TtlvItemType.TextString);
        ClientExtensionRegistry registry = registry(List.of(beta, alpha), defaults());

        assertEquals(2, registry.definitionCount());
        assertEquals(identity(ALPHA_NAME), ExtensionDefinition.identity(registry.definitionAt(0).orElseThrow()));
        assertEquals(identity("beta"), ExtensionDefinition.identity(registry.definitionAt(1).orElseThrow()));
        assertEquals(identity(ALPHA_NAME), ExtensionDefinition.identity(
                ClientExtensionRegistry.definitionForIdentity(registry, identity(ALPHA_NAME)).orElseThrow()));
        assertEquals(identity("beta"), ExtensionDefinition.identity(
                ClientExtensionRegistry.definitionForIdentity(registry, identity("beta")).orElseThrow()));
    }

    @Test
    void everyRegistryLimitExposesDefaultsAndAcceptsItsLowerAndHardBoundaries() throws Exception {
        ExtensionRegistryLimits defaults = defaults();
        List<LimitCase> cases = limitCases();
        assertEquals(12, cases.size());

        for (int index = 0; index < cases.size(); index++) {
            final int limitIndex = index;
            LimitCase limit = cases.get(index);
            assertEquals(limit.defaultValue(), limit.readValue().applyAsLong(defaults), limit.name());

            ExtensionRegistryLimits lowered = limitsWithOverride(index, 0);
            assertEquals(0, limit.readValue().applyAsLong(lowered), limit.name() + " may be lowered");

            ExtensionRegistryLimits atHardMaximum = limitsWithOverride(index, limit.hardMaximum());
            assertEquals(limit.hardMaximum(), limit.readValue().applyAsLong(atHardMaximum),
                    limit.name() + " accepts its hard maximum");

            if (limit.defaultValue() < limit.hardMaximum()) {
                ExtensionRegistryLimits raised = limitsWithOverride(index, limit.defaultValue() + 1);
                assertEquals(limit.defaultValue() + 1, limit.readValue().applyAsLong(raised),
                        limit.name() + " may be raised above its default");
            }

            assertThrows(ResourceLimitException.class,
                    () -> limitsWithOverride(limitIndex, limit.hardMaximum() + 1),
                    limit.name() + " rejects values above its hard maximum");
        }
    }

    @Test
    void schemaAndLookupBudgetsFailWithoutReturningPartialRecognition() throws Exception {
        ExtensionRegistryLimits insufficientSchemaNodes = limitsWithOverride(1, 1);
        assertThrows(ResourceLimitException.class,
                () -> registry(List.of(alphaDefinition()), insufficientSchemaNodes));

        ExtensionRegistryLimits insufficientLookupWork = limitsWithOverride(10, 1);
        ClientExtensionRegistry lookupLimited = registry(List.of(alphaDefinition()), insufficientLookupWork);
        TtlvStructure widePayload = alphaPayload("preserve-this-child-order");
        assertThrows(ResourceLimitException.class,
                () -> lookupLimited.inspect(VENDOR, widePayload, CODEC_LIMITS));

        ExtensionRegistryLimits insufficientPayloadIndex = limitsWithOverride(9, 1);
        ClientExtensionRegistry indexLimited = registry(List.of(alphaDefinition()), insufficientPayloadIndex);
        assertThrows(ResourceLimitException.class,
                () -> indexLimited.inspect(VENDOR, widePayload, CODEC_LIMITS));
    }

    @Test
    void vendorIdentificationAloneDoesNotRecognizeAnUnmatchedDiscriminator() throws Exception {
        ClientExtensionRegistry registry = registry(List.of(alphaDefinition()), defaults());
        ExtensionRecognition recognition = registry.inspect(
                VENDOR, payloadWithMarker("different-marker", "preserve-this-child-order"), CODEC_LIMITS);

        assertFalse(ExtensionRecognition.isRecognized(recognition));
        assertTrue(ExtensionRecognition.validatedValue(recognition).isEmpty());
        assertEquals(5, ExtensionRecognition.genericValue(recognition).itemCount());
    }

    @Test
    void extensionInformationUsesTheOrderedTable365Fields() throws Exception {
        ExtensionInformation information = ExtensionInformation.create("alpha");
        information = ExtensionInformation.with_tag(information, MARKER_TAG);
        information = ExtensionInformation.with_type(information, TtlvItemType.TextString);
        information = ExtensionInformation.with_enumeration(information, 7);
        information = ExtensionInformation.with_attribute(information, true);
        information = ExtensionInformation.with_parent_structure_tag(information, 0x420010);
        information = ExtensionInformation.with_description(information, "alpha description");

        TtlvStructure ttlv = ExtensionInformation.toTtlv(information);
        TtlvStructureView view = ttlv.view();
        assertEquals(7, view.itemCount());
        assertEquals(List.of(0x4200A5, 0x4200A6, 0x4200A7, 0x420129, 0x42012A, 0x42012B, 0x42012C),
                List.of(
                        view.itemAt(0).tag().raw(),
                        view.itemAt(1).tag().raw(),
                        view.itemAt(2).tag().raw(),
                        view.itemAt(3).tag().raw(),
                        view.itemAt(4).tag().raw(),
                        view.itemAt(5).tag().raw(),
                        view.itemAt(6).tag().raw()));
    }

    @Test
    void invalidSchemaErrorsAndValidatedValueFormattingRedactPayloadText() throws Exception {
        String secret = "secret-value-must-not-appear-in-diagnostics";
        ExtensionDefinition withRequiredNumericField = definitionWithExtraRequiredField();
        ClientExtensionRegistry registry = registry(List.of(withRequiredNumericField), defaults());
        TtlvStructure secretPayload = alphaPayload(secret);

        InvalidExtensionSchemaException error = assertThrows(
                InvalidExtensionSchemaException.class,
                () -> registry.validateExtensionValue(
                        identity(ALPHA_NAME), secretPayload, CODEC_LIMITS));
        assertFalse(error.getMessage().contains(secret));
        assertFalse(error.toString().contains(secret));

        RegisteredExtensionValue value = registry(List.of(alphaDefinition()), defaults())
                .validateExtensionValue(identity(ALPHA_NAME), secretPayload, CODEC_LIMITS);
        assertFalse(value.toString().contains(secret));
    }

    @Test
    void closedRegistryHandlesFailWithInvalidInputBeforeNativeAccess() throws Exception {
        ClientExtensionRegistry registry = registry(List.of(alphaDefinition()), defaults());
        registry.close();

        assertThrows(InvalidInputException.class, registry::definitionCount);
    }

    @Test
    void requestUseFactoryAcceptsBothExplicitCriticalityChoices() throws Exception {
        ClientExtensionRegistry registry = registry(List.of(alphaDefinition()), defaults());
        RegisteredExtensionValue nonCriticalValue = registry.validateExtensionValue(
                identity(ALPHA_NAME), alphaPayload("preserve-this-child-order"), CODEC_LIMITS);
        RegisteredExtensionValue criticalValue = registry.validateExtensionValue(
                identity(ALPHA_NAME), alphaPayload("preserve-this-child-order"), CODEC_LIMITS);

        ClientRequestMessageExtension nonCritical = ClientRequestMessageExtension.create(nonCriticalValue, false);
        ClientRequestMessageExtension critical = ClientRequestMessageExtension.create(criticalValue, true);

        assertNotNull(nonCritical);
        assertNotNull(critical);
    }

    @Test
    void discoverVersionsBatchItemPreservesRepeatedExtensionOrderIdentityAndCriticality() throws Exception {
        ExtensionDefinition alpha = alphaDefinition();
        ExtensionDefinition beta = definition("beta", "beta-v1", TtlvItemType.TextString);
        ClientExtensionRegistry registry = registry(List.of(alpha, beta), defaults());
        RegisteredExtensionValue alphaValue = registry.validateExtensionValue(
                identity(ALPHA_NAME), alphaPayload("alpha-payload"), CODEC_LIMITS);
        RegisteredExtensionValue betaValue = registry.validateExtensionValue(
                identity("beta"), payloadWithMarker("beta-v1", "beta-payload"), CODEC_LIMITS);

        ClientRequestMessageExtension nonCritical = ClientRequestMessageExtension.create(alphaValue, false);
        ClientRequestMessageExtension critical = ClientRequestMessageExtension.create(betaValue, true);
        ClientBatchItem item = ClientBatchItem.discoverVersions()
                .withExtension(nonCritical)
                .withExtension(critical);

        assertEquals(2, item.extensionCount());
        assertEquals(identity(ALPHA_NAME), item.extensionIdentityAt(0));
        assertFalse(item.extensionCriticalityIndicatorAt(0));
        assertEquals(identity("beta"), item.extensionIdentityAt(1));
        assertTrue(item.extensionCriticalityIndicatorAt(1));
        assertThrows(InvalidInputException.class, () -> item.extensionIdentityAt(2));
        assertThrows(InvalidInputException.class, () -> item.extensionCriticalityIndicatorAt(2));
    }

    private static ExtensionDefinition alphaDefinition() throws Exception {
        return definition(ALPHA_NAME, ALPHA_MARKER, TtlvItemType.TextString);
    }

    private static ExtensionDefinition definition(
            String name, String marker, TtlvItemType markerType) throws Exception {
        ExtensionIdentity identity = identity(name);
        Compatibility compatibility = Compatibility.create(2, 1, 2, 1, "0.0.0", "99.0.0");
        Tag markerTag = tag(MARKER_TAG);
        TtlvValue markerValue = TtlvValue.TextString(marker.getBytes(StandardCharsets.UTF_8), CODEC_LIMITS);
        Discriminator discriminator = Discriminator.create(TtlvPath.create(markerTag), markerValue);
        ExtensionSchema childSchema = ExtensionSchema.scalar(markerType);
        ExtensionSchema schema = ExtensionSchema.structure(
                List.of(ExtensionChildRule.required(markerTag, childSchema)), List.of(), true);
        ExtensionDefinition definition = ExtensionDefinition.create(identity, compatibility, discriminator, schema);
        return ExtensionDefinition.withInformation(definition, ExtensionInformation.create(name));
    }

    private static ExtensionDefinition definitionWithExtraRequiredField() throws Exception {
        ExtensionIdentity identity = identity(ALPHA_NAME);
        Compatibility compatibility = Compatibility.create(2, 1, 2, 1, "0.0.0", "99.0.0");
        Tag markerTag = tag(MARKER_TAG);
        TtlvValue markerValue = TtlvValue.TextString(ALPHA_MARKER.getBytes(StandardCharsets.UTF_8), CODEC_LIMITS);
        Discriminator discriminator = Discriminator.create(TtlvPath.create(markerTag), markerValue);
        ExtensionSchema textSchema = ExtensionSchema.scalar(TtlvItemType.TextString);
        ExtensionSchema numericSchema = ExtensionSchema.scalar(TtlvItemType.LongInteger);
        ExtensionSchema schema = ExtensionSchema.structure(
                List.of(
                        ExtensionChildRule.required(markerTag, textSchema),
                        ExtensionChildRule.required(tag(0x420006), numericSchema)),
                List.of(), true);
        ExtensionDefinition definition = ExtensionDefinition.create(identity, compatibility, discriminator, schema);
        return ExtensionDefinition.withInformation(definition, ExtensionInformation.create(ALPHA_NAME));
    }

    private static ExtensionDefinition sharedDefinition(SharedExtensionFixtures.Definition fixture) throws Exception {
        ExtensionIdentity identity = ExtensionIdentity.create(
                fixture.vendor(), fixture.name(), fixture.version());
        Compatibility compatibility = Compatibility.create(2, 1, 2, 1, "0.0.0", "99.0.0");
        Tag firstTag = tag(fixture.path().get(0));
        TtlvPath path = TtlvPath.create(firstTag);
        for (int index = 1; index < fixture.path().size(); index++) {
            path = TtlvPath.withChildTag(path, tag(fixture.path().get(index)));
        }
        TtlvValue discriminatorValue = sharedScalar(
                fixture.discriminatorType(), fixture.discriminatorText(), fixture.discriminatorNumber());
        Discriminator discriminator = Discriminator.create(path, discriminatorValue);
        ExtensionSchema schema = sharedSchema(fixture.schema());
        return ExtensionDefinition.create(identity, compatibility, discriminator, schema);
    }

    private static ExtensionSchema sharedSchema(SharedExtensionFixtures.Schema fixture) throws Exception {
        TtlvItemType itemType = TtlvItemType.fromCode(fixture.type());
        if (itemType != TtlvItemType.Structure) {
            ExtensionSchema scalar = ExtensionSchema.scalar(itemType);
            return fixture.hasRange()
                    ? ExtensionSchema.with_signed_range(scalar, fixture.minimum(), fixture.maximum())
                    : scalar;
        }
        List<ExtensionChildRule> children = new ArrayList<>();
        for (SharedExtensionFixtures.ChildRule child : fixture.children()) {
            children.add(ExtensionChildRule.required(tag(child.tag()), sharedSchema(child.schema())));
        }
        return ExtensionSchema.structure(children, List.of(), true);
    }

    private static TtlvStructure sharedPayload(List<SharedExtensionFixtures.Item> items) throws Exception {
        TtlvStructure structure = TtlvStructure.create();
        for (SharedExtensionFixtures.Item item : items) {
            TtlvValue value = sharedValue(item);
            TtlvItem ttlvItem = TtlvItem.create(tag(item.tag()), value, CODEC_LIMITS);
            structure = TtlvStructure.withItem(structure, ttlvItem, CODEC_LIMITS);
        }
        return structure;
    }

    private static TtlvValue sharedValue(SharedExtensionFixtures.Item item) throws Exception {
        TtlvItemType itemType = TtlvItemType.fromCode(item.type());
        if (itemType == TtlvItemType.Structure) {
            return TtlvValue.structure(sharedPayload(item.children()), CODEC_LIMITS);
        }
        return sharedScalar(item.type(), item.text(), item.number());
    }

    private static TtlvValue sharedScalar(int type, String text, long number) {
        TtlvItemType itemType = TtlvItemType.fromCode(type);
        return switch (itemType) {
            case TextString -> TtlvValue.TextString(text.getBytes(StandardCharsets.UTF_8), CODEC_LIMITS);
            case Enumeration -> TtlvValue.Enumeration(number);
            case LongInteger -> TtlvValue.LongInteger(number);
            default -> throw new IllegalArgumentException("Unsupported generated fixture scalar type");
        };
    }

    private static void assertSharedStructure(TtlvStructureView actual,
            List<SharedExtensionFixtures.Item> expected, String caseId) {
        assertEquals(expected.size(), actual.itemCount(), caseId);
        for (int index = 0; index < expected.size(); index++) {
            SharedExtensionFixtures.Item fixture = expected.get(index);
            var item = actual.itemAt(index);
            assertEquals(fixture.tag(), item.tag().raw(), caseId + " item " + index + " tag");
            assertEquals(fixture.type(), item.itemType().code(), caseId + " item " + index + " type");
            var value = item.value();
            TtlvItemType type = TtlvItemType.fromCode(fixture.type());
            switch (type) {
                case Structure -> assertSharedStructure(value.structure(), fixture.children(), caseId);
                case TextString -> {
                    byte[] bytes = fixture.text().getBytes(StandardCharsets.UTF_8);
                    assertEquals(bytes.length, value.byteLength(), caseId + " text length");
                    for (int offset = 0; offset < bytes.length; offset++) {
                        assertEquals(bytes[offset], (byte) value.byteAt(offset), caseId + " text byte");
                    }
                }
                case Enumeration -> assertEquals(fixture.number(), value.enumeration(), caseId);
                case LongInteger -> assertEquals(fixture.number(), value.longInteger(), caseId);
                default -> throw new AssertionError("Unsupported generated fixture TTLV type");
            }
        }
    }

    private static TtlvStructure alphaPayload(String trailingText) throws Exception {
        return payloadWithMarker(ALPHA_MARKER, trailingText);
    }

    private static TtlvStructure payloadWithMarker(String marker, String trailingText) throws Exception {
        TtlvStructure structure = TtlvStructure.create();
        structure = append(structure, MARKER_TAG,
                TtlvValue.TextString(marker.getBytes(StandardCharsets.UTF_8), CODEC_LIMITS));
        structure = append(structure, 0x420002, TtlvValue.Enumeration(4_294_967_295L));
        structure = append(structure, 0x420004, TtlvValue.LongInteger(42));
        structure = append(structure, 0x420006,
                TtlvValue.TextString(trailingText.getBytes(StandardCharsets.UTF_8), CODEC_LIMITS));
        structure = append(structure, VENDOR_RANGE_TAG,
                TtlvValue.TextString("preserve-vendor-range-tag".getBytes(StandardCharsets.UTF_8), CODEC_LIMITS));
        return structure;
    }

    private static TtlvStructure append(TtlvStructure structure, int rawTag, TtlvValue value) throws Exception {
        TtlvItem item = TtlvItem.create(tag(rawTag), value, CODEC_LIMITS);
        return TtlvStructure.withItem(structure, item, CODEC_LIMITS);
    }

    private static String textValue(TtlvValueView value) {
        int length = Math.toIntExact(value.byteLength());
        byte[] bytes = new byte[length];
        for (int index = 0; index < length; index++) {
            bytes[index] = (byte) value.byteAt(index);
        }
        return new String(bytes, StandardCharsets.UTF_8);
    }

    private static ExtensionIdentity identity(String name) throws Exception {
        return ExtensionIdentity.create(VENDOR, name, ALPHA_VERSION);
    }

    private static ClientExtensionRegistry registry(
            List<ExtensionDefinition> definitions, ExtensionRegistryLimits limits) throws Exception {
        return ClientExtensionRegistry.create(definitions, limits);
    }

    private static ExtensionRegistryLimits defaults() {
        return ExtensionRegistryLimits.defaults();
    }

    private static Tag tag(int raw) throws Exception {
        return RawTag.fromRaw(raw).checked();
    }

    private static ExtensionRegistryLimits limitsWithOverride(int index, long replacement) throws Exception {
        long[] values = {
                256, 16_384, 256, 4_096, 1_048_576, 4_096,
                1_048_576, 256, 16_384, 200_000, 1_048_576, 64
        };
        assertTrue(index >= 0 && index < values.length);
        values[index] = replacement;
        return ExtensionRegistryLimits.withValues(
                values[0], values[1], values[2], values[3], values[4], values[5],
                values[6], values[7], values[8], values[9], values[10], values[11]);
    }

    private static List<LimitCase> limitCases() {
        return List.of(
                new LimitCase("maxDefinitions", 256, 1_024, ExtensionRegistryLimits::maxDefinitions),
                new LimitCase("maxSchemaNodes", 16_384, 100_000, ExtensionRegistryLimits::maxSchemaNodes),
                new LimitCase("maxChildRulesPerStructure", 256, 4_096,
                        ExtensionRegistryLimits::maxChildRulesPerStructure),
                new LimitCase("maxTextBytesPerField", 4_096, 4_096,
                        ExtensionRegistryLimits::maxTextBytesPerField),
                new LimitCase("maxRegistryTextBytes", 1_048_576, 16_777_216,
                        ExtensionRegistryLimits::maxRegistryTextBytes),
                new LimitCase("maxDiscriminatorScalarBytes", 4_096, 4_096,
                        ExtensionRegistryLimits::maxDiscriminatorScalarBytes),
                new LimitCase("maxTotalDiscriminatorScalarBytes", 1_048_576, 16_777_216,
                        ExtensionRegistryLimits::maxTotalDiscriminatorScalarBytes),
                new LimitCase("maxConstraintMembersPerRule", 256, 4_096,
                        ExtensionRegistryLimits::maxConstraintMembersPerRule),
                new LimitCase("maxTotalConstraintMembers", 16_384, 100_000,
                        ExtensionRegistryLimits::maxTotalConstraintMembers),
                new LimitCase("maxPayloadIndexRecords", 200_000, 200_000,
                        ExtensionRegistryLimits::maxPayloadIndexRecords),
                new LimitCase("maxLookupComparisons", 1_048_576, 4_194_304,
                        ExtensionRegistryLimits::maxLookupComparisons),
                new LimitCase("maxDepth", 64, 64, ExtensionRegistryLimits::maxDepth));
    }

}
