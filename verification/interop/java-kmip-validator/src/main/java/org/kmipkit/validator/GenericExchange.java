package org.kmipkit.validator;

import org.kmipkit.message.ResponseMessage;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.protocol.VersionGate;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;

import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;
import java.io.ByteArrayOutputStream;
import java.io.EOFException;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.Socket;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.GeneralSecurityException;
import java.security.KeyFactory;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.spec.InvalidKeySpecException;
import java.security.spec.PKCS8EncodedKeySpec;
import java.util.Arrays;
import java.util.Base64;
import java.util.HexFormat;
import java.util.List;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Predicate;

final class GenericExchange {
    private static final CodecLimits LIMITS = CodecLimits.defaults();
    private static final int CREATE = 0x01;
    private static final int CREATE_KEY_PAIR = 0x02;
    private static final int REGISTER = 0x03;
    private static final int RE_KEY = 0x04;
    private static final int DERIVE_KEY = 0x05;
    private static final int CERTIFY = 0x06;
    private static final int RE_CERTIFY = 0x07;
    private static final int CHECK = 0x09;
    private static final int GET = 0x0a;
    private static final int GET_ATTRIBUTES = 0x0b;
    private static final int GET_ATTRIBUTE_LIST = 0x0c;
    private static final int ADD_ATTRIBUTE = 0x0d;
    private static final int MODIFY_ATTRIBUTE = 0x0e;
    private static final int DELETE_ATTRIBUTE = 0x0f;
    private static final int SET_ATTRIBUTE = 0x31;
    private static final int DESCRIPTION = 0x4200fc;
    private static final byte[] CHANGED_DESCRIPTION = "kmipkit-changed-description".getBytes(StandardCharsets.US_ASCII);
    private static final int OBTAIN_LEASE = 0x10;
    private static final int GET_USAGE_ALLOCATION = 0x11;
    private static final int ACTIVATE = 0x12;
    private static final int REVOKE = 0x13;
    private static final int DESTROY = 0x14;
    private static final int ARCHIVE = 0x15;
    private static final int RECOVER = 0x16;
    private static final int VALIDATE = 0x17;
    private static final int CANCEL = 0x19;
    private static final int POLL = 0x1a;
    private static final int RE_KEY_KEY_PAIR = 0x1d;
    private static final int ENCRYPT = 0x1f;
    private static final int DECRYPT = 0x20;
    private static final int SIGN = 0x21;
    private static final int SIGNATURE_VERIFY = 0x22;
    private static final int MAC = 0x23;
    private static final int MAC_VERIFY = 0x24;
    private static final int CREATE_SPLIT_KEY = 0x28;
    private static final int JOIN_SPLIT_KEY = 0x29;
    private static final int IMPORT = 0x2a;
    private static final int EXPORT = 0x2b;
    private static final int LOCATE = 0x08;
    private static final int QUERY = 0x18;
    private static final int DISCOVER_VERSIONS = 0x1e;
    private static final int RNG_RETRIEVE = 0x25;
    private static final int RNG_SEED = 0x26;
    private static final byte[] RNG_SEED_INPUT = "A".repeat(32).getBytes(StandardCharsets.US_ASCII);
    private static final int HASH = 0x27;
    private static final byte[] TEXT_ATTRIBUTE_FIXTURE = "kmipkit-attribute-fixture".getBytes(StandardCharsets.US_ASCII);
    private static final byte[] HASH_INPUT = "abc".getBytes(StandardCharsets.US_ASCII);
    private static final byte[] HASH_DIGEST = HexFormat.of().parseHex(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    private static final int PING = 0x3b;

    private GenericExchange() { }

    static boolean supports(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        int operation = operation(testCase, connection);
        return "server_exchange".equals(testCase.execution().mode()) && operation != 0
                && ((usesOwnedKey(operation) || usesExtendedOp(operation))
                        ? "mutating".equals(testCase.execution().mutation())
                        : ("read_only".equals(testCase.execution().mutation()) || operation == RNG_RETRIEVE || operation == RNG_SEED
                            || JavaKmipDriver.multipartHash(testCase)))
                && "raw_tls".equals(connection.transport()) && "TTLV".equals(connection.encoding())
                && connection.usernamePassword() == null;
    }

    private static int operation(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        String version = connection.protocolVersion();
        if (!"2.0".equals(version) && !"2.1".equals(version)) return 0;
        if (keyAttributeReference(testCase.caseId()) != 0) return GET_ATTRIBUTES;
        if (JavaKmipDriver.rotateLocateCase(testCase)) return LOCATE;
        if ("Query".equals(testCase.operation()) && testCase.requires().has("query_function")) return QUERY;
        return switch (testCase.caseId()) {
            case "test-op-create" -> CREATE;
            case "test-op-create-key-pair" -> CREATE_KEY_PAIR;
            case "test-op-register" -> REGISTER;
            case "test-op-re-key" -> RE_KEY;
            case "test-op-derive-key" -> DERIVE_KEY;
            case "test-op-certify" -> CERTIFY;
            case "test-op-re-certify" -> RE_CERTIFY;
            case "test-op-check" -> CHECK;
            case "test-op-get" -> GET;
            case "test-op-get-attribute-list" -> GET_ATTRIBUTE_LIST;
            case "test-op-add-attribute" -> ADD_ATTRIBUTE;
            case "test-op-modify-attribute" -> MODIFY_ATTRIBUTE;
            case "test-op-delete-attribute" -> DELETE_ATTRIBUTE;
            case "test-op-set-attribute" -> SET_ATTRIBUTE;
            case "test-op-obtain-lease" -> OBTAIN_LEASE;
            case "test-op-get-usage-allocation" -> GET_USAGE_ALLOCATION;
            case "test-op-activate" -> ACTIVATE;
            case "test-op-revoke" -> REVOKE;
            case "test-op-destroy" -> DESTROY;
            case "test-op-archive" -> ARCHIVE;
            case "test-op-recover" -> RECOVER;
            case "test-op-validate" -> VALIDATE;
            case "test-op-cancel" -> CANCEL;
            case "test-op-poll" -> POLL;
            case "test-op-re-key-key-pair" -> RE_KEY_KEY_PAIR;
            case "test-op-locate" -> LOCATE;
            case "test-op-query" -> QUERY;
            case "test-op-discover-versions" -> DISCOVER_VERSIONS;
            case "test-op-rng-retrieve", "test-crypto-rng-retrieve-rng-32" -> RNG_RETRIEVE;
            case "test-op-rng-seed", "test-crypto-rng-seed-rng-32" -> RNG_SEED;
            case "test-op-hash", "test-crypto-hash-sha_256-0", "test-multipart-hash-sha256" -> HASH;
            case "test-op-encrypt", "test-crypto-encrypt-aes-256-cbc", "test-crypto-encrypt-aes-256-gcm" -> ENCRYPT;
            case "test-op-decrypt", "test-crypto-decrypt-aes-256-cbc", "test-crypto-decrypt-aes-256-gcm",
                    "test-crypto-decrypt-aes-256-gcm-invalid-tag" -> DECRYPT;
            case "test-op-sign", "test-crypto-sign-rsa-2048-pkcs1_v1_5", "test-crypto-sign-ecdsa-256-p_256" -> SIGN;
            case "test-op-signature-verify", "test-crypto-signature-verify-rsa-2048-pkcs1_v1_5",
                    "test-crypto-signature-verify-ecdsa-256-p_256" -> SIGNATURE_VERIFY;
            case "test-op-mac", "test-crypto-mac-hmac_sha256-256" -> MAC;
            case "test-op-mac-verify", "test-crypto-mac-verify-hmac_sha256-256" -> MAC_VERIFY;
            case "test-op-create-split-key" -> CREATE_SPLIT_KEY;
            case "test-op-join-split-key" -> JOIN_SPLIT_KEY;
            case "test-op-import" -> IMPORT;
            case "test-op-export" -> EXPORT;
            case "test-op-ping" -> "2.1".equals(version) ? PING : 0;
            default -> 0;
        };
    }

    static JavaKmipDriver.CaseOutcome run(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        long start = System.nanoTime();
        int minor = "2.1".equals(connection.protocolVersion()) ? 1 : 0;
        if (JavaKmipDriver.multipartHash(testCase)) return multipartHash(connection, minor, start);
        if (JavaKmipDriver.rotateLocateCase(testCase)) return locateRotateAttribute(testCase, connection, minor, start);
        int operation = operation(testCase, connection);
        if (operation == QUERY && testCase.requires().has("query_function"))
            return queryField(testCase, connection, minor, start);
        int attributeTag = keyAttributeReference(testCase.caseId());
        if (usesOwnedKey(operation)) return runOwnedKey(connection, operation, minor, attributeTag, start);
        if (usesExtendedOp(operation)) return runExtended(testCase, connection, operation, minor, start);
        return execute(connection, operation, minor, null, payload -> matches(payload, operation, minor), start);
    }

    private static JavaKmipDriver.CaseOutcome queryField(CatalogLoader.CatalogCase testCase,
            ConnectionConfig connection, int minor, long start) {
        int function = testCase.requires().path("query_function").asInt(-1);
        int resultTag = testCase.requires().path("query_result_tag").asInt(-1);
        if (function < 0 || resultTag < 0)
            return JavaKmipDriver.CaseOutcome.failed("Invalid Query field fixture.", elapsed(start));
        try (var payload = structureItem(0x420079, item(0x420074, TtlvValue.Enumeration(function)))) {
            return executePayload(connection, QUERY, minor, payload,
                    result -> hasQueryStructure(result, resultTag), start);
        }
    }

    static boolean hasQueryStructure(TtlvStructureView payload, int resultTag) {
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var child = payload.itemAt(index); var tag = child.tag()) {
                if (tag.raw() == resultTag && child.itemType() == TtlvItemType.Structure) {
                    try (var value = child.value(); var structure = value.structure()) {
                        if (structure.itemCount() > 0) return true;
                    }
                }
            }
        }
        return false;
    }

    private static int keyAttributeReference(String caseId) {
        return switch (caseId) {
            case "test-attr-cryptographic-algorithm" -> 0x420028;
            case "test-op-get-attributes", "test-attr-cryptographic-length" -> 0x42002a;
            case "test-attr-cryptographic-usage-mask" -> 0x42002c;
            case "test-attr-object-type" -> 0x420057;
            case "test-attr-state" -> 0x42008d;
            case "test-attr-contact-information" -> 0x420022;
            case "test-attr-description" -> 0x4200fc;
            case "test-attr-comment" -> 0x4200fd;
            case "test-attr-sensitive" -> 0x420120;
            case "test-attr-always-sensitive" -> 0x420121;
            case "test-attr-extractable" -> 0x420122;
            case "test-attr-never-extractable" -> 0x420123;
            default -> 0;
        };
    }

    private static boolean usesOwnedKey(int operation) {
        return switch (operation) {
            case CREATE, CHECK, GET, GET_ATTRIBUTES, GET_ATTRIBUTE_LIST, OBTAIN_LEASE,
                    ACTIVATE, REVOKE, DESTROY, ARCHIVE, RECOVER -> true;
            default -> changesDescription(operation);
        };
    }

    private static boolean changesDescription(int operation) {
        return operation == ADD_ATTRIBUTE || operation == MODIFY_ATTRIBUTE
                || operation == DELETE_ATTRIBUTE || operation == SET_ATTRIBUTE;
    }

    private static JavaKmipDriver.CaseOutcome runOwnedKey(ConnectionConfig connection, int operation,
            int minor, int attributeTag, long start) {
        var createdId = new AtomicReference<byte[]>();
        int initialTag = changesDescription(operation) && operation != ADD_ATTRIBUTE ? DESCRIPTION : attributeTag;
        var create = execute(connection, CREATE, minor, null, initialTag, payload -> {
            byte[] id = textIdentifier(payload);
            createdId.set(id);
            return id != null && hasField(payload, 0x420057, TtlvItemType.Enumeration, 2);
        }, start);
        byte[] id = createdId.get();
        if (id == null) {
            if (operation == CREATE || "failed".equals(create.status())) return create;
            return JavaKmipDriver.CaseOutcome.notRun("Server rejected isolated Create setup.", elapsed(start));
        }
        Predicate<TtlvStructureView> sameId = payload -> Arrays.equals(id, textIdentifier(payload));
        if ("passed".equals(create.status()) && operation == DESTROY) {
            return execute(connection, DESTROY, minor, id, sameId, start);
        }

        var target = create;
        if ("passed".equals(create.status()) && operation == GET) {
            target = execute(connection, GET, minor, id, payload -> sameId.test(payload)
                    && hasField(payload, 0x420057, TtlvItemType.Enumeration, 2)
                    && hasField(payload, 0x42008f, TtlvItemType.Structure, 0), start);
        }
        if ("passed".equals(create.status()) && operation == OBTAIN_LEASE) {
            target = execute(connection, OBTAIN_LEASE, minor, id, payload -> sameId.test(payload)
                    && hasField(payload, 0x420049, TtlvItemType.Interval, 0)
                    && hasField(payload, 0x420048, TtlvItemType.DateTime, 0), start);
        }
        if ("passed".equals(create.status()) && operation == GET_ATTRIBUTES) {
            target = execute(connection, GET_ATTRIBUTES, minor, id, attributeTag,
                    payload -> sameId.test(payload) && hasKeyAttribute(payload, attributeTag), start);
        }
        if ("passed".equals(create.status()) && operation == GET_ATTRIBUTE_LIST) {
            target = execute(connection, GET_ATTRIBUTE_LIST, minor, id,
                    payload -> sameId.test(payload) && hasLengthReference(payload), start);
        }
        if ("passed".equals(create.status()) && (operation == ACTIVATE || operation == REVOKE || operation == CHECK)) {
            var activate = execute(connection, ACTIVATE, minor, id, sameId, start);
            target = activate;
            if ("passed".equals(activate.status()) && operation == CHECK) {
                target = execute(connection, CHECK, minor, id,
                        payload -> sameId.test(payload) && matchesCheckUsage(payload), start);
            }
            if (activate.resultStatusCode() == null || activate.resultStatusCode() == 0) {
                var revoke = execute(connection, REVOKE, minor, id, sameId, start);
                if ("passed".equals(activate.status())) {
                    if (operation == REVOKE) target = revoke;
                    else if (!"passed".equals(revoke.status())) {
                        target = JavaKmipDriver.CaseOutcome.failed("Cleanup Revoke failed for the activated key.", elapsed(start));
                    }
                }
            } else if (operation != ACTIVATE) {
                target = JavaKmipDriver.CaseOutcome.notRun("Server rejected isolated Activate setup.", elapsed(start));
            }
        }
        if ("passed".equals(create.status()) && changesDescription(operation)) {
            target = changeDescription(connection, operation, minor, id, sameId, start);
        }
        if ("passed".equals(create.status()) && (operation == ARCHIVE || operation == RECOVER)) {
            var archive = execute(connection, ARCHIVE, minor, id, sameId, start);
            target = archive;
            if (Long.valueOf(0).equals(archive.resultStatusCode())) {
                var recover = execute(connection, RECOVER, minor, id, sameId, start);
                if ("passed".equals(archive.status())) {
                    target = operation == RECOVER ? recover : archive;
                    if (operation == ARCHIVE && !"passed".equals(recover.status())) {
                        target = JavaKmipDriver.CaseOutcome.failed("Cleanup Recover failed.", elapsed(start));
                    }
                }
            } else if (operation == RECOVER && !"failed".equals(archive.status())) {
                target = JavaKmipDriver.CaseOutcome.notRun("Server rejected Archive setup.", elapsed(start));
            }
        }

        var cleanup = execute(connection, DESTROY, minor, id, sameId, start);
        return "passed".equals(cleanup.status()) ? target
                : JavaKmipDriver.CaseOutcome.failed("Cleanup Destroy failed for the isolated key.", elapsed(start));
    }

    private record HashFrame(int index, byte[] chunk, byte[] correlation) { }

    private static JavaKmipDriver.CaseOutcome multipartHash(ConnectionConfig connection, int minor, long start) {
        byte[] expected = HexFormat.of().parseHex("484e69bd8ae45ed2440bf93f35e464f86ed9557258f0377c0c070a4f786b5b0a");
        var correlation = new AtomicReference<byte[]>();
        var chunks = List.of("KMIP ", "Hash", "!");
        try {
            JavaKmipDriver.CaseOutcome outcome = null;
            for (int index = 0; index < chunks.size(); index++) {
                var frame = new HashFrame(index, chunks.get(index).getBytes(StandardCharsets.US_ASCII), correlation.get());
                outcome = execute(connection, HASH, minor, null, 0, frame, payload -> {
                    if (frame.index() == 1) return payload.itemCount() == 0;
                    if (payload.itemCount() != 1) return false;
                    try (var field = payload.itemAt(0); var tag = field.tag(); var value = field.value()) {
                        if (field.itemType() != TtlvItemType.ByteString) return false;
                        if (frame.index() == 0 && tag.raw() == 0x4200d6) {
                            byte[] bytes = value.bytes();
                            correlation.set(bytes);
                            return bytes.length != 0;
                        }
                        return frame.index() == 2 && tag.raw() == 0x4200c2
                                && Arrays.equals(value.bytes(), expected);
                    }
                }, start);
                if (!"passed".equals(outcome.status())) return outcome;
            }
            return outcome;
        } finally {
            byte[] bytes = correlation.get();
            if (bytes != null) Arrays.fill(bytes, (byte) 0);
        }
    }

    private static JavaKmipDriver.CaseOutcome execute(ConnectionConfig connection, int operation, int minor,
            byte[] identifier, Predicate<TtlvStructureView> expected, long start) {
        return execute(connection, operation, minor, identifier, 0, expected, start);
    }

    private static JavaKmipDriver.CaseOutcome execute(ConnectionConfig connection, int operation, int minor,
            byte[] identifier, int attributeTag, Predicate<TtlvStructureView> expected, long start) {
        return execute(connection, operation, minor, identifier, attributeTag, null, expected, start);
    }

    private static JavaKmipDriver.CaseOutcome execute(ConnectionConfig connection, int operation, int minor,
            byte[] identifier, int attributeTag, HashFrame frame, Predicate<TtlvStructureView> expected, long start) {
        return executeConstructed(connection, operation, minor,
                () -> request(operation, minor, identifier, attributeTag, frame, null), expected, start);
    }

    private static JavaKmipDriver.CaseOutcome executeConstructed(ConnectionConfig connection, int operation, int minor,
            java.util.function.Supplier<TtlvItem> builder, Predicate<TtlvStructureView> expected, long start) {
        try (var request = builder.get(); var view = request.view()) {
            byte[] wire = exchange(connection, encode(view));
            try {
                return check(wire, operation, minor, start, expected);
            } finally {
                Arrays.fill(wire, (byte) 0);
            }
        } catch (GeneralSecurityException | IOException | RuntimeException error) {
            return JavaKmipDriver.CaseOutcome.failed("Generic TTLV exchange or validation failed ("
                    + error.getClass().getSimpleName() + ").", elapsed(start));
        }
    }

    private static JavaKmipDriver.CaseOutcome locateRotateAttribute(CatalogLoader.CatalogCase testCase,
            ConnectionConfig connection, int minor, long start) {
        try (var attribute = JavaKmipDriver.rotateAttribute(testCase)) {
            if ("NotSent".equals(testCase.requires().path("expected_status").asText())) {
                try (var view = attribute.view(); var tag = view.tag()) {
                    return VersionGate.validateTag(2, minor, tag.raw())
                            ? JavaKmipDriver.CaseOutcome.failed("The Rotate tag was accepted under KMIP 2.0.", elapsed(start))
                            : JavaKmipDriver.CaseOutcome.localModelPassed(elapsed(start));
                }
            }
            return executeConstructed(connection, LOCATE, minor,
                    () -> request(LOCATE, minor, null, 0, null, attribute), GenericExchange::matchesLocate, start);
        } catch (RuntimeException exception) {
            return JavaKmipDriver.CaseOutcome.failed("Invalid Rotate attribute fixture.", elapsed(start));
        }
    }

    private static JavaKmipDriver.CaseOutcome changeDescription(ConnectionConfig connection, int operation,
            int minor, byte[] id, Predicate<TtlvStructureView> sameId, long start) {
        byte[] initial = operation == ADD_ATTRIBUTE ? null : TEXT_ATTRIBUTE_FIXTURE;
        var before = execute(connection, GET_ATTRIBUTES, minor, id, DESCRIPTION,
                payload -> sameId.test(payload) && descriptionMatches(payload, initial), start);
        if (!"passed".equals(before.status())) {
            return JavaKmipDriver.CaseOutcome.failed("Could not read the initial Description.", elapsed(start));
        }
        var target = execute(connection, operation, minor, id, sameId, start);
        if ("passed".equals(target.status())) {
            byte[] expected = operation == DELETE_ATTRIBUTE ? null : CHANGED_DESCRIPTION;
            var after = execute(connection, GET_ATTRIBUTES, minor, id, DESCRIPTION,
                    payload -> sameId.test(payload) && descriptionMatches(payload, expected), start);
            if (!"passed".equals(after.status())) {
                return JavaKmipDriver.CaseOutcome.failed("Could not verify the Description after mutation.", elapsed(start));
            }
        }
        return target;
    }

    private static boolean descriptionMatches(TtlvStructureView payload, byte[] expected) {
        if (expected == null) return payload.itemCount() == 1;
        if (payload.itemCount() != 2) return false;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag()) {
                if (tag.raw() != 0x420125) continue;
                if (field.itemType() != TtlvItemType.Structure) return false;
                try (var value = field.value(); var attributes = value.structure()) {
                    if (attributes.itemCount() != 1) return false;
                    try (var attr = attributes.itemAt(0); var attrTag = attr.tag(); var text = attr.value()) {
                        return attrTag.raw() == DESCRIPTION && attr.itemType() == TtlvItemType.TextString
                                && Arrays.equals(expected, text.bytes());
                    }
                }
            }
        }
        return false;
    }

    private static TtlvItem request(int operation, int minor, byte[] objectId, int attributeTag,
            HashFrame frame, TtlvItem rotate) {
        try (var version = version(minor);
                var count = item(0x42000d, TtlvValue.Integer(1));
                var header = structureItem(0x420077, version, count);
                var identifier = item(0x42005c, TtlvValue.Enumeration(operation));
                var payload = switch (operation) {
                    case CREATE -> createPayload(attributeTag);
                    case CHECK -> structureItem(0x420079,
                            item(0x420094, TtlvValue.TextString(objectId, LIMITS)),
                            item(0x42002c, TtlvValue.Integer(4)));
                    case GET_ATTRIBUTES -> structureItem(0x420079,
                            item(0x420094, TtlvValue.TextString(objectId, LIMITS)),
                            item(0x42013b, TtlvValue.Enumeration(attributeTag)));
                    case ADD_ATTRIBUTE, MODIFY_ATTRIBUTE, DELETE_ATTRIBUTE, SET_ATTRIBUTE ->
                            descriptionPayload(operation, objectId);
                    case REVOKE -> revokePayload(objectId);
                    case GET, GET_ATTRIBUTE_LIST, OBTAIN_LEASE, ACTIVATE, DESTROY, ARCHIVE, RECOVER -> structureItem(0x420079,
                            item(0x420094, TtlvValue.TextString(objectId, LIMITS)));
                    case LOCATE -> locatePayload(rotate);
                    case QUERY -> structureItem(0x420079, item(0x420074, TtlvValue.Enumeration(1)));
                    case DISCOVER_VERSIONS -> structureItem(0x420079, version(minor));
                    case RNG_RETRIEVE -> structureItem(0x420079, item(0x4200c4, TtlvValue.Integer(32)));
                    case RNG_SEED -> structureItem(0x420079,
                            item(0x4200c2, TtlvValue.ByteString(RNG_SEED_INPUT, LIMITS)));
                    case HASH -> hashPayload(frame);
                    case PING -> structureItem(0x420079);
                    default -> throw new IllegalArgumentException("Unsupported generic operation");
                };
                var batch = structureItem(0x42000f, identifier, payload)) {
            return structureItem(0x420078, header, batch);
        }
    }

    private static TtlvItem locatePayload(TtlvItem rotate) {
        try (var maximum = item(0x42004f, TtlvValue.Integer(1));
                var offset = item(0x4200d4, TtlvValue.Integer(0))) {
            if (rotate == null) {
                try (var attributes = structureItem(0x420125)) {
                    return structureItem(0x420079, maximum, offset, attributes);
                }
            }
            try (var attributes = structureItem(0x420125, rotate)) {
                return structureItem(0x420079, maximum, offset, attributes);
            }
        }
    }

    private static TtlvItem descriptionPayload(int operation, byte[] objectId) {
        try (var id = item(0x420094, TtlvValue.TextString(objectId, LIMITS))) {
            if (operation == DELETE_ATTRIBUTE) {
                try (var reference = item(0x42013b, TtlvValue.Enumeration(DESCRIPTION))) {
                    return structureItem(0x420079, id, reference);
                }
            }
            try (var replacement = structureItem(0x42013d,
                    item(DESCRIPTION, TtlvValue.TextString(CHANGED_DESCRIPTION, LIMITS)))) {
                if (operation == MODIFY_ATTRIBUTE) {
                    try (var current = structureItem(0x42013c,
                            item(DESCRIPTION, TtlvValue.TextString(TEXT_ATTRIBUTE_FIXTURE, LIMITS)))) {
                        return structureItem(0x420079, id, current, replacement);
                    }
                }
                return structureItem(0x420079, id, replacement);
            }
        }
    }

    private static TtlvItem revokePayload(byte[] objectId) {
        try (var code = item(0x420082, TtlvValue.Enumeration(6));
                var reason = structureItem(0x420081, code);
                var identifier = item(0x420094, TtlvValue.TextString(objectId, LIMITS))) {
            return structureItem(0x420079, identifier, reason);
        }
    }

    private static TtlvItem createPayload(int attributeTag) {
        try (var algorithm = item(0x420028, TtlvValue.Enumeration(3));
                var length = item(0x42002a, TtlvValue.Integer(256));
                var usage = item(0x42002c, TtlvValue.Integer(4));
                var sensitive = item(0x420120, TtlvValue.Boolean(true));
                var extractable = item(0x420122, TtlvValue.Boolean(false));
                var attributes = switch (attributeTag) {
                    case 0x420120, 0x420121, 0x420122, 0x420123 ->
                            structureItem(0x420125, algorithm, length, usage, sensitive, extractable);
                    case 0x420022, 0x4200fc, 0x4200fd ->
                            structureItem(0x420125, algorithm, length, usage,
                                    item(attributeTag, TtlvValue.TextString(TEXT_ATTRIBUTE_FIXTURE, LIMITS)));
                    default -> structureItem(0x420125, algorithm, length, usage);
                };
                var objectType = item(0x420057, TtlvValue.Enumeration(2))) {
            return structureItem(0x420079, objectType, attributes);
        }
    }

    private static TtlvItem hashPayload(HashFrame frame) {
        try (var parameters = structureItem(0x42002b, item(0x420038, TtlvValue.Enumeration(6)));
                var data = item(0x4200c2, TtlvValue.ByteString(frame == null ? HASH_INPUT : frame.chunk(), LIMITS))) {
            if (frame == null) return structureItem(0x420079, parameters, data);
            if (frame.index() == 0) {
                try (var init = item(0x4200d7, TtlvValue.Boolean(true))) {
                    return structureItem(0x420079, parameters, data, init);
                }
            }
            try (var correlation = item(0x4200d6, TtlvValue.ByteString(frame.correlation(), LIMITS))) {
                if (frame.index() == 1) return structureItem(0x420079, parameters, data, correlation);
                try (var end = item(0x4200d8, TtlvValue.Boolean(true))) {
                    return structureItem(0x420079, parameters, data, correlation, end);
                }
            }
        }
    }

    private static TtlvItem version(int minor) {
        try (var major = item(0x42006a, TtlvValue.Integer(2));
                var minorItem = item(0x42006b, TtlvValue.Integer(minor))) {
            return structureItem(0x420069, major, minorItem);
        }
    }

    static TtlvItem item(int code, TtlvValue value) {
        try (value; var raw = RawTag.fromRaw(code); var tag = raw.checked()) {
            return TtlvItem.create(tag, value, LIMITS);
        }
    }

    // withItem consumes its arguments. Closing already consumed handles is harmless.
    static TtlvItem structureItem(int code, TtlvItem... items) {
        TtlvStructure structure = TtlvStructure.create();
        try {
            for (var child : items) {
                structure = TtlvStructure.withItem(structure, child, LIMITS);
            }
            return item(code, TtlvValue.structure(structure, LIMITS));
        } finally {
            structure.close();
            for (var child : items) child.close();
        }
    }

    static byte[] encode(TtlvItemView item) {
        try (var tag = item.tag(); var value = item.value()) {
            byte[] body = switch (item.itemType()) {
                case Structure -> {
                    var children = new ByteArrayOutputStream();
                    try (var structure = value.structure()) {
                        for (long index = 0; index < structure.itemCount(); index++) {
                            try (var child = structure.itemAt(index)) {
                                children.writeBytes(encode(child));
                            }
                        }
                    }
                    yield children.toByteArray();
                }
                case Integer -> ByteBuffer.allocate(4).putInt(value.integer()).array();
                case Enumeration -> ByteBuffer.allocate(4).putInt((int) value.enumeration()).array();
                case LongInteger -> ByteBuffer.allocate(8).putLong(value.longInteger()).array();
                case BigInteger -> {
                    byte[] integer = value.bytes();
                    if (integer.length == 0 || integer.length % 8 != 0) {
                        throw new IllegalArgumentException("Invalid Big Integer size");
                    }
                    yield integer;
                }
                case Boolean -> ByteBuffer.allocate(8).putLong(value.booleanValue() ? 1 : 0).array();
                case ByteString, TextString -> value.bytes();
                case DateTime -> ByteBuffer.allocate(8).putLong(value.dateTime()).array();
                case Interval -> ByteBuffer.allocate(4).putInt((int) value.interval()).array();
                case DateTimeExtended -> ByteBuffer.allocate(8).putLong(value.dateTimeExtended()).array();
                default -> throw new IllegalArgumentException("Unsupported generic probe item type");
            };
            return ByteBuffer.allocate(8 + ((body.length + 7) & ~7))
                    .putInt((tag.raw() << 8) | item.itemType().code()).putInt(body.length).put(body).array();
        }
    }

    private static byte[] exchange(ConnectionConfig connection, byte[] request)
            throws GeneralSecurityException, IOException {
        var tls = connection.tls();
        var endpoint = new InetSocketAddress(connection.host(), connection.port());
        String peerName = tls.serverNameOverride().isBlank() ? connection.host() : tls.serverNameOverride();
        try (var tcp = new Socket()) {
            tcp.connect(endpoint, Math.multiplyExact(tls.connectTimeoutSeconds(), 1000));
            try (var socket = (SSLSocket) context(tls).getSocketFactory().createSocket(
                    tcp, peerName, connection.port(), true)) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                var parameters = socket.getSSLParameters();
                parameters.setEndpointIdentificationAlgorithm("HTTPS");
                socket.setSSLParameters(parameters);
                socket.setSoTimeout(Math.multiplyExact(tls.readTimeoutSeconds(), 1000));
                socket.startHandshake();
                socket.getOutputStream().write(request);
                return readFrame(socket.getInputStream());
            }
        }
    }

    static byte[] readFrame(java.io.InputStream input) throws IOException {
        byte[] header = input.readNBytes(8);
        if (header.length != 8) throw new EOFException();
        long size = Integer.toUnsignedLong(ByteBuffer.wrap(header).getInt(4));
        long padded = (size + 7) & ~7L;
        if (8 + padded > LIMITS.maxMessageBytes()) throw new IOException("TTLV limit exceeded");
        byte[] body = input.readNBytes((int) padded);
        try {
            if (body.length != padded) throw new EOFException();
            return ByteBuffer.allocate(8 + body.length).put(header).put(body).array();
        } finally {
            Arrays.fill(body, (byte) 0);
        }
    }

    private static SSLContext context(ConnectionConfig.Tls tls) throws GeneralSecurityException, IOException {
        return context(Path.of(tls.caCertificatePath()), Path.of(tls.clientCertificatePath()), Path.of(tls.clientKeyPath()));
    }

    static SSLContext context(Path ca, Path certificatePath, Path key) throws GeneralSecurityException, IOException {
        var certs = CertificateFactory.getInstance("X.509");
        var trust = KeyStore.getInstance("PKCS12");
        trust.load(null);
        try (var input = Files.newInputStream(ca)) {
            int index = 0;
            for (var certificate : certs.generateCertificates(input)) {
                trust.setCertificateEntry("ca-" + index++, certificate);
            }
        }
        var identity = KeyStore.getInstance("PKCS12");
        identity.load(null);
        char[] password = {};
        try (var input = Files.newInputStream(certificatePath)) {
            var chain = certs.generateCertificates(input).toArray(Certificate[]::new);
            identity.setKeyEntry("client", readKey(key), password, chain);
        }
        var keys = KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm());
        keys.init(identity, password);
        var managers = TrustManagerFactory.getInstance(TrustManagerFactory.getDefaultAlgorithm());
        managers.init(trust);
        var context = SSLContext.getInstance("TLSv1.3");
        context.init(keys.getKeyManagers(), managers.getTrustManagers(), null);
        return context;
    }

    private static PrivateKey readKey(Path path) throws IOException, GeneralSecurityException {
        String pem = Files.readString(path).strip();
        byte[] decoded = null;
        byte[] pkcs8 = null;
        try {
            if (pem.contains("-----BEGIN PRIVATE KEY-----")) {
                decoded = decodePemBlock(pem, "PRIVATE KEY");
                pkcs8 = decoded;
            } else if (pem.contains("-----BEGIN RSA PRIVATE KEY-----")) {
                decoded = decodePemBlock(pem, "RSA PRIVATE KEY");
                byte[] algId = derTlv(0x30, concat(
                        new byte[] {0x06, 0x09, 0x2a, (byte) 0x86, 0x48, (byte) 0x86, (byte) 0xf7, 0x0d, 0x01, 0x01, 0x01},
                        new byte[] {0x05, 0x00}));
                pkcs8 = derTlv(0x30, concat(new byte[] {0x02, 0x01, 0x00}, algId, derTlv(0x04, decoded)));
            } else if (pem.contains("-----BEGIN EC PRIVATE KEY-----")) {
                decoded = decodePemBlock(pem, "EC PRIVATE KEY");
                byte[] curveOidTlv = extractSec1NamedCurveOid(decoded);
                byte[] ecOid = new byte[] {0x06, 0x07, 0x2a, (byte) 0x86, 0x48, (byte) 0xce, 0x3d, 0x02, 0x01};
                byte[] algId = derTlv(0x30, curveOidTlv != null ? concat(ecOid, curveOidTlv) : ecOid);
                pkcs8 = derTlv(0x30, concat(new byte[] {0x02, 0x01, 0x00}, algId, derTlv(0x04, decoded)));
            } else {
                throw new InvalidKeySpecException("Unsupported PEM client key format");
            }
            for (String algorithm : new String[] {"RSA", "EC", "Ed25519", "Ed448"}) {
                try {
                    return KeyFactory.getInstance(algorithm).generatePrivate(new PKCS8EncodedKeySpec(pkcs8));
                } catch (InvalidKeySpecException ignored) {
                    // Try the next available key type.
                }
            }
            throw new InvalidKeySpecException("Client key type is unsupported");
        } finally {
            if (decoded != null) Arrays.fill(decoded, (byte) 0);
            if (pkcs8 != null) Arrays.fill(pkcs8, (byte) 0);
        }
    }

    private static byte[] decodePemBlock(String pem, String label) throws InvalidKeySpecException {
        String start = "-----BEGIN " + label + "-----";
        String end = "-----END " + label + "-----";
        int s = pem.indexOf(start);
        int e = pem.indexOf(end, s + start.length());
        if (s < 0 || e < 0) throw new InvalidKeySpecException("Malformed PEM " + label + " block");
        return Base64.getMimeDecoder().decode(pem.substring(s + start.length(), e));
    }

    private static byte[] extractSec1NamedCurveOid(byte[] sec1) throws InvalidKeySpecException {
        int[] offset = {0};
        if (sec1.length < 2 || sec1[offset[0]++] != 0x30) throw new InvalidKeySpecException("Invalid SEC1 EC key");
        int seqLen = readDerLength(sec1, offset);
        int end = Math.min(sec1.length, offset[0] + seqLen);
        while (offset[0] < end) {
            int tag = sec1[offset[0]++] & 0xff;
            int len = readDerLength(sec1, offset);
            if (offset[0] + len > end) break;
            if (tag == 0xa0 && len >= 2 && sec1[offset[0]] == 0x06) {
                return Arrays.copyOfRange(sec1, offset[0], offset[0] + len);
            }
            offset[0] += len;
        }
        return null;
    }

    private static int readDerLength(byte[] der, int[] offset) throws InvalidKeySpecException {
        if (offset[0] >= der.length) throw new InvalidKeySpecException("Truncated DER length");
        int first = der[offset[0]++] & 0xff;
        if ((first & 0x80) == 0) return first;
        int numBytes = first & 0x7f;
        if (numBytes == 0 || numBytes > 4 || offset[0] + numBytes > der.length) {
            throw new InvalidKeySpecException("Invalid DER length");
        }
        int len = 0;
        for (int i = 0; i < numBytes; i++) {
            len = (len << 8) | (der[offset[0]++] & 0xff);
        }
        return len;
    }

    private static byte[] derTlv(int tag, byte[] content) throws IOException {
        ByteArrayOutputStream out = new ByteArrayOutputStream(content.length + 6);
        out.write(tag);
        int len = content.length;
        if (len < 0x80) {
            out.write(len);
        } else {
            int n = (32 - Integer.numberOfLeadingZeros(len) + 7) / 8;
            out.write(0x80 | n);
            for (int shift = (n - 1) * 8; shift >= 0; shift -= 8) {
                out.write((len >>> shift) & 0xff);
            }
        }
        out.write(content);
        return out.toByteArray();
    }

    private static byte[] concat(byte[]... parts) throws IOException {
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        for (byte[] p : parts) out.write(p);
        return out.toByteArray();
    }

    static TtlvStructure copyStructure(TtlvStructureView view) {
        TtlvStructure copy = TtlvStructure.create();
        try {
            for (long index = 0; index < view.itemCount(); index++) {
                try (var child = view.itemAt(index); var owned = child.copy()) {
                    copy = TtlvStructure.withItem(copy, owned, LIMITS);
                }
            }
            TtlvStructure result = copy;
            copy = null;
            return result;
        } finally {
            if (copy != null) copy.close();
        }
    }

    private static JavaKmipDriver.CaseOutcome check(byte[] wire, int operation, int minor, long start,
            Predicate<TtlvStructureView> expected) {
        return check(wire, operation, minor, start, expected, null);
    }

    private static JavaKmipDriver.CaseOutcome check(byte[] wire, int operation, int minor, long start,
            Predicate<TtlvStructureView> expected, AtomicReference<byte[]> pending) {
        try (var root = TtlvCodec.decode(wire); var view = root.view();
                var tag = view.tag(); var value = view.value()) {
            if (tag.raw() != 0x42007b) throw new IllegalArgumentException("Wrong response root");
            try (var structure = value.structure(); var response = ResponseMessage.fromTtlv(copyStructure(structure));
                    var header = response.header()) {
                var version = header.protocolVersion();
                if (version.major() != 2 || version.minor() != minor || response.batchItemCount() != 1) {
                    throw new IllegalArgumentException("Response does not match request version and count");
                }
                try (var batch = response.batchItemAt(0)) {
                    long status = batch.resultStatus().orElseThrow();
                    long opVal = batch.operation().orElse(-1);
                    if (opVal != operation && !(batch.operation().isEmpty() && status == 1)) {
                        throw new IllegalArgumentException("Wrong response operation");
                    }
                    Long reason = batch.resultReason().isPresent() ? batch.resultReason().getAsLong() : null;
                    double duration = elapsed(start);
                    if (status == 0 && reason == null && batch.withResponsePayload(expected::test).orElse(false)) {
                        return JavaKmipDriver.CaseOutcome.passed(duration, status, null);
                    }
                    if (status == 2 && reason == null && pending != null) {
                        var payload = batch.withResponsePayload(TtlvStructureView::itemCount);
                        boolean expectedShape = operation == CREATE ? payload.orElse(-1L) == 0 : payload.isEmpty();
                        var token = batch.asynchronousCorrelationValue().orElse(null);
                        if (expectedShape && token != null && token.length != 0) {
                            pending.set(token);
                            return JavaKmipDriver.CaseOutcome.passed(duration, status, null);
                        }
                        if (token != null) Arrays.fill(token, (byte) 0);
                    }
                    if (status == 1 && reason != null) {
                        if (reason == KmipValues.ResultReason.OPERATION_NOT_SUPPORTED
                                || reason == KmipValues.ResultReason.FEATURE_NOT_SUPPORTED) {
                            return JavaKmipDriver.CaseOutcome.unsupported(duration, status, reason);
                        }
                        if (reason == KmipValues.ResultReason.PERMISSION_DENIED
                                || reason == KmipValues.ResultReason.AUTHENTICATION_NOT_SUCCESSFUL) {
                            return JavaKmipDriver.CaseOutcome.policyRestricted(duration, status, reason);
                        }
                    }
                    return JavaKmipDriver.CaseOutcome.failed("Generic response did not satisfy the case.", duration, status, reason);
                }
            }
        }
    }

    private static boolean matches(TtlvStructureView payload, int operation, int minor) {
        if (operation == LOCATE) return matchesLocate(payload);
        if (operation == PING) return payload.itemCount() == 0;
        if (operation == RNG_SEED) {
            if (payload.itemCount() != 1) return false;
            try (var field = payload.itemAt(0); var tag = field.tag(); var value = field.value()) {
                if (tag.raw() != 0x4200c4 || field.itemType() != TtlvItemType.Integer) return false;
                long length = value.integer();
                return length >= 0 && length <= RNG_SEED_INPUT.length;
            }
        }
        if (operation == HASH || operation == RNG_RETRIEVE) {
            if (payload.itemCount() != 1) return false;
            try (var field = payload.itemAt(0); var tag = field.tag(); var value = field.value()) {
                return tag.raw() == 0x4200c2 && field.itemType() == TtlvItemType.ByteString
                        && (operation == RNG_RETRIEVE ? value.byteLength() == 32
                                : Arrays.equals(value.bytes(), HASH_DIGEST));
            }
        }
        if (operation == DISCOVER_VERSIONS) {
            if (payload.itemCount() != 1) return false;
            try (var actual = payload.itemAt(0); var expected = version(minor); var expectedView = expected.view()) {
                return Arrays.equals(encode(actual), encode(expectedView));
            }
        }
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                if (tag.raw() == 0x42005c && field.itemType() == TtlvItemType.Enumeration
                        && value.enumeration() != 0) return true;
            }
        }
        return false;
    }

    private static byte[] textIdentifier(TtlvStructureView payload) {
        byte[] id = null;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag()) {
                if (tag.raw() != 0x420094) continue;
                if (id != null || field.itemType() != TtlvItemType.TextString) return null;
                try (var value = field.value()) {
                    if (value.byteLength() == 0) return null;
                    id = value.bytes();
                }
            }
        }
        return id;
    }

    private static boolean hasField(TtlvStructureView payload, int wanted, TtlvItemType type, int expected) {
        boolean found = false;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                if (tag.raw() != wanted) continue;
                if (found || field.itemType() != type) return false;
                found = switch (type) {
                    case Enumeration -> value.enumeration() == expected;
                    case Integer -> value.integer() == expected;
                    case Boolean -> value.booleanValue() == (expected != 0);
                    case TextString -> Arrays.equals(value.bytes(), TEXT_ATTRIBUTE_FIXTURE);
                    default -> true;
                };
                if (!found) return false;
            }
        }
        return found;
    }

    private static boolean matchesCheckUsage(TtlvStructureView payload) {
        boolean found = false;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                if (tag.raw() != 0x42002c) continue;
                if (found || field.itemType() != TtlvItemType.Integer || (value.integer() & 4) == 0) return false;
                found = true;
            }
        }
        return true;
    }

    private static boolean hasLengthReference(TtlvStructureView payload) {
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                if (tag.raw() == 0x42013b && field.itemType() == TtlvItemType.Enumeration
                        && value.enumeration() == 0x42002a) return true;
            }
        }
        return false;
    }

    private static boolean hasKeyAttribute(TtlvStructureView payload, int attributeTag) {
        if (!hasField(payload, 0x420125, TtlvItemType.Structure, 0)) return false;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag()) {
                if (tag.raw() != 0x420125) continue;
                try (var value = field.value(); var attrs = value.structure()) {
                    if (attrs.itemCount() != 1) return false;
                    return switch (attributeTag) {
                        case 0x420028 -> hasField(attrs, attributeTag, TtlvItemType.Enumeration, 3);
                        case 0x42002a -> hasField(attrs, attributeTag, TtlvItemType.Integer, 256);
                        case 0x42002c -> hasField(attrs, attributeTag, TtlvItemType.Integer, 4);
                        case 0x420057 -> hasField(attrs, attributeTag, TtlvItemType.Enumeration, 2);
                        case 0x42008d -> hasField(attrs, attributeTag, TtlvItemType.Enumeration, 1);
                        case 0x420120, 0x420121, 0x420123 -> hasField(attrs, attributeTag, TtlvItemType.Boolean, 1);
                        case 0x420122 -> hasField(attrs, attributeTag, TtlvItemType.Boolean, 0);
                        case 0x420022, 0x4200fc, 0x4200fd -> hasField(attrs, attributeTag, TtlvItemType.TextString, 0);
                        default -> false;
                    };
                }
            }
        }
        return false;
    }

    private static boolean matchesLocate(TtlvStructureView payload) {
        Integer total = null;
        int identifiers = 0;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                var type = field.itemType();
                if (tag.raw() == 0x4200d5 && type == TtlvItemType.Integer && total == null) {
                    total = value.integer();
                } else if (tag.raw() == 0x420094 && (type == TtlvItemType.TextString
                        || type == TtlvItemType.Integer || type == TtlvItemType.Enumeration)) {
                    identifiers++;
                } else {
                    return false;
                }
            }
        }
        return identifiers <= 1 && (total == null || total >= identifiers);
    }

    private static boolean usesExtendedOp(int operation) {
        return switch (operation) {
            case CREATE_KEY_PAIR, REGISTER, RE_KEY, DERIVE_KEY, CERTIFY, RE_CERTIFY,
                    GET_USAGE_ALLOCATION, VALIDATE, CANCEL, POLL, RE_KEY_KEY_PAIR,
                    ENCRYPT, DECRYPT, SIGN, SIGNATURE_VERIFY, MAC, MAC_VERIFY,
                    CREATE_SPLIT_KEY, JOIN_SPLIT_KEY, IMPORT, EXPORT -> true;
            default -> false;
        };
    }

    private static JavaKmipDriver.CaseOutcome executePayload(ConnectionConfig connection, int operation, int minor,
            TtlvItem customPayload, Predicate<TtlvStructureView> expected, long start) {
        return executePayload(connection, operation, minor, customPayload, expected, start, null);
    }

    private static JavaKmipDriver.CaseOutcome executePayload(ConnectionConfig connection, int operation, int minor,
            TtlvItem customPayload, Predicate<TtlvStructureView> expected, long start,
            AtomicReference<byte[]> pending) {
        try (var version = version(minor);
                var count = item(0x42000d, TtlvValue.Integer(1));
                var header = pending != null && operation == CREATE
                        ? structureItem(0x420077, version, item(0x420007, TtlvValue.Enumeration(1)), count)
                        : structureItem(0x420077, version, count);
                var identifier = item(0x42005c, TtlvValue.Enumeration(operation));
                var batch = structureItem(0x42000f, identifier, customPayload);
                var req = structureItem(0x420078, header, batch);
                var view = req.view()) {
            byte[] request = encode(view);
            try {
                byte[] wire = exchange(connection, request);
                try {
                    return check(wire, operation, minor, start, expected, pending);
                } finally {
                    Arrays.fill(wire, (byte) 0);
                }
            } finally {
                Arrays.fill(request, (byte) 0);
            }
        } catch (GeneralSecurityException | IOException | RuntimeException error) {
            return JavaKmipDriver.CaseOutcome.failed("Generic TTLV exchange or validation failed ("
                    + error.getClass().getSimpleName() + ").", elapsed(start));
        }
    }

    private static byte[] fieldText(TtlvStructureView payload, int wantedTag) {
        for (long i = 0; i < payload.itemCount(); i++) {
            try (var f = payload.itemAt(i); var t = f.tag(); var v = f.value()) {
                if (t.raw() == wantedTag && f.itemType() == TtlvItemType.TextString) {
                    byte[] bytes = v.bytes();
                    if (bytes.length != 0) return bytes;
                }
            }
        }
        return null;
    }

    private static List<byte[]> allIdentifiers(TtlvStructureView payload) {
        var ids = new java.util.ArrayList<byte[]>();
        for (long i = 0; i < payload.itemCount(); i++) {
            try (var f = payload.itemAt(i); var t = f.tag(); var v = f.value()) {
                if (t.raw() == 0x420094 && f.itemType() == TtlvItemType.TextString) {
                    byte[] bytes = v.bytes();
                    if (bytes.length != 0) ids.add(bytes);
                }
            }
        }
        return ids;
    }

    private static byte[] fieldBytes(TtlvStructureView payload, int wantedTag) {
        for (long i = 0; i < payload.itemCount(); i++) {
            try (var f = payload.itemAt(i); var t = f.tag(); var v = f.value()) {
                if (t.raw() == wantedTag && f.itemType() == TtlvItemType.ByteString) {
                    byte[] bytes = v.bytes();
                    if (bytes.length != 0) return bytes;
                }
            }
        }
        return null;
    }

    private static void destroyAll(ConnectionConfig connection, int minor, List<byte[]> ids, long start) {
        for (byte[] id : ids) {
            if (id == null) continue;
            execute(connection, REVOKE, minor, id, p -> true, start);
            execute(connection, DESTROY, minor, id, p -> true, start);
        }
    }

    static byte[] asynchronousCreateIdentifier(TtlvStructureView payload) {
        return hasField(payload, 0x420057, TtlvItemType.Enumeration, 2) ? textIdentifier(payload) : null;
    }

    private static boolean asynchronousCreateResult(TtlvStructureView payload, AtomicReference<byte[]> owned) {
        byte[] identifier = asynchronousCreateIdentifier(payload);
        if (identifier == null) return false;
        owned.set(identifier);
        return true;
    }

    private static boolean asynchronousCancelResult(TtlvStructureView payload, byte[] token) {
        if (payload.itemCount() != 2) return false;
        byte[] echoed = fieldBytes(payload, 0x420006);
        if (echoed == null) return false;
        try {
            if (!Arrays.equals(echoed, token)) return false;
            for (int result = 1; result <= 5; result++) {
                if (hasField(payload, 0x420012, TtlvItemType.Enumeration, result)) return true;
            }
            return false;
        } finally {
            Arrays.fill(echoed, (byte) 0);
        }
    }

    private static JavaKmipDriver.CaseOutcome asynchronousTarget(
            ConnectionConfig connection, int operation, int minor, byte[] token,
            AtomicReference<byte[]> pending, AtomicReference<byte[]> owned, long start) {
        try (var payload = structureItem(0x420079, item(0x420006, TtlvValue.ByteString(token, LIMITS)))) {
            return executePayload(connection, operation, minor, payload,
                    operation == CANCEL ? p -> asynchronousCancelResult(p, token)
                            : p -> asynchronousCreateResult(p, owned),
                    start, operation == POLL ? pending : null);
        }
    }

    private static JavaKmipDriver.CaseOutcome runAsynchronous(
            ConnectionConfig connection, int operation, int minor, long start) {
        var owned = new AtomicReference<byte[]>();
        var pending = new AtomicReference<byte[]>();
        byte[] name = ("kmipkit-java-async-" + UUID.randomUUID()).getBytes(StandardCharsets.US_ASCII);
        try (var attributes = structureItem(0x420125,
                    structureItem(0x420053, item(0x420055, TtlvValue.TextString(name, LIMITS)),
                            item(0x420054, TtlvValue.Enumeration(1))),
                    item(0x420028, TtlvValue.Enumeration(3)), item(0x42002a, TtlvValue.Integer(128)),
                    item(0x42002c, TtlvValue.Integer(4)));
                var payload = structureItem(0x420079, item(0x420057, TtlvValue.Enumeration(2)), attributes)) {
            executePayload(connection, CREATE, minor, payload, p -> asynchronousCreateResult(p, owned), start, pending);
        }
        var result = JavaKmipDriver.CaseOutcome.notRun("The prerequisite Create returned no Pending correlation.", elapsed(start));
        if (pending.get() != null) {
            for (int poll = 0; poll < 16; poll++) {
                byte[] token = pending.getAndSet(null);
                try {
                    result = asynchronousTarget(connection, operation, minor, token, pending, owned, start);
                } finally {
                    Arrays.fill(token, (byte) 0);
                }
                if (pending.get() == null) break;
                if (poll == 15) {
                    byte[] finalToken = pending.getAndSet(null);
                    try {
                        var cleanup = asynchronousTarget(connection, CANCEL, minor, finalToken, pending, owned, start);
                        result = "passed".equals(cleanup.status())
                                ? JavaKmipDriver.CaseOutcome.notRun("The server remained Pending after the bounded Poll sequence.", elapsed(start))
                                : JavaKmipDriver.CaseOutcome.failed("Could not cancel the fixture after the bounded Poll sequence.", elapsed(start));
                    } finally {
                        Arrays.fill(finalToken, (byte) 0);
                    }
                }
            }
        }
        byte[] identifier = owned.get();
        if (identifier != null) {
            var cleanup = execute(connection, DESTROY, minor, identifier,
                    payload -> Arrays.equals(textIdentifier(payload), identifier), start);
            if (!"passed".equals(cleanup.status())) {
                return JavaKmipDriver.CaseOutcome.failed("Could not destroy the asynchronous Create fixture.", elapsed(start));
            }
        }
        return result;
    }

    private static JavaKmipDriver.CaseOutcome runExtended(CatalogLoader.CatalogCase testCase,
            ConnectionConfig connection, int operation,
            int minor, long start) {
        var cleanup = new java.util.ArrayList<byte[]>();
        try {
            if (operation == POLL || operation == CANCEL) {
                return runAsynchronous(connection, operation, minor, start);
            }
            if (operation == REGISTER || operation == IMPORT) {
                return runExtendedRegisterImport(connection, operation, minor, cleanup, start);
            }
            if (operation == CREATE_KEY_PAIR || operation == RE_KEY_KEY_PAIR) {
                return runExtendedKeyPair(connection, operation, minor, cleanup, start);
            }
            if (operation == CERTIFY || operation == RE_CERTIFY || operation == VALIDATE) {
                return runExtendedCert(connection, operation, minor, cleanup, start);
            }
            if (operation == ENCRYPT || operation == DECRYPT) {
                if ("GCM".equals(testCase.requires().path("block_cipher_mode").asText())) {
                    boolean invalid = "OperationFailed".equals(testCase.requires().path("expected_status").asText());
                    return runExtendedGcm(connection, operation, minor, cleanup, start, invalid);
                }
                return runExtendedEncrypt(connection, operation, minor, cleanup, start);
            }
            if (operation == MAC || operation == MAC_VERIFY) {
                return runExtendedMac(connection, operation, minor, cleanup, start);
            }
            if (operation == SIGN || operation == SIGNATURE_VERIFY) {
                return runExtendedSign(connection, operation, minor, cleanup, start);
            }
            return runExtendedSym(connection, operation, minor, cleanup, start);
        } finally {
            destroyAll(connection, minor, cleanup, start);
        }
    }

    private static TtlvItem symKeyBlock(byte fill) {
        try (var fmt = item(0x420042, TtlvValue.Enumeration(1));
                var mat = item(0x420043, TtlvValue.ByteString(new byte[]{
                        fill, fill, fill, fill, fill, fill, fill, fill,
                        fill, fill, fill, fill, fill, fill, fill, fill}, LIMITS));
                var kv = structureItem(0x420045, mat);
                var kb = structureItem(0x420040, fmt, kv)) {
            return structureItem(0x42008f, kb);
        }
    }

    private static JavaKmipDriver.CaseOutcome runExtendedRegisterImport(ConnectionConfig connection, int operation,
            int minor, List<byte[]> cleanup, long start) {
        try (var ot = item(0x420057, TtlvValue.Enumeration(2));
                var alg = item(0x420028, TtlvValue.Enumeration(3));
                var len = item(0x42002a, TtlvValue.Integer(128));
                var mask = item(0x42002c, TtlvValue.Integer(4));
                var attrs = structureItem(0x420125, alg, len, mask);
                var sk = symKeyBlock((byte) 0x11);
                var uid = item(0x420094, TtlvValue.TextString("kmipkit-generic-import".getBytes(StandardCharsets.US_ASCII), LIMITS));
                var rep = item(0x420124, TtlvValue.Boolean(false));
                var payload = operation == IMPORT
                        ? structureItem(0x420079, uid, ot, rep, attrs, sk)
                        : structureItem(0x420079, ot, attrs, sk)) {
            return executePayload(connection, operation, minor, payload, p -> {
                byte[] id = textIdentifier(p);
                if (id != null) cleanup.add(id);
                return id != null;
            }, start);
        }
    }

    private static JavaKmipDriver.CaseOutcome createAesKey(ConnectionConfig connection, int minor, int usageMask,
            boolean extractable, List<byte[]> cleanup, AtomicReference<byte[]> outId, long start) {
        try (var ot = item(0x420057, TtlvValue.Enumeration(2));
                var alg = item(0x420028, TtlvValue.Enumeration(3));
                var len = item(0x42002a, TtlvValue.Integer(256));
                var mask = item(0x42002c, TtlvValue.Integer(usageMask));
                var ext = item(0x420122, TtlvValue.Boolean(true));
                var attrs = extractable ? structureItem(0x420125, alg, len, mask, ext) : structureItem(0x420125, alg, len, mask);
                var payload = structureItem(0x420079, ot, attrs)) {
            return executePayload(connection, CREATE, minor, payload, p -> {
                byte[] id = textIdentifier(p);
                if (id != null) {
                    outId.set(id);
                    cleanup.add(id);
                }
                return id != null;
            }, start);
        }
    }

    private static JavaKmipDriver.CaseOutcome createRsaPair(ConnectionConfig connection, int minor,
            List<byte[]> cleanup, AtomicReference<byte[]> privOut, AtomicReference<byte[]> pubOut, long start) {
        try (var alg = item(0x420028, TtlvValue.Enumeration(4));
                var len = item(0x42002a, TtlvValue.Integer(2048));
                var common = structureItem(0x420126, alg, len);
                var privMask = item(0x42002c, TtlvValue.Integer(1));
                var privAttrs = structureItem(0x420127, privMask);
                var pubMask = item(0x42002c, TtlvValue.Integer(2));
                var pubAttrs = structureItem(0x420128, pubMask);
                var payload = structureItem(0x420079, common, privAttrs, pubAttrs)) {
            return executePayload(connection, CREATE_KEY_PAIR, minor, payload, p -> {
                byte[] priv = fieldText(p, 0x420066);
                byte[] pub = fieldText(p, 0x42006f);
                if (priv != null) { privOut.set(priv); cleanup.add(priv); }
                if (pub != null) { pubOut.set(pub); cleanup.add(pub); }
                return priv != null && pub != null;
            }, start);
        }
    }

    private static JavaKmipDriver.CaseOutcome runExtendedKeyPair(ConnectionConfig connection, int operation,
            int minor, List<byte[]> cleanup, long start) {
        var priv = new AtomicReference<byte[]>();
        var pub = new AtomicReference<byte[]>();
        var kp = createRsaPair(connection, minor, cleanup, priv, pub, start);
        if (operation == CREATE_KEY_PAIR || !"passed".equals(kp.status())) return kp;
        try (var pid = item(0x420066, TtlvValue.TextString(priv.get(), LIMITS));
                var offset = item(0x420058, TtlvValue.Interval(3600));
                var payload = structureItem(0x420079, pid, offset)) {
            return executePayload(connection, RE_KEY_KEY_PAIR, minor, payload, p -> {
                byte[] np = fieldText(p, 0x420066);
                byte[] nu = fieldText(p, 0x42006f);
                if (np != null) cleanup.add(np);
                if (nu != null) cleanup.add(nu);
                return np != null && nu != null;
            }, start);
        }
    }

    private static JavaKmipDriver.CaseOutcome runExtendedSym(ConnectionConfig connection, int operation,
            int minor, List<byte[]> cleanup, long start) {
        var keyRef = new AtomicReference<byte[]>();
        var keyRes = createAesKey(connection, minor, operation == DERIVE_KEY ? 512 : 12,
                operation == EXPORT, cleanup, keyRef, start);
        if (!"passed".equals(keyRes.status())) return keyRes;
        byte[] keyId = keyRef.get();
        if (operation == RE_KEY) {
            try (var uid = item(0x420094, TtlvValue.TextString(keyId, LIMITS));
                    var offset = item(0x420058, TtlvValue.Interval(3600));
                    var payload = structureItem(0x420079, uid, offset)) {
                return executePayload(connection, RE_KEY, minor, payload, p -> {
                    byte[] nid = textIdentifier(p);
                    if (nid != null) cleanup.add(nid);
                    return nid != null && !Arrays.equals(nid, keyId);
                }, start);
            }
        }
        if (operation == EXPORT) {
            try (var uid = item(0x420094, TtlvValue.TextString(keyId, LIMITS));
                    var fmt = item(0x420042, TtlvValue.Enumeration(1));
                    var payload = structureItem(0x420079, uid, fmt)) {
                return executePayload(connection, EXPORT, minor, payload,
                        p -> Arrays.equals(keyId, textIdentifier(p)) && hasField(p, 0x42008f, TtlvItemType.Structure, 0), start);
            }
        }
        if (operation == GET_USAGE_ALLOCATION) {
            try (var uid = item(0x420094, TtlvValue.TextString(keyId, LIMITS));
                    var cnt = item(0x420096, TtlvValue.LongInteger(32));
                    var payload = structureItem(0x420079, uid, cnt)) {
                return executePayload(connection, GET_USAGE_ALLOCATION, minor, payload,
                        p -> Arrays.equals(keyId, textIdentifier(p)), start);
            }
        }
        if (operation == DERIVE_KEY) {
            execute(connection, ACTIVATE, minor, keyId, p -> true, start);
            try (var ot = item(0x420057, TtlvValue.Enumeration(2));
                    var uid = item(0x420094, TtlvValue.TextString(keyId, LIMITS));
                    var dm = item(0x420031, TtlvValue.Enumeration(2));
                    var ha = item(0x420038, TtlvValue.Enumeration(6));
                    var dp = structureItem(0x420032, ha);
                    var alg = item(0x420028, TtlvValue.Enumeration(3));
                    var len = item(0x42002a, TtlvValue.Integer(128));
                    var mask = item(0x42002c, TtlvValue.Integer(4));
                    var attrs = structureItem(0x420125, alg, len, mask);
                    var payload = structureItem(0x420079, ot, uid, dm, dp, attrs)) {
                return executePayload(connection, DERIVE_KEY, minor, payload, p -> {
                    byte[] did = textIdentifier(p);
                    if (did != null) cleanup.add(did);
                    return did != null;
                }, start);
            }
        }
        var parts = new AtomicReference<List<byte[]>>(List.of());
        JavaKmipDriver.CaseOutcome splitRes;
        try (var ot = item(0x420057, TtlvValue.Enumeration(2));
                var uid = item(0x420094, TtlvValue.TextString(keyId, LIMITS));
                var p1 = item(0x42008b, TtlvValue.Integer(2));
                var th = item(0x42008c, TtlvValue.Integer(2));
                var m = item(0x42008a, TtlvValue.Enumeration(1));
                var alg = item(0x420028, TtlvValue.Enumeration(3));
                var len = item(0x42002a, TtlvValue.Integer(256));
                var mask = item(0x42002c, TtlvValue.Integer(12));
                var attrs = structureItem(0x420125, alg, len, mask);
                var payload = structureItem(0x420079, ot, uid, p1, th, m, attrs)) {
            splitRes = executePayload(connection, CREATE_SPLIT_KEY, minor, payload, p -> {
                var found = allIdentifiers(p);
                parts.set(found);
                cleanup.addAll(found);
                return found.size() == 2;
            }, start);
        }
        if (operation == CREATE_SPLIT_KEY || !"passed".equals(splitRes.status())) return splitRes;
        try (var ot = item(0x420057, TtlvValue.Enumeration(2));
                var u1 = item(0x420094, TtlvValue.TextString(parts.get().get(0), LIMITS));
                var u2 = item(0x420094, TtlvValue.TextString(parts.get().get(1), LIMITS));
                var alg = item(0x420028, TtlvValue.Enumeration(3));
                var len = item(0x42002a, TtlvValue.Integer(256));
                var mask = item(0x42002c, TtlvValue.Integer(12));
                var attrs = structureItem(0x420125, alg, len, mask);
                var payload = structureItem(0x420079, ot, u1, u2, attrs)) {
            return executePayload(connection, JOIN_SPLIT_KEY, minor, payload, p -> {
                byte[] jid = textIdentifier(p);
                if (jid != null) cleanup.add(jid);
                return jid != null;
            }, start);
        }
    }

    private static JavaKmipDriver.CaseOutcome runExtendedCert(ConnectionConfig connection, int operation,
            int minor, List<byte[]> cleanup, long start) {
        var priv = new AtomicReference<byte[]>();
        var pub = new AtomicReference<byte[]>();
        var kp = createRsaPair(connection, minor, cleanup, priv, pub, start);
        if (!"passed".equals(kp.status())) {
            return JavaKmipDriver.CaseOutcome.notRun("The prerequisite Create Key Pair request failed.", elapsed(start));
        }
        var certRef = new AtomicReference<byte[]>();
        JavaKmipDriver.CaseOutcome certRes;
        try (var uid = item(0x420094, TtlvValue.TextString(pub.get(), LIMITS));
                var attributes = structureItem(0x420125, item(0x42001d, TtlvValue.Enumeration(1)));
                var payload = structureItem(0x420079, uid, attributes)) {
            certRes = executePayload(connection, CERTIFY, minor, payload, p -> {
                byte[] cid = textIdentifier(p);
                if (cid != null) { certRef.set(cid); cleanup.add(cid); }
                return cid != null;
            }, start);
        }
        if (operation == CERTIFY) return certRes;
        if (!"passed".equals(certRes.status())) {
            return JavaKmipDriver.CaseOutcome.notRun("The server rejected the prerequisite Certify request.", elapsed(start));
        }
        try (var uid = item(0x420094, TtlvValue.TextString(certRef.get(), LIMITS));
                var payload = structureItem(0x420079, uid)) {
            if (operation == RE_CERTIFY) {
                return executePayload(connection, RE_CERTIFY, minor, payload, p -> {
                    byte[] nid = textIdentifier(p);
                    if (nid != null) cleanup.add(nid);
                    return nid != null;
                }, start);
            }
            return executePayload(connection, VALIDATE, minor, payload,
                    p -> hasField(p, 0x42009b, TtlvItemType.Enumeration, 1), start);
        }
    }

    private static JavaKmipDriver.CaseOutcome runExtendedEncrypt(ConnectionConfig connection, int operation,
            int minor, List<byte[]> cleanup, long start) {
        var keyRef = new AtomicReference<byte[]>();
        var keyRes = createAesKey(connection, minor, 12, false, cleanup, keyRef, start);
        if (!"passed".equals(keyRes.status())) return keyRes;
        byte[] keyId = keyRef.get();
        var act = execute(connection, ACTIVATE, minor, keyId, p -> true, start);
        if (!"passed".equals(act.status())) return act;
        byte[] iv = new byte[16];
        Arrays.fill(iv, (byte) 0x01);
        byte[] plaintext = "0123456789abcdef".getBytes(StandardCharsets.US_ASCII);
        var cipherRef = new AtomicReference<byte[]>();
        JavaKmipDriver.CaseOutcome encRes;
        try (var uid = item(0x420094, TtlvValue.TextString(keyId, LIMITS));
                var bcm = item(0x420011, TtlvValue.Enumeration(1));
                var pad = item(0x42005f, TtlvValue.Enumeration(3));
                var params = structureItem(0x42002b, bcm, pad);
                var data = item(0x4200c2, TtlvValue.ByteString(plaintext, LIMITS));
                var ivItem = item(0x42003d, TtlvValue.ByteString(iv, LIMITS));
                var payload = structureItem(0x420079, uid, params, data, ivItem)) {
            encRes = executePayload(connection, ENCRYPT, minor, payload, p -> {
                byte[] c = fieldBytes(p, 0x4200c2);
                cipherRef.set(c);
                return Arrays.equals(keyId, textIdentifier(p)) && c != null;
            }, start);
        }
        if (operation == ENCRYPT || !"passed".equals(encRes.status())) return encRes;
        try (var uid = item(0x420094, TtlvValue.TextString(keyId, LIMITS));
                var bcm = item(0x420011, TtlvValue.Enumeration(1));
                var pad = item(0x42005f, TtlvValue.Enumeration(3));
                var params = structureItem(0x42002b, bcm, pad);
                var data = item(0x4200c2, TtlvValue.ByteString(cipherRef.get(), LIMITS));
                var ivItem = item(0x42003d, TtlvValue.ByteString(iv, LIMITS));
                var payload = structureItem(0x420079, uid, params, data, ivItem)) {
            return executePayload(connection, DECRYPT, minor, payload,
                    p -> Arrays.equals(keyId, textIdentifier(p)) && Arrays.equals(plaintext, fieldBytes(p, 0x4200c2)), start);
        }
    }

    private static JavaKmipDriver.CaseOutcome runExtendedGcm(ConnectionConfig connection, int operation,
            int minor, List<byte[]> cleanup, long start, boolean invalidTag) {
        var keyRef = new AtomicReference<byte[]>();
        var keyRes = createAesKey(connection, minor, 12, false, cleanup, keyRef, start);
        if (!"passed".equals(keyRes.status())) {
            return JavaKmipDriver.CaseOutcome.notRun("The prerequisite Create request failed.", elapsed(start));
        }
        byte[] keyId = keyRef.get();
        var act = execute(connection, ACTIVATE, minor, keyId, p -> Arrays.equals(keyId, textIdentifier(p)), start);
        if (!"passed".equals(act.status())) {
            return JavaKmipDriver.CaseOutcome.notRun("The prerequisite Activate request failed.", elapsed(start));
        }
        byte[] plain = "AAAAAAAAAAAAAAAA".getBytes(StandardCharsets.US_ASCII);
        var cipherRef = new AtomicReference<byte[]>();
        var tagRef = new AtomicReference<byte[]>();
        try (var payload = gcmPayload(keyId, plain, null)) {
            var enc = executePayload(connection, ENCRYPT, minor, payload, p -> {
                byte[] ciphertext = fieldBytes(p, 0x4200c2);
                byte[] tag = fieldBytes(p, 0x4200ff);
                cipherRef.set(ciphertext);
                tagRef.set(tag);
                return Arrays.equals(keyId, textIdentifier(p)) && ciphertext != null && ciphertext.length == plain.length
                        && tag != null && tag.length == 16;
            }, start);
            if (operation == ENCRYPT) return enc;
            if (!"passed".equals(enc.status())) {
                return JavaKmipDriver.CaseOutcome.notRun("The prerequisite Encrypt request failed.", elapsed(start));
            }
        }
        byte[] tag = tagRef.get();
        if (invalidTag) tag[0] ^= 1;
        try (var payload = gcmPayload(keyId, cipherRef.get(), tag)) {
            var result = executePayload(connection, DECRYPT, minor, payload,
                    p -> !invalidTag && Arrays.equals(keyId, textIdentifier(p))
                            && Arrays.equals(plain, fieldBytes(p, 0x4200c2)), start);
            if (invalidTag && Long.valueOf(1).equals(result.resultStatusCode())
                    && Long.valueOf(10).equals(result.resultReasonCode())) {
                return JavaKmipDriver.CaseOutcome.passed(elapsed(start), 1L, 10L);
            }
            return result;
        }
    }

    private static TtlvItem gcmPayload(byte[] identifier, byte[] data, byte[] tag) {
        byte[] aad = "kmipkit authenticated data".getBytes(StandardCharsets.US_ASCII);
        try (var uid = item(0x420094, TtlvValue.TextString(identifier, LIMITS));
                var mode = item(0x420011, TtlvValue.Enumeration(9));
                var padding = item(0x42005f, TtlvValue.Enumeration(1));
                var ivLength = item(0x4200cd, TtlvValue.Integer(96));
                var tagLength = item(0x4200ce, TtlvValue.Integer(16));
                var params = structureItem(0x42002b, mode, padding, ivLength, tagLength);
                var contents = item(0x4200c2, TtlvValue.ByteString(data, LIMITS));
                var iv = item(0x42003d, TtlvValue.ByteString(new byte[12], LIMITS));
                var additional = item(0x4200fe, TtlvValue.ByteString(aad, LIMITS))) {
            if (tag == null) return structureItem(0x420079, uid, params, contents, iv, additional);
            try (var authTag = item(0x4200ff, TtlvValue.ByteString(tag, LIMITS))) {
                return structureItem(0x420079, uid, params, contents, iv, additional, authTag);
            }
        }
    }

    private static JavaKmipDriver.CaseOutcome runExtendedMac(ConnectionConfig connection, int operation,
            int minor, List<byte[]> cleanup, long start) {
        var macRef = new AtomicReference<byte[]>();
        JavaKmipDriver.CaseOutcome mkRes;
        try (var ot = item(0x420057, TtlvValue.Enumeration(2));
                var alg = item(0x420028, TtlvValue.Enumeration(9));
                var len = item(0x42002a, TtlvValue.Integer(256));
                var mask = item(0x42002c, TtlvValue.Integer(384));
                var attrs = structureItem(0x420125, alg, len, mask);
                var payload = structureItem(0x420079, ot, attrs)) {
            mkRes = executePayload(connection, CREATE, minor, payload, p -> {
                byte[] id = textIdentifier(p);
                if (id != null) { macRef.set(id); cleanup.add(id); }
                return id != null;
            }, start);
        }
        if (!"passed".equals(mkRes.status())) return mkRes;
        byte[] macId = macRef.get();
        var act = execute(connection, ACTIVATE, minor, macId, p -> true, start);
        if (!"passed".equals(act.status())) return act;
        var tagRef = new AtomicReference<byte[]>();
        JavaKmipDriver.CaseOutcome macRes;
        try (var uid = item(0x420094, TtlvValue.TextString(macId, LIMITS));
                var ha = item(0x420038, TtlvValue.Enumeration(6));
                var params = structureItem(0x42002b, ha);
                var data = item(0x4200c2, TtlvValue.ByteString(HASH_INPUT, LIMITS));
                var payload = structureItem(0x420079, uid, params, data)) {
            macRes = executePayload(connection, MAC, minor, payload, p -> {
                byte[] tag = fieldBytes(p, 0x4200c6);
                tagRef.set(tag);
                return Arrays.equals(macId, textIdentifier(p)) && tag != null;
            }, start);
        }
        if (operation == MAC || !"passed".equals(macRes.status())) return macRes;
        try (var uid = item(0x420094, TtlvValue.TextString(macId, LIMITS));
                var ha = item(0x420038, TtlvValue.Enumeration(6));
                var params = structureItem(0x42002b, ha);
                var data = item(0x4200c2, TtlvValue.ByteString(HASH_INPUT, LIMITS));
                var macData = item(0x4200c6, TtlvValue.ByteString(tagRef.get(), LIMITS));
                var payload = structureItem(0x420079, uid, params, data, macData)) {
            return executePayload(connection, MAC_VERIFY, minor, payload,
                    p -> hasField(p, 0x42009b, TtlvItemType.Enumeration, 1), start);
        }
    }

    private static JavaKmipDriver.CaseOutcome runExtendedSign(ConnectionConfig connection, int operation,
            int minor, List<byte[]> cleanup, long start) {
        var priv = new AtomicReference<byte[]>();
        var pub = new AtomicReference<byte[]>();
        var kp = createRsaPair(connection, minor, cleanup, priv, pub, start);
        if (!"passed".equals(kp.status())) return kp;
        var act1 = execute(connection, ACTIVATE, minor, priv.get(), p -> true, start);
        if (!"passed".equals(act1.status())) return act1;
        var act2 = execute(connection, ACTIVATE, minor, pub.get(), p -> true, start);
        if (!"passed".equals(act2.status())) return act2;
        var sigRef = new AtomicReference<byte[]>();
        JavaKmipDriver.CaseOutcome signRes;
        try (var uid = item(0x420094, TtlvValue.TextString(priv.get(), LIMITS));
                var alg = item(0x420028, TtlvValue.Enumeration(4));
                var ha = item(0x420038, TtlvValue.Enumeration(6));
                var pad = item(0x42005f, TtlvValue.Enumeration(8));
                var dsa = item(0x4200ae, TtlvValue.Enumeration(5));
                var params = structureItem(0x42002b, alg, ha, pad, dsa);
                var data = item(0x4200c2, TtlvValue.ByteString(HASH_INPUT, LIMITS));
                var payload = structureItem(0x420079, uid, params, data)) {
            signRes = executePayload(connection, SIGN, minor, payload, p -> {
                byte[] sig = fieldBytes(p, 0x4200c3);
                sigRef.set(sig);
                return Arrays.equals(priv.get(), textIdentifier(p)) && sig != null;
            }, start);
        }
        if (operation == SIGN || !"passed".equals(signRes.status())) return signRes;
        try (var uid = item(0x420094, TtlvValue.TextString(pub.get(), LIMITS));
                var alg = item(0x420028, TtlvValue.Enumeration(4));
                var ha = item(0x420038, TtlvValue.Enumeration(6));
                var pad = item(0x42005f, TtlvValue.Enumeration(8));
                var dsa = item(0x4200ae, TtlvValue.Enumeration(5));
                var params = structureItem(0x42002b, alg, ha, pad, dsa);
                var data = item(0x4200c2, TtlvValue.ByteString(HASH_INPUT, LIMITS));
                var sigData = item(0x4200c3, TtlvValue.ByteString(sigRef.get(), LIMITS));
                var payload = structureItem(0x420079, uid, params, data, sigData)) {
            return executePayload(connection, SIGNATURE_VERIFY, minor, payload,
                    p -> hasField(p, 0x42009b, TtlvItemType.Enumeration, 1), start);
        }
    }

    private static double elapsed(long start) {
        return (System.nanoTime() - start) / 1_000_000.0;
    }
}
