package org.kmipkit.validator;

import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.RegisterRequest;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.extensions.RevokeRequest;
import org.kmipkit.extensions.SplitKeyReference;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/** Constructs deterministic public Java requests from the installed KMIPKit package. */
final class TypedRequestFactory {
    private static final CodecLimits LIMITS = CodecLimits.defaults();
    private static final byte[] BATCH_ITEM_ID = "item-0".getBytes(StandardCharsets.US_ASCII);
    private static final byte[] LOG_MESSAGE =
            "KMIPKit interoperability diagnostic".getBytes(StandardCharsets.UTF_8);
    private static final java.util.Set<String> KMIP_3_DRAFT_FACTORIES = java.util.Set.of(
            "createGroup", "obliterate", "createUser", "createCredential", "deactivate", "encapsulate",
            "decapsulate");
    private static final Map<String, String> FACTORY_METHODS = factoryMethods();

    private TypedRequestFactory() { }

    /** Constructs one catalog operation through its public ClientBatchItem factory, if available. */
    static ClientBatchItem create(String operation) {
        return create(operation, null);
    }

    /** Constructs one operation while replacing generic object identifiers with an owned fixture ID. */
    static ClientBatchItem create(String operation, UniqueIdentifier fixtureIdentifier) {
        String factoryName = FACTORY_METHODS.get(operation);
        if (factoryName == null) return null;
        Method factory = Arrays.stream(ClientBatchItem.class.getMethods())
                .filter(method -> method.getName().equals(factoryName)
                        && method.getReturnType() == ClientBatchItem.class
                        && Modifier.isStatic(method.getModifiers()))
                .max(Comparator.comparingInt(Method::getParameterCount))
                .orElseThrow(() -> new IllegalStateException("Installed Java package lacks " + factoryName));
        try (RequestInputs inputs = new RequestInputs(fixtureIdentifier)) {
            return invoke(factory, operation, inputs);
        }
    }

    /** Exercises every public static ClientBatchItem factory overload with a deterministic recipe. */
    static Map<String, String> constructEveryFactoryVariant() {
        var results = new LinkedHashMap<String, String>();
        var methods = Arrays.stream(ClientBatchItem.class.getMethods())
                .filter(method -> method.getReturnType() == ClientBatchItem.class
                        && Modifier.isStatic(method.getModifiers())
                        && !KMIP_3_DRAFT_FACTORIES.contains(method.getName()))
                .sorted(Comparator.comparing(Method::toGenericString))
                .toList();
        for (Method method : methods) {
            try (RequestInputs inputs = new RequestInputs(null);
                    ClientBatchItem request = invoke(method, method.getName(), inputs)) {
                String operation = KmipValues.Operation.name(request.operation());
                if (operation == null || operation.isBlank()) {
                    throw new IllegalStateException("Java factory returned an unknown KMIP operation: "
                            + method.toGenericString());
                }
                results.put(method.toGenericString(), operation);
            }
        }
        return Map.copyOf(results);
    }

    private static ClientBatchItem invoke(Method factory, String operation, RequestInputs inputs) {
        try {
            Type[] parameterTypes = factory.getGenericParameterTypes();
            Object[] arguments = new Object[parameterTypes.length];
            for (int index = 0; index < parameterTypes.length; index++) {
                if ("deriveKey".equals(factory.getName())) {
                    arguments[index] = switch (index) {
                        case 0 -> 7L;
                        case 1 -> inputs.argument(parameterTypes[index]);
                        case 2 -> 0xfeed_beefL;
                        case 3 -> inputs.argument(parameterTypes[index]);
                        case 4 -> inputs.deriveKeyAttributes();
                        default -> throw new IllegalArgumentException("Unexpected Derive Key factory parameter");
                    };
                } else if ("hash".equals(factory.getName())) {
                    arguments[index] = switch (index) {
                        case 0 -> inputs.hashParameters();
                        case 1 -> inputs.argument(parameterTypes[index]);
                        default -> null;
                    };
                } else if ("pkcs11".equals(factory.getName())) {
                    arguments[index] = switch (index) {
                        case 0 -> inputs.pkcs11Function();
                        case 4 -> List.of();
                        default -> null;
                    };
                } else if ("log".equals(factory.getName())) {
                    arguments[index] = LOG_MESSAGE.clone();
                } else if ("validate".equals(factory.getName())) {
                    arguments[index] = switch (index) {
                        case 0, 3 -> List.of();
                        case 1 -> List.of(inputs.identifier());
                        default -> null;
                    };
                } else if ("export".equals(factory.getName())) {
                    arguments[index] = index == 0 ? inputs.argument(parameterTypes[index]) : null;
                } else if ("get".equals(factory.getName())) {
                    arguments[index] = index == 0 ? inputs.argument(parameterTypes[index]) : null;
                } else if (("encrypt".equals(factory.getName()) || "decrypt".equals(factory.getName()))
                        && index >= 3) {
                    arguments[index] = null;
                } else if ("reprovision".equals(factory.getName())) {
                    arguments[index] = null;
                } else if ("delegatedLogin".equals(factory.getName())) {
                    arguments[index] = switch (index) {
                        case 0 -> inputs.argument(parameterTypes[index]);
                        case 1 -> 60L;
                        case 2 -> null;
                        case 3 -> 3;
                        default -> null;
                    };
                } else if ("login".equals(factory.getName())) {
                    arguments[index] = null;
                } else if ("deleteAttribute".equals(factory.getName()) && index == 1) {
                    arguments[index] = null;
                } else if ("logout".equals(factory.getName()) && index == 0) {
                    arguments[index] = inputs.logoutTicket();
                } else if ("importObject".equals(factory.getName())) {
                    arguments[index] = switch (index) {
                        case 1 -> 8L;
                        case 2, 3 -> null;
                        case 5 -> inputs.importAnyObject();
                        default -> inputs.argument(parameterTypes[index]);
                    };
                } else {
                    arguments[index] = inputs.argument(parameterTypes[index]);
                }
            }
            ClientBatchItem request = (ClientBatchItem) factory.invoke(null, arguments);
            return request.withUniqueBatchItemId(BATCH_ITEM_ID);
        } catch (InvocationTargetException invocationFailure) {
            Throwable cause = invocationFailure.getCause();
            throw new IllegalArgumentException("Could not construct public Java request for " + operation,
                    cause instanceof RuntimeException ? cause : invocationFailure);
        } catch (ReflectiveOperationException constructionFailure) {
            throw new IllegalStateException("Could not invoke public Java request factory for " + operation,
                    constructionFailure);
        }
    }

    /** Returns catalog operations backed by a public ClientBatchItem factory. */
    static Map<String, String> availableFactories() {
        var available = new LinkedHashMap<String, String>();
        for (var entry : FACTORY_METHODS.entrySet()) {
            boolean present = Arrays.stream(ClientBatchItem.class.getMethods())
                    .anyMatch(method -> method.getName().equals(entry.getValue())
                            && method.getReturnType() == ClientBatchItem.class
                            && Modifier.isStatic(method.getModifiers()));
            if (present) available.put(entry.getKey(), entry.getValue());
        }
        return Map.copyOf(available);
    }

    /** Returns the public typed response reader paired with an installed request factory. */
    static String responseAccessor(String operation) {
        String factoryName = FACTORY_METHODS.get(operation);
        if (factoryName == null || !availableFactories().containsKey(operation)) {
            throw new IllegalArgumentException("No installed Java request factory exists for " + operation);
        }
        String accessor = switch (operation) {
            case "Discover Versions" -> "supportedVersions";
            case "Query" -> "typedQueryResponse";
            case "Create Split Key" -> "createSplitKeyUniqueIdentifiers";
            case "Import" -> "importResponse";
            default -> factoryName + "Response";
        };
        boolean present = Arrays.stream(ClientBatchItemResult.class.getMethods())
                .anyMatch(method -> method.getName().equals(accessor) && method.getParameterCount() == 0);
        if (!present) {
            throw new IllegalStateException("Installed Java package lacks the typed response reader for " + operation);
        }
        return accessor;
    }

    private static Map<String, String> factoryMethods() {
        return Map.ofEntries(
                Map.entry("Discover Versions", "discoverVersions"),
                Map.entry("Ping", "ping"),
                Map.entry("Query", "query"),
                Map.entry("Log", "log"),
                Map.entry("Get Constraints", "getConstraints"),
                Map.entry("Get Attribute List", "getAttributeList"),
                Map.entry("Activate", "activate"),
                Map.entry("Add Attribute", "addAttribute"),
                Map.entry("Adjust Attribute", "adjustAttribute"),
                Map.entry("Archive", "archive"),
                Map.entry("Certify", "certify"),
                Map.entry("Check", "check"),
                Map.entry("Create", "create"),
                Map.entry("Create Key Pair", "createKeyPair"),
                Map.entry("Create Split Key", "createSplitKey"),
                Map.entry("Decrypt", "decrypt"),
                Map.entry("Delegated Login", "delegatedLogin"),
                Map.entry("Delete Attribute", "deleteAttribute"),
                Map.entry("Derive Key", "deriveKey"),
                Map.entry("Destroy", "destroy"),
                Map.entry("Encrypt", "encrypt"),
                Map.entry("Export", "export"),
                Map.entry("Get", "get"),
                Map.entry("Get Attributes", "getAttributes"),
                Map.entry("Get Usage Allocation", "getUsageAllocation"),
                Map.entry("Hash", "hash"),
                Map.entry("Import", "importObject"),
                Map.entry("Interop", "interop"),
                Map.entry("Join Split Key", "joinSplitKey"),
                Map.entry("Locate", "locate"),
                Map.entry("Login", "login"),
                Map.entry("Logout", "logout"),
                Map.entry("MAC", "mac"),
                Map.entry("MAC Verify", "macVerify"),
                Map.entry("Modify Attribute", "modifyAttribute"),
                Map.entry("Obtain Lease", "obtainLease"),
                Map.entry("PKCS#11", "pkcs11"),
                Map.entry("Re-Provision", "reprovision"),
                Map.entry("Re-certify", "recertify"),
                Map.entry("Re-key", "rekey"),
                Map.entry("Re-key Key Pair", "rekeyKeyPair"),
                Map.entry("Recover", "recover"),
                Map.entry("Register", "register"),
                Map.entry("Revoke", "revoke"),
                Map.entry("RNG Retrieve", "rngRetrieve"),
                Map.entry("RNG Seed", "rngSeed"),
                Map.entry("Set Attribute", "setAttribute"),
                Map.entry("Set Constraints", "setConstraints"),
                Map.entry("Set Defaults", "setDefaults"),
                Map.entry("Sign", "sign"),
                Map.entry("Signature Verify", "signatureVerify"),
                Map.entry("Validate", "validate"));
    }

    private static final class RequestInputs implements AutoCloseable {
        private final List<AutoCloseable> owned = new ArrayList<>();
        private final UniqueIdentifier fixtureIdentifier;

        private RequestInputs(UniqueIdentifier fixtureIdentifier) {
            this.fixtureIdentifier = fixtureIdentifier;
        }

        private Object argument(Type type) {
            if (type instanceof Class<?> rawType) return argument(rawType);
            if (type instanceof ParameterizedType parameterized) {
                Type rawType = parameterized.getRawType();
                if (rawType == List.class) return listArgument(parameterized.getActualTypeArguments()[0]);
                if (rawType instanceof Class<?> rawClass) return argument(rawClass);
            }
            throw new IllegalArgumentException("Unsupported installed Java request argument type");
        }

        private Object argument(Class<?> type) {
            if (type == TtlvStructure.class) return own(TtlvStructure.create());
            if (type == TtlvItem.class) return item();
            if (type == TtlvValue.class) return own(TtlvValue.ByteString(new byte[] {1}, LIMITS));
            if (type == UniqueIdentifier.class) return identifier();
            if (type == AttributeReference.class) return attributeReference();
            if (type == RegisterRequest.class) return registerRequest();
            if (type == RevokeRequest.class) return revokeRequest();
            if (type == CodecLimits.class) return LIMITS;
            if (type == byte[].class) return new byte[] {1};
            if (type == int[].class) return new int[] {1};
            if (type == String[].class) return new String[] {"Ping"};
            if (type == long.class || type == Long.class) return 1L;
            if (type == int.class || type == Integer.class) return 1;
            if (type == boolean.class || type == Boolean.class) return Boolean.TRUE;
            throw new IllegalArgumentException("Unsupported installed Java request argument type: " + type.getName());
        }

        private List<?> listArgument(Type elementType) {
            Class<?> elementClass = elementType instanceof Class<?> rawType ? rawType : Object.class;
            if (elementClass == TtlvStructure.class) return List.of(own(TtlvStructure.create()));
            if (elementClass == UniqueIdentifier.class) return List.of(identifier());
            if (elementClass == AttributeReference.class) return List.of(attributeReference());
            if (elementClass == TtlvItem.class) return List.of(item());
            if (elementClass == SplitKeyReference.class) {
                return List.of(own(new SplitKeyReference(identifier(), 1)));
            }
            throw new IllegalArgumentException("Unsupported installed Java request collection argument type");
        }

        private UniqueIdentifier identifier() {
            return fixtureIdentifier == null
                    ? UniqueIdentifier.textString("kmipkit-validator-object") : fixtureIdentifier;
        }

        private AttributeReference attributeReference() {
            return AttributeReference.tag(0x42000a);
        }

        private RegisterRequest registerRequest() {
            TtlvStructure attributes = own(TtlvStructure.create());
            TtlvStructure objectContents = own(TtlvStructure.create());
            TtlvStructure withType = own(TtlvStructure.withItem(objectContents,
                    item(0x420059, TtlvValue.Enumeration(0x80000001L)), LIMITS));
            TtlvStructure withValue = own(TtlvStructure.withItem(withType,
                    item(0x42005a, TtlvValue.ByteString(new byte[] {1}, LIMITS)), LIMITS));
            TtlvItem object = item(0x42005b, TtlvValue.structure(withValue, LIMITS));
            return own(RegisterRequest.create(8, attributes, object));
        }

        private TtlvStructure importAnyObject() {
            TtlvStructure anyObject = own(TtlvStructure.create());
            TtlvStructure opaqueFields = own(TtlvStructure.create());
            TtlvItem opaqueType = item(0x420059, TtlvValue.Enumeration(0x80000001L));
            opaqueFields = own(TtlvStructure.withItem(opaqueFields, opaqueType, LIMITS));
            TtlvItem opaqueValue = item(0x42005a,
                    TtlvValue.ByteString("validator-object".getBytes(StandardCharsets.US_ASCII), LIMITS));
            opaqueFields = own(TtlvStructure.withItem(opaqueFields, opaqueValue, LIMITS));
            TtlvItem opaqueObject = item(0x42005b, TtlvValue.structure(opaqueFields, LIMITS));
            return own(TtlvStructure.withItem(anyObject, opaqueObject, LIMITS));
        }

        private TtlvStructure logoutTicket() {
            TtlvStructure ticket = own(TtlvStructure.create());
            RawTag typeRawTag = own(RawTag.fromRaw(0x42014a));
            Tag typeTag = own(typeRawTag.checked());
            TtlvValue typeValue = own(TtlvValue.Enumeration(1));
            TtlvItem typeItem = own(TtlvItem.create(typeTag, typeValue, LIMITS));
            TtlvStructure withType = own(TtlvStructure.withItem(ticket, typeItem, LIMITS));
            RawTag valueRawTag = own(RawTag.fromRaw(0x42014b));
            Tag valueTag = own(valueRawTag.checked());
            TtlvValue ticketValue = own(TtlvValue.ByteString(new byte[] {1, 2, 3}, LIMITS));
            TtlvItem valueItem = own(TtlvItem.create(valueTag, ticketValue, LIMITS));
            return own(TtlvStructure.withItem(withType, valueItem, LIMITS));
        }

        private RevokeRequest revokeRequest() {
            RevocationReason reason = own(RevocationReason.create(1, "KMIPKit interoperability test"));
            return own(RevokeRequest.create(identifier(), reason, null));
        }

        private TtlvItem item() {
            TtlvStructure name = own(TtlvStructure.create());
            TtlvItem value = item(0x4200a5,
                    TtlvValue.TextString("kmipkit-interoperability-name".getBytes(StandardCharsets.UTF_8), LIMITS));
            name = own(TtlvStructure.withItem(name, value, LIMITS));
            TtlvItem type = item(0x4200a6, TtlvValue.Enumeration(1));
            name = own(TtlvStructure.withItem(name, type, LIMITS));
            return item(0x42000a, TtlvValue.structure(name, LIMITS));
        }

        private TtlvItem item(int raw, TtlvValue value) {
            RawTag rawTag = own(RawTag.fromRaw(raw));
            Tag tag = own(rawTag.checked());
            own(value);
            return own(TtlvItem.create(tag, value, LIMITS));
        }

        private TtlvStructure deriveKeyAttributes() {
            TtlvStructure attributes = own(TtlvStructure.create());
            TtlvItem keyLength = item(0x42002a, TtlvValue.Integer(128));
            return own(TtlvStructure.withItem(attributes, keyLength, LIMITS));
        }

        private TtlvStructure hashParameters() {
            TtlvStructure parameters = own(TtlvStructure.create());
            TtlvItem hashingAlgorithm = item(0x420038,
                    TtlvValue.Enumeration(KmipValues.HashingAlgorithm.SHA_256));
            return own(TtlvStructure.withItem(parameters, hashingAlgorithm, LIMITS));
        }

        private TtlvStructure pkcs11Function() {
            TtlvStructure function = own(TtlvStructure.create());
            TtlvItem functionValue = item(0x42015a, TtlvValue.Integer(1));
            return own(TtlvStructure.withItem(function, functionValue, LIMITS));
        }

        private <T extends AutoCloseable> T own(T value) {
            owned.add(value);
            return value;
        }

        @Override
        public void close() {
            for (int index = owned.size() - 1; index >= 0; index--) {
                try {
                    owned.get(index).close();
                } catch (Exception failure) {
                    throw new IllegalStateException("Failed to close a temporary Java KMIP request value", failure);
                }
            }
        }
    }
}
