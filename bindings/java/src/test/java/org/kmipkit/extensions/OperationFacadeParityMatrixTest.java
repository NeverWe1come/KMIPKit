package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
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
        Map<String, Long> operationRawValues = operationFixture();
        Set<String> exercised = new HashSet<>();
        for (BatchRoute route : batchRoutes()) {
            assertTrue(exercised.add(route.operationId()), route.operationId());
            try (ClientBatch batch = ClientBatch.create();
                    ClientBatchItem item = route.factory().get()) {
                assertSame(batch, batch.add(item), route.operationId());
                assertEquals(1, batch.itemCount(), route.operationId());
                try (ClientBatchItem readBack = batch.itemAt(0)) {
                    assertEquals(0, readBack.extensionCount(), route.operationId());
                    assertEquals(
                            operationRawValues.get(route.operationId()),
                            readBack.operation(),
                            route.operationId());
                }
            }
        }

        for (AsyncRoute route : asyncRoutes()) {
            assertTrue(exercised.add(route.operationId()), route.operationId());
            route.constructAndReadPayload().run();
        }

        assertEquals(56, exercised.size());
        assertEquals(operationRawValues.keySet(), exercised);
        assertResponseAccessorMatrix(exercised);
    }

    private static void assertResponseAccessorMatrix(Set<String> exercisedOperations) {
        Set<String> batchResponseOperations = new HashSet<>();
        for (BatchResponseAccessor route : batchResponseAccessors()) {
            assertTrue(batchResponseOperations.add(route.operationId()), route.operationId());
            assertTrue(exercisedOperations.contains(route.operationId()), route.operationId());
            assertTrue(route.accessor() != null, route.operationId());
        }

        Set<String> asyncResponseOperations = new HashSet<>();
        for (AsyncResponseAccessor route : asyncResponseAccessors()) {
            assertTrue(asyncResponseOperations.add(route.operationId()), route.operationId());
            assertTrue(exercisedOperations.contains(route.operationId()), route.operationId());
            assertTrue(route.accessor() != null, route.operationId());
        }

        Set<String> allResponseOperations = new HashSet<>(batchResponseOperations);
        allResponseOperations.addAll(asyncResponseOperations);
        assertEquals(exercisedOperations, allResponseOperations);
    }

    /** Compile-time route map; result mapping and value assertions live in the operation tests. */
    private static List<BatchResponseAccessor> batchResponseAccessors() {
        return List.of(
                result("KMIPKIT-ELEM-OP-C2S-ACTIVATE", ClientBatchItemResult::activateResponse),
                result("KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE", ClientBatchItemResult::addAttributeResponse),
                result("KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE", ClientBatchItemResult::adjustAttributeResponse),
                result("KMIPKIT-ELEM-OP-C2S-ARCHIVE", ClientBatchItemResult::archiveResponse),
                result("KMIPKIT-ELEM-OP-C2S-CERTIFY", ClientBatchItemResult::certifyResponse),
                result("KMIPKIT-ELEM-OP-C2S-CHECK", ClientBatchItemResult::checkResponse),
                result("KMIPKIT-ELEM-OP-C2S-CREATE", ClientBatchItemResult::createResponse),
                result("KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR", ClientBatchItemResult::createKeyPairResponse),
                result("KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY", ClientBatchItemResult::createSplitKeyUniqueIdentifiers),
                result("KMIPKIT-ELEM-OP-C2S-DECRYPT", ClientBatchItemResult::decryptResponse),
                result("KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN", ClientBatchItemResult::delegatedLoginResponse),
                result("KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE", ClientBatchItemResult::deleteAttributeResponse),
                result("KMIPKIT-ELEM-OP-C2S-DERIVE-KEY", ClientBatchItemResult::deriveKeyResponse),
                result("KMIPKIT-ELEM-OP-C2S-DESTROY", ClientBatchItemResult::destroyResponse),
                result("KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS", ClientBatchItemResult::supportedVersions),
                result("KMIPKIT-ELEM-OP-C2S-ENCRYPT", ClientBatchItemResult::encryptResponse),
                result("KMIPKIT-ELEM-OP-C2S-EXPORT", ClientBatchItemResult::exportResponse),
                result("KMIPKIT-ELEM-OP-C2S-GET", ClientBatchItemResult::getResponse),
                result("KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES", ClientBatchItemResult::getAttributesResponse),
                result("KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST", ClientBatchItemResult::getAttributeListResponse),
                result("KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS", ClientBatchItemResult::getConstraintsResponse),
                result("KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION", ClientBatchItemResult::getUsageAllocationResponse),
                result("KMIPKIT-ELEM-OP-C2S-HASH", ClientBatchItemResult::hashResponse),
                result("KMIPKIT-ELEM-OP-C2S-IMPORT", ClientBatchItemResult::importResponse),
                result("KMIPKIT-ELEM-OP-C2S-INTEROP", ClientBatchItemResult::interopResponse),
                result("KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY", ClientBatchItemResult::joinSplitKeyResponse),
                result("KMIPKIT-ELEM-OP-C2S-LOCATE", ClientBatchItemResult::locateResponse),
                result("KMIPKIT-ELEM-OP-C2S-LOG", ClientBatchItemResult::logResponse),
                result("KMIPKIT-ELEM-OP-C2S-LOGIN", ClientBatchItemResult::loginResponse),
                result("KMIPKIT-ELEM-OP-C2S-LOGOUT", ClientBatchItemResult::logoutResponse),
                result("KMIPKIT-ELEM-OP-C2S-MAC", ClientBatchItemResult::macResponse),
                result("KMIPKIT-ELEM-OP-C2S-MAC-VERIFY", ClientBatchItemResult::macVerifyResponse),
                result("KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE", ClientBatchItemResult::modifyAttributeResponse),
                result("KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE", ClientBatchItemResult::obtainLeaseResponse),
                result("KMIPKIT-ELEM-OP-C2S-PING", ClientBatchItemResult::pingResponse),
                result("KMIPKIT-ELEM-OP-C2S-PKCS-11", ClientBatchItemResult::pkcs11Response),
                result("KMIPKIT-ELEM-OP-C2S-QUERY", ClientBatchItemResult::typedQueryResponse),
                result("KMIPKIT-ELEM-OP-C2S-RE-CERTIFY", ClientBatchItemResult::recertifyResponse),
                result("KMIPKIT-ELEM-OP-C2S-RECOVER", ClientBatchItemResult::recoverResponse),
                result("KMIPKIT-ELEM-OP-C2S-REGISTER", ClientBatchItemResult::registerResponse),
                result("KMIPKIT-ELEM-OP-C2S-RE-KEY", ClientBatchItemResult::rekeyResponse),
                result("KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR", ClientBatchItemResult::rekeyKeyPairResponse),
                result("KMIPKIT-ELEM-OP-C2S-RE-PROVISION", ClientBatchItemResult::reprovisionResponse),
                result("KMIPKIT-ELEM-OP-C2S-REVOKE", ClientBatchItemResult::revokeResponse),
                result("KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE", ClientBatchItemResult::rngRetrieveResponse),
                result("KMIPKIT-ELEM-OP-C2S-RNG-SEED", ClientBatchItemResult::rngSeedResponse),
                result("KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE", ClientBatchItemResult::setAttributeResponse),
                result("KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS", ClientBatchItemResult::setConstraintsResponse),
                result("KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS", ClientBatchItemResult::setDefaultsResponse),
                result("KMIPKIT-ELEM-OP-C2S-SIGN", ClientBatchItemResult::signResponse),
                result("KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY", ClientBatchItemResult::signatureVerifyResponse),
                result("KMIPKIT-ELEM-OP-C2S-VALIDATE", ClientBatchItemResult::validateResponse));
    }

    private static List<AsyncResponseAccessor> asyncResponseAccessors() {
        return List.of(
                new AsyncResponseAccessor("KMIPKIT-ELEM-OP-C2S-CANCEL", AsyncOperationOutcome::metadataV1),
                new AsyncResponseAccessor("KMIPKIT-ELEM-OP-C2S-POLL", AsyncOperationOutcome::metadataV1),
                new AsyncResponseAccessor("KMIPKIT-ELEM-OP-C2S-PROCESS", AsyncOperationOutcome::metadataV1),
                new AsyncResponseAccessor(
                        "KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS", AsyncOperationOutcome::responsePayloadTtlv));
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
        response.typedQueryResponse();
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
                route("KMIPKIT-ELEM-OP-C2S-ACTIVATE", () -> ClientBatchItem.activate(null)),
                route("KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE", () -> withCommentAttribute(ClientBatchItem::addAttribute)),
                route("KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE", () -> ClientBatchItem.adjustAttribute(
                        null, AttributeReference.tag(StandardAttribute.COMMENT.tag()), 1, null)),
                route("KMIPKIT-ELEM-OP-C2S-ARCHIVE", () -> ClientBatchItem.archive(null)),
                route("KMIPKIT-ELEM-OP-C2S-CERTIFY", () -> ClientBatchItem.certify(null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-CHECK", () -> ClientBatchItem.check(null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-CREATE", () -> withStructure(attributes ->
                        ClientBatchItem.create(0, attributes, null))),
                route("KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR", () -> ClientBatchItem.createKeyPair(
                        null, null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY", () -> withStructure(attributes ->
                        ClientBatchItem.createSplitKey(0, null, 1, 1, 0,
                                null, attributes, null))),
                route("KMIPKIT-ELEM-OP-C2S-DECRYPT", () -> ClientBatchItem.decrypt(
                        null, null, null, null, null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN", () -> withStructure(rights ->
                        ClientBatchItem.delegatedLogin(rights, null, null, null, null))),
                route("KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE", ClientBatchItem::deleteAttribute),
                route("KMIPKIT-ELEM-OP-C2S-DERIVE-KEY", () -> withTwoStructures((parameters, attributes) ->
                        ClientBatchItem.deriveKey(0, List.of(UniqueIdentifier.textString("matrix")),
                                0, parameters, attributes))),
                route("KMIPKIT-ELEM-OP-C2S-DESTROY", () -> ClientBatchItem.destroy(null)),
                route("KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS", ClientBatchItem::discoverVersions),
                route("KMIPKIT-ELEM-OP-C2S-ENCRYPT", () -> ClientBatchItem.encrypt(
                        null, null, null, null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-EXPORT", () -> ClientBatchItem.export(null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-GET", () -> ClientBatchItem.get(null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES", () -> ClientBatchItem.getAttributes(null, List.of())),
                route("KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST", ClientBatchItem::getAttributeList),
                route("KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS", ClientBatchItem::getConstraints),
                route("KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION", () -> ClientBatchItem.getUsageAllocation(null, 1)),
                route("KMIPKIT-ELEM-OP-C2S-HASH", () -> withStructure(parameters ->
                        ClientBatchItem.hash(parameters, null, null, null, null))),
                route("KMIPKIT-ELEM-OP-C2S-IMPORT", OperationFacadeParityMatrixTest::importObject),
                route("KMIPKIT-ELEM-OP-C2S-INTEROP", () -> ClientBatchItem.interop(0, new byte[] {0x41})),
                route("KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY", () -> {
                    try (SplitKeyReference reference = new SplitKeyReference(
                            UniqueIdentifier.textString("matrix"), 1)) {
                        return ClientBatchItem.joinSplitKey(0, List.of(reference));
                    }
                }),
                route("KMIPKIT-ELEM-OP-C2S-LOCATE", () -> withStructure(attributes ->
                        ClientBatchItem.locate(null, null, null, null, attributes))),
                route("KMIPKIT-ELEM-OP-C2S-LOG", () -> ClientBatchItem.log("matrix".getBytes(StandardCharsets.UTF_8))),
                route("KMIPKIT-ELEM-OP-C2S-LOGIN", () -> ClientBatchItem.login(null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-LOGOUT", () -> {
                    try (TtlvStructure ticket = ticket()) {
                        return ClientBatchItem.logout(ticket);
                    }
                }),
                route("KMIPKIT-ELEM-OP-C2S-MAC", () -> ClientBatchItem.mac(null, null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-MAC-VERIFY", () -> ClientBatchItem.macVerify(
                        null, null, null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE", () -> withCommentAttribute(item ->
                        ClientBatchItem.modifyAttribute(null, null, item))),
                route("KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE", () -> ClientBatchItem.obtainLease(null)),
                route("KMIPKIT-ELEM-OP-C2S-PING", ClientBatchItem::ping),
                route("KMIPKIT-ELEM-OP-C2S-PKCS-11", () -> withStructure(function ->
                        ClientBatchItem.pkcs11(function, null, null, null))),
                route("KMIPKIT-ELEM-OP-C2S-QUERY", () -> ClientBatchItem.query(new int[] {0}, null)),
                route("KMIPKIT-ELEM-OP-C2S-RE-CERTIFY", () -> ClientBatchItem.recertify(
                        null, null, null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-RECOVER", () -> ClientBatchItem.recover(null)),
                route("KMIPKIT-ELEM-OP-C2S-REGISTER", OperationFacadeParityMatrixTest::register),
                route("KMIPKIT-ELEM-OP-C2S-RE-KEY", () -> ClientBatchItem.rekey(null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR", () -> ClientBatchItem.rekeyKeyPair(
                        null, null, null, null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-RE-PROVISION", () -> ClientBatchItem.reprovision(null, null)),
                route("KMIPKIT-ELEM-OP-C2S-REVOKE", OperationFacadeParityMatrixTest::revoke),
                route("KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE", () -> ClientBatchItem.rngRetrieve(0)),
                route("KMIPKIT-ELEM-OP-C2S-RNG-SEED", () -> ClientBatchItem.rngSeed(new byte[] {0x01})),
                route("KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE", () -> withCommentAttribute(ClientBatchItem::setAttribute)),
                route("KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS", () -> withStructure(ClientBatchItem::setConstraints)),
                route("KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS", ClientBatchItem::setDefaults),
                route("KMIPKIT-ELEM-OP-C2S-SIGN", () -> ClientBatchItem.sign(
                        null, null, null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY", () -> ClientBatchItem.signatureVerify(
                        null, null, null, null, null, null, null, null)),
                route("KMIPKIT-ELEM-OP-C2S-VALIDATE", () -> ClientBatchItem.validate(List.of(), List.of(), null)));
    }

    private static List<AsyncRoute> asyncRoutes() {
        return List.of(
                new AsyncRoute("KMIPKIT-ELEM-OP-C2S-CANCEL", () -> {
                    try (CancelRequest request = CancelRequest.create(CORRELATION);
                            TtlvStructure payload = request.toTtlvPayload()) {
                        assertCorrelationPayload(payload, CORRELATION_VALUE_TAG, CORRELATION);
                    }
                }),
                new AsyncRoute("KMIPKIT-ELEM-OP-C2S-POLL", () -> {
                    try (PollRequest request = PollRequest.create(CORRELATION);
                            TtlvStructure payload = request.toTtlvPayload()) {
                        assertCorrelationPayload(payload, CORRELATION_VALUE_TAG, CORRELATION);
                    }
                }),
                new AsyncRoute("KMIPKIT-ELEM-OP-C2S-PROCESS", () -> {
                    try (ProcessRequest request = ProcessRequest.create(CORRELATION);
                            TtlvStructure payload = request.toTtlvPayload()) {
                        assertCorrelationPayload(payload, CORRELATION_VALUE_TAG, CORRELATION);
                    }
                }),
                new AsyncRoute("KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS", () -> {
                    try (QueryAsyncRequestsRequest request = QueryAsyncRequestsRequest.create();
                            QueryAsyncRequestsRequest withCorrelations = request.withCorrelationValues(
                                    List.of(CORRELATION, SECOND_CORRELATION));
                            QueryAsyncRequestsRequest filtered = withCorrelations.withOperations(
                                    List.of(0x1aL));
                            TtlvStructure payload = filtered.toTtlvPayload();
                            TtlvStructureView view = payload.view()) {
                        assertEquals(2, view.itemCount(), "KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS");
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

    private static BatchRoute route(String operationId, Supplier<ClientBatchItem> factory) {
        return new BatchRoute(operationId, factory);
    }

    private static Map<String, Long> operationFixture() {
        Path fixture = Path.of("..", "..", "tests", "fixtures", "api",
                "operation_catalog_parity.tsv");
        try {
            List<String> lines = Files.readAllLines(fixture, StandardCharsets.UTF_8);
            if (lines.isEmpty() || !lines.get(0).equals("operationId\toperationRaw")) {
                throw new AssertionError(
                        "invalid shared KMIP Operation fixture header: " + fixture);
            }
            Map<String, Long> values = new HashMap<>();
            for (String line : lines.subList(1, lines.size())) {
                String[] columns = line.split("\\t", -1);
                if (columns.length != 2 || values.putIfAbsent(
                        columns[0], Long.parseLong(columns[1])) != null) {
                    throw new AssertionError("invalid or duplicate operation fixture row: " + line);
                }
            }
            return Map.copyOf(values);
        } catch (IOException exception) {
            throw new AssertionError(
                    "unable to read shared KMIP Operation fixture: " + fixture, exception);
        }
    }

    private record BatchRoute(String operationId, Supplier<ClientBatchItem> factory) { }

    private record AsyncRoute(String operationId, Runnable constructAndReadPayload) { }

    private record BatchResponseAccessor(
            String operationId, Function<ClientBatchItemResult, ?> accessor) { }

    private record AsyncResponseAccessor(
            String operationId, Function<AsyncOperationOutcome, ?> accessor) { }
}
