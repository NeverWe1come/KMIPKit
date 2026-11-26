package org.kmipkit.validator;

import org.kmipkit.ClientOperationException;
import org.kmipkit.credentials.Authentication;
import org.kmipkit.credentials.Credential;
import org.kmipkit.credentials.SecretText;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.Client;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.OpaqueObjectContext;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.extensions.RegisterRequest;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.extensions.RevokeRequest;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.extensions.WireEncoding;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.ManagedObject;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
import java.util.HexFormat;
import java.util.List;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.BiFunction;
import java.util.function.Predicate;

/** Executes the Java public typed client against a KMIP server without logging remote payloads. */
public final class JavaKmipDriver implements AutoCloseable {
    private static final CodecLimits CODEC_LIMITS = CodecLimits.defaults();
    private static final byte[] HASH_INPUT = "abc".getBytes(StandardCharsets.US_ASCII);
    private static final byte[] SHA256_EXPECTED = HexFormat.of().parseHex(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    private static final long OPAQUE_DATA_TYPE = 0x8000_0001L;
    private static final byte[] OPAQUE_FIXTURE = "KMIPKit dedicated opaque fixture".getBytes(StandardCharsets.US_ASCII);
    private final KmipClient client;
    private final ConnectionConfig connection;

    private JavaKmipDriver(KmipClient client, ConnectionConfig connection) {
        this.client = client;
        this.connection = connection;
    }

    /** Creates one TLS client for a run; client construction itself performs no exchange. */
    public static JavaKmipDriver connect(ConnectionConfig connection) {
        return new JavaKmipDriver(configuredClient(connection, KmipClient::create), connection);
    }

    private static <T> T configuredClient(ConnectionConfig connection,
            BiFunction<ClientConfiguration, TransportConfiguration, T> factory) {
        ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                List.of(), ExtensionRegistryLimits.defaults());
        ClientConfiguration baseConfiguration = ClientConfiguration.create(registry);
        ClientConfiguration encodingConfiguration = baseConfiguration.withWireEncoding(
                WireEncoding.valueOf(connection.encoding()));
        ClientConfiguration protocolConfiguration = encodingConfiguration.withProtocolVersion(
                toProtocolVersion(connection.protocolVersion()));
        TransportConfiguration transport = buildTransport(connection);
        try (registry; baseConfiguration; encodingConfiguration; protocolConfiguration; transport) {
            return factory.apply(protocolConfiguration, transport);
        }
    }

    /** Executes one available typed operation and records only numeric KMIP result codes. */
    public CaseOutcome execute(CatalogLoader.CatalogCase testCase) {
        long start = System.nanoTime();
        if (asymmetricFormat(testCase) != 0) {
            return executeAsymmetricKey(testCase, asymmetricFormat(testCase), start);
        }
        if (keyBlockCase(testCase)) {
            return executeKeyBlock(testCase, start);
        }
        if ("Query".equals(testCase.operation())) {
            return executeQuery(testCase, start);
        }
        if (registerOpaqueCase(testCase)) {
            return executeRegisterOpaque(testCase, start);
        }
        if (createSplitKeyCase(testCase)) {
            return executeCreateSplitKey(testCase, start);
        }
        if ("Create".equals(testCase.operation()) || ownedObjectCase(testCase)) {
            return executeCreateWithCleanup(testCase, start);
        }
        if (multipartHash(testCase)) {
            return executeMultipartHash(testCase, start);
        }
        if (typedRngCase(testCase)) {
            return executeRngCase(testCase, start);
        }
        if (readOnlyProbe(testCase)) {
            return executeReadOnlyProbe(testCase, start);
        }
        if (extendedTypedCase(testCase)) {
            return executeExtended(testCase, start);
        }
        ClientBatchItem request = "fluent".equals(connection.apiLevel()) && "Ping".equals(testCase.operation())
                ? withBatchId(client.request().ping().build())
                : TypedRequestFactory.create(testCase.operation(), !isDraft(), toProtocolVersion(connection.protocolVersion()));
        return executeRequest(testCase, request, result -> verifyTypedResponse(testCase.operation(), result), start);
    }

    private CaseOutcome executeQuery(CatalogLoader.CatalogCase testCase, long start) {
        boolean field = queryFieldCase(testCase);
        int function = field ? testCase.requires().path("query_function").asInt(-1) : 1;
        int tag = field ? testCase.requires().path("query_result_tag").asInt(-1) : 0;
        if (function < 0 || tag < 0) {
            return CaseOutcome.failed("Invalid Query field fixture.", elapsedMillis(start));
        }
        Predicate<ClientBatchItemResult> verify = result -> field
                ? result.typedQueryResponse().filter(query -> {
                    try (query; var payload = query.rawView()) {
                        return GenericExchange.hasQueryStructure(payload, tag);
                    }
                }).isPresent()
                : verifyTypedResponse("Query", result);
        if ("high-level".equals(connection.apiLevel())) {
            try (var response = client.query(new int[]{function}, CODEC_LIMITS, null)) {
                return verifyResponse(testCase, KmipValues.Operation.QUERY, response, verify, start);
            } catch (ClientOperationException exception) {
                return exchangeFailure(exception, start);
            } catch (Exception exception) {
                return CaseOutcome.failed("KMIP Query exchange failed; remote diagnostic text was suppressed.",
                        elapsedMillis(start));
            }
        }
        if ("fluent".equals(connection.apiLevel())) {
            try (var builder = client.request().query()) {
                return executeRequest(testCase, withBatchId(builder.function(function).build()), verify, start);
            }
        }
        return executeRequest(testCase, withBatchId(ClientBatchItem.query(new int[]{function}, null)), verify, start);
    }

    static boolean queryFieldCase(CatalogLoader.CatalogCase testCase) {
        return "Query".equals(testCase.operation()) && testCase.requires().has("query_function");
    }

    static boolean readOnlyProbe(CatalogLoader.CatalogCase testCase) {
        if (queryFieldCase(testCase)) return true;
        return rotateLocateCase(testCase) || Set.of("test-op-locate", "test-op-hash", "test-crypto-hash-sha_256-0")
                .contains(testCase.caseId());
    }

    static boolean rotateLocateCase(CatalogLoader.CatalogCase testCase) {
        return "Locate".equals(testCase.operation()) && testCase.requires().path("rotate_attribute_ttlv").isTextual();
    }

    static boolean multipartHash(CatalogLoader.CatalogCase testCase) {
        return "test-multipart-hash-sha256".equals(testCase.caseId());
    }

    private static final Set<String> EXTENDED_FLUENT_OPERATIONS = Set.of(
            "Create Key Pair", "Register", "Re-key", "Re-key Key Pair", "Certify", "Re-certify", "Validate",
            "Derive Key", "Join Split Key", "Import", "Export", "Get Usage Allocation",
            "Add Attribute", "Modify Attribute", "Delete Attribute", "Set Attribute",
            "Encrypt", "Decrypt", "MAC", "MAC Verify", "Sign", "Signature Verify"
    );

    private static final Set<String> EXTENDED_TYPED_OPERATIONS = Set.of(
            "Create Key Pair", "Register", "Re-key", "Re-key Key Pair", "Certify", "Re-certify", "Validate",
            "Derive Key", "Join Split Key", "Import", "Export", "Get Usage Allocation",
            "Poll", "Cancel", "Add Attribute", "Modify Attribute", "Delete Attribute", "Set Attribute",
            "Encrypt", "Decrypt", "MAC", "MAC Verify", "Sign", "Signature Verify"
    );

    static boolean extendedTypedCase(CatalogLoader.CatalogCase testCase) {
        String wf = testCase.requires().path("workflow_type").asText();
        return !"server_to_client".equals(wf) && !"batch_id_placeholder".equals(wf)
                && EXTENDED_TYPED_OPERATIONS.contains(testCase.operation());
    }

    static boolean supportsFluent(CatalogLoader.CatalogCase testCase) {
        String wf = testCase.requires().path("workflow_type").asText();
        if ("server_to_client".equals(wf) || "async_poll".equals(wf) || "batch_id_placeholder".equals(wf)) {
            return false;
        }
        return !typedOnlyKeyProbe(testCase) && (registerOpaqueCase(testCase) || createSplitKeyCase(testCase)
                || ownedObjectCase(testCase) || readOnlyProbe(testCase) || multipartHash(testCase)
                || EXTENDED_FLUENT_OPERATIONS.contains(testCase.operation())
                || Set.of("Create", "Ping", "Query", "Discover Versions", "Check", "Obtain Lease",
                          "RNG Retrieve", "RNG Seed").contains(testCase.operation()));
    }

    static boolean typedRngCase(CatalogLoader.CatalogCase testCase) {
        return Set.of("test-op-rng-retrieve", "test-op-rng-seed",
                "test-crypto-rng-retrieve-rng-32", "test-crypto-rng-seed-rng-32").contains(testCase.caseId());
    }

    private CaseOutcome executeRngCase(CatalogLoader.CatalogCase testCase, long start) {
        int length = 32;
        if ("RNG Seed".equals(testCase.operation())) {
            var seed = "A".repeat(length).getBytes(StandardCharsets.US_ASCII);
            return executeRequest(testCase, withBatchId(ClientBatchItem.rngSeed(seed)), result ->
                    result.rngSeedResponse().filter(payload -> {
                        try (payload) {
                            int used = payload.dataLength().orElse(-1);
                            return used >= 0 && used <= length;
                        }
                    }).isPresent(), start);
        }
        return executeRequest(testCase, withBatchId(ClientBatchItem.rngRetrieve(length)), result ->
                result.rngRetrieveResponse().filter(payload -> {
                    try (payload) {
                        byte[] data = payload.data().orElse(null);
                        if (data == null) return false;
                        try {
                            return data.length == length;
                        } finally {
                            Arrays.fill(data, (byte) 0);
                        }
                    }
                }).isPresent(), start);
    }

    private CaseOutcome executeReadOnlyProbe(CatalogLoader.CatalogCase testCase, long start) {
        boolean fluent = "fluent".equals(connection.apiLevel());
        try {
            if ("Hash".equals(testCase.operation())) {
                return executeRequest(testCase, withBatchId(hashRequest(fluent, HASH_INPUT, null, null, null)), result -> {
                    var payload = result.hashResponse().orElse(null);
                    if (payload == null) return false;
                    try (var ttlv = payload.responseTtlv()) {
                        return payload.correlationValue().isEmpty()
                                && payload.data().map(bytes -> Arrays.equals(bytes, SHA256_EXPECTED)).orElse(false);
                    }
                }, start);
            }

            if (rotateLocateCase(testCase)) {
                var request = withBatchId(locateRotateRequest(testCase, fluent));
                if ("NotSent".equals(testCase.requires().path("expected_status").asText())) {
                    return executeRotateBoundary(request, start);
                }
                return executeRequest(testCase, request, result ->
                        result.locateResponse().map(payload -> payload.uniqueIdentifiers().size() <= 1).orElse(false),
                        start);
            }
            return executeRequest(testCase, withBatchId(locateRequest(fluent)), result ->
                    result.locateResponse().map(payload -> payload.uniqueIdentifiers().size() <= 1
                            && payload.locatedItems().orElse(payload.uniqueIdentifiers().size())
                                    >= payload.uniqueIdentifiers().size()).orElse(false), start);
        } catch (RuntimeException exception) {
            return exchangeFailure(exception, start);
        }
    }

    private CaseOutcome executeMultipartHash(CatalogLoader.CatalogCase testCase, long start) {
        boolean fluent = "fluent".equals(connection.apiLevel());
        var returnedCorrelation = new AtomicReference<byte[]>();
        byte[] expected = HexFormat.of().parseHex("484e69bd8ae45ed2440bf93f35e464f86ed9557258f0377c0c070a4f786b5b0a");
        try {
            byte[] first = "KMIP ".getBytes(StandardCharsets.US_ASCII);
            CaseOutcome initial = executeRequest(testCase,
                    withBatchId(hashRequest(fluent, first, null, true, null)), result -> {
                        var payload = result.hashResponse().orElse(null);
                        if (payload == null) return false;
                        try (var ttlv = payload.responseTtlv()) {
                            byte[] correlation = payload.correlationValue().orElse(null);
                            returnedCorrelation.set(correlation);
                            return payload.data().isEmpty() && correlation != null && correlation.length > 0;
                        }
                    }, start);
            if (!"passed".equals(initial.status())) return initial;
            byte[] correlation = returnedCorrelation.get();
            byte[] update = "Hash".getBytes(StandardCharsets.US_ASCII);
            CaseOutcome updated = executeRequest(testCase,
                    withBatchId(hashRequest(fluent, update, correlation, null, null)),
                    result -> checkHashOutput(result, null), start);
            if (!"passed".equals(updated.status())) return updated;
            byte[] last = "!".getBytes(StandardCharsets.US_ASCII);
            return executeRequest(testCase, withBatchId(hashRequest(fluent, last, correlation, null, true)),
                    result -> checkHashOutput(result, expected), start);
        } catch (RuntimeException exception) {
            return exchangeFailure(exception, start);
        } finally {
            byte[] correlation = returnedCorrelation.get();
            if (correlation != null) Arrays.fill(correlation, (byte) 0);
        }
    }

    private static boolean checkHashOutput(ClientBatchItemResult result, byte[] expected) {
        var payload = result.hashResponse().orElse(null);
        if (payload == null) return false;
        try (var ttlv = payload.responseTtlv()) {
            return payload.correlationValue().isEmpty() && Arrays.equals(payload.data().orElse(null), expected);
        }
    }

    private ClientBatchItem hashRequest(boolean fluent, byte[] input, byte[] correlation,
            Boolean init, Boolean end) {
        try (var parameters = appendOwnedValue(TtlvStructure.create(), 0x0042_0038, TtlvValue.Enumeration(6))) {
            if (fluent) {
                try (var builder = client.request().hash()) {
                    builder.cryptographicParametersStructure(parameters).data(input);
                    if (correlation != null) builder.correlationValue(correlation);
                    if (init != null) builder.initIndicator(init);
                    if (end != null) builder.finalIndicator(end);
                    return builder.build();
                }
            }
            try (var data = TtlvValue.ByteString(input, CODEC_LIMITS)) {
                return ClientBatchItem.hash(parameters, data, correlation, init, end);
            }
        }
    }

    static TtlvItem rotateAttribute(CatalogLoader.CatalogCase testCase) {
        byte[] wire = HexFormat.of().parseHex(testCase.requires().path("rotate_attribute_ttlv").asText());
        var attribute = TtlvCodec.decode(wire);
        try (var view = attribute.view(); var tag = view.tag()) {
            long rawTag = tag.raw();
            if (!(rawTag >= 0x42016a && rawTag <= 0x42016f || rawTag == 0x420172)) {
                throw new IllegalArgumentException("Rotate fixture has an unexpected tag");
            }
            return attribute;
        } catch (RuntimeException exception) {
            attribute.close();
            throw exception;
        }
    }

    private ClientBatchItem locateRotateRequest(CatalogLoader.CatalogCase testCase, boolean fluent) {
        try (var attr = rotateAttribute(testCase); var empty = TtlvStructure.create();
                var source = TtlvStructure.withItem(empty, attr, CODEC_LIMITS)) {
            return locateRequest(source, fluent);
        }
    }

    @SuppressWarnings("try")
    private CaseOutcome executeRotateBoundary(ClientBatchItem request, long start) {
        try (request; var batch = ClientBatch.create()) {
            batch.add(request);
            if (connection.usernamePassword() != null) {
                batch.setAuthentication(usernamePasswordAuthentication(connection.usernamePassword()));
            }
            try (var ignored = client.execute(batch, CODEC_LIMITS)) {
                return CaseOutcome.failed("Locate with the Rotate attribute was sent under KMIP 2.0.", elapsedMillis(start));
            }
        } catch (ClientOperationException exception) {
            if (exception.deliveryStateCode() == 1 && exception.categoryCode() == 1 && exception.causeCode() == 1) {
                return CaseOutcome.localModelPassed(elapsedMillis(start));
            }
            return exchangeFailure(exception, start);
        }
    }

    private ClientBatchItem locateRequest(boolean fluent) {
        // Description is a Text String in all three versions.
        byte[] isolatedDescription = ("kmipkit-locate-" + UUID.randomUUID()).getBytes(StandardCharsets.US_ASCII);
        try (var source = appendOwnedValue(TtlvStructure.create(), 0x0042_00fc,
                TtlvValue.TextString(isolatedDescription, CODEC_LIMITS))) {
            return locateRequest(source, fluent);
        }
    }

    private ClientBatchItem locateRequest(TtlvStructure source, boolean fluent) {
        if (fluent) {
            try (var attributes = AttributeSet.tryNew(source); var builder = client.request().locate(attributes)) {
                return builder.maximumItems(1).offsetItems(0).build();
            }
        }
        return ClientBatchItem.locate(1, 0, null, null, source);
    }

    private CaseOutcome executeRequest(CatalogLoader.CatalogCase testCase, ClientBatchItem request,
            Predicate<ClientBatchItemResult> verify, long start) {
        if (request == null) {
            return CaseOutcome.notRun("No Java typed request route is implemented for this catalog operation.",
                    elapsedMillis(start));
        }
        long requestOperation = request.operation();
        try (request; ClientBatch batch = ClientBatch.create()) {
            batch.add(request);
            if (connection.usernamePassword() != null) {
                batch.setAuthentication(usernamePasswordAuthentication(connection.usernamePassword()));
            }
            try (ClientBatchResponse response = client.execute(batch, CODEC_LIMITS)) {
                return verifyResponse(testCase, requestOperation, response, verify, start);
            }
        } catch (ClientOperationException exception) {
            if ("NotSent".equals(testCase != null ? testCase.requires().path("expected_status").asText() : "")
                    && exception.deliveryStateCode() == 1
                    && exception.categoryCode() == 1
                    && exception.causeCode() == 1) {
                return CaseOutcome.localModelPassed(elapsedMillis(start));
            }
            return exchangeFailure(exception, start);
        } catch (Exception exception) {
            return CaseOutcome.failed("KMIP client exchange failed (" + exception.getClass().getSimpleName()
                    + "); remote diagnostic text was suppressed.", elapsedMillis(start));
        }
    }

    private static CaseOutcome verifyResponse(CatalogLoader.CatalogCase testCase, long requestOperation,
            ClientBatchResponse response, Predicate<ClientBatchItemResult> verify, long start) {
        if (response.itemCount() != 1) {
            return CaseOutcome.failed("Response batch item count did not match the request.", elapsedMillis(start));
        }
        try (ClientBatchItemResult result = response.itemAt(0)) {
            if (result.operation() != requestOperation) {
                return CaseOutcome.failed("Response operation did not match the request.", elapsedMillis(start));
            }
            long statusCode = result.resultStatus();
            long reasonCode = result.resultReason().orElse(-1);
            if (statusCode == 0) {
                if (!verify.test(result)) {
                    return CaseOutcome.failed("The typed response shape did not match the request.",
                            elapsedMillis(start), statusCode, optionalCode(reasonCode));
                }
                return CaseOutcome.passed(elapsedMillis(start), statusCode, optionalCode(reasonCode));
            }
            if (reasonCode == KmipValues.ResultReason.OPERATION_NOT_SUPPORTED
                    || reasonCode == KmipValues.ResultReason.FEATURE_NOT_SUPPORTED) {
                return CaseOutcome.unsupported(elapsedMillis(start), statusCode, reasonCode);
            }
            if (reasonCode == KmipValues.ResultReason.PERMISSION_DENIED
                    || reasonCode == KmipValues.ResultReason.AUTHENTICATION_NOT_SUCCESSFUL) {
                return CaseOutcome.policyRestricted(elapsedMillis(start), statusCode, reasonCode);
            }
            if (expectedReason(testCase, reasonCode)) {
                return CaseOutcome.passed(elapsedMillis(start), statusCode, reasonCode);
            }
            return CaseOutcome.failed("The KMIP server returned a non-success result.",
                    elapsedMillis(start), statusCode, optionalCode(reasonCode));
        }
    }

    static int asymmetricFormat(CatalogLoader.CatalogCase testCase) {
        if (!"Get".equals(testCase.operation())) return 0;
        int format = testCase.requires().path("key_material_format").asInt(0);
        return switch (format) {
            case 10, 11, 20, 21 -> format;
            default -> 0;
        };
    }

    static boolean keyBlockCase(CatalogLoader.CatalogCase testCase) {
        return "Get".equals(testCase.operation()) && testCase.requires().has("key_block_shape");
    }

    static boolean ownedObjectCase(CatalogLoader.CatalogCase testCase) {
        return asymmetricFormat(testCase) != 0 || keyBlockCase(testCase) || attributeRead(testCase) || typedOnlyKeyProbe(testCase)
                || Set.of("test-op-get", "test-op-destroy", "test-op-archive",
                "test-op-recover", "test-op-activate", "test-op-revoke").contains(testCase.caseId());
    }

    private static boolean typedOnlyKeyProbe(CatalogLoader.CatalogCase testCase) {
        return Set.of("test-op-check", "test-op-obtain-lease").contains(testCase.caseId());
    }

    static boolean registerOpaqueCase(CatalogLoader.CatalogCase testCase) {
        return "test-obj-opaque-object".equals(testCase.caseId());
    }

    private CaseOutcome executeAsymmetricKey(CatalogLoader.CatalogCase testCase, int format, long start) {
        boolean ec = format >= 20;
        var ids = new ArrayList<UniqueIdentifier>();
        try (var algorithm = appendValue(TtlvStructure.create(), 0x420028, TtlvValue.Enumeration(ec ? 26 : 4));
                var length = appendValue(algorithm, 0x42002a, TtlvValue.Integer(ec ? 256 : 2048));
                var curve = appendValue(TtlvStructure.create(), 0x420075, TtlvValue.Enumeration(7));
                var common = ec ? appendValue(length, 0x420029, TtlvValue.structure(curve, CODEC_LIMITS)) : length.copy();
                var sign = appendValue(TtlvStructure.create(), 0x42002c, TtlvValue.Integer(1));
                var verify = appendValue(TtlvStructure.create(), 0x42002c, TtlvValue.Integer(2));
                var privateAttributes = appendValue(sign, 0x420122, TtlvValue.Boolean(true));
                var publicAttributes = appendValue(verify, 0x420122, TtlvValue.Boolean(true))) {
            var created = executeRequest(testCase, withBatchId(ClientBatchItem.createKeyPair(
                    common, privateAttributes, publicAttributes, null, null, null)),
                    response -> response.createKeyPairResponse().map(pair -> {
                        ids.add(pair.privateKeyUniqueIdentifier());
                        ids.add(pair.publicKeyUniqueIdentifier());
                        return !sameIdentifier(ids.get(0), ids.get(1));
                    }).orElse(false), start);
            if (!"passed".equals(created.status()))
                return CaseOutcome.notRun("The prerequisite Create Key Pair failed.", elapsedMillis(start));
            boolean publicKey = format == 11 || format == 21;
            var target = ids.get(publicKey ? 1 : 0);
            ClientBatchItem request;
            if ("fluent".equals(connection.apiLevel())) {
                try (var builder = client.request().get()) {
                    request = builder.uniqueIdentifier(target).keyFormatType(format).build();
                }
            } else {
                request = ClientBatchItem.get(target, (long) format, null, null, null);
            }
            return executeRequest(testCase, withBatchId(request), response -> response.getResponse().map(get -> {
                try (get; var object = get.object().map(TtlvStructure::view).orElse(null)) {
                    return get.objectTypeRaw().orElse(-1) == (publicKey ? 3 : 4) && object != null
                            && get.uniqueIdentifier().filter(id -> sameIdentifier(target, id)).isPresent()
                            && GenericExchange.keyBlockMatches(object, null, format);
                }
            }).orElse(false), start);
        } finally {
            for (var key : ids) executeRequest(testCase, withBatchId(ClientBatchItem.destroy(key)), ignored -> true, start);
        }
    }

    private CaseOutcome executeKeyBlock(CatalogLoader.CatalogCase testCase, long start) {
        boolean wrapped = "wrapped".equals(testCase.requires().path("key_block_shape").asText());
        var keys = new ArrayList<UniqueIdentifier>();
        UniqueIdentifier active = null;
        try {
            UniqueIdentifier target = createExtractableFixture(testCase, 12, keys, start);
            if (target == null) return CaseOutcome.notRun("The prerequisite Create request failed.", elapsedMillis(start));
            UniqueIdentifier wrappingKey = wrapped ? createExtractableFixture(testCase, 48, keys, start) : null;
            if (wrapped) {
                if (wrappingKey == null)
                    return CaseOutcome.notRun("The prerequisite Create request failed.", elapsedMillis(start));
                var activation = executeRequest(testCase, withBatchId(ClientBatchItem.activate(wrappingKey)),
                        item -> item.activateResponse().map(payload -> sameIdentifier(wrappingKey, payload.uniqueIdentifier()))
                                .orElse(false), start);
                if (!"passed".equals(activation.status()))
                    return CaseOutcome.notRun("The prerequisite Activate request failed.", elapsedMillis(start));
                active = wrappingKey;
            }
            byte[] wrappingIdentifier = wrapped ? wrappingKey.nativeText() : null;
            try (var specification = wrapped ? GenericExchange.keyWrappingFields(wrappingIdentifier) : null) {
                ClientBatchItem request;
                if ("fluent".equals(connection.apiLevel())) {
                    try (var builder = client.request().get()) {
                        builder.uniqueIdentifier(target);
                        if (specification == null) builder.keyFormatType(7);
                        else builder.keyWrappingSpecification(specification);
                        request = builder.build();
                    }
                } else {
                    request = ClientBatchItem.get(target, wrapped ? null : 7L, null, null, specification);
                }
                return executeRequest(testCase, withBatchId(request), observed -> observed.getResponse().map(get -> {
                    try (get; var objectView = get.object().map(TtlvStructure::view).orElse(null)) {
                        return get.objectTypeRaw().orElse(-1) == 2 && objectView != null
                                && get.uniqueIdentifier().filter(id -> sameIdentifier(target, id)).isPresent()
                                && GenericExchange.keyBlockMatches(objectView, wrappingIdentifier);
                    }
                }).orElse(false), start);
            }
        } finally {
            if (active != null) executeRequest(testCase, withBatchId(revokeRequest(active, false)), ignored -> true, start);
            for (var key : keys)
                executeRequest(testCase, withBatchId(ClientBatchItem.destroy(key)), ignored -> true, start);
        }
    }

    private UniqueIdentifier createExtractableFixture(CatalogLoader.CatalogCase testCase, int usageMask,
            List<UniqueIdentifier> keys, long start) {
        var id = new AtomicReference<UniqueIdentifier>();
        try (var attributes = symmetricKeyAttributes("key-block-" + UUID.randomUUID(), 256, usageMask)) {
            var result = executeRequest(testCase, withBatchId(ClientBatchItem.create(2L, attributes, null)),
                    response -> response.createResponse().map(payload -> {
                        keys.add(payload.uniqueIdentifier());
                        id.set(payload.uniqueIdentifier());
                        return payload.objectType() == 2;
                    }).orElse(false), start);
            return "passed".equals(result.status()) ? id.get() : null;
        }
    }

    private CaseOutcome executeRegisterOpaque(CatalogLoader.CatalogCase testCase, long start) {
        var registered = new AtomicReference<UniqueIdentifier>();
        CaseOutcome target;
        try (var type = GenericExchange.item(0x420059, TtlvValue.Enumeration(OPAQUE_DATA_TYPE));
                var data = GenericExchange.item(0x42005a, TtlvValue.ByteString(OPAQUE_FIXTURE, CODEC_LIMITS));
                var opaque = ManagedObject.opaqueObject(List.of(type, data))) {
            var request = buildRegisterOpaque(opaque);
            target = executeRequest(testCase, withBatchId(request), result ->
                    result.registerResponse().filter(payload -> {
                        try (payload) {
                            registered.set(payload.uniqueIdentifier());
                            return registered.get() != null;
                        }
                    }).isPresent(), start);
        } catch (RuntimeException exception) {
            target = exchangeFailure(exception, start);
        }
        var id = registered.get();
        if (id == null) return target;
        if ("passed".equals(target.status())) {
            target = executeRequest(testCase, withBatchId(ClientBatchItem.get(id, null, null, null, null)),
                    result -> verifyOpaqueReadback(result, id), start);
        }
        var cleanup = executeRequest(testCase, withBatchId(ClientBatchItem.destroy(id)), result ->
                result.destroyResponse().map(value -> sameIdentifier(value.uniqueIdentifier(), id)).orElse(false), start);
        return "passed".equals(cleanup.status()) ? target
                : CaseOutcome.failed("Cleanup Destroy failed for the registered Opaque Object.", elapsedMillis(start));
    }

    private ClientBatchItem buildRegisterOpaque(ManagedObject opaque) {
        if ("fluent".equals(connection.apiLevel())) {
            try (var builder = client.request().register()) {
                return builder.object(opaque).opaqueObjectContext(OpaqueObjectContext.GENERIC).build();
            }
        }
        try (var view = opaque.view(); var item = view.copy(); var attrs = TtlvStructure.create();
                var request = RegisterRequest.create((int) opaque.objectType(), attrs, item);
                var contextual = request.withOpaqueObjectContext(OpaqueObjectContext.GENERIC)) {
            return ClientBatchItem.register(contextual);
        }
    }

    private static boolean verifyOpaqueReadback(ClientBatchItemResult result, UniqueIdentifier id) {
        var response = result.getResponse().orElse(null);
        if (response == null) return false;
        try (response) {
            if (response.objectTypeRaw().orElse(-1) != 8
                    || response.uniqueIdentifier().filter(value -> sameIdentifier(id, value)).isEmpty()
                    || response.object().isEmpty()) return false;
            try (var object = response.object().orElseThrow().view()) {
                if (object.itemCount() != 1) return false;
                try (var item = object.itemAt(0); var wire = item.copy(); var opaque = ManagedObject.fromItem(8, wire)) {
                    return opaque.opaqueDataType() == OPAQUE_DATA_TYPE
                            && Arrays.equals(opaque.opaqueDataValue(), OPAQUE_FIXTURE);
                }
            }
        }
    }

    static boolean createSplitKeyCase(CatalogLoader.CatalogCase testCase) {
        return Set.of("test-op-create-split-key", "test-obj-split-key").contains(testCase.caseId());
    }

    private CaseOutcome executeCreateSplitKey(CatalogLoader.CatalogCase testCase, long start) {
        String name = testCase.isolationRecipe().uniqueNamePrefix() + "-" + UUID.randomUUID();
        boolean fluent = "fluent".equals(connection.apiLevel());
        var created = new ArrayList<UniqueIdentifier>();
        CaseOutcome target;
        try (var source = createTestKeyAttributes(name, isDraft())) {
            var request = splitKeyRequest(source, fluent);
            target = executeRequest(testCase, withBatchId(request), result -> {
                var identifiers = result.createSplitKeyUniqueIdentifiers().orElse(List.of());
                for (var id : identifiers) {
                    if (created.stream().noneMatch(previous -> sameIdentifier(previous, id))) {
                        created.add(id);
                    }
                }
                return identifiers.size() == 2 && created.size() == 2;
            }, start);
        } catch (RuntimeException exception) {
            target = exchangeFailure(exception, start);
        }

        if ("passed".equals(target.status()) && "test-obj-split-key".equals(testCase.caseId())) {
            var partNumbers = new HashSet<Integer>();
            for (var id : created) {
                var read = executeRequest(testCase, withBatchId(ClientBatchItem.get(id, null, null, null, null)),
                        response -> readSplitKey(response, id, partNumbers), start);
                if (!"passed".equals(read.status())) {
                    target = read;
                    break;
                }
            }
        }
        boolean cleaned = true;
        for (var id : created) {
            var cleanup = executeRequest(testCase, withBatchId(ClientBatchItem.destroy(id)), result ->
                    result.destroyResponse().map(value -> sameIdentifier(value.uniqueIdentifier(), id)).orElse(false), start);
            cleaned &= "passed".equals(cleanup.status());
        }
        return cleaned ? target : CaseOutcome.failed("Cleanup Destroy failed for a split key part.", elapsedMillis(start));
    }

    private ClientBatchItem splitKeyRequest(TtlvStructure source, boolean fluent) {
        long method = KmipValues.SplitKeyMethod.XOR;
        if (!fluent) return ClientBatchItem.createSplitKey(2, null, 2, 2, method, null, source, null);
        try (var attributes = AttributeSet.tryNew(source); var builder = client.request().createSplitKey()) {
            return builder.objectType(2).parts(2).threshold(2).method(method).attributes(attributes).build();
        }
    }

    private static boolean readSplitKey(ClientBatchItemResult result, UniqueIdentifier id, Set<Integer> partNumbers) {
        var response = result.getResponse().orElse(null);
        if (response == null) return false;
        try (response) {
            if (response.objectTypeRaw().orElse(-1) != 5
                    || response.uniqueIdentifier().filter(value -> sameIdentifier(value, id)).isEmpty()
                    || response.object().isEmpty()) return false;
            try (var object = response.object().orElseThrow().view()) {
                if (object.itemCount() != 1) return false;
                try (var item = object.itemAt(0); var wire = item.copy(); var part = ManagedObject.fromItem(5, wire)) {
                    int index = part.keyPartIdentifier();
                    return part.splitKeyParts() == 2 && part.splitKeyThreshold() == 2
                            && part.splitKeyMethod() == KmipValues.SplitKeyMethod.XOR
                            && index >= 1 && index <= 2 && partNumbers.add(index);
                }
            }
        }
    }

    private TtlvStructure createOwnedKeyAttributes(CatalogLoader.CatalogCase testCase, String name) {
        TtlvStructure attributes = createTestKeyAttributes(name, isDraft());
        if (!"test-op-check".equals(testCase.caseId())) return attributes;
        return appendOwnedValue(attributes, 0x0042_002c, TtlvValue.Integer(4));
    }

    private CaseOutcome executeCreateWithCleanup(CatalogLoader.CatalogCase testCase, long start) {
        String objectName = testCase.isolationRecipe().uniqueNamePrefix() + "-" + UUID.randomUUID();
        UniqueIdentifier createdIdentifier = null;
        long createStatus = -1;
        long createReason = -1;
        long createObjectType = -1;
        try (TtlvStructure attributes = createOwnedKeyAttributes(testCase, objectName);
                ClientBatchItem request = withBatchId(buildCreate(attributes, !ownedObjectCase(testCase)));
                ClientBatch batch = ClientBatch.create()) {
            long requestOperation = request.operation();
            batch.add(request);
            if (connection.usernamePassword() != null) {
                batch.setAuthentication(usernamePasswordAuthentication(connection.usernamePassword()));
            }
            try (ClientBatchResponse response = client.execute(batch, CODEC_LIMITS)) {
                if (response.itemCount() != 1) {
                    return CaseOutcome.failed("Create response batch item count did not match the request.",
                            elapsedMillis(start));
                }
                try (ClientBatchItemResult result = response.itemAt(0)) {
                    if (result.operation() != requestOperation) {
                        return CaseOutcome.failed("Create response operation did not match the request.",
                                elapsedMillis(start));
                    }
                    createStatus = result.resultStatus();
                    createReason = result.resultReason().orElse(-1);
                    if (createStatus != 0) {
                        if (ownedObjectCase(testCase)) {
                            return CaseOutcome.notRun("The server rejected the isolated key setup.",
                                    elapsedMillis(start));
                        }
                        return nonSuccessOutcome(testCase, createStatus, createReason, start);
                    }
                    var payload = result.createResponse();
                    if (payload.isEmpty()) {
                        return CaseOutcome.failed("Create success response omitted its typed payload.",
                                elapsedMillis(start), createStatus, optionalCode(createReason));
                    }
                    var created = payload.orElseThrow();
                    createObjectType = created.objectType();
                    createdIdentifier = created.uniqueIdentifier();
                }
            }
        } catch (Exception exception) {
            return exchangeFailure(exception, start);
        }

        if (createdIdentifier == null) {
            return CaseOutcome.failed("Create success response omitted the object identifier; cleanup could not run.",
                    elapsedMillis(start), createStatus, optionalCode(createReason));
        }

        if ("test-op-destroy".equals(testCase.caseId()) && createObjectType == 2) {
            return destroyTarget(testCase, createdIdentifier, start);
        }

        CaseOutcome target = null;
        if (ownedObjectCase(testCase) && createObjectType == 2) {
            try {
                target = switch (testCase.caseId()) {
                    case "test-op-get" -> getTarget(testCase, createdIdentifier, start);
                    case "test-op-activate", "test-op-revoke", "test-op-check" ->
                            activateAndRevoke(testCase, createdIdentifier, start);
                    case "test-op-obtain-lease" -> obtainLeaseTarget(testCase, createdIdentifier, start);
                    case "test-op-archive", "test-op-recover" -> archiveAndRecover(testCase, createdIdentifier, start);
                    default -> readAttributes(testCase, createdIdentifier, objectName, start);
                };
            } catch (RuntimeException exception) {
                target = exchangeFailure(exception, start);
            }
        }

        try (ClientBatchItem request = withBatchId(ClientBatchItem.destroy(createdIdentifier));
                ClientBatch batch = ClientBatch.create()) {
            long requestOperation = request.operation();
            batch.add(request);
            if (connection.usernamePassword() != null) {
                batch.setAuthentication(usernamePasswordAuthentication(connection.usernamePassword()));
            }
            try (ClientBatchResponse response = client.execute(batch, CODEC_LIMITS)) {
                if (response.itemCount() != 1) {
                    return CaseOutcome.failed("Create succeeded, but cleanup response count was invalid.",
                            elapsedMillis(start), createStatus, optionalCode(createReason));
                }
                try (ClientBatchItemResult result = response.itemAt(0)) {
                    if (result.operation() != requestOperation || result.resultStatus() != 0) {
                        return CaseOutcome.failed("Create succeeded, but cleanup Destroy did not succeed.",
                                elapsedMillis(start), optionalCode(result.resultStatus()),
                                optionalCode(result.resultReason().orElse(-1)));
                    }
                    var payload = result.destroyResponse();
                    if (payload.isEmpty()
                            || !sameIdentifier(createdIdentifier, payload.orElseThrow().uniqueIdentifier())) {
                        return CaseOutcome.failed("Create succeeded, but Destroy did not confirm the same object.",
                                elapsedMillis(start), createStatus, optionalCode(createReason));
                    }
                }
            }
        } catch (Exception exception) {
            return CaseOutcome.failed("Create succeeded, but cleanup exchange failed ("
                    + exception.getClass().getSimpleName() + "); remote diagnostic text was suppressed.",
                    elapsedMillis(start), createStatus, optionalCode(createReason));
        }

        if (createObjectType != 2) {
            return CaseOutcome.failed("Create returned an object type different from Symmetric Key.",
                    elapsedMillis(start), createStatus, optionalCode(createReason));
        }
        return target == null ? CaseOutcome.passed(elapsedMillis(start), createStatus, optionalCode(createReason)) : target;
    }

    private CaseOutcome activateAndRevoke(CatalogLoader.CatalogCase testCase, UniqueIdentifier id, long start) {
        boolean activateTarget = "test-op-activate".equals(testCase.caseId());
        boolean revokeTarget = "test-op-revoke".equals(testCase.caseId());
        boolean fluent = "fluent".equals(connection.apiLevel());
        ClientBatchItem request = activateTarget && fluent
                ? client.request().activate(id).build() : ClientBatchItem.activate(id);
        var activated = executeRequest(testCase, withBatchId(request), result -> result.activateResponse()
                .map(payload -> sameIdentifier(id, payload.uniqueIdentifier())).orElse(false), start);
        if (activated.resultStatusCode() != null && activated.resultStatusCode() != 0) {
            return activateTarget ? activated
                    : CaseOutcome.notRun("The server rejected isolated Activate setup.", elapsedMillis(start));
        }
        CaseOutcome target = "passed".equals(activated.status()) && "test-op-check".equals(testCase.caseId())
                ? checkTarget(testCase, id, start) : activated;
        var revoked = executeRequest(testCase, withBatchId(revokeRequest(id, fluent && revokeTarget)), result -> {
            var response = result.revokeResponse().orElse(null);
            if (response == null) return false;
            try (response) {
                return sameIdentifier(id, response.uniqueIdentifier());
            }
        }, start);
        if (!"passed".equals(activated.status())) return activated;
        if (revokeTarget) return revoked;
        return "passed".equals(revoked.status()) ? target
                : CaseOutcome.failed("Cleanup Revoke failed for the activated key.", elapsedMillis(start));
    }

    private CaseOutcome checkTarget(CatalogLoader.CatalogCase testCase, UniqueIdentifier id, long start) {
        int usageMask = 4;
        return executeRequest(testCase, withBatchId(ClientBatchItem.check(id, null, usageMask, null)), result ->
                result.checkResponse().filter(payload -> {
                    try (payload) {
                        return payload.uniqueIdentifier().filter(value -> sameIdentifier(id, value)).isPresent()
                                && (payload.cryptographicUsageMask().orElse(usageMask) & usageMask) == usageMask;
                    }
                }).isPresent(), start);
    }

    private CaseOutcome obtainLeaseTarget(CatalogLoader.CatalogCase testCase, UniqueIdentifier id, long start) {
        return executeRequest(testCase, withBatchId(ClientBatchItem.obtainLease(id)), result ->
                result.obtainLeaseResponse().filter(payload -> {
                    try (payload) {
                        return sameIdentifier(id, payload.uniqueIdentifier());
                    }
                }).isPresent(), start);
    }

    private ClientBatchItem revokeRequest(UniqueIdentifier id, boolean fluent) {
        long reasonCode = 6;
        if (fluent) return client.request().revoke(reasonCode).uniqueIdentifier(id).build();
        try (var reason = RevocationReason.create(reasonCode, null);
                var request = RevokeRequest.create(id, reason, null)) {
            return ClientBatchItem.revoke(request);
        }
    }

    private CaseOutcome archiveAndRecover(CatalogLoader.CatalogCase testCase, UniqueIdentifier id, long start) {
        boolean archiveTarget = "test-op-archive".equals(testCase.caseId());
        boolean fluent = "fluent".equals(connection.apiLevel());
        ClientBatchItem archive = archiveTarget && fluent
                ? client.request().archive(id).build() : ClientBatchItem.archive(id);
        CaseOutcome archived = executeRequest(testCase, withBatchId(archive), result -> result.archiveResponse()
                .map(payload -> sameIdentifier(id, payload.uniqueIdentifier())).orElse(false), start);
        if (archived.resultStatusCode() == null || archived.resultStatusCode() != 0) {
            return archiveTarget || "failed".equals(archived.status()) ? archived
                    : CaseOutcome.notRun("The server rejected Archive during Recover setup.", elapsedMillis(start));
        }
        ClientBatchItem recover = !archiveTarget && fluent
                ? client.request().recover(id).build() : ClientBatchItem.recover(id);
        CaseOutcome recovered = executeRequest(testCase, withBatchId(recover), result -> result.recoverResponse()
                .map(payload -> sameIdentifier(id, payload.uniqueIdentifier())).orElse(false), start);
        if (!"passed".equals(archived.status())) return archived;
        if (!archiveTarget) return recovered;
        return "passed".equals(recovered.status()) ? archived
                : CaseOutcome.failed("Recover did not confirm that the archived key was restored.", elapsedMillis(start));
    }

    private CaseOutcome getTarget(CatalogLoader.CatalogCase testCase, UniqueIdentifier id, long start) {
        ClientBatchItem request = "fluent".equals(connection.apiLevel())
                ? client.request().get(id).build() : ClientBatchItem.get(id, null, null, null, null);
        return executeRequest(testCase, withBatchId(request), result -> {
            var payload = result.getResponse().orElse(null);
            if (payload == null) return false;
            try (payload) {
                return payload.objectTypeRaw().orElse(-1) == 2 && payload.object().isPresent()
                        && payload.uniqueIdentifier().filter(value -> sameIdentifier(value, id)).isPresent();
            }
        }, start);
    }

    private CaseOutcome destroyTarget(CatalogLoader.CatalogCase testCase, UniqueIdentifier id, long start) {
        try {
            ClientBatchItem request = "fluent".equals(connection.apiLevel())
                    ? client.request().destroy(id).build() : ClientBatchItem.destroy(id);
            return executeRequest(testCase, withBatchId(request), result -> result.destroyResponse()
                    .map(payload -> sameIdentifier(id, payload.uniqueIdentifier())).orElse(false), start);
        } catch (RuntimeException exception) {
            return exchangeFailure(exception, start);
        }
    }

    private ClientBatchItem buildCreate(TtlvStructure source, boolean target) {
        if (!target || !"fluent".equals(connection.apiLevel())) {
            return ClientBatchItem.create(2, source, null);
        }
        try (var attributes = AttributeSet.tryNew(source); var builder = client.request().create()) {
            return builder.objectType(2).attributes(attributes).build();
        }
    }

    static boolean attributeRead(CatalogLoader.CatalogCase testCase) {
        return testCase.caseId().startsWith("test-attr-") || switch (testCase.caseId()) {
            case "test-op-get-attributes", "test-op-get-attribute-list" -> true;
            default -> false;
        };
    }

    private static int catalogAttributeTag(String name) {
        return switch (name) {
            case "Activation Date" -> 0x0042_0001;
            case "Alternative Name" -> 0x0042_00bf;
            case "Always Sensitive" -> 0x0042_0121;
            case "Application Specific Information" -> 0x0042_0004;
            case "Archive Date" -> 0x0042_0005;
            case "Certificate Length" -> 0x0042_00ad;
            case "Certificate Type" -> 0x0042_001d;
            case "Comment" -> 0x0042_00fd;
            case "Compromise Date" -> 0x0042_0020;
            case "Compromise Occurrence Date" -> 0x0042_0021;
            case "Contact Information" -> 0x0042_0022;
            case "Cryptographic Algorithm" -> 0x0042_0028;
            case "Cryptographic Domain Parameters" -> 0x0042_0029;
            case "Cryptographic Length" -> 0x0042_002a;
            case "Cryptographic Parameters" -> 0x0042_002b;
            case "Cryptographic Usage Mask" -> 0x0042_002c;
            case "Deactivation Date" -> 0x0042_002f;
            case "Description" -> 0x0042_00fc;
            case "Destroy Date" -> 0x0042_0033;
            case "Digest" -> 0x0042_0034;
            case "Digital Signature Algorithm" -> 0x0042_00ae;
            case "Extractable" -> 0x0042_0122;
            case "Fresh" -> 0x0042_00a8;
            case "Initial Date" -> 0x0042_0039;
            case "Key Format Type" -> 0x0042_0042;
            case "Key Value Location" -> 0x0042_00b8;
            case "Key Value Present" -> 0x0042_00bb;
            case "Last Change Date" -> 0x0042_0048;
            case "Lease Time" -> 0x0042_0049;
            case "Link" -> 0x0042_004a;
            case "Name" -> 0x0042_0053;
            case "Never Extractable" -> 0x0042_0123;
            case "NIST Key Type" -> 0x0042_013a;
            case "Object Group" -> 0x0042_0056;
            case "Object Type" -> 0x0042_0057;
            case "Opaque Data Type" -> 0x0042_0059;
            case "Original Creation Date" -> 0x0042_00bc;
            case "PKCS#12 Friendly Name" -> 0x0042_00fb;
            case "Process Start Date" -> 0x0042_0067;
            case "Protect Stop Date" -> 0x0042_0068;
            case "Protection Level" -> 0x0042_0145;
            case "Protection Period" -> 0x0042_0146;
            case "Protection Storage Mask" -> 0x0042_015e;
            case "Quantum Safe" -> 0x0042_0147;
            case "Random Number Generator" -> 0x0042_00de;
            case "Revocation Reason" -> 0x0042_0081;
            case "Rotate Automatic" -> 0x0042_016b;
            case "Rotate Date" -> 0x0042_016d;
            case "Rotate Generation" -> 0x0042_016e;
            case "Rotate Interval" -> 0x0042_016a;
            case "Rotate Latest" -> 0x0042_0172;
            case "Rotate Name" -> 0x0042_016f;
            case "Rotate Offset" -> 0x0042_016c;
            case "Sensitive" -> 0x0042_0120;
            case "Short Unique Identifier" -> 0x0042_0136;
            case "State" -> 0x0042_008d;
            case "Unique Identifier" -> 0x0042_0094;
            case "Usage Limits" -> 0x0042_0095;
            case "X.509 Certificate Identifier" -> 0x0042_00b5;
            case "X.509 Certificate Issuer" -> 0x0042_00b6;
            case "X.509 Certificate Subject" -> 0x0042_00b7;
            default -> 0;
        };
    }

    private CaseOutcome readAttributes(CatalogLoader.CatalogCase testCase, UniqueIdentifier id,
                                       String objectName, long start) {
        boolean fluent = "fluent".equals(connection.apiLevel());
        if ("Get Attribute List".equals(testCase.operation())) {
            var request = fluent ? client.request().getAttributeList(id).build() : ClientBatchItem.getAttributeList(id);
            return executeRequest(testCase, request, result -> {
                var payload = result.getAttributeListResponse().orElse(null);
                if (payload == null) return false;
                try (payload) {
                    return sameIdentifier(id, payload.uniqueIdentifier())
                            && payload.attributeReferences().contains(AttributeReference.tag(0x0042_0053));
                }
            }, start);
        }

        String attrName = testCase.requires().path("attribute").asText("");
        List<AttributeReference> references;
        int strictTag;
        if ("Vendor Attribute".equals(attrName)) {
            references = List.of(AttributeReference.name("KMIPKit.Interop", "probe"));
            strictTag = 0;
        } else if ("Certificate Attributes".equals(attrName)) {
            references = List.of(AttributeReference.tag(0x0042_00b5), AttributeReference.tag(0x0042_00b6));
            strictTag = 0;
        } else {
            int lookedUp = switch (testCase.caseId()) {
                case "test-attr-cryptographic-algorithm" -> 0x0042_0028;
                case "test-attr-cryptographic-length" -> 0x0042_002a;
                case "test-op-get-attributes", "test-attr-name" -> 0x0042_0053;
                default -> catalogAttributeTag(attrName);
            };
            if (lookedUp == 0) {
                return CaseOutcome.notRun("Attribute tag is not mapped for " + testCase.caseId(), elapsedMillis(start));
            }
            references = List.of(AttributeReference.tag(lookedUp));
            strictTag = (lookedUp == 0x0042_0028 || lookedUp == 0x0042_002a || lookedUp == 0x0042_0053) ? lookedUp : 0;
        }
        var request = fluent ? client.request().getAttributes(id, references).build()
                : ClientBatchItem.getAttributes(id, references);
        return executeRequest(testCase, request, result -> {
            var payload = result.getAttributesResponse().orElse(null);
            if (payload == null) return false;
            try (payload) {
                if (!id.text().equals(payload.uniqueIdentifier())) return false;
                return strictTag == 0 || payload.attributes()
                        .map(attributes -> containsAttribute(attributes, strictTag, objectName)).orElse(false);
            }
        }, start);
    }

    private static boolean containsAttribute(TtlvStructure attributes, int rawTag, String objectName) {
        try (var view = attributes.view()) {
            for (long index = 0; index < view.itemCount(); index++) {
                try (var item = view.itemAt(index); var tag = item.tag()) {
                    if (tag.raw() != rawTag) continue;
                    try (var value = item.value()) {
                        return switch (rawTag) {
                            case 0x0042_0028 -> value.enumeration() == 3;
                            case 0x0042_002a -> value.integer() == 256;
                            default -> containsName(value, objectName);
                        };
                    }
                }
            }
        }
        return false;
    }

    private static boolean containsName(org.kmipkit.ttlv.TtlvValueView value, String objectName) {
        if (value.itemType() != org.kmipkit.ttlv.TtlvItemType.Structure) return false;
        try (var name = value.structure()) {
            for (long index = 0; index < name.itemCount(); index++) {
                try (var field = name.itemAt(index); var tag = field.tag()) {
                    if (tag.raw() != 0x0042_0055) continue;
                    try (var text = field.value()) {
                        return java.util.Arrays.equals(text.bytes(),
                                objectName.getBytes(java.nio.charset.StandardCharsets.UTF_8));
                    }
                }
            }
        }
        return false;
    }

    private boolean isDraft() {
        return "3.0".equals(connection.protocolVersion());
    }

    private ClientBatchItem withBatchId(ClientBatchItem request) {
        return isDraft() ? request : request.withUniqueBatchItemId(
                "item-0".getBytes(java.nio.charset.StandardCharsets.US_ASCII));
    }

    private static TtlvStructure createTestKeyAttributes(String objectName, boolean draft) {
        if (draft) {
            var attributes = appendOwnedValue(TtlvStructure.create(), 0x0042_00fc,
                    TtlvValue.TextString(objectName.getBytes(java.nio.charset.StandardCharsets.UTF_8), CODEC_LIMITS));
            attributes = appendOwnedValue(attributes, 0x0042_0028, TtlvValue.Enumeration(3));
            return appendOwnedValue(attributes, 0x0042_002a, TtlvValue.Integer(256));
        }
        TtlvStructure name = TtlvStructure.create();
        TtlvStructure attributes = null;
        try {
            name = appendOwnedValue(name, 0x0042_0055,
                    TtlvValue.TextString(objectName.getBytes(java.nio.charset.StandardCharsets.UTF_8), CODEC_LIMITS));
            name = appendOwnedValue(name, 0x0042_0054, TtlvValue.Enumeration(1));

            attributes = TtlvStructure.create();
            attributes = appendOwnedValue(attributes, 0x0042_0053, TtlvValue.structure(name, CODEC_LIMITS));
            name.close();
            name = null;
            attributes = appendOwnedValue(attributes, 0x0042_0028, TtlvValue.Enumeration(3));
            attributes = appendOwnedValue(attributes, 0x0042_002a, TtlvValue.Integer(256));
            TtlvStructure result = attributes;
            attributes = null;
            return result;
        } finally {
            if (name != null) name.close();
            if (attributes != null) attributes.close();
        }
    }

    private static TtlvStructure appendOwnedValue(TtlvStructure previous, int rawTag, TtlvValue value) {
        try {
            return appendValue(previous, rawTag, value);
        } finally {
            previous.close();
        }
    }

    private static boolean sameIdentifier(UniqueIdentifier left, UniqueIdentifier right) {
        if (left.kind() != right.kind()) return false;
        return left.kind() == UniqueIdentifier.Kind.TEXT_STRING || left.kind() == UniqueIdentifier.Kind.IDENTIFIER
                ? left.text().equals(right.text()) : left.number() == right.number();
    }

    private static CaseOutcome nonSuccessOutcome(
            CatalogLoader.CatalogCase testCase, long status, long reason, long start) {
        if (reason == KmipValues.ResultReason.OPERATION_NOT_SUPPORTED
                || reason == KmipValues.ResultReason.FEATURE_NOT_SUPPORTED) {
            return CaseOutcome.unsupported(elapsedMillis(start), status, reason);
        }
        if (reason == KmipValues.ResultReason.PERMISSION_DENIED
                || reason == KmipValues.ResultReason.AUTHENTICATION_NOT_SUCCESSFUL) {
            return CaseOutcome.policyRestricted(elapsedMillis(start), status, reason);
        }
        if (expectedReason(testCase, reason)) {
            return CaseOutcome.passed(elapsedMillis(start), status, reason);
        }
        return CaseOutcome.failed("The KMIP server returned a non-success Create result.",
                elapsedMillis(start), status, optionalCode(reason));
    }

    private static CaseOutcome exchangeFailure(Exception exception, long start) {
        return CaseOutcome.failed("KMIP client exchange failed (" + exception.getClass().getSimpleName()
                + "); remote diagnostic text was suppressed.", elapsedMillis(start));
    }

    private boolean verifyTypedResponse(String operation, ClientBatchItemResult result) {
        return switch (operation) {
            case "Ping" -> result.pingResponse().orElse(false);
            case "Query" -> {
                var payload = result.typedQueryResponse();
                if (payload.isEmpty()) yield false;
                try (var query = payload.orElseThrow()) {
                    yield JavaResponseChecks.queryAdvertisesOperations(query.operations());
                }
            }
            case "Log" -> {
                var payload = result.logResponse();
                if (payload.isEmpty()) yield false;
                payload.orElseThrow().close();
                yield true;
            }
            case "Set Endpoint Role" -> result.setEndpointRoleResponse()
                    .map(payload -> payload.endpointRole() == 1).orElse(false);
            case "Create" -> false;
            case "Get Constraints" -> {
                var payload = result.getConstraintsResponse();
                if (payload.isEmpty()) yield false;
                payload.orElseThrow().close();
                yield true;
            }
            case "Get Attribute List" -> {
                var payload = result.getAttributeListResponse();
                if (payload.isEmpty()) yield false;
                payload.orElseThrow().close();
                yield true;
            }
            case "Discover Versions" -> JavaResponseChecks.advertisesRequestedVersion(
                    result.supportedVersions().orElse(List.of()), toProtocolVersion(connection.protocolVersion()));
            default -> false;
        };
    }

    private static TransportConfiguration buildTransport(ConnectionConfig connection) {
        var tls = connection.tls();
        try (TransportConfiguration.Builder builder = TransportConfiguration.builder()) {
            if ("raw_tls".equals(connection.transport())) {
                builder.rawTls(connection.host(), connection.port());
            } else {
                builder.https(connection.httpsUri().toString());
            }
            builder.clientIdentityFiles(List.of(Path.of(tls.clientCertificatePath())),
                    Path.of(tls.clientKeyPath()), TransportConfiguration.Encoding.PEM);
            builder.trustCertificateFiles(List.of(Path.of(tls.caCertificatePath())),
                    TransportConfiguration.Encoding.PEM);
            String serverName = tls.serverNameOverride();
            if (serverName.isBlank()) {
                serverName = "raw_tls".equals(connection.transport())
                        ? connection.host() : connection.httpsUri().getHost();
            }
            builder.tlsServerName(serverName);
            Duration connect = Duration.ofSeconds(tls.connectTimeoutSeconds());
            Duration read = Duration.ofSeconds(tls.readTimeoutSeconds());
            builder.timeouts(TransportConfiguration.Timeout.bounded(connect),
                    TransportConfiguration.Timeout.bounded(read),
                    TransportConfiguration.Timeout.bounded(read),
                    TransportConfiguration.Timeout.bounded(read));
            return builder.build();
        }
    }

    private static Authentication usernamePasswordAuthentication(ConnectionConfig.UsernamePassword names) {
        String username = System.getenv(names.usernameEnvironmentName());
        String password = System.getenv(names.passwordEnvironmentName());
        if (username == null || username.isBlank() || password == null || password.isBlank()) {
            throw new IllegalArgumentException("Configured credential environment variables are unavailable");
        }
        // KMIPKit owns and zeroizes its native copies. The JVM's environment Strings cannot be erased.
        TtlvStructure valueStructure = TtlvStructure.create();
        valueStructure = appendSecret(valueStructure, 0x0042_0099, username);
        valueStructure = appendSecret(valueStructure, 0x0042_00a1, password);
        TtlvStructure credentialStructure = TtlvStructure.create();
        credentialStructure = appendValue(credentialStructure, 0x0042_0024,
                TtlvValue.Enumeration(1));
        credentialStructure = appendValue(credentialStructure, 0x0042_0025,
                TtlvValue.structure(valueStructure, CODEC_LIMITS));
        Credential credential = Credential.fromTtlv(credentialStructure);
        return Authentication.create(List.of(credential));
    }

    private CaseOutcome executeAsynchronous(CatalogLoader.CatalogCase testCase, boolean cancel, long start) {
        if (connection.usernamePassword() != null) {
            return CaseOutcome.notRun("The asynchronous client API cannot attach Request Header Authentication.",
                    elapsedMillis(start));
        }
        byte[] token = null;
        UniqueIdentifier owned = null;
        CaseOutcome target;
        try (var nativeClient = configuredClient(connection, Client::create);
                var attrs = symmetricKeyAttributes("async-" + UUID.randomUUID(), 128, 4);
                var request = ClientBatchItem.create(2, attrs, null);
                var batch = ClientBatch.create()) {
            batch.asynchronousIndicator(1);
            batch.add(request);
            try (var response = nativeClient.execute(batch, CODEC_LIMITS)) {
                if (response.itemCount() != 1) {
                    return CaseOutcome.failed("The asynchronous Create response count was invalid.", elapsedMillis(start));
                }
                try (var item = response.itemAt(0)) {
                    if (item.resultStatus() == 0) {
                        owned = item.createResponse().orElseThrow().uniqueIdentifier();
                    } else if (item.resultStatus() == 2) {
                        token = item.pendingCorrelationValue().orElse(null);
                    }
                }
            }
            if (token == null || token.length == 0) {
                target = CaseOutcome.notRun("The prerequisite Create returned no Pending correlation.", elapsedMillis(start));
            } else if (cancel) {
                try (var response = nativeClient.executeCancel(token, CODEC_LIMITS)) {
                    var metadata = response.metadataV1();
                    if (metadata.resultStatus() != 0) {
                        target = nonSuccessOutcome(testCase, metadata.resultStatus(), metadata.resultReason().orElse(-1), start);
                    } else {
                        long value = response.cancelResponse().orElseThrow().cancellationResult().orElse(-1);
                        target = value >= 1 && value <= 5 ? CaseOutcome.passed(elapsedMillis(start), 0L, null)
                                : CaseOutcome.failed("The Cancel response omitted a valid Cancellation Result.", elapsedMillis(start));
                    }
                }
            } else {
                target = null;
                for (int poll = 0; poll < 16; poll++) {
                    try (var response = nativeClient.executePoll(token, CODEC_LIMITS)) {
                        var typed = response.pollResponse().orElseThrow();
                        var metadata = response.metadataV1();
                        if (typed.isPending()) {
                            var updated = new AtomicReference<byte[]>();
                            typed.withAsynchronousCorrelationValue(value -> {
                                byte[] copy = new byte[value.remaining()];
                                value.get(copy);
                                updated.set(copy);
                            });
                            Arrays.fill(token, (byte) 0);
                            token = updated.get();
                            if (token == null || token.length == 0) {
                                target = CaseOutcome.failed("The Pending Poll response omitted its correlation.", elapsedMillis(start));
                                break;
                            }
                            continue;
                        }
                        if (metadata.resultStatus() != 0) {
                            target = nonSuccessOutcome(testCase, metadata.resultStatus(), metadata.resultReason().orElse(-1), start);
                            break;
                        }
                        try (var payload = response.responsePayloadTtlv().orElseThrow();
                                var view = payload.view()) {
                            byte[] identifier = GenericExchange.asynchronousCreateIdentifier(view);
                            if (identifier != null) {
                                owned = UniqueIdentifier.textString(new String(identifier, StandardCharsets.UTF_8));
                            }
                        }
                        target = owned == null
                                ? CaseOutcome.failed("Completed Poll omitted the owned Create identifier.", elapsedMillis(start))
                                : CaseOutcome.passed(elapsedMillis(start), 0L, null);
                        break;
                    }
                }
                if (target == null) {
                    try (var cleanup = nativeClient.executeCancel(token, CODEC_LIMITS)) {
                        long cancelled = cleanup.metadataV1().cancellationResult().orElse(-1);
                        target = cancelled == 1
                                ? CaseOutcome.notRun("The server remained Pending after the bounded Poll sequence.", elapsedMillis(start))
                                : CaseOutcome.failed("Could not cancel the fixture after the bounded Poll sequence.", elapsedMillis(start));
                    }
                }
            }
        } catch (RuntimeException error) {
            target = exchangeFailure(error, start);
        } finally {
            if (token != null) Arrays.fill(token, (byte) 0);
        }
        if (owned != null) {
            var cleanup = destroyTarget(testCase, owned, start);
            if (!"passed".equals(cleanup.status())) {
                return CaseOutcome.failed("Could not destroy the asynchronous Create fixture.", elapsedMillis(start));
            }
        }
        return target;
    }

    private CaseOutcome executeExtended(CatalogLoader.CatalogCase testCase, long start) {
        String op = testCase.operation();
        boolean fluent = "fluent".equals(connection.apiLevel());
        var cleanup = new ArrayList<UniqueIdentifier>();
        try {
            if ("Poll".equals(op) || "Cancel".equals(op)) {
                return executeAsynchronous(testCase, "Cancel".equals(op), start);
            }
            if ("Register".equals(op)) {
                String objectType = testCase.requires().path("object_type").asText("");
                if ("PGP Key".equals(objectType)) {
                    try (var attrs = symmetricKeyAttributes("register-pgp-" + UUID.randomUUID(), 256, 12);
                            var fmt = TtlvValue.Enumeration(1);
                            var mat = TtlvValue.ByteString(new byte[32], CODEC_LIMITS);
                            var kv = appendValue(TtlvStructure.create(), 0x420043, mat);
                            var kb0 = appendValue(TtlvStructure.create(), 0x420042, fmt);
                            var kb1 = appendValue(kb0, 0x420045, TtlvValue.structure(kv, CODEC_LIMITS));
                            var kb2 = appendValue(kb1, 0x420028, TtlvValue.Enumeration(3));
                            var kb = appendValue(kb2, 0x42002a, TtlvValue.Integer(256));
                            var verItem = GenericExchange.item(0x4200be, TtlvValue.Integer(4));
                            var kbItem = GenericExchange.item(0x420040, TtlvValue.structure(kb, CODEC_LIMITS));
                            var managed = ManagedObject.pgpKey(List.of(verItem, kbItem))) {
                        ClientBatchItem req;
                        if (fluent) {
                            try (var attrSet = AttributeSet.tryNew(attrs);
                                    var b = client.request().register()) {
                                req = withBatchId(b.object(managed).attributes(attrSet).build());
                            }
                        } else {
                            try (var view = managed.view();
                                    var pgpItem = view.copy();
                                    var registerReq = RegisterRequest.create(9, attrs, pgpItem)) {
                                req = withBatchId(ClientBatchItem.register(registerReq));
                            }
                        }
                        return executeRequest(testCase, req, r -> r.registerResponse().map(p -> {
                            cleanup.add(p.uniqueIdentifier());
                            return true;
                        }).orElse(false), start);
                    }
                }
                try (var attrs = symmetricKeyAttributes("register-" + UUID.randomUUID(), 256, 12);
                        var fmt = TtlvValue.Enumeration(1);
                        var mat = TtlvValue.ByteString(new byte[32], CODEC_LIMITS);
                        var kv = appendValue(TtlvStructure.create(), 0x420043, mat);
                        var kb0 = appendValue(TtlvStructure.create(), 0x420042, fmt);
                        var kb1 = appendValue(kb0, 0x420045, TtlvValue.structure(kv, CODEC_LIMITS));
                        var kb2 = appendValue(kb1, 0x420028, TtlvValue.Enumeration(3));
                        var kb = appendValue(kb2, 0x42002a, TtlvValue.Integer(256));
                        var skItem = GenericExchange.item(0x420040, TtlvValue.structure(kb, CODEC_LIMITS));
                        var managed = ManagedObject.symmetricKey(List.of(skItem))) {
                    ClientBatchItem req;
                    if (fluent) {
                        try (var attrSet = AttributeSet.tryNew(attrs);
                                var b = client.request().register()) {
                            req = withBatchId(b.object(managed).attributes(attrSet).build());
                        }
                    } else {
                        try (var view = managed.view();
                                var symItem = view.copy();
                                var registerReq = RegisterRequest.create(2, attrs, symItem)) {
                            req = withBatchId(ClientBatchItem.register(registerReq));
                        }
                    }
                    return executeRequest(testCase, req, r -> r.registerResponse().map(p -> {
                        cleanup.add(p.uniqueIdentifier());
                        return true;
                    }).orElse(false), start);
                }
            }
            if ("Import".equals(op)) {
                var uid = UniqueIdentifier.textString("kmipkit-java-import-" + UUID.randomUUID());
                try (var attrs = symmetricKeyAttributes("import-" + UUID.randomUUID(), 128, 4);
                        var fmt = TtlvValue.Enumeration(1);
                        var mat = TtlvValue.ByteString(new byte[16], CODEC_LIMITS);
                        var kv = appendValue(TtlvStructure.create(), 0x420043, mat);
                        var kb0 = appendValue(TtlvStructure.create(), 0x420042, fmt);
                        var kb = appendValue(kb0, 0x420045, TtlvValue.structure(kv, CODEC_LIMITS));
                        var sym = appendValue(TtlvStructure.create(), 0x420040, TtlvValue.structure(kb, CODEC_LIMITS));
                        var sk = appendValue(TtlvStructure.create(), 0x42008f, TtlvValue.structure(sym, CODEC_LIMITS))) {
                    var req = withBatchId(ClientBatchItem.importObject(uid, 2L, false, null, attrs, sk));
                    return executeRequest(testCase, req, r -> r.importResponse().map(p -> {
                        cleanup.add(p.uniqueIdentifier());
                        return true;
                    }).orElse(false), start);
                }
            }
            if ("Create Key Pair".equals(op) || "Re-key Key Pair".equals(op)
                    || "Certify".equals(op) || "Re-certify".equals(op) || "Validate".equals(op)
                    || "Sign".equals(op) || "Signature Verify".equals(op)) {
                return executeExtendedKeyPairGroup(testCase, op, fluent, cleanup, start);
            }
            if ("Encrypt".equals(op) || "Decrypt".equals(op)) {
                return executeExtendedEncryptGroup(testCase, op, fluent, cleanup, start);
            }
            if ("MAC".equals(op) || "MAC Verify".equals(op)) {
                return executeExtendedMacGroup(testCase, op, fluent, cleanup, start);
            }
            return executeExtendedSymGroup(testCase, op, fluent, cleanup, start);
        } catch (RuntimeException exception) {
            return exchangeFailure(exception, start);
        } finally {
            for (UniqueIdentifier id : cleanup) {
                try {
                    revokeAndDestroy(id, start);
                } catch (RuntimeException ignored) { }
            }
        }
    }

    private void revokeAndDestroy(UniqueIdentifier id, long start) {
        try (var reason = RevocationReason.create(6, "cleanup");
                var req = RevokeRequest.create(id, reason, null)) {
            executeRequest(null, withBatchId(ClientBatchItem.revoke(req)), r -> true, start);
        } catch (RuntimeException ignored) { }
        try {
            executeRequest(null, withBatchId(ClientBatchItem.destroy(id)), r -> true, start);
        } catch (RuntimeException ignored) { }
    }

    private TtlvStructure symmetricKeyAttributes(String name, int bits, int usageMask) {
        return symmetricKeyAttributes(name, bits, usageMask, true);
    }

    private TtlvStructure symmetricKeyAttributes(
            String name, int bits, int usageMask, boolean includeDescription) {
        try (var s0 = appendValue(TtlvStructure.create(), 0x420028, TtlvValue.Enumeration(3));
                var s1 = appendValue(s0, 0x42002a, TtlvValue.Integer(bits));
                var s2 = appendValue(s1, 0x42002c, TtlvValue.Integer(usageMask));
                var s3 = includeDescription
                        ? appendValue(s2, 0x4200fc, TtlvValue.TextString(
                                ("kmipkit-" + name).getBytes(StandardCharsets.US_ASCII), CODEC_LIMITS))
                        : GenericExchange.copyStructure(s2.view())) {
            return TtlvStructure.withItem(s3, TtlvItem.create(
                    RawTag.fromRaw(0x420122).checked(), TtlvValue.Boolean(true), CODEC_LIMITS), CODEC_LIMITS);
        }
    }

    private CaseOutcome executeExtendedKeyPairGroup(CatalogLoader.CatalogCase testCase, String op,
            boolean fluent, List<UniqueIdentifier> cleanup, long start) {
        var privRef = new AtomicReference<UniqueIdentifier>();
        var pubRef = new AtomicReference<UniqueIdentifier>();
        try (var common0 = appendValue(TtlvStructure.create(), 0x420028, TtlvValue.Enumeration(4));
                var common = appendValue(common0, 0x42002a, TtlvValue.Integer(2048));
                var privA = appendValue(TtlvStructure.create(), 0x42002c, TtlvValue.Integer(1));
                var pubA = appendValue(TtlvStructure.create(), 0x42002c, TtlvValue.Integer(2))) {
            ClientBatchItem kpReq;
            if (fluent) {
                try (var b = client.request().createKeyPair()) {
                    kpReq = withBatchId(b.algorithm(4).length(2048).privateUsageMask(1).publicUsageMask(2).build());
                }
            } else {
                kpReq = withBatchId(ClientBatchItem.createKeyPair(common, privA, pubA, null, null, null));
            }
            var kpRes = executeRequest(testCase, kpReq, r -> r.createKeyPairResponse().map(p -> {
                privRef.set(p.privateKeyUniqueIdentifier());
                pubRef.set(p.publicKeyUniqueIdentifier());
                cleanup.add(p.privateKeyUniqueIdentifier());
                cleanup.add(p.publicKeyUniqueIdentifier());
                return true;
            }).orElse(false), start);
            if ("Create Key Pair".equals(op)) return kpRes;
            if (!"passed".equals(kpRes.status())) {
                return CaseOutcome.notRun("The prerequisite Create Key Pair request failed.", elapsedMillis(start));
            }
            if ("Re-key Key Pair".equals(op)) {
                ClientBatchItem rkReq;
                if (fluent) {
                    try (var b = client.request().rekeyKeyPair()) {
                        rkReq = withBatchId(b.privateKeyUniqueIdentifier(privRef.get()).offset(3600L).build());
                    }
                } else {
                    rkReq = withBatchId(ClientBatchItem.rekeyKeyPair(privRef.get(), 3600L, null, null, null, null, null, null));
                }
                return executeRequest(testCase, rkReq, r -> r.rekeyKeyPairResponse().map(p -> {
                    cleanup.add(p.privateKeyUniqueIdentifier());
                    cleanup.add(p.publicKeyUniqueIdentifier());
                    return true;
                }).orElse(false), start);
            }
            if ("Certify".equals(op) || "Re-certify".equals(op) || "Validate".equals(op)) {
                var certRef = new AtomicReference<UniqueIdentifier>();
                CaseOutcome certRes;
                try (var attrs = appendOwnedValue(TtlvStructure.create(), 0x42001d, TtlvValue.Enumeration(1))) {
                    certRes = executeRequest(testCase,
                            withBatchId(ClientBatchItem.certify(pubRef.get(), null, null, attrs, null)),
                            r -> r.certifyResponse().filter(payload -> {
                                try (payload) {
                                    certRef.set(payload.uniqueIdentifier());
                                    cleanup.add(payload.uniqueIdentifier());
                                    return true;
                                }
                            }).isPresent(), start);
                }
                if ("Certify".equals(op)) return certRes;
                if (!"passed".equals(certRes.status())) {
                    return CaseOutcome.notRun("The server rejected the prerequisite Certify request.", elapsedMillis(start));
                }
                if ("Re-certify".equals(op)) {
                    ClientBatchItem rcReq;
                    if (fluent) {
                        try (var b = client.request().recertify()) {
                            rcReq = withBatchId(b.uniqueIdentifier(certRef.get()).build());
                        }
                    } else {
                        rcReq = withBatchId(ClientBatchItem.recertify(certRef.get(), null, null, null, null, null, null));
                    }
                    return executeRequest(testCase, rcReq, r -> r.recertifyResponse().map(p -> {
                        cleanup.add(p.uniqueIdentifier());
                        return true;
                    }).orElse(false), start);
                }
                return executeRequest(testCase,
                        withBatchId(ClientBatchItem.validate(List.of(), List.of(certRef.get()), null)),
                        r -> r.validateResponse().isPresent(), start);
            }
            var act1 = executeRequest(testCase, withBatchId(ClientBatchItem.activate(privRef.get())), r -> true, start);
            if (!"passed".equals(act1.status())) return act1;
            var act2 = executeRequest(testCase, withBatchId(ClientBatchItem.activate(pubRef.get())), r -> true, start);
            if (!"passed".equals(act2.status())) return act2;
            var sigRef = new AtomicReference<byte[]>();
            try (var p0 = appendValue(TtlvStructure.create(), 0x42005f, TtlvValue.Enumeration(8));
                    var params = appendValue(p0, 0x420038, TtlvValue.Enumeration(6));
                    var dataVal = TtlvValue.ByteString(HASH_INPUT, CODEC_LIMITS)) {
                ClientBatchItem signReq;
                if (fluent) {
                    try (var b = client.request().sign()) {
                        signReq = withBatchId(b.uniqueIdentifier(privRef.get())
                                .cryptographicParametersStructure(params).data(HASH_INPUT).build());
                    }
                } else {
                    signReq = withBatchId(ClientBatchItem.sign(privRef.get(), params, dataVal, null, null, null, null));
                }
                var signRes = executeRequest(testCase, signReq, r -> r.signResponse().map(p -> {
                    sigRef.set(p.signatureData().orElse(null));
                    return sigRef.get() != null && sigRef.get().length > 0;
                }).orElse(false), start);
                if ("Sign".equals(op) || !"passed".equals(signRes.status())) return signRes;
                ClientBatchItem verReq;
                if (fluent) {
                    try (var b = client.request().signatureVerify()) {
                        verReq = withBatchId(b.uniqueIdentifier(pubRef.get())
                                .cryptographicParametersStructure(params).data(HASH_INPUT).signatureData(sigRef.get()).build());
                    }
                } else {
                    try (var dv = TtlvValue.ByteString(HASH_INPUT, CODEC_LIMITS)) {
                        verReq = withBatchId(ClientBatchItem.signatureVerify(
                                pubRef.get(), params, dv, null, sigRef.get(), null, null, null));
                    }
                }
                return executeRequest(testCase, verReq, r -> r.signatureVerifyResponse().isPresent(), start);
            }
        }
    }

    private CaseOutcome executeExtendedEncryptGroup(CatalogLoader.CatalogCase testCase, String op,
            boolean fluent, List<UniqueIdentifier> cleanup, long start) {
        boolean gcm = "GCM".equals(testCase.requires().path("block_cipher_mode").asText());
        boolean invalidTag = gcm && "OperationFailed".equals(testCase.requires().path("expected_status").asText());
        var keyRef = new AtomicReference<UniqueIdentifier>();
        try (var attrs = symmetricKeyAttributes("enc-" + UUID.randomUUID(), 256, 12)) {
            var cRes = executeRequest(testCase, withBatchId(ClientBatchItem.create(2L, attrs, null)), r -> r.createResponse().map(p -> {
                keyRef.set(p.uniqueIdentifier());
                cleanup.add(p.uniqueIdentifier());
                return true;
            }).orElse(false), start);
            if (!"passed".equals(cRes.status()) || keyRef.get() == null)
                return gcm ? CaseOutcome.notRun("The prerequisite Create request failed.", elapsedMillis(start)) : cRes;
            var act = executeRequest(testCase, withBatchId(ClientBatchItem.activate(keyRef.get())), r -> true, start);
            if (!"passed".equals(act.status()) || !Long.valueOf(0).equals(act.resultStatusCode()))
                return gcm ? CaseOutcome.notRun("The prerequisite Activate request failed.", elapsedMillis(start)) : act;
            byte[] iv = new byte[gcm ? 12 : 16];
            if (!gcm) Arrays.fill(iv, (byte) 0x01);
            byte[] pt = (gcm ? "AAAAAAAAAAAAAAAA" : "0123456789abcdef").getBytes(StandardCharsets.US_ASCII);
            byte[] aad = gcm ? "kmipkit authenticated data".getBytes(StandardCharsets.US_ASCII) : null;
            var cipherRef = new AtomicReference<byte[]>();
            var tagRef = new AtomicReference<byte[]>();
            try (var params = encryptionParameters(gcm);
                    var ptVal = TtlvValue.ByteString(pt, CODEC_LIMITS)) {
                ClientBatchItem encReq;
                if (fluent) {
                    try (var b = client.request().encrypt()) {
                        b.uniqueIdentifier(keyRef.get()).cryptographicParametersStructure(params).data(pt).ivCounterNonce(iv);
                        if (gcm) b.authenticatedEncryptionAdditionalData(aad);
                        encReq = withBatchId(b.build());
                    }
                } else {
                    encReq = withBatchId(ClientBatchItem.encrypt(keyRef.get(), params, ptVal, iv, null, null, null, aad));
                }
                var encRes = executeRequest(testCase, encReq, r -> r.encryptResponse().map(p -> {
                    try (p) {
                        byte[] ciphertext = p.data().orElse(null);
                        cipherRef.set(ciphertext);
                        if (!gcm) return ciphertext != null && ciphertext.length > 0;
                        byte[] tag = p.authenticatedEncryptionTag().orElse(null);
                        tagRef.set(tag);
                        return sameIdentifier(keyRef.get(), p.uniqueIdentifier()) && ciphertext != null
                                && ciphertext.length == pt.length && tag != null && tag.length == 16;
                    }
                }).orElse(false), start);
                if ("Encrypt".equals(op)) return encRes;
                if (!"passed".equals(encRes.status()) || cipherRef.get() == null)
                    return gcm ? CaseOutcome.notRun("The prerequisite Encrypt request failed.", elapsedMillis(start)) : encRes;
                byte[] tag = tagRef.get();
                if (invalidTag) tag[0] ^= 1;
                ClientBatchItem decReq;
                if (fluent) {
                    try (var b = client.request().decrypt()) {
                        b.uniqueIdentifier(keyRef.get()).cryptographicParametersStructure(params)
                                .data(cipherRef.get()).ivCounterNonce(iv);
                        if (gcm) b.authenticatedEncryptionAdditionalData(aad).authenticatedEncryptionTag(tag);
                        decReq = withBatchId(b.build());
                    }
                } else {
                    try (var cv = TtlvValue.ByteString(cipherRef.get(), CODEC_LIMITS)) {
                        decReq = withBatchId(ClientBatchItem.decrypt(keyRef.get(), params, cv, iv, tag, null, null, null, aad));
                    }
                }
                return executeRequest(testCase, decReq, r -> r.decryptResponse().map(p -> {
                    try (p) {
                        return !invalidTag && (!gcm || sameIdentifier(keyRef.get(), p.uniqueIdentifier()))
                                && Arrays.equals(pt, p.data().orElse(null));
                    }
                }).orElse(false), start);
            }
        }
    }

    private static TtlvStructure encryptionParameters(boolean gcm) {
        try (var mode = appendValue(TtlvStructure.create(), 0x420011, TtlvValue.Enumeration(gcm ? 9 : 1))) {
            var padding = appendValue(mode, 0x42005f, TtlvValue.Enumeration(gcm ? 1 : 3));
            if (!gcm) return padding;
            try (padding; var ivLength = appendValue(padding, 0x4200cd, TtlvValue.Integer(96))) {
                return appendValue(ivLength, 0x4200ce, TtlvValue.Integer(16));
            }
        }
    }

    private CaseOutcome executeExtendedMacGroup(CatalogLoader.CatalogCase testCase, String op,
            boolean fluent, List<UniqueIdentifier> cleanup, long start) {
        var keyRef = new AtomicReference<UniqueIdentifier>();
        try (var s0 = appendValue(TtlvStructure.create(), 0x420028, TtlvValue.Enumeration(9));
                var s1 = appendValue(s0, 0x42002a, TtlvValue.Integer(256));
                var attrs = appendValue(s1, 0x42002c, TtlvValue.Integer(384))) {
            var cRes = executeRequest(testCase, withBatchId(ClientBatchItem.create(2L, attrs, null)), r -> r.createResponse().map(p -> {
                keyRef.set(p.uniqueIdentifier());
                cleanup.add(p.uniqueIdentifier());
                return true;
            }).orElse(false), start);
            if (!"passed".equals(cRes.status())) return cRes;
            var act = executeRequest(testCase, withBatchId(ClientBatchItem.activate(keyRef.get())), r -> true, start);
            if (!"passed".equals(act.status())) return act;
            var tagRef = new AtomicReference<byte[]>();
            try (var params = appendValue(TtlvStructure.create(), 0x420038, TtlvValue.Enumeration(6));
                    var dv = TtlvValue.ByteString(HASH_INPUT, CODEC_LIMITS)) {
                ClientBatchItem macReq;
                if (fluent) {
                    try (var b = client.request().mac()) {
                        macReq = withBatchId(b.uniqueIdentifier(keyRef.get())
                                .cryptographicParametersStructure(params).data(HASH_INPUT).build());
                    }
                } else {
                    macReq = withBatchId(ClientBatchItem.mac(keyRef.get(), params, dv, null, null, null));
                }
                var macRes = executeRequest(testCase, macReq, r -> r.macResponse().map(p -> {
                    tagRef.set(p.macData().orElse(null));
                    return tagRef.get() != null && tagRef.get().length > 0;
                }).orElse(false), start);
                if ("MAC".equals(op) || !"passed".equals(macRes.status())) return macRes;
                ClientBatchItem verReq;
                if (fluent) {
                    try (var b = client.request().macVerify()) {
                        verReq = withBatchId(b.uniqueIdentifier(keyRef.get())
                                .cryptographicParametersStructure(params).data(HASH_INPUT).macData(tagRef.get()).build());
                    }
                } else {
                    try (var dv2 = TtlvValue.ByteString(HASH_INPUT, CODEC_LIMITS)) {
                        verReq = withBatchId(ClientBatchItem.macVerify(keyRef.get(), params, dv2, tagRef.get(), null, null, null));
                    }
                }
                return executeRequest(testCase, verReq, r -> r.macVerifyResponse().isPresent(), start);
            }
        }
    }

    private CaseOutcome executeExtendedSymGroup(CatalogLoader.CatalogCase testCase, String op,
            boolean fluent, List<UniqueIdentifier> cleanup, long start) {
        var keyRef = new AtomicReference<UniqueIdentifier>();
        int usage = "Derive Key".equals(op) ? 512 : 12;
        try (var attrs = symmetricKeyAttributes("sym-" + UUID.randomUUID(), 256, usage, !"Add Attribute".equals(op))) {
            var cRes = executeRequest(testCase, withBatchId(ClientBatchItem.create(2L, attrs, null)), r -> r.createResponse().map(p -> {
                keyRef.set(p.uniqueIdentifier());
                cleanup.add(p.uniqueIdentifier());
                return true;
            }).orElse(false), start);
            if (!"passed".equals(cRes.status())) return cRes;
            UniqueIdentifier keyId = keyRef.get();
            if ("Re-key".equals(op)) {
                ClientBatchItem rkReq;
                if (fluent) {
                    try (var b = client.request().rekey()) {
                        rkReq = withBatchId(b.uniqueIdentifier(keyId).offset(3600L).build());
                    }
                } else {
                    rkReq = withBatchId(ClientBatchItem.rekey(keyId, 3600L, null, null));
                }
                return executeRequest(testCase, rkReq, r -> r.rekeyResponse().map(p -> {
                    cleanup.add(p.uniqueIdentifier());
                    return true;
                }).orElse(false), start);
            }
            if ("Export".equals(op)) {
                return executeRequest(testCase, withBatchId(ClientBatchItem.export(keyId, 1L, null, null, null)),
                        r -> r.exportResponse().isPresent(), start);
            }
            if ("Get Usage Allocation".equals(op)) {
                return executeRequest(testCase, withBatchId(ClientBatchItem.getUsageAllocation(keyId, 32L)),
                        r -> r.getUsageAllocationResponse().isPresent(), start);
            }
            if ("Derive Key".equals(op)) {
                executeRequest(testCase, withBatchId(ClientBatchItem.activate(keyId)), r -> true, start);
                try (var cp = appendValue(TtlvStructure.create(), 0x420038, TtlvValue.Enumeration(6));
                        var dp = appendValue(TtlvStructure.create(), 0x42002b, TtlvValue.structure(cp, CODEC_LIMITS));
                        var da = symmetricKeyAttributes("derived-" + UUID.randomUUID(), 128, 4)) {
                    return executeRequest(testCase,
                            withBatchId(ClientBatchItem.deriveKey(2L, List.of(keyId), 2L, dp, da)),
                            r -> r.deriveKeyResponse().map(p -> {
                                cleanup.add(p.uniqueIdentifier());
                                return true;
                            }).orElse(false), start);
                }
            }
            if ("Join Split Key".equals(op)) {
                var parts = new ArrayList<UniqueIdentifier>();
                var splitRes = executeRequest(testCase,
                        withBatchId(ClientBatchItem.createSplitKey(2L, keyId, 2, 2, 1L, null, attrs, null)),
                        r -> r.createSplitKeyUniqueIdentifiers().map(list -> {
                            parts.addAll(list);
                            cleanup.addAll(list);
                            return parts.size() == 2;
                        }).orElse(false), start);
                if (!"passed".equals(splitRes.status())) return splitRes;
                var refs = List.of(
                        new org.kmipkit.extensions.SplitKeyReference(parts.get(0), 2L),
                        new org.kmipkit.extensions.SplitKeyReference(parts.get(1), 2L));
                return executeRequest(testCase, withBatchId(ClientBatchItem.joinSplitKey(2L, refs, null, attrs, null)),
                        r -> r.joinSplitKeyResponse().map(p -> {
                            cleanup.add(p.uniqueIdentifier());
                            return true;
                        }).orElse(false), start);
            }
            try (var attrVal = TtlvValue.TextString("kmipkit-attr".getBytes(StandardCharsets.US_ASCII), CODEC_LIMITS);
                    var attrItem = TtlvItem.create(RawTag.fromRaw(0x4200fc).checked(), attrVal, CODEC_LIMITS);
                    var newVal = TtlvValue.TextString("kmipkit-changed".getBytes(StandardCharsets.US_ASCII), CODEC_LIMITS);
                    var newItem = TtlvItem.create(RawTag.fromRaw(0x4200fc).checked(), newVal, CODEC_LIMITS)) {
                if ("Add Attribute".equals(op)) {
                    ClientBatchItem req;
                    if (fluent) {
                        try (var b = client.request().addAttribute()) {
                            req = withBatchId(b.uniqueIdentifier(keyId.text()).attributeItem(newItem).build());
                        }
                    } else {
                        req = withBatchId(ClientBatchItem.addAttribute(keyId, newItem));
                    }
                    return executeRequest(testCase, req, r -> r.addAttributeResponse().isPresent(), start);
                }
                if ("Modify Attribute".equals(op)) {
                    return executeRequest(testCase, withBatchId(ClientBatchItem.modifyAttribute(keyId, attrItem, newItem)),
                            r -> r.modifyAttributeResponse().isPresent(), start);
                }
                if ("Delete Attribute".equals(op)) {
                    ClientBatchItem req;
                    if (fluent) {
                        try (var b = client.request().deleteAttribute()) {
                            req = withBatchId(b.uniqueIdentifier(keyId.text()).attributeReference(AttributeReference.tag(0x4200fc)).build());
                        }
                    } else {
                        req = withBatchId(ClientBatchItem.deleteAttribute(keyId, null, AttributeReference.tag(0x4200fc)));
                    }
                    return executeRequest(testCase, req, r -> r.deleteAttributeResponse().isPresent(), start);
                }
                ClientBatchItem req;
                if (fluent) {
                    try (var b = client.request().setAttribute()) {
                        req = withBatchId(b.uniqueIdentifier(keyId.text()).attributeItem(newItem).build());
                    }
                } else {
                    req = withBatchId(ClientBatchItem.setAttribute(keyId, newItem));
                }
                return executeRequest(testCase, req, r -> r.setAttributeResponse().isPresent(), start);
            }
        }
    }

    private static TtlvStructure appendSecret(TtlvStructure structure, int rawTag, String secret) {
        try (SecretText text = SecretText.fromString(secret)) {
            return appendValue(structure, rawTag, text.intoTtlvValue());
        }
    }

    private static TtlvStructure appendValue(TtlvStructure structure, int rawTag, TtlvValue value) {
        try (value; RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            TtlvItem item = TtlvItem.create(tag, value, CODEC_LIMITS);
            return TtlvStructure.withItem(structure, item, CODEC_LIMITS);
        }
    }

    private static boolean expectedReason(CatalogLoader.CatalogCase testCase, long reason) {
        var reasons = testCase.requires().get("expected_result_reasons");
        if (reasons == null || !reasons.isArray()) return false;
        for (var expected : reasons) {
            if (expected.canConvertToLong() && expected.longValue() == reason) return true;
        }
        return false;
    }

    private static ProtocolVersion toProtocolVersion(String version) {
        String[] parts = version.split("\\.");
        return new ProtocolVersion(Integer.parseInt(parts[0]), Integer.parseInt(parts[1]));
    }

    private static Long optionalCode(long code) {
        return code < 0 ? null : code;
    }

    private static double elapsedMillis(long start) {
        return Duration.ofNanos(System.nanoTime() - start).toNanos() / 1_000_000.0;
    }

    @Override
    public void close() {
        client.close();
    }

    public record CaseOutcome(String status, String libraryCorrectness, String serverCompatibility,
                              String parity, double durationMs, Long resultStatusCode,
                              Long resultReasonCode, String diagnostic) {
        static CaseOutcome passed(double elapsed, Long status, Long reason) {
            return new CaseOutcome("passed", "passed", "supported", "not_compared", elapsed,
                    status, reason, null);
        }

        static CaseOutcome localModelPassed(double elapsed) {
            return new CaseOutcome("passed", "passed", "unknown", "not_compared", elapsed,
                    null, null, null);
        }

        static CaseOutcome failed(String diagnostic, double elapsed) {
            return failed(diagnostic, elapsed, null, null);
        }

        static CaseOutcome failed(String diagnostic, double elapsed, Long status, Long reason) {
            return new CaseOutcome("failed", "not_checked", "unknown", "not_compared", elapsed,
                    status, reason, diagnostic);
        }

        static CaseOutcome unsupported(double elapsed, long status, long reason) {
            return new CaseOutcome("server_unsupported", "passed", "unsupported", "not_compared",
                    elapsed, status, reason, "The server rejected the operation as unsupported.");
        }

        static CaseOutcome policyRestricted(double elapsed, long status, long reason) {
            return new CaseOutcome("policy_restricted", "passed", "policy_restricted", "not_compared",
                    elapsed, status, reason, "The server policy rejected the operation.");
        }

        static CaseOutcome notRun(String diagnostic, double elapsed) {
            return new CaseOutcome("not_run", "not_checked", "unknown", "not_compared",
                    elapsed, null, null, diagnostic);
        }
    }
}
