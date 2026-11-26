package org.kmipkit.validator;

import org.kmipkit.credentials.Authentication;
import org.kmipkit.credentials.Credential;
import org.kmipkit.credentials.SecretText;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.AttributeReference;
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
        ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                List.of(), ExtensionRegistryLimits.defaults());
        ClientConfiguration baseConfiguration = ClientConfiguration.create(registry);
        ClientConfiguration encodingConfiguration = baseConfiguration.withWireEncoding(
                WireEncoding.valueOf(connection.encoding()));
        ClientConfiguration protocolConfiguration = encodingConfiguration.withProtocolVersion(
                toProtocolVersion(connection.protocolVersion()));
        TransportConfiguration transport = buildTransport(connection);
        try (registry; baseConfiguration; encodingConfiguration; protocolConfiguration; transport) {
            return new JavaKmipDriver(KmipClient.create(protocolConfiguration, transport), connection);
        }
    }

    /** Executes one available typed operation and records only numeric KMIP result codes. */
    public CaseOutcome execute(CatalogLoader.CatalogCase testCase) {
        long start = System.nanoTime();
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
        ClientBatchItem request = "fluent".equals(connection.apiLevel()) && "Ping".equals(testCase.operation())
                ? withBatchId(client.request().ping().build())
                : TypedRequestFactory.create(testCase.operation(), !isDraft());
        return executeRequest(testCase, request, result -> verifyTypedResponse(testCase.operation(), result), start);
    }

    static boolean readOnlyProbe(CatalogLoader.CatalogCase testCase) {
        return Set.of("test-op-locate", "test-op-hash", "test-crypto-hash-sha_256-0")
                .contains(testCase.caseId());
    }

    static boolean multipartHash(CatalogLoader.CatalogCase testCase) {
        return "test-multipart-hash-sha256".equals(testCase.caseId());
    }

    static boolean supportsFluent(CatalogLoader.CatalogCase testCase) {
        return !typedOnlyKeyProbe(testCase) && (registerOpaqueCase(testCase) || createSplitKeyCase(testCase)
                || ownedObjectCase(testCase) || readOnlyProbe(testCase) || multipartHash(testCase)
                || Set.of("Create", "Ping").contains(testCase.operation()));
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

            return executeRequest(testCase, withBatchId(locateRequest(fluent)), result ->
                    result.locateResponse().map(payload -> payload.locatedItems().orElse(0) == 0
                            && payload.uniqueIdentifiers().isEmpty()).orElse(false), start);
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

    private ClientBatchItem locateRequest(boolean fluent) {
        // Description is a Text String in all three versions.
        byte[] isolatedDescription = ("kmipkit-locate-" + UUID.randomUUID()).getBytes(StandardCharsets.US_ASCII);
        try (var source = appendOwnedValue(TtlvStructure.create(), 0x0042_00fc,
                TtlvValue.TextString(isolatedDescription, CODEC_LIMITS))) {
            if (fluent) {
                try (var attributes = AttributeSet.tryNew(source); var builder = client.request().locate(attributes)) {
                    return builder.maximumItems(1).offsetItems(0).build();
                }
            }
            return ClientBatchItem.locate(1, 0, null, null, source);
        }
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
        } catch (Exception exception) {
            return CaseOutcome.failed("KMIP client exchange failed (" + exception.getClass().getSimpleName()
                    + "); remote diagnostic text was suppressed.", elapsedMillis(start));
        }
    }

    static boolean ownedObjectCase(CatalogLoader.CatalogCase testCase) {
        return attributeRead(testCase) || typedOnlyKeyProbe(testCase)
                || Set.of("test-op-get", "test-op-destroy", "test-op-archive",
                "test-op-recover", "test-op-activate", "test-op-revoke").contains(testCase.caseId());
    }

    private static boolean typedOnlyKeyProbe(CatalogLoader.CatalogCase testCase) {
        return Set.of("test-op-check", "test-op-obtain-lease").contains(testCase.caseId());
    }

    static boolean registerOpaqueCase(CatalogLoader.CatalogCase testCase) {
        return Set.of("test-op-register", "test-obj-opaque-object").contains(testCase.caseId());
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
        return switch (testCase.caseId()) {
            case "test-op-get-attributes", "test-op-get-attribute-list", "test-attr-name",
                    "test-attr-cryptographic-algorithm", "test-attr-cryptographic-length" -> true;
            default -> false;
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

        int rawTag = switch (testCase.caseId()) {
            case "test-attr-cryptographic-algorithm" -> 0x0042_0028;
            case "test-attr-cryptographic-length" -> 0x0042_002a;
            default -> 0x0042_0053;
        };
        var references = List.of(AttributeReference.tag(rawTag));
        var request = fluent ? client.request().getAttributes(id, references).build()
                : ClientBatchItem.getAttributes(id, references);
        return executeRequest(testCase, request, result -> {
            var payload = result.getAttributesResponse().orElse(null);
            if (payload == null) return false;
            try (payload) {
                return id.text().equals(payload.uniqueIdentifier()) && payload.attributes()
                        .map(attributes -> containsAttribute(attributes, rawTag, objectName)).orElse(false);
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
