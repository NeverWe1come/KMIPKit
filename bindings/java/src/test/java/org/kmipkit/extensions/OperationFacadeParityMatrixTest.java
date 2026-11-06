package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import java.util.function.Function;
import java.util.function.Supplier;
import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

/** KMIP 2.1 §6.1 request-route smoke matrix for all client operations in KMIPKIT-0053. */
final class OperationFacadeParityMatrixTest {
    private static final int CORRELATION_VALUE_TAG = 0x0042_0006;
    private static final int CORRELATION_VALUES_TAG = 0x0042_0176;
    private static final int OPERATIONS_TAG = 0x0042_014f;
    private static final int OPERATION_TAG = 0x0042_005c;
    private static final byte[] CORRELATION = new byte[] {0, (byte) 0xff, 0x2a};
    private static final byte[] SECOND_CORRELATION = new byte[] {0x10, 0, 0x7f};

    @Test
    void allFiftySixClientOperationsConstructARequestOrReachTheTypedBatchApi() {
        Set<String> exercised = new HashSet<>();
        for (BatchRoute route : batchRoutes()) {
            assertTrue(exercised.add(route.operation()), route.operation());
            try (ClientBatch batch = ClientBatch.create();
                    ClientBatchItem item = route.factory().get()) {
                assertSame(batch, batch.add(item), route.operation());
                assertEquals(1, batch.itemCount(), route.operation());
                try (ClientBatchItem readBack = batch.itemAt(0)) {
                    assertEquals(0, readBack.extensionCount(), route.operation());
                    assertEquals(operationRaw(route.operation()), readBack.operation(), route.operation());
                }
            }
        }

        for (AsyncRoute route : asyncRoutes()) {
            assertTrue(exercised.add(route.operation()), route.operation());
            route.constructAndReadPayload().run();
        }

        assertEquals(56, exercised.size());
        assertResponseAccessorMatrix(exercised);
    }

    private static void assertResponseAccessorMatrix(Set<String> exercisedOperations) {
        Set<String> batchResponseOperations = new HashSet<>();
        for (BatchResponseAccessor route : batchResponseAccessors()) {
            assertTrue(batchResponseOperations.add(route.operation()), route.operation());
            assertTrue(exercisedOperations.contains(route.operation()), route.operation());
            assertTrue(route.accessor() != null, route.operation());
        }

        Set<String> asyncResponseOperations = new HashSet<>();
        for (AsyncResponseAccessor route : asyncResponseAccessors()) {
            assertTrue(asyncResponseOperations.add(route.operation()), route.operation());
            assertTrue(exercisedOperations.contains(route.operation()), route.operation());
            assertTrue(route.accessor() != null, route.operation());
        }

        Set<String> allResponseOperations = new HashSet<>(batchResponseOperations);
        allResponseOperations.addAll(asyncResponseOperations);
        assertEquals(exercisedOperations, allResponseOperations);
    }

    /** Compile-time route map; result mapping and value assertions live in the operation tests. */
    private static List<BatchResponseAccessor> batchResponseAccessors() {
        return List.of(
                result("Activate", ClientBatchItemResult::activateResponse),
                result("Add Attribute", ClientBatchItemResult::addAttributeResponse),
                result("Adjust Attribute", ClientBatchItemResult::adjustAttributeResponse),
                result("Archive", ClientBatchItemResult::archiveResponse),
                result("Certify", ClientBatchItemResult::certifyResponse),
                result("Check", ClientBatchItemResult::checkResponse),
                result("Create", ClientBatchItemResult::createResponse),
                result("Create Key Pair", ClientBatchItemResult::createKeyPairResponse),
                result("Create Split Key", ClientBatchItemResult::createSplitKeyUniqueIdentifiers),
                result("Decrypt", ClientBatchItemResult::decryptResponse),
                result("Delegated Login", ClientBatchItemResult::delegatedLoginResponse),
                result("Delete Attribute", ClientBatchItemResult::deleteAttributeResponse),
                result("Derive Key", ClientBatchItemResult::deriveKeyResponse),
                result("Destroy", ClientBatchItemResult::destroyResponse),
                result("Discover Versions", ClientBatchItemResult::supportedVersions),
                result("Encrypt", ClientBatchItemResult::encryptResponse),
                result("Export", ClientBatchItemResult::exportResponse),
                result("Get", ClientBatchItemResult::getResponse),
                result("Get Attributes", ClientBatchItemResult::getAttributesResponse),
                result("Get Attribute List", ClientBatchItemResult::getAttributeListResponse),
                result("Get Constraints", ClientBatchItemResult::getConstraintsResponse),
                result("Get Usage Allocation", ClientBatchItemResult::getUsageAllocationResponse),
                result("Hash", ClientBatchItemResult::hashResponse),
                result("Import", ClientBatchItemResult::importResponse),
                result("Interop", ClientBatchItemResult::interopResponse),
                result("Join Split Key", ClientBatchItemResult::joinSplitKeyResponse),
                result("Locate", ClientBatchItemResult::locateResponse),
                result("Log", ClientBatchItemResult::logResponse),
                result("Login", ClientBatchItemResult::loginResponse),
                result("Logout", ClientBatchItemResult::logoutResponse),
                result("MAC", ClientBatchItemResult::macResponse),
                result("MAC Verify", ClientBatchItemResult::macVerifyResponse),
                result("Modify Attribute", ClientBatchItemResult::modifyAttributeResponse),
                result("Obtain Lease", ClientBatchItemResult::obtainLeaseResponse),
                result("Ping", ClientBatchItemResult::pingResponse),
                result("PKCS#11", ClientBatchItemResult::pkcs11Response),
                result("Query", ClientBatchItemResult::queryResponse),
                result("Re-certify", ClientBatchItemResult::recertifyResponse),
                result("Recover", ClientBatchItemResult::recoverResponse),
                result("Register", ClientBatchItemResult::registerResponse),
                result("Re-key", ClientBatchItemResult::rekeyResponse),
                result("Re-key Key Pair", ClientBatchItemResult::rekeyKeyPairResponse),
                result("Re-Provision", ClientBatchItemResult::reprovisionResponse),
                result("Revoke", ClientBatchItemResult::revokeResponse),
                result("RNG Retrieve", ClientBatchItemResult::rngRetrieveResponse),
                result("RNG Seed", ClientBatchItemResult::rngSeedResponse),
                result("Set Attribute", ClientBatchItemResult::setAttributeResponse),
                result("Set Constraints", ClientBatchItemResult::setConstraintsResponse),
                result("Set Defaults", ClientBatchItemResult::setDefaultsResponse),
                result("Sign", ClientBatchItemResult::signResponse),
                result("Signature Verify", ClientBatchItemResult::signatureVerifyResponse),
                result("Validate", ClientBatchItemResult::validateResponse));
    }

    private static List<AsyncResponseAccessor> asyncResponseAccessors() {
        return List.of(
                new AsyncResponseAccessor("Cancel", AsyncOperationOutcome::metadataV1),
                new AsyncResponseAccessor("Poll", AsyncOperationOutcome::metadataV1),
                new AsyncResponseAccessor("Process", AsyncOperationOutcome::metadataV1),
                new AsyncResponseAccessor(
                        "Query Asynchronous Requests", AsyncOperationOutcome::responsePayloadTtlv));
    }

    /** Compile-checks every synchronous typed result getter against one response item. */
    private static void invokeSynchronousResponseGetters(ClientBatchItemResult response) {
        response.activateResponse();
        response.addAttributeResponse();
        response.adjustAttributeResponse();
        response.archiveResponse();
        response.certifyResponse();
        response.checkResponse();
        response.createResponse();
        response.createKeyPairResponse();
        response.createSplitKeyUniqueIdentifiers();
        response.decryptResponse();
        response.delegatedLoginResponse();
        response.deleteAttributeResponse();
        response.deriveKeyResponse();
        response.destroyResponse();
        response.supportedVersions();
        response.encryptResponse();
        response.exportResponse();
        response.getResponse();
        response.getAttributesResponse();
        response.getAttributeListResponse();
        response.getConstraintsResponse();
        response.getUsageAllocationResponse();
        response.hashResponse();
        response.importResponse();
        response.interopResponse();
        response.joinSplitKeyResponse();
        response.locateResponse();
        response.logResponse();
        response.loginResponse();
        response.logoutResponse();
        response.macResponse();
        response.macVerifyResponse();
        response.modifyAttributeResponse();
        response.obtainLeaseResponse();
        response.pingResponse();
        response.pkcs11Response();
        response.queryResponse();
        response.recoverResponse();
        response.registerResponse();
        response.rekeyResponse();
        response.rekeyKeyPairResponse();
        response.reprovisionResponse();
        response.revokeResponse();
        response.rngRetrieveResponse();
        response.rngSeedResponse();
        response.setAttributeResponse();
        response.setConstraintsResponse();
        response.setDefaultsResponse();
        response.signResponse();
        response.signatureVerifyResponse();
        response.validateResponse();
    }

    private static BatchResponseAccessor result(
            String operation, Function<ClientBatchItemResult, ?> accessor) {
        return new BatchResponseAccessor(operation, accessor);
    }

    private static List<BatchRoute> batchRoutes() {
        return List.of(
                route("Activate", () -> ClientBatchItem.activate(null)),
                route("Add Attribute", () -> withCommentAttribute(ClientBatchItem::addAttribute)),
                route("Adjust Attribute", () -> ClientBatchItem.adjustAttribute(
                        null, AttributeReference.tag(StandardAttribute.COMMENT.tag()), 1, null)),
                route("Archive", () -> ClientBatchItem.archive(null)),
                route("Certify", () -> ClientBatchItem.certify(null, null, null, null, null)),
                route("Check", () -> ClientBatchItem.check(null, null, null, null)),
                route("Create", () -> withStructure(attributes ->
                        ClientBatchItem.create(0, attributes, null))),
                route("Create Key Pair", () -> ClientBatchItem.createKeyPair(
                        null, null, null, null, null, null)),
                route("Create Split Key", () -> withStructure(attributes ->
                        ClientBatchItem.createSplitKey(0, null, 1, 1, 0,
                                null, attributes, null))),
                route("Decrypt", () -> ClientBatchItem.decrypt(
                        null, null, null, null, null, null, null, null, null)),
                route("Delegated Login", () -> withStructure(rights ->
                        ClientBatchItem.delegatedLogin(rights, null, null, null, null))),
                route("Delete Attribute", ClientBatchItem::deleteAttribute),
                route("Derive Key", () -> withTwoStructures((parameters, attributes) ->
                        ClientBatchItem.deriveKey(0, List.of(UniqueIdentifier.textString("matrix")),
                                0, parameters, attributes))),
                route("Destroy", () -> ClientBatchItem.destroy(null)),
                route("Discover Versions", ClientBatchItem::discoverVersions),
                route("Encrypt", () -> ClientBatchItem.encrypt(
                        null, null, null, null, null, null, null, null)),
                route("Export", () -> ClientBatchItem.export(null, null, null, null, null)),
                route("Get", () -> ClientBatchItem.get(null, null, null, null, null)),
                route("Get Attributes", () -> ClientBatchItem.getAttributes(null, List.of())),
                route("Get Attribute List", ClientBatchItem::getAttributeList),
                route("Get Constraints", ClientBatchItem::getConstraints),
                route("Get Usage Allocation", () -> ClientBatchItem.getUsageAllocation(null, 1)),
                route("Hash", () -> withStructure(parameters ->
                        ClientBatchItem.hash(parameters, null, null, null, null))),
                route("Import", OperationFacadeParityMatrixTest::importObject),
                route("Interop", () -> ClientBatchItem.interop(0, new byte[] {0x41})),
                route("Join Split Key", () -> {
                    try (SplitKeyReference reference = new SplitKeyReference(
                            UniqueIdentifier.textString("matrix"), 1)) {
                        return ClientBatchItem.joinSplitKey(0, List.of(reference));
                    }
                }),
                route("Locate", () -> withStructure(attributes ->
                        ClientBatchItem.locate(null, null, null, null, attributes))),
                route("Log", () -> ClientBatchItem.log("matrix".getBytes(StandardCharsets.UTF_8))),
                route("Login", () -> ClientBatchItem.login(null, null, null, null)),
                route("Logout", () -> {
                    try (TtlvStructure ticket = ticket()) {
                        return ClientBatchItem.logout(ticket);
                    }
                }),
                route("MAC", () -> ClientBatchItem.mac(null, null, null, null, null, null)),
                route("MAC Verify", () -> ClientBatchItem.macVerify(
                        null, null, null, null, null, null, null)),
                route("Modify Attribute", () -> withCommentAttribute(item ->
                        ClientBatchItem.modifyAttribute(null, null, item))),
                route("Obtain Lease", () -> ClientBatchItem.obtainLease(null)),
                route("Ping", ClientBatchItem::ping),
                route("PKCS#11", () -> withStructure(function ->
                        ClientBatchItem.pkcs11(function, null, null, null))),
                route("Query", () -> ClientBatchItem.query(new int[] {0}, null)),
                route("Re-certify", () -> ClientBatchItem.recertify(
                        null, null, null, null, null, null, null)),
                route("Recover", () -> ClientBatchItem.recover(null)),
                route("Register", OperationFacadeParityMatrixTest::register),
                route("Re-key", () -> ClientBatchItem.rekey(null, null, null, null)),
                route("Re-key Key Pair", () -> ClientBatchItem.rekeyKeyPair(
                        null, null, null, null, null, null, null, null)),
                route("Re-Provision", () -> ClientBatchItem.reprovision(null, null)),
                route("Revoke", OperationFacadeParityMatrixTest::revoke),
                route("RNG Retrieve", () -> ClientBatchItem.rngRetrieve(0)),
                route("RNG Seed", () -> ClientBatchItem.rngSeed(new byte[] {0x01})),
                route("Set Attribute", () -> withCommentAttribute(ClientBatchItem::setAttribute)),
                route("Set Constraints", () -> withStructure(ClientBatchItem::setConstraints)),
                route("Set Defaults", ClientBatchItem::setDefaults),
                route("Sign", () -> ClientBatchItem.sign(
                        null, null, null, null, null, null, null)),
                route("Signature Verify", () -> ClientBatchItem.signatureVerify(
                        null, null, null, null, null, null, null, null)),
                route("Validate", () -> ClientBatchItem.validate(List.of(), List.of(), null)));
    }

    private static List<AsyncRoute> asyncRoutes() {
        return List.of(
                new AsyncRoute("Cancel", () -> {
                    try (CancelRequest request = CancelRequest.create(CORRELATION);
                            TtlvStructure payload = request.toTtlvPayload()) {
                        assertCorrelationPayload(payload, CORRELATION_VALUE_TAG, CORRELATION);
                    }
                }),
                new AsyncRoute("Poll", () -> {
                    try (PollRequest request = PollRequest.create(CORRELATION);
                            TtlvStructure payload = request.toTtlvPayload()) {
                        assertCorrelationPayload(payload, CORRELATION_VALUE_TAG, CORRELATION);
                    }
                }),
                new AsyncRoute("Process", () -> {
                    try (ProcessRequest request = ProcessRequest.create(CORRELATION);
                            TtlvStructure payload = request.toTtlvPayload()) {
                        assertCorrelationPayload(payload, CORRELATION_VALUE_TAG, CORRELATION);
                    }
                }),
                new AsyncRoute("Query Asynchronous Requests", () -> {
                    try (QueryAsyncRequestsRequest request = QueryAsyncRequestsRequest.create();
                            QueryAsyncRequestsRequest withCorrelations = request.withCorrelationValues(
                                    List.of(CORRELATION, SECOND_CORRELATION));
                            QueryAsyncRequestsRequest filtered = withCorrelations.withOperations(
                                    List.of(0x1aL));
                            TtlvStructure payload = filtered.toTtlvPayload();
                            TtlvStructureView view = payload.view()) {
                        assertEquals(2, view.itemCount(), "Query Asynchronous Requests");
                        assertTagAt(view, 0, CORRELATION_VALUES_TAG);
                        assertNestedByteStringItems(view, 0, CORRELATION_VALUE_TAG,
                                List.of(CORRELATION, SECOND_CORRELATION));
                        assertTagAt(view, 1, OPERATIONS_TAG);
                        assertNestedEnumerationItems(view, 1, OPERATION_TAG, List.of(0x1aL));
                    }
                }));
    }

    private static void assertCorrelationPayload(
            TtlvStructure payload, int expectedTag, byte[] expectedValue) {
        try (TtlvStructureView view = payload.view()) {
            assertEquals(1, view.itemCount());
            assertTagAt(view, 0, expectedTag);
            try (TtlvItemView item = view.itemAt(0); TtlvValueView value = item.value()) {
                assertByteString(value, expectedValue);
            }
        }
    }

    private static void assertTagAt(TtlvStructureView structure, long index, int expectedTag) {
        try (TtlvItemView item = structure.itemAt(index); Tag tag = item.tag()) {
            assertEquals(expectedTag, tag.raw());
        }
    }

    private static void assertNestedByteStringItems(
            TtlvStructureView parent, long index, int childTag, List<byte[]> expectedValues) {
        try (TtlvItemView item = parent.itemAt(index);
                TtlvValueView value = item.value();
                TtlvStructureView children = value.structure()) {
            assertEquals(expectedValues.size(), children.itemCount());
            for (int child = 0; child < expectedValues.size(); child++) {
                assertTagAt(children, child, childTag);
                try (TtlvItemView childItem = children.itemAt(child);
                        TtlvValueView childValue = childItem.value()) {
                    assertByteString(childValue, expectedValues.get(child));
                }
            }
        }
    }

    private static void assertNestedEnumerationItems(
            TtlvStructureView parent, long index, int childTag, List<Long> expectedValues) {
        try (TtlvItemView item = parent.itemAt(index);
                TtlvValueView value = item.value();
                TtlvStructureView children = value.structure()) {
            assertEquals(expectedValues.size(), children.itemCount());
            for (int child = 0; child < expectedValues.size(); child++) {
                assertTagAt(children, child, childTag);
                try (TtlvItemView childItem = children.itemAt(child);
                        TtlvValueView childValue = childItem.value()) {
                    assertEquals(expectedValues.get(child).longValue(), childValue.enumeration());
                }
            }
        }
    }

    private static void assertByteString(TtlvValueView value, byte[] expected) {
        assertEquals(org.kmipkit.ttlv.TtlvItemType.ByteString, value.itemType());
        assertEquals(expected.length, value.byteLength());
        for (int index = 0; index < expected.length; index++) {
            assertEquals(Byte.toUnsignedInt(expected[index]),
                    Short.toUnsignedInt(value.byteAt(index)));
        }
    }

    private static ClientBatchItem register() {
        try (TtlvStructure attributes = TtlvStructure.create();
                TtlvItem object = opaqueObject();
                RegisterRequest request = RegisterRequest.create(8, attributes, object)) {
            return ClientBatchItem.register(request);
        }
    }

    private static TtlvItem opaqueObject() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure fields = TtlvStructure.create();
        try (Tag typeTag = RawTag.fromRaw(0x0042_0059).checked();
                TtlvValue typeValue = TtlvValue.Enumeration(0);
                TtlvItem typeItem = TtlvItem.create(typeTag, typeValue, limits)) {
            fields = TtlvStructure.withItem(fields, typeItem, limits);
        }
        try (Tag valueTag = RawTag.fromRaw(0x0042_005a).checked();
                TtlvValue dataValue = TtlvValue.ByteString(new byte[] {0x01}, limits);
                TtlvItem valueItem = TtlvItem.create(valueTag, dataValue, limits)) {
            fields = TtlvStructure.withItem(fields, valueItem, limits);
        }
        try (Tag objectTag = RawTag.fromRaw(0x0042_005b).checked()) {
            return TtlvItem.create(objectTag, TtlvValue.structure(fields, limits), limits);
        }
    }

    private static ClientBatchItem revoke() {
        try (RevocationReason reason = RevocationReason.create(0, null);
                RevokeRequest request = RevokeRequest.create(null, reason, null)) {
            return ClientBatchItem.revoke(request);
        }
    }

    private static ClientBatchItem importObject() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure attributes = TtlvStructure.create();
        TtlvStructure anyObject = TtlvStructure.create();
        TtlvStructure nestedObject = TtlvStructure.create();
        try {
            try (Tag objectTag = RawTag.fromRaw(0x0042_008f).checked();
                    TtlvItem object = TtlvItem.create(
                            objectTag, TtlvValue.structure(nestedObject, limits), limits)) {
                anyObject = TtlvStructure.withItem(anyObject, object, limits);
            }
            return ClientBatchItem.importObject(UniqueIdentifier.textString("matrix"),
                    0, null, null, attributes, anyObject);
        } finally {
            attributes.close();
            anyObject.close();
            nestedObject.close();
        }
    }

    private static ClientBatchItem withCommentAttribute(
            Function<TtlvItem, ClientBatchItem> factory) {
        try (TtlvItem attribute = commentAttribute()) {
            return factory.apply(attribute);
        }
    }

    private static TtlvItem commentAttribute() {
        try (RawTag rawTag = RawTag.fromRaw(StandardAttribute.COMMENT.tag());
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.TextString(
                        "matrix".getBytes(StandardCharsets.UTF_8), CodecLimits.defaults())) {
            return TtlvItem.create(tag, value, CodecLimits.defaults());
        }
    }

    private static TtlvStructure ticket() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure ticket = TtlvStructure.create();
        try (Tag typeTag = RawTag.fromRaw(0x0042_014a).checked();
                TtlvValue typeValue = TtlvValue.Enumeration(1);
                TtlvItem typeItem = TtlvItem.create(typeTag, typeValue, limits)) {
            ticket = TtlvStructure.withItem(ticket, typeItem, limits);
        }
        try (Tag valueTag = RawTag.fromRaw(0x0042_014b).checked();
                TtlvValue ticketValue = TtlvValue.ByteString(new byte[] {1, 2, 3}, limits);
                TtlvItem valueItem = TtlvItem.create(valueTag, ticketValue, limits)) {
            return TtlvStructure.withItem(ticket, valueItem, limits);
        }
    }

    private static ClientBatchItem withStructure(Function<TtlvStructure, ClientBatchItem> factory) {
        try (TtlvStructure structure = TtlvStructure.create()) {
            return factory.apply(structure);
        }
    }

    private static ClientBatchItem withTwoStructures(
            java.util.function.BiFunction<TtlvStructure, TtlvStructure, ClientBatchItem> factory) {
        try (TtlvStructure first = TtlvStructure.create();
                TtlvStructure second = TtlvStructure.create()) {
            return factory.apply(first, second);
        }
    }

    private static BatchRoute route(String operation, Supplier<ClientBatchItem> factory) {
        return new BatchRoute(operation, factory);
    }

    private static long operationRaw(String operation) {
        String fieldName = operation.replaceAll("[^A-Za-z0-9]+", "_")
                .toUpperCase(Locale.ROOT);
        try {
            return org.kmipkit.protocol.KmipValues.Operation.class
                    .getField(fieldName).getLong(null);
        } catch (ReflectiveOperationException exception) {
            throw new AssertionError("missing KMIP 2.1 Operation value: " + operation, exception);
        }
    }

    private record BatchRoute(String operation, Supplier<ClientBatchItem> factory) { }

    private record AsyncRoute(String operation, Runnable constructAndReadPayload) { }

    private record BatchResponseAccessor(
            String operation, Function<ClientBatchItemResult, ?> accessor) { }

    private record AsyncResponseAccessor(
            String operation, Function<AsyncOperationOutcome, ?> accessor) { }
}
