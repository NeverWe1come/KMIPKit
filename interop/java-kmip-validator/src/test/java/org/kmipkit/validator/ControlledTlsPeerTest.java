package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.AddAttributeRequest;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ExtensionInformation;
import org.kmipkit.extensions.GetAttributesRequest;
import org.kmipkit.extensions.ModifyAttributeRequest;
import org.kmipkit.extensions.SetAttributeRequest;
import org.kmipkit.extensions.QueryAsyncRequestsRequest;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.extensions.QueryResponse;
import org.kmipkit.extensions.QueryResponseField;
import org.kmipkit.extensions.QueryResponseFieldKind;
import org.kmipkit.credentials.AttestationCredential;
import org.kmipkit.credentials.Credential;
import org.kmipkit.credentials.CredentialType;
import org.kmipkit.credentials.DeviceCredential;
import org.kmipkit.credentials.HashedPasswordCredential;
import org.kmipkit.credentials.UsernameAndPasswordCredential;
import org.kmipkit.message.ResponseMessage;
import org.kmipkit.message.RequestMessage;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvDecodeErrorKind;
import org.kmipkit.ttlv.TtlvDecodeException;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TypedAttribute;

import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLServerSocket;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;
import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.PrintStream;
import java.net.InetAddress;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.KeyFactory;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.SecureRandom;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.security.spec.PKCS8EncodedKeySpec;
import java.util.Base64;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ControlledTlsPeerTest {
    private static final List<String> KMIP_2_0_AND_2_1 = List.of("2.0", "2.1");
    private static final List<String> GENERIC_STRUCTURE_ROUTE_IDS = List.of(
            "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.create",
            "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.withitem",
            "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure",
            "java.generic-ttlv.ttlv_item.create",
            "java.generic-ttlv.function.ttlv_item_view_from_item",
            "java.generic-ttlv.ttlv_codec.decode");

    private record NewAttributeMutationFixture(String methodName, String title, String ordinal,
            int operationValue) { }

    private static final List<NewAttributeMutationFixture> NEW_ATTRIBUTE_MUTATION_FIXTURES = List.of(
            new NewAttributeMutationFixture("addAttribute", "Add Attribute", "002", 0x0d),
            new NewAttributeMutationFixture("modifyAttribute", "Modify Attribute", "003", 0x0e),
            new NewAttributeMutationFixture("setAttribute", "Set Attribute", "002", 0x31));

    private record StructureMemberStructureFixture(String caseId, String targetId, int rawTag,
            int parentTag, boolean repeatable) { }

    private record OperationStructureFixture(String caseId, String targetId, int structureTag,
            int memberTag, TtlvItemType memberType, String textValue, long enumerationValue) { }

    private record ScalarOperationStructureFixture(String caseId, String targetId, int tag,
            TtlvItemType valueType, boolean optional) { }

    private record ProfileInformationMemberFixture(String caseId, String targetId, int rawTag,
            TtlvItemType itemType, int integerValue, String textValue, boolean structure) { }

    private static final String PROFILE_INFORMATION_CASE_ID =
            "test-norm-operation-structure-operation-structure-7-31-profile-information";
    private static final String PROFILE_INFORMATION_TARGET_ID =
            "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-31-PROFILE-INFORMATION";
    private static final String PROFILE_VERSION_CASE_ID =
            "test-norm-operation-structure-operation-structure-7-32-profile-version";
    private static final String PROFILE_VERSION_TARGET_ID =
            "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-32-PROFILE-VERSION";
    private static final String OBJECT_TYPE_MEMBER_CASE_ID =
            "test-norm-structure-member-structure-member-7-24-object-type";
    private static final String OBJECT_TYPE_MEMBER_TARGET_ID =
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-24-OBJECT-TYPE";
    private static final String CONSTRAINT_CASE_ID =
            "test-norm-operation-structure-operation-structure-7-6-constraint";
    private static final String CONSTRAINT_TARGET_ID =
            "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-6-CONSTRAINT";
    private static final String CONSTRAINTS_CASE_ID =
            "test-norm-operation-structure-operation-structure-7-7-constraints";
    private static final String CONSTRAINTS_TARGET_ID =
            "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-7-CONSTRAINTS";
    private static final List<ProfileInformationMemberFixture> PROFILE_INFORMATION_MEMBER_FIXTURES = List.of(
            new ProfileInformationMemberFixture(
                    "test-norm-structure-member-structure-member-7-31-profile-version",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-PROFILE-VERSION", 0x4200ef,
                    TtlvItemType.Structure, 0, "", true),
            new ProfileInformationMemberFixture(
                    "test-norm-structure-member-structure-member-7-31-server-port",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-SERVER-PORT", 0x4200ee,
                    TtlvItemType.Integer, 5696, "", false),
            new ProfileInformationMemberFixture(
                    "test-norm-structure-member-structure-member-7-31-server-uri",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-31-SERVER-URI", 0x4200ed,
                    TtlvItemType.TextString, 0, "https://kms.example.invalid", false),
            new ProfileInformationMemberFixture(
                    "test-norm-structure-member-structure-member-7-32-profile-version-major",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-32-PROFILE-VERSION-MAJOR", 0x4200f0,
                    TtlvItemType.Integer, 1, "", false),
            new ProfileInformationMemberFixture(
                    "test-norm-structure-member-structure-member-7-32-profile-version-minor",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-32-PROFILE-VERSION-MINOR", 0x4200f1,
                    TtlvItemType.Integer, 0, "", false));
    private static final List<String> PROFILE_INFORMATION_CASE_IDS = List.of(
            PROFILE_INFORMATION_CASE_ID, PROFILE_VERSION_CASE_ID,
            PROFILE_INFORMATION_MEMBER_FIXTURES.get(0).caseId(),
            PROFILE_INFORMATION_MEMBER_FIXTURES.get(1).caseId(),
            PROFILE_INFORMATION_MEMBER_FIXTURES.get(2).caseId(),
            PROFILE_INFORMATION_MEMBER_FIXTURES.get(3).caseId(),
            PROFILE_INFORMATION_MEMBER_FIXTURES.get(4).caseId());

    private static final List<OperationStructureFixture> OPERATION_STRUCTURE_FIXTURES = List.of(
            new OperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-23-object-groups",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-23-OBJECT-GROUPS",
                    0x420126, 0x420056, TtlvItemType.TextString, "kmipkit-object-group", 0),
            new OperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-24-object-types",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-24-OBJECT-TYPES",
                    0x420125, 0x420057, TtlvItemType.Enumeration, "", 2));

    private static final List<ScalarOperationStructureFixture> SCALAR_OPERATION_STRUCTURE_FIXTURES = List.of(
            new ScalarOperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-10-data-length",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-10-DATA-LENGTH", 0x4200c4, TtlvItemType.Integer, false),
            new ScalarOperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-14-final-indicator",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-14-FINAL-INDICATOR", 0x4200d8, TtlvItemType.Boolean, false),
            new ScalarOperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-16-interop-identifier",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-16-INTEROP-IDENTIFIER", 0x420161,
                    TtlvItemType.TextString, false),
            new ScalarOperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-19-log-message",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-19-LOG-MESSAGE", 0x420141, TtlvItemType.TextString, false),
            new ScalarOperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-20-mac-data",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-20-MAC-DATA", 0x4200c6, TtlvItemType.ByteString, false),
            new ScalarOperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-3-authenticated-encryption-additional-data",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-3-AUTHENTICATED-ENCRYPTION-ADDITIONAL-DATA",
                    0x4200fe, TtlvItemType.ByteString, true),
            new ScalarOperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-4-authenticated-encryption-tag",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-4-AUTHENTICATED-ENCRYPTION-TAG",
                    0x4200ff, TtlvItemType.ByteString, true),
            new ScalarOperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-17-init-indicator",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-17-INIT-INDICATOR",
                    0x4200d7, TtlvItemType.Boolean, false),
            new ScalarOperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-8-correlation-value",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE",
                    0x4200d6, TtlvItemType.ByteString, false));

    private static final ScalarOperationStructureFixture SIGNATURE_DATA_FIXTURE =
            new ScalarOperationStructureFixture(
                    "test-norm-operation-structure-operation-structure-7-38-signature-data",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-38-SIGNATURE-DATA",
                    0x4200c3, TtlvItemType.ByteString, false);

    private static final List<StructureMemberStructureFixture> STRUCTURE_MEMBER_STRUCTURE_FIXTURES = List.of(
            new StructureMemberStructureFixture(
                    "test-norm-structure-member-structure-member-7-6-attributes",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-6-ATTRIBUTES", 0x420125, 0x420169, false),
            new StructureMemberStructureFixture(
                    "test-norm-structure-member-structure-member-7-6-object-groups",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-6-OBJECT-GROUPS", 0x420166, 0x420169, false),
            new StructureMemberStructureFixture(
                    "test-norm-structure-member-structure-member-7-6-object-types",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-6-OBJECT-TYPES", 0x420167, 0x420169, false),
            new StructureMemberStructureFixture(
                    "test-norm-structure-member-structure-member-7-7-constraint",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-7-CONSTRAINT", 0x420169, 0x420168, true));

    private record StructureMemberByteStringFixture(String caseId, String targetId, boolean typedQueryFilter) { }

    private static final List<StructureMemberByteStringFixture> STRUCTURE_MEMBER_BYTE_STRING_FIXTURES = List.of(
            new StructureMemberByteStringFixture(
                    "test-norm-structure-member-structure-member-7-1-asynchronous-correlation-value",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-1-ASYNCHRONOUS-CORRELATION-VALUE", true),
            new StructureMemberByteStringFixture(
                    "test-norm-structure-member-structure-member-7-2-asynchronous-correlation-value",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-ASYNCHRONOUS-CORRELATION-VALUE", false));

    private record StructureMemberValueFixture(String caseId, String targetId, int rawTag,
            TtlvItemType itemType, long value) { }

    private static final List<StructureMemberValueFixture> STRUCTURE_MEMBER_VALUE_FIXTURES = List.of(
            new StructureMemberValueFixture(
                    "test-norm-structure-member-structure-member-7-2-operation",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-OPERATION", 0x42005c,
                    TtlvItemType.Enumeration, 0x18),
            new StructureMemberValueFixture(
                    "test-norm-structure-member-structure-member-7-2-processing-stage",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-PROCESSING-STAGE", 0x420175,
                    TtlvItemType.Enumeration, 2),
            new StructureMemberValueFixture(
                    "test-norm-structure-member-structure-member-7-2-submission-date",
                    "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-SUBMISSION-DATE", 0x420174,
                    TtlvItemType.DateTimeExtended, 1_800_000_000L));

    private record NormativeTarget(String caseId, String targetId) { }

    private static final String LOG_MESSAGE = "KMIPKit interoperability diagnostic";
    private static final String CREATED_OBJECT_ID = "controlled-create-object-id";
    private static final List<TagRangeFixture> TAG_RANGE_FIXTURES = List.of(
            new TagRangeFixture("test-norm-tag-range-kmipkit-range-001", "KMIPKIT-RANGE-001", 0x000002, "unused"),
            new TagRangeFixture("test-norm-tag-range-kmipkit-range-002", "KMIPKIT-RANGE-002", 0x420009, "reserved"),
            new TagRangeFixture("test-norm-tag-range-kmipkit-range-003", "KMIPKIT-RANGE-003", 0x430000, "unused"),
            new TagRangeFixture("test-norm-tag-range-kmipkit-range-004", "KMIPKIT-RANGE-004", 0x540001, "extension"),
            new TagRangeFixture("test-norm-tag-range-kmipkit-range-005", "KMIPKIT-RANGE-005", 0x550000, "unused"));
    private static final Map<String, String> CRYPTOGRAPHIC_PARAMETER_BUILDER_ROUTES = Map.of(
            "BLOCK-CIPHER-MODE", "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.blockciphermode",
            "CRYPTOGRAPHIC-ALGORITHM", "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.cryptographicalgorithm",
            "DIGITAL-SIGNATURE-ALGORITHM", "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.digitalsignaturealgorithm",
            "HASHING-ALGORITHM", "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.hashingalgorithm",
            "KEY-ROLE-TYPE", "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.keyroletype",
            "MASK-GENERATOR", "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.maskgenerator",
            "PADDING-METHOD", "java.typed-protocol.model.org.kmipkit.protocol.cryptographicparameters.builder.paddingmethod");
    private static final String CREDENTIAL_TYPE_FROM_RAW_ROUTE =
            "java.typed-protocol.model.org.kmipkit.credentials.credentialtype.fromraw";
    private static final String KEY_FORMAT_RAW_CASE_ID =
            "test-norm-enumeration-value-enum-value-key-format-type-raw-00000001";
    private static final String KEY_FORMAT_RAW_AGGREGATE_CASE_ID = "test-enum-key-format-type";
    private static final String KEY_FORMAT_RAW_TARGET_ID =
            "KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RAW-00000001";
    private static final String CLIENT_BATCH_ITEM_RAW_TTLV_ROUTE =
            "java.typed-protocol.function.client_batch_item_response_raw_ttlv";
    private static final String ENDPOINT_ROLE_CLIENT_CASE_ID =
            "test-norm-enumeration-value-enum-value-endpoint-role-client-00000001";
    private static final String ENDPOINT_ROLE_CLIENT_TARGET_ID =
            "KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-CLIENT-00000001";
    private static final String ENDPOINT_ROLE_SERVER_CASE_ID =
            "test-norm-enumeration-value-enum-value-endpoint-role-server-00000002";
    private static final String ENDPOINT_ROLE_SERVER_TARGET_ID =
            "KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-SERVER-00000002";
    private static final List<String> ENDPOINT_ROLE_GENERIC_ROUTES = List.of(
            "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
            "java.generic-ttlv.ttlv_item.create",
            "java.generic-ttlv.function.ttlv_item_view_from_item",
            "java.generic-ttlv.ttlv_codec.decode");
    private static final List<String> TYPED_ATTRIBUTE_ENUMERATION_ROUTES = List.of(
            "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
            "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
            "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew");
    private static final String RESPONSE_BATCH_ITEM_CASE_ID =
            "test-norm-message-field-message-field-8-4-batch-item";
    // Exercise KMIP 2.1-assigned values as unknown Enumeration values in KMIP 2.0.
    private static final Set<String> ENUMS_WITH_KMIP_2_1_ONLY_VALUES = Set.of("Operation", "Result Reason");
    private static final String RESPONSE_BATCH_ITEM_TARGET_ID =
            "KMIPKIT-ELEM-MESSAGE-FIELD-8-4-BATCH-ITEM";
    private static final List<NormativeTarget> RESPONSE_MESSAGE_PARSE_TARGETS = List.of(
            new NormativeTarget("test-norm-message-field-message-field-8-5-batch-count",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-BATCH-COUNT"),
            new NormativeTarget("test-norm-message-field-message-field-8-6-batch-item",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM"),
            new NormativeTarget("test-norm-message-field-message-field-8-6-message-extension",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-MESSAGE-EXTENSION"),
            new NormativeTarget("test-norm-message-field-message-field-9-5-batch-count",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT"),
            new NormativeTarget("test-norm-message-field-message-field-9-7-batch-item",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM"),
            new NormativeTarget("test-norm-message-field-message-field-9-13-message-extension",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION"));
    private static final List<NormativeTarget> REQUEST_MESSAGE_PARSE_TARGETS = List.of(
            new NormativeTarget("test-norm-message-field-message-field-9-5-batch-count",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT"),
            new NormativeTarget("test-norm-message-field-message-field-9-7-batch-item",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM"),
            new NormativeTarget("test-norm-message-field-message-field-9-13-message-extension",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION"));
    private static final List<NormativeTarget> BATCH_ITEM_FIELD_TARGETS = List.of(
            new NormativeTarget("test-norm-message-field-message-field-8-6-batch-item",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM"),
            new NormativeTarget("test-norm-message-field-message-field-9-7-batch-item",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM"));
    private static final Map<String, String> CREDENTIAL_TYPE_MODEL_ROUTES = Map.of(
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-USERNAME-AND-PASSWORD-00000001",
            "java.typed-protocol.model.org.kmipkit.credentials.usernameandpasswordcredential.fromttlv",
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-DEVICE-00000002",
            "java.typed-protocol.model.org.kmipkit.credentials.devicecredential.fromttlv",
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ATTESTATION-00000003",
            "java.typed-protocol.model.org.kmipkit.credentials.attestationcredential.fromttlv",
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-HASHED-PASSWORD-00000005",
            "java.typed-protocol.model.org.kmipkit.credentials.hashedpasswordcredential.fromttlv");
    private static final Map<String, String> CREDENTIAL_TYPE_MODEL_CASE_IDS = Map.of(
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-USERNAME-AND-PASSWORD-00000001",
            "test-norm-credential-credential-username-and-password",
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-DEVICE-00000002",
            "test-norm-credential-credential-device",
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ATTESTATION-00000003",
            "test-norm-credential-credential-attestation",
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-HASHED-PASSWORD-00000005",
            "test-norm-credential-credential-hashed-password");
    private static final Map<String, String> CREDENTIAL_TYPE_MODEL_TARGET_IDS = Map.of(
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-USERNAME-AND-PASSWORD-00000001",
            "KMIPKIT-ELEM-CREDENTIAL-USERNAME-AND-PASSWORD",
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-DEVICE-00000002",
            "KMIPKIT-ELEM-CREDENTIAL-DEVICE",
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ATTESTATION-00000003",
            "KMIPKIT-ELEM-CREDENTIAL-ATTESTATION",
            "KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-HASHED-PASSWORD-00000005",
            "KMIPKIT-ELEM-CREDENTIAL-HASHED-PASSWORD");
    @TempDir
    Path temporaryDirectory;

    @Test
    void enumerationFixturesCoverEveryNamedDirectTagFamily() throws Exception {
        JsonNode catalog = readRouteEvidenceCatalog();
        Map<String, JsonNode> targets = new HashMap<>();
        for (JsonNode target : catalog.path("normative_targets")) {
            targets.put(target.path("target_id").asText(), target);
        }
        Set<String> expectedCaseIds = new java.util.HashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"enumeration_value".equals(requires.path("target_kind").asText())) continue;
            JsonNode target = targets.get(requires.path("normative_target_id").asText());
            if (target == null || !target.path("parent_element_ids").isArray()
                    || target.path("parent_element_ids").isEmpty()) continue;
            String parentName = target.path("parent_element_ids").get(0).asText()
                    .replace("KMIPKIT-ELEM-ENUMERATION-", "");
            boolean hasStandaloneTag = targets.values().stream()
                    .filter(candidate -> "tag".equals(candidate.path("kind").asText()))
                    .anyMatch(candidate -> normalizeTargetName(candidate.path("name").asText())
                            .equals(normalizeTargetName(parentName)));
            boolean hasRepresentativeScalar = !target.path("wire_value").asText().contains("-")
                    || "reserved".equals(target.path("allocation").asText());
            if (hasStandaloneTag && hasRepresentativeScalar) {
                expectedCaseIds.add(testCase.path("case_id").asText());
            }
        }
        Set<String> actualCaseIds = genericEnumerationFixtures().stream()
                .map(EnumerationFixture::caseId).collect(java.util.stream.Collectors.toSet());

        assertEquals(expectedCaseIds, actualCaseIds,
                "Every named enum value with a same-named standalone tag must exercise the generic TTLV routes.");
        assertEndpointRoleCatalogValue(actualCaseIds, ENDPOINT_ROLE_CLIENT_CASE_ID,
                ENDPOINT_ROLE_CLIENT_TARGET_ID, KmipValues.EndpointRole.CLIENT, "Client");
        assertEndpointRoleCatalogValue(actualCaseIds, ENDPOINT_ROLE_SERVER_CASE_ID,
                ENDPOINT_ROLE_SERVER_TARGET_ID, KmipValues.EndpointRole.SERVER, "Server");
    }

    @Test
    void profileNameReservedRangeHasPerTargetEnumerationFixtures() throws Exception {
        String caseId = "test-norm-enumeration-value-enum-value-profile-name-reserved-00000001-00000103";
        List<EnumerationFixture> fixtures = genericEnumerationFixtures().stream()
                .filter(fixture -> fixture.caseId().equals(caseId))
                .toList();

        assertEquals(1, fixtures.size(),
                "The Profile Name reserved-range case must exercise its own generic TTLV routes.");
        assertEquals(0x1L, fixtures.get(0).value(),
                "The generic scalar probe must use the catalogued reserved-range lower boundary.");
        assertEquals(KMIP_2_0_AND_2_1, fixtures.get(0).protocolVersions());
    }

    @Test
    void bitmaskFixturesCoverEveryCataloguedPerTargetValueCase() throws Exception {
        JsonNode catalog = readRouteEvidenceCatalog();
        Map<String, JsonNode> targets = new HashMap<>();
        for (JsonNode target : catalog.path("normative_targets")) {
            targets.put(target.path("target_id").asText(), target);
        }
        Set<String> expected = new java.util.HashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"bitmask_value".equals(requires.path("target_kind").asText())
                    || !"local_model".equals(testCase.path("execution").path("mode").asText())) continue;
            JsonNode target = targets.get(requires.path("normative_target_id").asText());
            if (target == null || !target.path("parent_element_ids").isArray()
                    || target.path("parent_element_ids").isEmpty()) continue;
            String familyName = target.path("parent_element_ids").get(0).asText()
                    .replace("KMIPKIT-ELEM-BITMASK-", "");
            boolean hasStandaloneTag = targets.values().stream()
                    .filter(candidate -> "tag".equals(candidate.path("kind").asText()))
                    .anyMatch(candidate -> normalizeTargetName(candidate.path("name").asText())
                            .equals(normalizeTargetName(familyName)));
            if (hasStandaloneTag) expected.add(testCase.path("case_id").asText());
        }
        Set<String> actual = genericBitmaskFixtures().stream()
                .map(BitmaskFixture::caseId)
                .filter(caseId -> caseId.startsWith("test-norm-bitmask-value-"))
                .collect(java.util.stream.Collectors.toSet());

        assertEquals(expected, actual,
                "Each catalogued bitmask value with a standalone tag needs its own generic and typed routes.");
    }

    // OASIS KMIP 2.1 §6.1.16 and §4.1.2: Discover Versions request and response.
    @Test
    void typedDiscoverVersionsUsesInstalledPackageAndVerifiedMutualTls() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        X509Certificate serverCertificate = certificate(serverCertificatePem);
        X509Certificate clientCertificate = certificate(clientCertificatePem);
        SSLServerSocket listener = listener(serverCertificate, privateKey(serverPrivateKeyPem), clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, discoverVersionsResponse()));
        Thread peerThread = new Thread(peer, "java-validator-controlled-kmip-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem));
        Path manifestPath = Path.of(System.getProperty("kmipkit.artifactManifest"));
        var connection = ConnectionConfig.load(connectionFile);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var testCase = catalog.cases().stream()
                .filter(value -> value.caseId().equals("test-op-discover-versions"))
                .findFirst().orElseThrow();

        ArtifactProvenance.load(manifestPath, PinnedLibraryCommit.read());
        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(testCase);
        } finally {
            listener.close();
        }
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertEquals("passed", outcome.status(), outcome.toString());
        assertEquals("supported", outcome.serverCompatibility());
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4, 0, 0, 0, 30}),
                "The controlled peer must receive the typed Discover Versions operation.");
    }

    @Test
    void typedQueryUsesAValidFunctionAndVerifiesTheTypedResponse() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        X509Certificate serverCertificate = certificate(serverCertificatePem);
        X509Certificate clientCertificate = certificate(clientCertificatePem);
        SSLServerSocket listener = listener(serverCertificate, privateKey(serverPrivateKeyPem), clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, queryResponse()));
        Thread peerThread = new Thread(peer, "java-validator-controlled-query-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("query-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem));
        var connection = ConnectionConfig.load(connectionFile);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var testCase = catalog.cases().stream()
                .filter(value -> value.caseId().equals("test-op-query"))
                .findFirst().orElseThrow();

        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(testCase);
        } finally {
            listener.close();
        }
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4, 0, 0, 0, 0x18}),
                "The controlled peer must receive the typed Query operation.");
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x74, 0x05, 0, 0, 0, 4, 0, 0, 0, 1}),
                "The Query request must include a non-empty Query Function.");
        assertEquals("passed", outcome.status(), outcome.toString());
    }

    @Test
    void objectTypeEnumerationDoesNotUseTypedAttributeRoutes() throws Exception {
        List<EnumerationFixture> objectTypeFixtures = genericEnumerationFixtures().stream()
                .filter(fixture -> "OBJECT-TYPE".equals(fixture.parentName()))
                .toList();

        assertTrue(!objectTypeFixtures.isEmpty(),
                "The normative Object Type enumeration values must have generic TTLV fixtures.");
        assertTrue(objectTypeFixtures.stream().allMatch(fixture -> fixture.attribute() == null),
                "KMIP 2.x §11.34 Object Type values are distinct from the §4.36 Object Type Attribute.");
    }

    @Test
    void controlledQueryProducesPerRouteEvidenceForBothProtocolVersions() throws Exception {
        Path reportPath = Path.of(System.getProperty("kmipkit.routeEvidenceOutput",
                "build/route-execution-evidence.json"));
        Files.deleteIfExists(reportPath);
        ObjectMapper mapper = new ObjectMapper();
        ArrayNode events = mapper.createArrayNode();
        List<EnumerationFixture> enumerationFixtures = genericEnumerationFixtures();
        List<EnumerationFixture> localEnumerationFixtures = genericLocalEnumerationFixtures();
        Set<String> expectedLocalEnumerationCases = cataloguedLocalEnumerationCaseIds(readRouteEvidenceCatalog());
        Set<String> actualLocalEnumerationCases = localEnumerationFixtures.stream()
                .map(EnumerationFixture::caseId).collect(java.util.stream.Collectors.toSet());
        assertEquals(expectedLocalEnumerationCases, actualLocalEnumerationCases,
                "Every local enum aggregate with normative values and a same-named TTLV Tag needs fixtures.");
        List<String> keyFormatTargets = enumerationTargetIds(localEnumerationFixtures, "KEY-FORMAT-TYPE");
        assertEquals(24, keyFormatTargets.size(),
                "The Key Format Type aggregate must cover every catalogued value.");
        List<String> digitalSignatureTargets = enumerationTargetIds(
                localEnumerationFixtures, "DIGITAL-SIGNATURE-ALGORITHM");
        assertEquals(20, digitalSignatureTargets.size(),
                "The Digital Signature Algorithm aggregate must cover every catalogued value.");
        List<String> operationTargets = enumerationTargetIds(localEnumerationFixtures, "OPERATION");
        assertEquals(60, operationTargets.size(), "The Operation aggregate must cover every catalogued value.");
        List<String> resultReasonTargets = enumerationTargetIds(localEnumerationFixtures, "RESULT-REASON");
        assertEquals(78, resultReasonTargets.size(),
                "The Result Reason aggregate must cover every catalogued value.");
        List<String> recommendedCurveTargets = enumerationTargetIds(localEnumerationFixtures, "RECOMMENDED-CURVE");
        assertEquals(71, recommendedCurveTargets.size(),
                "The Recommended Curve aggregate must cover every catalogued value.");
        List<String> nistKeyTypeTargets = enumerationTargetIds(localEnumerationFixtures, "NIST-KEY-TYPE");
        assertEquals(20, nistKeyTypeTargets.size(),
                "The NIST Key Type aggregate must cover every catalogued value.");
        List<String> keyRoleTypeTargets = enumerationTargetIds(localEnumerationFixtures, "KEY-ROLE-TYPE");
        assertEquals(25, keyRoleTypeTargets.size(),
                "The Key Role Type aggregate must cover every catalogued value.");
        List<String> hashingAlgorithmTargets = enumerationTargetIds(
                localEnumerationFixtures, "HASHING-ALGORITHM");
        assertEquals(18, hashingAlgorithmTargets.size(),
                "The Hashing Algorithm aggregate must cover every catalogued value.");
        List<String> linkTypeTargets = enumerationTargetIds(localEnumerationFixtures, "LINK-TYPE");
        assertEquals(15, linkTypeTargets.size(),
                "The Link Type aggregate must cover every catalogued value.");
        List<String> paddingMethodTargets = enumerationTargetIds(
                localEnumerationFixtures, "PADDING-METHOD");
        assertEquals(11, paddingMethodTargets.size(),
                "The Padding Method aggregate must cover every catalogued value.");
        List<String> profileNameTargets = enumerationTargetIds(localEnumerationFixtures, "PROFILE-NAME");
        assertEquals(43, profileNameTargets.size(),
                "The Profile Name aggregate must cover every assigned, reserved, and extension target.");
        List<BitmaskFixture> bitmaskFixtures = genericBitmaskFixtures();
        List<DataTypeFixture> dataTypeFixtures = genericDataTypeFixtures();
        Map<String, JsonNode> dataTypeVectors = dataTypeBoundaryVectors();
        byte[] queryOperation = queryOperationTtlv();
        String requestTreeHash = normalizedTreeSha256(queryOperation);
        int typedTagIdentityEventCount = 0;

        for (String protocolVersion : KMIP_2_0_AND_2_1) {
            appendLocalQueryRouteEvents(events, protocolVersion, requestTreeHash);
            appendTagRangeRouteEvents(events, protocolVersion);
            appendActivationDateLocalRouteEvents(events, protocolVersion);
            appendGenericEnumerationRouteEvents(events, enumerationFixtures, protocolVersion);
            appendGenericLocalEnumerationRouteEvents(events, localEnumerationFixtures, protocolVersion);
            appendGenericBitmaskRouteEvents(events, bitmaskFixtures, protocolVersion);
            appendGenericDataTypeRouteEvents(events, dataTypeFixtures, dataTypeVectors, protocolVersion);
            appendNormativeMessageFieldGenericRouteEvents(events, protocolVersion);
            appendNormativeRequestMessageParseEvents(events, protocolVersion);
            if ("2.1".equals(protocolVersion)) {
                appendAsynchronousRequestOperationStructureRouteEvents(events);
                for (OperationStructureFixture fixture : OPERATION_STRUCTURE_FIXTURES) {
                    appendOperationStructureRouteEvents(events, fixture);
                }
                appendObjectTypeMemberRouteEvents(events);
                appendConstraintStructureRouteEvents(events);
            }
            for (ScalarOperationStructureFixture fixture : SCALAR_OPERATION_STRUCTURE_FIXTURES) {
                appendScalarOperationStructureRouteEvents(events, fixture, protocolVersion);
            }
            appendExtensionInformationRouteEvents(events, protocolVersion);
            appendProfileInformationRouteEvents(events, protocolVersion);
            LocalCatalogRouteChecks.executeTags(readRouteEvidenceCatalog(), protocolVersion, genericOnlyTagCaseIds(),
                    invocation -> appendLocalCatalogInvocation(events, protocolVersion, invocation));
            Set<String> typedTagIdentityCases = missingTypedTagIdentityCaseIds(events, protocolVersion);
            typedTagIdentityEventCount += typedTagIdentityCases.size() * 8;
            LocalCatalogRouteChecks.executeTags(readRouteEvidenceCatalog(), protocolVersion, typedTagIdentityCases,
                    invocation -> appendLocalCatalogInvocation(events, protocolVersion, invocation));
            LocalCatalogRouteChecks.executeTagIdentities(readRouteEvidenceCatalog(), protocolVersion,
                    typedTagIdentityCases,
                    invocation -> appendLocalCatalogInvocation(events, protocolVersion, invocation));
            LocalCatalogRouteChecks.executeReferencedEnums(readRouteEvidenceCatalog(), protocolVersion,
                    invocation -> appendLocalCatalogInvocation(events, protocolVersion, invocation));
            byte[] serverCertificatePem = fixture("server-cert.pem");
            byte[] serverPrivateKeyPem = fixture("server-key.pem");
            byte[] clientCertificatePem = fixture("client-cert.pem");
            byte[] clientPrivateKeyPem = fixture("client-key.pem");
            SSLServerSocket listener = listener(certificate(serverCertificatePem),
                    privateKey(serverPrivateKeyPem), certificate(clientCertificatePem));
            byte[] responseBytes = queryResponse("2.0".equals(protocolVersion) ? 0 : 1);
            assertTrue(contains(responseBytes, new byte[] {0x42, 0x00, 0x0f, 0x01}),
                    "The controlled Query response must contain an 8.4 Batch Item Structure.");
            String responseHash = normalizedTreeSha256(responseBytes);
            appendBatchItemGenericRouteEvents(events, protocolVersion);
            FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, responseBytes));
            Thread peerThread = new Thread(peer, "java-validator-query-route-evidence-peer-" + protocolVersion);
            peerThread.setDaemon(true);
            peerThread.start();

            Path connectionFile = temporaryDirectory.resolve("route-evidence-" + protocolVersion + ".json");
            Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                    clientCertificatePem, clientPrivateKeyPem, false, protocolVersion));
            try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
                try (ClientBatch batch = ClientBatch.create()) {
                    ClientBatchItem request = ClientBatchItem.query(new int[] {1}, null)
                            .withUniqueBatchItemId("item-0".getBytes(StandardCharsets.US_ASCII));
                    assertEquals(0x18L, request.operation());
                    batch.add(request);
                    try (ClientBatchResponse response = driver.executeBatchForEvidence(batch)) {
                        assertEquals(1, response.itemCount());
                        try (ClientBatchItemResult result = response.itemAt(0)) {
                            assertEquals(1, result.extensionCount());
                            appendRouteEvent(events, "test-norm-message-field-message-field-8-6-message-extension",
                                    "java.typed-protocol.function.client_batch_item_response_extension_count",
                                    "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-MESSAGE-EXTENSION",
                                    "org.kmipkit.extensions.ClientBatchItemResult.extensionCount",
                                    null, null, "decode", protocolVersion, null, null, responseHash);
                            appendRouteEvent(events, "test-norm-message-field-message-field-9-13-message-extension",
                                    "java.typed-protocol.function.client_batch_item_response_extension_count",
                                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION",
                                    "org.kmipkit.extensions.ClientBatchItemResult.extensionCount",
                                    null, null, "decode", protocolVersion, null, null, responseHash);
                            appendRouteEvent(events, "test-norm-tag-tag-420051",
                                    "java.typed-protocol.function.client_batch_item_response_extension_count",
                                    "KMIPKIT-ELEM-TAG-420051",
                                    "org.kmipkit.extensions.ClientBatchItemResult.extensionCount",
                                    null, null, "decode", protocolVersion, null, null, responseHash);
                            assertBatchItemResponseRouteEvidence(events, result, protocolVersion, responseHash);
                            appendRouteEvent(events, "test-norm-message-field-message-field-8-5-batch-count",
                                    "java.typed-protocol.function.client_batch_response_item_count",
                                    "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-BATCH-COUNT",
                                    "org.kmipkit.extensions.ClientBatchResponse.itemCount",
                                    null, null, "decode", protocolVersion, null, null, responseHash);
                            appendRouteEvent(events, "test-norm-message-field-message-field-9-5-batch-count",
                                    "java.typed-protocol.function.client_batch_response_item_count",
                                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT",
                                    "org.kmipkit.extensions.ClientBatchResponse.itemCount",
                                    null, null, "decode", protocolVersion, null, null, responseHash);
                            appendRouteEvent(events, "test-norm-message-field-message-field-8-6-batch-item",
                                    "java.typed-protocol.function.client_batch_response_item_at",
                                    "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM",
                                    "org.kmipkit.extensions.ClientBatchResponse.itemAt",
                                    null, null, "decode", protocolVersion, null, null, responseHash);
                            appendRouteEvent(events, "test-norm-message-field-message-field-9-7-batch-item",
                                    "java.typed-protocol.function.client_batch_response_item_at",
                                    "KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM",
                                    "org.kmipkit.extensions.ClientBatchResponse.itemAt",
                                    null, null, "decode", protocolVersion, null, null, responseHash);
                            assertEquals(0x18L, result.operation());
                            assertEquals(0L, result.resultStatus());
                            try (var genericPayload = result.queryResponse().orElseThrow()) {
                                try (var view = genericPayload.view()) {
                                    assertEquals(3, view.itemCount());
                                }
                            }
                            try (QueryResponse typed = result.typedQueryResponse().orElseThrow()) {
                                assertEquals(3, typed.fieldCount());
                                assertEquals(List.of(0x42005c, 0x4200eb, 0x42015f), typed.rawFieldTags());
                                QueryResponseField operation = typed.fieldAt(0);
                                assertEquals(QueryResponseFieldKind.ENUMERATION, operation.kind());
                                assertEquals(0x3bL, operation.enumeration().orElseThrow());
                                // OASIS KMIP 2.0 §7.25 Table 345 and KMIP 2.1 §7.31 Table 383.
                                List<TtlvItemView> profiles = typed.profileInformation();
                                assertEquals(1, profiles.size(),
                                        "The typed Query reader must expose the Profile Information structure.");
                                try (TtlvItemView profile = profiles.get(0);
                                        Tag profileTag = profile.tag();
                                        var profileValue = profile.value();
                                        var profileFields = profileValue.structure()) {
                                    assertEquals(0x4200eb, profileTag.raw());
                                    assertEquals(TtlvItemType.Structure, profile.itemType());
                                    assertProfileInformationFields(profileFields);
                                }
                            }
                        }
                    }
                }
            } finally {
                listener.close();
            }
            byte[] request = peer.get(15, TimeUnit.SECONDS);
            assertEquals(0x18L, requestOperation(request),
                    "The controlled peer must parse the installed Query request operation.");
            assertTrue(contains(request, item(0x420074, 5, intBytes(1))),
                    "The controlled peer must observe the Query Operations function.");
            assertEquals("2.0".equals(protocolVersion) ? 0 : 1, requestProtocolMinorVersion(request));
            String requestHash = normalizedTreeSha256(request);
            appendNetworkQueryRouteEvents(events, protocolVersion, requestHash, responseHash);
            appendControlledActivationDateNetworkEvents(events, protocolVersion);
            appendControlledKeyFormatRawResponseRouteEvents(events, protocolVersion);
            appendPendingQueryCorrelationEvidence(events, protocolVersion, serverCertificatePem,
                    serverPrivateKeyPem, clientCertificatePem, clientPrivateKeyPem);
            appendNewAttributeMutationEvents(events, protocolVersion, serverCertificatePem,
                    serverPrivateKeyPem, clientCertificatePem, clientPrivateKeyPem);
            if ("2.1".equals(protocolVersion)) appendObjectGroupLocalRouteEvents(events);
            if ("2.1".equals(protocolVersion)) appendStructureMemberStructureRouteEvents(events);
            if ("2.1".equals(protocolVersion)) appendStructureMemberByteStringRouteEvents(events);
            if ("2.1".equals(protocolVersion)) appendStructureMemberValueRouteEvents(events,
                    serverCertificatePem, serverPrivateKeyPem, clientCertificatePem, clientPrivateKeyPem);
            assertNewlyCataloguedLocalRoutesAreInvoked(events, protocolVersion);
        }

        writeRouteExecutionReport(reportPath, mapper, events);
        assertTrue(Files.isRegularFile(reportPath),
                "Installed Query calls against the controlled peer must emit route execution evidence.");
        JsonNode report = new ObjectMapper().readTree(reportPath.toFile());
        assertEquals("java", report.path("language").asText());
        assertGenericOnlyTagInventoryEvidence(report.path("events"));
        assertTypedTagIdentityEvidence(report.path("events"));
        assertReferencedPkcs11Evidence(report.path("events"));
        assertTrue(enumerationFixtures.stream().noneMatch(fixture -> fixture.caseId().equals(
                        "test-norm-enumeration-value-profile-name-reserved-00000001-00000103")),
                "A finite Profile Name range must remain pending until each named value is represented.");
        assertEquals(2, routeEventCount(report.path("events"),
                        KEY_FORMAT_RAW_CASE_ID, CLIENT_BATCH_ITEM_RAW_TTLV_ROUTE),
                "The controlled Get response must exercise raw TTLV access to the named Key Format Type value.");
        assertEndpointRoleRouteEvidence(report.path("events"), ENDPOINT_ROLE_CLIENT_CASE_ID,
                ENDPOINT_ROLE_CLIENT_TARGET_ID, KmipValues.EndpointRole.CLIENT, "Client");
        assertEndpointRoleRouteEvidence(report.path("events"), ENDPOINT_ROLE_SERVER_CASE_ID,
                ENDPOINT_ROLE_SERVER_TARGET_ID, KmipValues.EndpointRole.SERVER, "Server");
        assertEquals(2, routeEventCount(report.path("events"),
                        "test-norm-message-field-message-field-8-5-batch-count",
                        "java.typed-protocol.function.client_batch_response_item_count"),
                "The installed batch count accessor must read response Batch Count in KMIP 2.0 and 2.1.");
        assertEquals(2, routeEventCount(report.path("events"),
                        "test-norm-message-field-message-field-8-5-batch-count",
                        "java.typed-protocol.model.org.kmipkit.message.responsemessage.fromttlv"),
                "The response message parser must be exercised against the exact Batch Count case.");
        assertEquals(2, routeEventCount(report.path("events"),
                        "test-norm-message-field-message-field-9-5-batch-count",
                        "java.typed-protocol.function.client_batch_response_item_count"),
                "The installed batch count accessor must read the KMIP 2.1 response Batch Count field.");
        assertEquals(2, routeEventCount(report.path("events"),
                        "test-norm-message-field-message-field-8-6-batch-item",
                        "java.typed-protocol.function.client_batch_response_item_at"),
                "The installed batch item accessor must read response Batch Item in KMIP 2.0 and 2.1.");
        assertEquals(2, routeEventCount(report.path("events"),
                        "test-norm-message-field-message-field-9-7-batch-item",
                        "java.typed-protocol.function.client_batch_response_item_at"),
                "The installed batch item accessor must read the KMIP 2.1 response Batch Item field.");
        assertEquals(0, routeEventCount(report.path("events"), RESPONSE_BATCH_ITEM_CASE_ID,
                        "java.typed-protocol.function.client_batch_item_response_extension_count"),
                "Extension Count must not be attributed to the enclosing Batch Item Structure.");
        assertEquals(2, routeEventCount(report.path("events"),
                        "test-norm-message-field-message-field-8-6-message-extension",
                        "java.typed-protocol.function.client_batch_item_response_extension_count"),
                "Extension Count must read the exact KMIP 2.x §8.6 Message Extension field.");
        assertEquals(2, routeEventCount(report.path("events"),
                        "test-norm-message-field-message-field-9-13-message-extension",
                        "java.typed-protocol.function.client_batch_item_response_extension_count"),
                "Extension Count must read the exact KMIP 2.x §9.13 Message Extension field.");
        assertEquals(2, routeEventCount(report.path("events"), "test-norm-tag-tag-420051",
                        "java.typed-protocol.function.client_batch_item_response_extension_count"),
                "Extension Count must retain the exact Message Extension tag identity.");
        assertEquals(0, routeEventCount(report.path("events"), RESPONSE_BATCH_ITEM_CASE_ID,
                        "java.typed-protocol.function.client_batch_item_response_raw_ttlv"),
                "Raw response TTLV must not be attributed to the request Batch Item field.");
        assertEquals(2, routeEventCount(report.path("events"),
                        "test-norm-message-field-message-field-8-6-batch-item",
                        "java.typed-protocol.function.client_batch_item_response_raw_ttlv"),
                "Raw response TTLV must use the KMIP 2.x §8.6 Batch Item field.");
        assertEquals(2, routeEventCount(report.path("events"),
                        "test-norm-message-field-message-field-9-7-batch-item",
                        "java.typed-protocol.function.client_batch_item_response_raw_ttlv"),
                "Raw response TTLV must use the KMIP 2.x §9.7 Batch Item field.");
        assertEquals(0, routeEventCount(report.path("events"), RESPONSE_BATCH_ITEM_CASE_ID,
                        "java.typed-protocol.function.client_batch_item_response_extension_ttlv_at"),
                "Extension TTLV access must not be attributed to the enclosing Batch Item field.");
        for (String caseId : List.of("test-norm-message-field-message-field-8-6-message-extension",
                "test-norm-message-field-message-field-9-13-message-extension", "test-norm-tag-tag-420051")) {
            assertEquals(2, routeEventCount(report.path("events"), caseId,
                            "java.typed-protocol.function.client_batch_item_response_extension_ttlv_at"),
                    "Extension TTLV access must preserve each exact Message Extension identity: " + caseId);
        }
        assertEquals(0, routeEventCount(report.path("events"), RESPONSE_BATCH_ITEM_CASE_ID,
                        "java.typed-protocol.function.client_batch_response_item_count"),
                "Batch Count access must not be attributed to the structural Batch Item case.");
        assertEquals(0, routeEventCount(report.path("events"), RESPONSE_BATCH_ITEM_CASE_ID,
                        "java.typed-protocol.function.client_batch_response_item_at"),
                "The response item accessor must use its exact normative Batch Item cases.");
        for (String protocolVersion : KMIP_2_0_AND_2_1) {
            assertEquals("KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-AEAD-00000012",
                    findRouteEvent(report.path("events"), "test-enum-block-cipher-mode",
                            "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                            protocolVersion, "KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-AEAD-00000012")
                            .path("target_id").asText(),
                    "Each named Block Cipher Mode value must be constructed through generic TTLV.");
            assertEquals("KMIPKIT-ELEM-MESSAGE-FIELD-8-5-BATCH-COUNT",
                    findRouteEvent(report.path("events"),
                            "test-norm-message-field-message-field-8-5-batch-count",
                            "java.typed-protocol.function.client_batch_response_item_count",
                            protocolVersion, "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-BATCH-COUNT")
                            .path("target_id").asText());
            assertEquals("KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT",
                    findRouteEvent(report.path("events"),
                            "test-norm-message-field-message-field-9-5-batch-count",
                            "java.typed-protocol.function.client_batch_response_item_count",
                            protocolVersion, "KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT")
                            .path("target_id").asText());
            assertEquals("KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM",
                    findRouteEvent(report.path("events"),
                            "test-norm-message-field-message-field-8-6-batch-item",
                            "java.typed-protocol.function.client_batch_response_item_at",
                            protocolVersion, "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM")
                            .path("target_id").asText());
            assertEquals("KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM",
                    findRouteEvent(report.path("events"),
                            "test-norm-message-field-message-field-9-7-batch-item",
                            "java.typed-protocol.function.client_batch_response_item_at",
                            protocolVersion, "KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM")
                            .path("target_id").asText());
        }
        List<String> rotateNameTypeTargets = List.of(
                "KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001",
                "KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-URI-00000002",
                "KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-EXTENSIONS-8XXXXXXX");
        List<String> rotateNameTypeRoutes = List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode");
        for (String route : rotateNameTypeRoutes) {
            assertEquals(rotateNameTypeTargets.size(),
                    routeEventCount(report.path("events"), "test-enum-rotate-name-type", route),
                    "Every Rotate Name Type value must complete each applicable generic TTLV route.");
            for (String targetId : rotateNameTypeTargets) {
                JsonNode event = findRouteEvent(report.path("events"), "test-enum-rotate-name-type",
                        route, "2.1", targetId);
                assertEquals("passed", event.path("status").asText());
            }
        }
        List<String> genericEnumerationRoutes = List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode");
        List<String> cryptographicAlgorithmTargets = enumerationTargetIds(
                genericEnumerationFixtures(), "CRYPTOGRAPHIC-ALGORITHM");
        assertEquals(57, cryptographicAlgorithmTargets.size(),
                "The local Cryptographic Algorithm aggregate must cover every catalogued target value.");
        JsonNode routeCatalog = readRouteEvidenceCatalog();
        List<String> pkcs11FunctionTargets = referencedEnumerationTargetIds(routeCatalog, "PKCS#11 Function");
        List<String> pkcs11ReturnCodeTargets = referencedEnumerationTargetIds(routeCatalog, "PKCS#11 Return Code");
        assertEquals(92, pkcs11FunctionTargets.size(), "The PKCS#11 Function aggregate must cover every value.");
        assertEquals(97, pkcs11ReturnCodeTargets.size(),
                "The PKCS#11 Return Code aggregate must cover every value.");
        List<LocalEnumerationScope> additionalEnumerationScopes = List.of(
                new LocalEnumerationScope("test-enum-key-format-type", keyFormatTargets,
                        KMIP_2_0_AND_2_1, TYPED_ATTRIBUTE_ENUMERATION_ROUTES),
                new LocalEnumerationScope("test-enum-digital-signature-algorithm", digitalSignatureTargets,
                        KMIP_2_0_AND_2_1, concatRoutes(
                                TYPED_ATTRIBUTE_ENUMERATION_ROUTES,
                                CRYPTOGRAPHIC_PARAMETER_BUILDER_ROUTES.get("DIGITAL-SIGNATURE-ALGORITHM"))),
                genericEnumerationScope("test-enum-operation", operationTargets, KMIP_2_0_AND_2_1),
                genericEnumerationScope("test-enum-result-reason", resultReasonTargets, KMIP_2_0_AND_2_1),
                genericEnumerationScope("test-enum-recommended-curve", recommendedCurveTargets,
                        KMIP_2_0_AND_2_1),
                new LocalEnumerationScope("test-enum-nist-key-type", nistKeyTypeTargets,
                        KMIP_2_0_AND_2_1, TYPED_ATTRIBUTE_ENUMERATION_ROUTES),
                cryptographicParameterBuilderScope("test-enum-key-role-type", keyRoleTypeTargets,
                        "KEY-ROLE-TYPE"),
                cryptographicParameterBuilderScope("test-enum-hashing-algorithm", hashingAlgorithmTargets,
                        "HASHING-ALGORITHM"),
                genericEnumerationScope("test-enum-link-type", linkTypeTargets, KMIP_2_0_AND_2_1),
                cryptographicParameterBuilderScope("test-enum-padding-method", paddingMethodTargets,
                        "PADDING-METHOD"),
                new LocalEnumerationScope("test-enum-opaque-data-type", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-OPAQUE-DATA-TYPE-EXTENSIONS-8XXXXXXX"),
                        KMIP_2_0_AND_2_1, List.of(
                                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                                "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew")),
                new LocalEnumerationScope("test-enum-processing-stage", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-COMPLETED-00000003",
                        "KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-IN-PROCESS-00000002",
                        "KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-SUBMITTED-00000001"),
                        List.of("2.1"), List.of()),
                new LocalEnumerationScope("test-enum-ticket-type", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-TICKET-TYPE-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-TICKET-TYPE-LOGIN-00000001"),
                        KMIP_2_0_AND_2_1, List.of()),
                cryptographicParameterBuilderScope("test-enum-mask-generator", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-MASK-GENERATOR-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-MASK-GENERATOR-MFG1-00000001"), "MASK-GENERATOR"),
                new LocalEnumerationScope("test-enum-encoding-option", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-NO-ENCODING-00000001",
                        "KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-TTLV-ENCODING-00000002"),
                        KMIP_2_0_AND_2_1, List.of()),
                new LocalEnumerationScope("test-enum-endpoint-role", List.of(
                        ENDPOINT_ROLE_CLIENT_TARGET_ID,
                        "KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-EXTENSIONS-8XXXXXXX",
                        ENDPOINT_ROLE_SERVER_TARGET_ID), KMIP_2_0_AND_2_1, List.of()),
                new LocalEnumerationScope("test-enum-key-value-location-type", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-UNINTERPRETED-TEXT-STRING-00000001",
                        "KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-URI-00000002"),
                        KMIP_2_0_AND_2_1, List.of()),
                new LocalEnumerationScope("test-enum-key-wrap-type", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-AS-REGISTERED-00000002",
                        "KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-NOT-WRAPPED-00000001"),
                        KMIP_2_0_AND_2_1, List.of()),
                new LocalEnumerationScope("test-enum-name-type", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001",
                        "KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-URI-00000002"),
                        KMIP_2_0_AND_2_1, List.of()),
                new LocalEnumerationScope("test-enum-object-group-member", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-GROUP-MEMBER-DEFAULT-00000002",
                        "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-GROUP-MEMBER-FRESH-00000001"),
                        KMIP_2_0_AND_2_1, List.of()),
                new LocalEnumerationScope("test-enum-put-function", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-NEW-00000001",
                        "KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-REPLACE-00000002"),
                        KMIP_2_0_AND_2_1, List.of()),
                new LocalEnumerationScope("test-enum-secret-data-type", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-PASSWORD-00000001",
                        "KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-SEED-00000002"),
                        KMIP_2_0_AND_2_1, List.of()),
                new LocalEnumerationScope("test-enum-usage-limits-unit", List.of(
                        "KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-BYTE-00000001",
                        "KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-EXTENSIONS-8XXXXXXX",
                        "KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-OBJECT-00000002"),
                        KMIP_2_0_AND_2_1, List.of()),
                new LocalEnumerationScope("test-enum-cryptographic-algorithm", cryptographicAlgorithmTargets,
                        KMIP_2_0_AND_2_1, concatRoutes(
                                TYPED_ATTRIBUTE_ENUMERATION_ROUTES,
                                CRYPTOGRAPHIC_PARAMETER_BUILDER_ROUTES.get("CRYPTOGRAPHIC-ALGORITHM"))),
                new LocalEnumerationScope("test-enum-pkcs#11-function", pkcs11FunctionTargets,
                        KMIP_2_0_AND_2_1, List.of()),
                new LocalEnumerationScope("test-enum-pkcs#11-return-code", pkcs11ReturnCodeTargets,
                        KMIP_2_0_AND_2_1, List.of()));
        for (LocalEnumerationScope scope : additionalEnumerationScopes) {
            List<String> routes = new ArrayList<>(genericEnumerationRoutes);
            routes.addAll(scope.additionalRoutes());
            for (String route : routes) {
                assertEquals(scope.targetIds().size() * scope.protocolVersions().size(),
                        routeEventCount(report.path("events"), scope.caseId(), route),
                        "Every catalogued value must complete each applicable route for " + scope.caseId());
                for (String version : scope.protocolVersions()) {
                    for (String targetId : scope.targetIds()) {
                        JsonNode event = findRouteEvent(report.path("events"), scope.caseId(), route,
                                version, targetId);
                        assertEquals("passed", event.path("status").asText());
                    }
                }
            }
        }
        assertEquals(2, routeEventCount(report.path("events"), "test-enum-key-format-type",
                        CLIENT_BATCH_ITEM_RAW_TTLV_ROUTE),
                "The aggregate Key Format Type case must capture the controlled Raw value responses.");
        for (String version : KMIP_2_0_AND_2_1) {
            JsonNode event = findRouteEvent(report.path("events"), "test-enum-key-format-type",
                    CLIENT_BATCH_ITEM_RAW_TTLV_ROUTE, version, KEY_FORMAT_RAW_TARGET_ID);
            assertEquals("passed", event.path("status").asText());
        }
        assertEquals(2, routeEventCount(report.path("events"), RESPONSE_BATCH_ITEM_CASE_ID,
                        "java.typed-protocol.function.client_batch_item_response_pending_correlation_value"),
                "A controlled Pending response must expose its server-issued correlation value.");
        for (EnumerationFixture fixture : enumerationFixtures) {
            String builderRoute = CRYPTOGRAPHIC_PARAMETER_BUILDER_ROUTES.get(fixture.parentName());
            if (builderRoute != null) {
                assertEquals(fixture.protocolVersions().size(),
                        routeEventCount(report.path("events"), fixture.caseId(), builderRoute),
                        "Every KMIP Cryptographic Parameters builder value needs a public-call event: "
                                + fixture.targetId());
            }
            if ("BATCH-ERROR-CONTINUATION-OPTION".equals(fixture.parentName())) {
                assertEquals(fixture.protocolVersions().size(),
                        routeEventCount(report.path("events"), fixture.caseId(),
                                "java.typed-protocol.function.client_batch_add_error_continuation_value"),
                        "Every Batch Error Continuation Option value must reach the installed ClientBatch API.");
            }
            if ("CREDENTIAL-TYPE".equals(fixture.parentName())) {
                assertEquals(fixture.protocolVersions().size(),
                        routeEventCount(report.path("events"), fixture.caseId(), CREDENTIAL_TYPE_FROM_RAW_ROUTE),
                        "Every Credential Type value needs an installed fromRaw call: " + fixture.targetId());
                String credentialRoute = CREDENTIAL_TYPE_MODEL_ROUTES.get(fixture.targetId());
                if (credentialRoute != null) {
                    String credentialCaseId = CREDENTIAL_TYPE_MODEL_CASE_IDS.get(fixture.targetId());
                    String credentialTargetId = CREDENTIAL_TYPE_MODEL_TARGET_IDS.get(fixture.targetId());
                    for (String version : fixture.protocolVersions()) {
                        JsonNode envelopeEvent = findRouteEvent(report.path("events"), credentialCaseId,
                                "java.typed-protocol.model.org.kmipkit.credentials.credential.fromttlv",
                                version, credentialTargetId);
                        assertEquals("org.kmipkit.credentials.Credential.fromTtlv",
                                envelopeEvent.path("public_api_symbol").asText());
                        assertEquals(credentialTargetId,
                                findRouteEvent(report.path("events"), credentialCaseId,
                                        credentialRoute, version, credentialTargetId)
                                        .path("target_id").asText(),
                                "Credential Value conversion evidence must use its structural catalog case.");
                        for (String genericRoute : List.of(
                                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure",
                                "java.generic-ttlv.ttlv_item.create",
                                "java.generic-ttlv.function.ttlv_item_view_from_item",
                                "java.generic-ttlv.ttlv_codec.decode")) {
                            JsonNode genericEvent = findRouteEvent(report.path("events"), credentialCaseId,
                                    genericRoute, version, credentialTargetId);
                            assertEquals("passed", genericEvent.path("status").asText());
                        }
                    }
                }
            }
        }
        int enumerationEventCount = enumerationFixtures.stream()
                .mapToInt(ControlledTlsPeerTest::expectedEnumerationRouteEvents).sum();
        int localEnumerationEventCount = localEnumerationFixtures.stream()
                .mapToInt(ControlledTlsPeerTest::expectedEnumerationRouteEvents).sum();
        int bitmaskEventCount = bitmaskFixtures.stream()
                .mapToInt(fixture -> fixture.protocolVersions().size() * (4 + (fixture.attribute() == null ? 0 : 3)))
                .sum();
        int batchItemAccessorEventCount = KMIP_2_0_AND_2_1.size() * 4;
        int batchItemExtensionCountEventCount = KMIP_2_0_AND_2_1.size() * 3;
        int batchItemGenericRouteEventCount = KMIP_2_0_AND_2_1.size() * 4;
        int batchItemAdditionalResponseRouteEventCount = KMIP_2_0_AND_2_1.size() * 12;
        int normativeMessageFieldGenericRouteEventCount = KMIP_2_0_AND_2_1.size() * 31;
        int normativeMessageFieldParserRouteEventCount = KMIP_2_0_AND_2_1.size() * 10;
        int normativeBatchItemAccessorRouteEventCount = KMIP_2_0_AND_2_1.size() * 10;
        int newAttributePayloadRouteEventCount = KMIP_2_0_AND_2_1.size() * 3 * 20;
        int objectGroupLocalRouteEventCount = 3;
        int asynchronousRequestOperationStructureRouteEventCount = 6;
        int operationStructureRouteEventCount = OPERATION_STRUCTURE_FIXTURES.size() * 6;
        int scalarOperationStructureRouteEventCount = SCALAR_OPERATION_STRUCTURE_FIXTURES.size()
                * KMIP_2_0_AND_2_1.size() * 6;
        int extensionInformationRouteEventCount = KMIP_2_0_AND_2_1.size() * 7;
        int profileInformationRouteEventCount = KMIP_2_0_AND_2_1.size()
                * (2 * GENERIC_STRUCTURE_ROUTE_IDS.size() + 4 * PROFILE_INFORMATION_MEMBER_FIXTURES.size());
        int objectTypeMemberRouteEventCount = 4;
        int constraintStructureRouteEventCount = 2 * GENERIC_STRUCTURE_ROUTE_IDS.size();
        int pendingCorrelationAccessorEventCount = KMIP_2_0_AND_2_1.size();
        int structureMemberStructureRouteEventCount = STRUCTURE_MEMBER_STRUCTURE_FIXTURES.size() * 4;
        int structureMemberByteStringRouteEventCount = STRUCTURE_MEMBER_BYTE_STRING_FIXTURES.stream()
                .mapToInt(fixture -> 4 + (fixture.typedQueryFilter() ? 1 : 0)).sum();
        int structureMemberValueRouteEventCount = STRUCTURE_MEMBER_VALUE_FIXTURES.size() * 4 + 1;
        int credentialGenericRouteEventCount = CREDENTIAL_TYPE_MODEL_CASE_IDS.size()
                * KMIP_2_0_AND_2_1.size() * 4;
        int credentialEnvelopeEventCount = CREDENTIAL_TYPE_MODEL_CASE_IDS.size()
                * KMIP_2_0_AND_2_1.size();
        int batchErrorContinuationEventCount = enumerationFixtures.stream()
                .filter(fixture -> "BATCH-ERROR-CONTINUATION-OPTION".equals(fixture.parentName()))
                .mapToInt(fixture -> fixture.protocolVersions().size()).sum();
        int genericTagEventCount = 0;
        int referencedEnumEventCount = 0;
        int referencedEnumAggregateEventCount = 0;
        int referencedEnumCaseCount = 0;
        Set<String> genericTagCaseIds = genericOnlyTagCaseIds();
        for (JsonNode testCase : readRouteEvidenceCatalog().path("test_cases")) {
            if ("referenced_enum_value".equals(testCase.path("requires").path("target_kind").asText())) {
                referencedEnumEventCount += 4 * testCase.path("execution").path("protocol_versions").size();
                referencedEnumAggregateEventCount +=
                        4 * testCase.path("execution").path("protocol_versions").size();
                referencedEnumCaseCount++;
            }
            if (!genericTagCaseIds.contains(testCase.path("case_id").asText())) continue;
            int routes = "reserved".equals(testCase.path("requires").path("allocation").asText()) ? 3 : 5;
            genericTagEventCount += routes * testCase.path("execution").path("protocol_versions").size();
        }
        assertEquals(68 + enumerationEventCount + localEnumerationEventCount + bitmaskEventCount + 94
                        + batchItemAccessorEventCount
                        + batchItemExtensionCountEventCount + batchItemGenericRouteEventCount
                        + batchItemAdditionalResponseRouteEventCount + credentialGenericRouteEventCount
                        + normativeMessageFieldGenericRouteEventCount + normativeMessageFieldParserRouteEventCount
                        + normativeBatchItemAccessorRouteEventCount + newAttributePayloadRouteEventCount
                        + credentialEnvelopeEventCount + objectGroupLocalRouteEventCount
                        + asynchronousRequestOperationStructureRouteEventCount
                        + operationStructureRouteEventCount
                        + scalarOperationStructureRouteEventCount
                        + extensionInformationRouteEventCount
                        + profileInformationRouteEventCount
                        + objectTypeMemberRouteEventCount
                        + constraintStructureRouteEventCount
                        + pendingCorrelationAccessorEventCount
                        + structureMemberStructureRouteEventCount + structureMemberByteStringRouteEventCount
                        + structureMemberValueRouteEventCount
                        + batchErrorContinuationEventCount + genericTagEventCount + typedTagIdentityEventCount
                        + referencedEnumEventCount + referencedEnumAggregateEventCount,
                report.path("events").size());
        long bitmaskEvents = 0;
        for (JsonNode event : report.path("events")) {
            if ("test-bitmask-cryptographic-usage-mask".equals(event.path("case_id").asText())) {
                bitmaskEvents++;
            }
        }
        assertEquals(350, bitmaskEvents,
                "Every named Cryptographic Usage Mask bit needs all seven local routes in both versions.");
        long dataTypeEvents = 0;
        for (JsonNode event : report.path("events")) {
            if (event.path("case_id").asText().startsWith("test-norm-data-type-data-type-")) {
                dataTypeEvents++;
            }
        }
        assertEquals(92, dataTypeEvents,
                "Every TTLV data type needs its installed constructor, item view, and decoder routes.");
        long bitmaskCaseCount = bitmaskFixtures.stream().map(BitmaskFixture::caseId).distinct().count();
        long dataTypeCaseCount = dataTypeFixtures.stream().map(DataTypeFixture::caseId).distinct().count();
        long localEnumerationCaseCount = localEnumerationFixtures.stream()
                .map(EnumerationFixture::caseId).distinct().count();
        assertEquals(16 + OPERATION_STRUCTURE_FIXTURES.size() + SCALAR_OPERATION_STRUCTURE_FIXTURES.size()
                        + enumerationFixtures.size() + localEnumerationCaseCount + bitmaskCaseCount
                        + dataTypeCaseCount
                        + CREDENTIAL_TYPE_MODEL_CASE_IDS.size() + genericTagCaseIds.size()
                        + 3
                        + typedTagIdentityCaseIds().size() + referencedEnumCaseCount
                        + 11 + STRUCTURE_MEMBER_STRUCTURE_FIXTURES.size()
                        + STRUCTURE_MEMBER_BYTE_STRING_FIXTURES.size()
                        + STRUCTURE_MEMBER_VALUE_FIXTURES.size() + PROFILE_INFORMATION_CASE_IDS.size(),
                report.path("case_ids").size());
        List<String> expectedCredentialCaseIds = CREDENTIAL_TYPE_MODEL_CASE_IDS.values().stream().sorted().toList();
        int credentialCaseStart = report.path("case_ids").size() - expectedCredentialCaseIds.size();
        List<String> reportedCredentialCaseIds = new ArrayList<>();
        for (int index = credentialCaseStart; index < report.path("case_ids").size(); index++) {
            reportedCredentialCaseIds.add(report.path("case_ids").get(index).asText());
        }
        assertEquals(expectedCredentialCaseIds, reportedCredentialCaseIds,
                "Credential structural case identifiers must keep deterministic catalog order.");
        assertEquals("test-op-query", report.path("case_ids").get(0).asText());
        for (int index = 1; index < 6; index++) {
            assertEquals("test-norm-tag-range-kmipkit-range-%03d".formatted(index),
                    report.path("case_ids").get(index).asText());
        }
        assertEquals("test-attr-activation-date", report.path("case_ids").get(6).asText());
        assertEquals(2, report.path("protocol_versions").size());
        assertEquals("org.kmipkit.extensions.Client.execute",
                findEvent(report.path("events"), "send", "2.0").path("execution_api_symbol").asText());
        assertEquals("org.kmipkit.extensions.ClientBatchItemResult.queryResponse",
                findRouteEvent(report.path("events"),
                        "java.typed-protocol.function.client_batch_item_response_query", "2.1")
                        .path("public_api_symbol").asText());
    }

    @Test
    void typedClientPreservesKmip20OnTheWire() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem),
                privateKey(serverPrivateKeyPem), certificate(clientCertificatePem));
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, queryResponse(0)));
        Thread peerThread = new Thread(peer, "java-validator-kmip-20-controlled-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("kmip-20-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, false, "2.0"));
        var connection = ConnectionConfig.load(connectionFile);
        assertEquals("2.0", connection.protocolVersion());
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var query = catalog.cases().stream().filter(value -> value.caseId().equals("test-op-query"))
                .findFirst().orElseThrow();

        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(query);
        } finally {
            listener.close();
        }
        assertEquals("passed", outcome.status(), outcome.toString());
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertTrue(contains(request, structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(0)))),
                "The installed Java client must preserve KMIP 2.0 in the request header: "
                        + java.util.HexFormat.of().formatHex(request));
    }

    @Test
    void asynchronousOperationsUseTheirInstalledDedicatedExecutorsAndTypedResponses() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var operationCases = casesById(catalog, "test-op-cancel", "test-op-poll", "test-op-process",
                "test-op-query-asynchronous-requests");
        var peer = new FutureTask<List<byte[]>>(() -> serveAsynchronousResponses(listener, operationCases.size()));
        Thread peerThread = new Thread(peer, "java-validator-controlled-async-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("async-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));
        byte[] correlation = {0x4b, 0x4d, 0x49, 0x50, 0x2d, 0x74, 0x65, 0x73, 0x74};
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
            for (var testCase : operationCases) {
                var outcome = driver.executeAsynchronousRequest(testCase, correlation);
                assertEquals("passed", outcome.status(), testCase.operation() + ": " + outcome);
            }
        } finally {
            listener.close();
        }

        var requests = peer.get(20, TimeUnit.SECONDS);
        assertEquals(List.of(0x19L, 0x1aL, 0x3aL, 0x39L),
                requests.stream().map(ControlledTlsPeerTest::requestOperation).toList());
        for (byte[] request : requests) {
            assertTrue(contains(request, item(0x420006, 8, correlation)),
                    "The public asynchronous route must send the caller-supplied correlation value.");
        }
    }

    @Test
    void asynchronousCancelAndPollPreserveKmip20WithTheInstalledExecutors() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var operationCases = casesById(catalog, "test-op-cancel", "test-op-poll");
        var peer = new FutureTask<List<byte[]>>(() -> serveAsynchronousResponses(listener, operationCases.size(), 0));
        Thread peerThread = new Thread(peer, "java-validator-controlled-kmip20-async-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("kmip20-async-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true, "2.0"));
        byte[] correlation = {0x20, 0x20, 0x20, 0x30};
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
            for (var testCase : operationCases) {
                var outcome = driver.executeAsynchronousRequest(testCase, correlation);
                assertEquals("passed", outcome.status(), testCase.operation() + ": " + outcome);
            }
        } finally {
            listener.close();
        }

        var requests = peer.get(20, TimeUnit.SECONDS);
        assertEquals(List.of(0x19L, 0x1aL),
                requests.stream().map(ControlledTlsPeerTest::requestOperation).toList());
        assertTrue(requests.stream().allMatch(request -> requestProtocolMinorVersion(request) == 0),
                "KMIP 2.0 asynchronous requests must retain the configured version on the wire.");
    }

    @Test
    void processAndQueryAsyncRejectKmip20BeforeOpeningTheTransport() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        listener.setSoTimeout(3_000);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var operationCases = casesById(catalog, "test-v20-reject-process", "test-v20-reject-query-asynchronous-requests");
        Path connectionFile = temporaryDirectory.resolve("kmip20-async-boundary-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, false, "2.0"));
        byte[] correlation = {0x20, 0x20, 0x20, 0x31};
        FutureTask<Boolean> peer = new FutureTask<>(() -> acceptsConnection(listener));
        Thread peerThread = new Thread(peer, "java-validator-kmip20-async-boundary-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        boolean accepted;
        try {
            try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
                for (var testCase : operationCases) {
                    var outcome = driver.executeAsynchronousRequest(testCase, correlation);
                    assertEquals("failed", outcome.status(), testCase.operation() + ": " + outcome);
                    assertTrue(outcome.diagnostic().contains("InvalidInputException"),
                            testCase.operation() + ": " + outcome);
                }
            }
            accepted = peer.get(5, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }
        assertEquals(false, accepted,
                "KMIP 2.0 must reject Process and Query Asynchronous Requests before transport invocation.");
    }

    @Test
    void typedLogSendsTheRequiredMessageAndVerifiesAnEmptyResponsePayload() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        X509Certificate serverCertificate = certificate(serverCertificatePem);
        X509Certificate clientCertificate = certificate(clientCertificatePem);
        SSLServerSocket listener = listener(serverCertificate, privateKey(serverPrivateKeyPem), clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, logResponse()));
        Thread peerThread = new Thread(peer, "java-validator-controlled-log-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("log-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem));
        var connection = ConnectionConfig.load(connectionFile);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var testCase = catalog.cases().stream()
                .filter(value -> value.caseId().equals("test-op-log"))
                .findFirst().orElseThrow();

        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(testCase);
        } finally {
            listener.close();
        }
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4, 0, 0, 0, 0x2c}),
                "The controlled peer must receive the typed Log operation.");
        assertTrue(contains(request, item(0x420141, 7, LOG_MESSAGE.getBytes(StandardCharsets.UTF_8))),
                "The typed Log request must contain the deterministic Log Message.");
        assertEquals("passed", outcome.status(), outcome.toString());
    }

    @Test
    void typedCreateUsesAnIsolatedNameAndDestroysTheCreatedObject() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        X509Certificate serverCertificate = certificate(serverCertificatePem);
        X509Certificate clientCertificate = certificate(clientCertificatePem);
        SSLServerSocket listener = listener(serverCertificate, privateKey(serverPrivateKeyPem), clientCertificate);
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> serveCreateCleanup(listener));
        Thread peerThread = new Thread(peer, "java-validator-controlled-create-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("create-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));
        var connection = ConnectionConfig.load(connectionFile);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var testCase = catalog.cases().stream()
                .filter(value -> value.caseId().equals("test-op-create"))
                .findFirst().orElseThrow();

        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(testCase);
        } finally {
            listener.close();
        }
        assertEquals("passed", outcome.status(), outcome.toString());
        List<byte[]> requests = peer.get(15, TimeUnit.SECONDS);
        assertEquals(2, requests.size());
        assertTrue(contains(requests.get(0), new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4, 0, 0, 0, 1}),
                "The controlled peer must receive Create before cleanup.");
        assertTrue(contains(requests.get(0), new byte[] {0x42, 0x00, 0x53, 1}),
                "The Create request must include an isolated Name attribute.");
        assertTrue(contains(requests.get(0), "kmipkit-create-".getBytes(StandardCharsets.UTF_8)),
                "The Create Name must use the catalog's unique prefix.");
        assertTrue(contains(requests.get(0), item(0x420054, 5, intBytes(1))),
                "The Create Name must include the Name Type enumeration.");
        assertTrue(contains(requests.get(0), item(0x420028, 5, intBytes(3))),
                "The isolated object must request AES.");
        assertTrue(contains(requests.get(0), item(0x42002a, 2, intBytes(256))),
                "The isolated object must request a 256-bit key.");
        assertTrue(contains(requests.get(1), new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4, 0, 0, 0, 0x14}),
                "The created object must be destroyed in a separate exchange.");
        assertTrue(contains(requests.get(1), item(0x420094, 7,
                CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8))));
    }

    @Test
    void everyPublicClientOperationReachesPeerAndClassifiesUnsupportedResponse() throws Exception {
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var availableOperations = TypedRequestFactory.availableFactories().keySet();
        var casesByOperation = new java.util.TreeMap<String, CatalogLoader.CatalogCase>();
        catalog.cases().stream()
                .filter(testCase -> "server_exchange".equals(testCase.execution().mode()))
                .filter(testCase -> testCase.execution().protocolVersions().contains("2.1"))
                .filter(testCase -> !"server_to_client".equals(testCase.requires().path("workflow_type").asText()))
                .filter(testCase -> availableOperations.contains(testCase.operation()))
                .forEach(testCase -> casesByOperation.putIfAbsent(testCase.operation(), testCase));
        catalog.cases().stream().filter(testCase -> testCase.caseId().equals("test-op-get-attributes"))
                .findFirst().ifPresent(testCase -> casesByOperation.put("Get Attributes", testCase));
        assertEquals(availableOperations.size(), casesByOperation.size(),
                "Every public KMIP 2.x client operation must have a catalog exchange case.");

        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        MatrixPlan matrixPlan = operationMatrixPlan(casesByOperation, 1);
        var peerTrace = new java.util.concurrent.CopyOnWriteArrayList<Long>();
        var peerRequests = new java.util.concurrent.CopyOnWriteArrayList<byte[]>();
        FutureTask<List<Long>> peer = new FutureTask<>(() -> serveOperationMatrixResponses(listener,
                matrixPlan, peerTrace, peerRequests));
        Thread peerThread = new Thread(peer, "java-validator-controlled-operation-matrix-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("operation-matrix-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
            for (var entry : casesByOperation.entrySet()) {
                JavaKmipDriver.CaseOutcome outcome = driver.execute(entry.getValue());
                assertPeerHealthy(peer);
                if ("Create".equals(entry.getKey())) {
                    assertEquals("passed", outcome.status(), entry.getKey() + ": " + outcome);
                    assertEquals(0L, outcome.resultStatusCode(), entry.getKey());
                } else {
                    assertEquals("server_unsupported", outcome.status(), entry.getKey() + ": " + outcome
                            + "; peer operations=" + peerTrace);
                    assertEquals("unsupported", outcome.serverCompatibility(), entry.getKey());
                    assertEquals(1L, outcome.resultStatusCode(), entry.getKey());
                    assertEquals(KmipValues.ResultReason.OPERATION_NOT_SUPPORTED,
                            outcome.resultReasonCode(), entry.getKey());
                }
            }
        } finally {
            listener.close();
        }
        List<Long> observedOperations = peer.get(60, TimeUnit.SECONDS);
        assertEquals(matrixPlan.expectedOperations().size(), observedOperations.size());
        byte[] unfilteredGetAttributes = peerRequests.stream()
                .filter(request -> requestOperation(request) == 0x0b)
                .findFirst().orElseThrow();
        assertTrue(!containsTag(unfilteredGetAttributes, 0x42013b),
                "The operation-level Get Attributes case has no Attribute Reference and must request all attributes.");
        for (var entry : casesByOperation.entrySet()) {
            try (ClientBatchItem expected = TypedRequestFactory.create(entry.getKey())) {
                assertTrue(observedOperations.contains(expected.operation()), entry.getKey());
                assertEquals("Operation Not Supported", KmipValues.ResultReason.name(
                        KmipValues.ResultReason.OPERATION_NOT_SUPPORTED));
            }
        }
    }

    // OASIS KMIP 2.0 §§6.1 and 11: every catalogued client operation applicable
    // to 2.0 must be emitted through the installed typed request factories.
    @Test
    void everyPublicClientOperationReachesPeerForKmip20() throws Exception {
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var availableOperations = TypedRequestFactory.availableFactories().keySet();
        var casesByOperation = new java.util.TreeMap<String, CatalogLoader.CatalogCase>();
        catalog.cases().stream()
                .filter(testCase -> "server_exchange".equals(testCase.execution().mode()))
                .filter(testCase -> testCase.execution().protocolVersions().contains("2.0"))
                .filter(testCase -> !"server_to_client".equals(testCase.requires().path("workflow_type").asText()))
                .filter(testCase -> availableOperations.contains(testCase.operation()))
                .forEach(testCase -> casesByOperation.putIfAbsent(testCase.operation(), testCase));
        catalog.cases().stream().filter(testCase -> testCase.caseId().equals("test-op-get-attributes"))
                .findFirst().ifPresent(testCase -> casesByOperation.put("Get Attributes", testCase));
        assertTrue(!casesByOperation.isEmpty(), "KMIP 2.0 must have applicable public client operations.");

        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        MatrixPlan matrixPlan = operationMatrixPlan(casesByOperation, 0);
        var peerTrace = new java.util.concurrent.CopyOnWriteArrayList<Long>();
        var peerRequests = new java.util.concurrent.CopyOnWriteArrayList<byte[]>();
        FutureTask<List<Long>> peer = new FutureTask<>(() -> serveOperationMatrixResponses(listener,
                matrixPlan, peerTrace, peerRequests));
        Thread peerThread = new Thread(peer, "java-validator-kmip20-operation-matrix-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("kmip20-operation-matrix-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true, "2.0"));
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
            for (var entry : casesByOperation.entrySet()) {
                JavaKmipDriver.CaseOutcome outcome = driver.execute(entry.getValue());
                assertPeerHealthy(peer);
                if ("Create".equals(entry.getKey())) {
                    assertEquals("passed", outcome.status(), entry.getKey() + ": " + outcome);
                    assertEquals(0L, outcome.resultStatusCode(), entry.getKey());
                } else {
                    assertEquals("server_unsupported", outcome.status(), entry.getKey() + ": " + outcome
                            + "; peer operations=" + peerTrace);
                    assertEquals("unsupported", outcome.serverCompatibility(), entry.getKey());
                    assertEquals(1L, outcome.resultStatusCode(), entry.getKey());
                    assertEquals(KmipValues.ResultReason.OPERATION_NOT_SUPPORTED,
                            outcome.resultReasonCode(), entry.getKey());
                }
            }
        } finally {
            listener.close();
        }
        List<Long> observedOperations = peer.get(60, TimeUnit.SECONDS);
        assertEquals(matrixPlan.expectedOperations().size(), observedOperations.size());
        byte[] unfilteredGetAttributes = peerRequests.stream()
                .filter(request -> requestOperation(request) == 0x0b)
                .findFirst().orElseThrow();
        assertTrue(!containsTag(unfilteredGetAttributes, 0x42013b),
                "The operation-level Get Attributes case has no Attribute Reference and must request all attributes.");
        for (String operation : casesByOperation.keySet()) {
            try (ClientBatchItem expected = TypedRequestFactory.create(operation)) {
                assertTrue(observedOperations.contains(expected.operation()), operation);
            }
        }
    }

    // OASIS KMIP 2.0 §6.1.32: the consumer CLI must preserve the configured
    // version through request creation, TLS exchange, and response decoding.
    @Test
    void cliRunExecutesQueryAtKmip20AndAcceptsItsTypedResponse() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, queryResponse(0)));
        Thread peerThread = new Thread(peer, "java-validator-kmip20-cli-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("kmip20-cli-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, false, "2.0"));
        Path sourceCatalog = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        ObjectMapper mapper = new ObjectMapper();
        ObjectNode catalog = (ObjectNode) mapper.readTree(sourceCatalog.toFile());
        ArrayNode selectedCases = mapper.createArrayNode();
        for (JsonNode testCase : catalog.path("test_cases")) {
            if ("test-op-query".equals(testCase.path("case_id").asText())) {
                selectedCases.add(testCase);
            }
        }
        assertEquals(1, selectedCases.size(), "The KMIP 2.0 Query case must exist in the shared catalog.");
        catalog.set("test_cases", selectedCases);
        ((ObjectNode) catalog.path("statistics")).put("total_cases", selectedCases.size());
        Path selectedCatalog = temporaryDirectory.resolve("kmip20-query-catalog.json");
        Files.writeString(selectedCatalog, mapper.writeValueAsString(catalog));

        Path outputDirectory = temporaryDirectory.resolve("kmip20-run-report");
        var standardOutput = new ByteArrayOutputStream();
        var standardError = new ByteArrayOutputStream();
        int exitCode;
        try {
            exitCode = Main.execute(new String[] {"run", "--connection", connectionFile.toString(), "--catalog",
                    selectedCatalog.toString(), "--output-dir", outputDirectory.toString()},
                    new PrintStream(standardOutput), new PrintStream(standardError));
        } finally {
            listener.close();
        }
        assertEquals(0, exitCode, standardError.toString(StandardCharsets.UTF_8));
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertEquals(0, requestProtocolMinorVersion(request));
        assertEquals(0x18L, requestOperation(request));
        JsonNode report = mapper.readTree(outputDirectory.resolve("results.json").toFile());
        assertEquals(1, report.path("summary").path("total").asInt());
        assertEquals("passed", report.path("cases").get(0).path("status").asText());
        assertEquals(0, report.path("cases").get(0).path("result_status_code").asInt());
    }

    @Test
    void cliRunDispatchesGetThroughItsPublicTypedFactory() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> serveObjectFixtureAndGet(listener));
        Thread peerThread = new Thread(peer, "java-validator-cli-get-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("get-cli-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));
        Path sourceCatalog = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        ObjectMapper mapper = new ObjectMapper();
        ObjectNode catalog = (ObjectNode) mapper.readTree(sourceCatalog.toFile());
        ArrayNode selectedCases = mapper.createArrayNode();
        for (JsonNode testCase : catalog.path("test_cases")) {
            if ("test-op-get".equals(testCase.path("case_id").asText())) selectedCases.add(testCase);
        }
        assertEquals(1, selectedCases.size(), "The Get operation case must exist in the shared catalog.");
        catalog.set("test_cases", selectedCases);
        ((ObjectNode) catalog.path("statistics")).put("total_cases", selectedCases.size());
        Path selectedCatalog = temporaryDirectory.resolve("get-catalog.json");
        Files.writeString(selectedCatalog, mapper.writeValueAsString(catalog));

        Path outputDirectory = temporaryDirectory.resolve("get-run-report");
        var standardOutput = new ByteArrayOutputStream();
        var standardError = new ByteArrayOutputStream();
        int exitCode;
        try {
            exitCode = Main.execute(new String[] {"run", "--connection", connectionFile.toString(), "--catalog",
                    selectedCatalog.toString(), "--output-dir", outputDirectory.toString(), "--enable-mutations"},
                    new PrintStream(standardOutput), new PrintStream(standardError));
        } finally {
            listener.close();
        }
        List<byte[]> requests = peer.get(10, TimeUnit.SECONDS);
        assertEquals(List.of(1L, 10L, 0x14L), requests.stream().map(ControlledTlsPeerTest::requestOperation).toList());
        assertTrue(contains(requests.get(1), item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8))),
                "Get must use the identifier returned by the fixture Create response.");
        assertEquals(0, exitCode, standardError.toString(StandardCharsets.UTF_8));
        JsonNode report = mapper.readTree(outputDirectory.resolve("results.json").toFile());
        assertEquals("passed", report.path("cases").get(0).path("status").asText());
        assertEquals(0L, report.path("cases").get(0).path("result_status_code").asLong());
    }

    @Test
    void cliRunReadsAttributeListAndValuesThroughTypedFactories() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        Path connectionFile = temporaryDirectory.resolve("attribute-read-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));

        Path sourceCatalog = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        ObjectMapper mapper = new ObjectMapper();
        ObjectNode catalog = (ObjectNode) mapper.readTree(sourceCatalog.toFile());
        Set<String> selectedIds = Set.of(
                "test-attr-cryptographic-algorithm", "test-attr-name");
        ArrayNode selectedCases = mapper.createArrayNode();
        var targetOperations = new ArrayList<Long>();
        var targetAttributeTags = new ArrayList<Long>();
        var loadedCases = CatalogLoader.load(sourceCatalog).cases().stream().collect(
                java.util.stream.Collectors.toMap(CatalogLoader.CatalogCase::caseId, testCase -> testCase));
        for (JsonNode testCase : catalog.path("test_cases")) {
            if (!selectedIds.contains(testCase.path("case_id").asText())) continue;
            selectedCases.add(testCase);
            long attributeTag = loadedCases.get(testCase.path("case_id").asText()).attributeTag();
            targetAttributeTags.add(attributeTag);
            try (ClientBatchItem request = TypedRequestFactory.create(
                    testCase.path("operation").asText(), null, attributeTag)) {
                targetOperations.add(request.operation());
            }
        }
        assertEquals(selectedIds.size(), selectedCases.size(), "Both attribute-specific read cases must exist.");
        catalog.set("test_cases", selectedCases);
        ((ObjectNode) catalog.path("statistics")).put("total_cases", selectedCases.size());
        Path selectedCatalog = temporaryDirectory.resolve("attribute-read-catalog.json");
        Files.writeString(selectedCatalog, mapper.writeValueAsString(catalog));

        FutureTask<List<byte[]>> peer = new FutureTask<>(
                () -> serveAttributeReadCases(listener, targetOperations, targetAttributeTags));
        Thread peerThread = new Thread(peer, "java-validator-attribute-read-peer");
        peerThread.setDaemon(true);
        peerThread.start();
        Path outputDirectory = temporaryDirectory.resolve("attribute-read-run-report");
        var standardError = new ByteArrayOutputStream();
        int exitCode;
        try {
            exitCode = Main.execute(new String[] {"run", "--connection", connectionFile.toString(), "--catalog",
                    selectedCatalog.toString(), "--output-dir", outputDirectory.toString(), "--enable-mutations"},
                    new PrintStream(new ByteArrayOutputStream()), new PrintStream(standardError));
        } finally {
            listener.close();
        }
        List<byte[]> requests = peer.get(10, TimeUnit.SECONDS);
        assertEquals(6, requests.size());
        for (int index = 0; index < targetOperations.size(); index++) {
            byte[] request = requests.get(index * 3 + 1);
            assertEquals(targetOperations.get(index), requestOperation(request));
            assertTrue(contains(request, item(0x420094, 7,
                    CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8))),
                    "The request must use the identifier from its setup Create response.");
            assertTrue(contains(request, item(0x42013b, 5,
                            intBytes(Math.toIntExact(targetAttributeTags.get(index))))),
                    "Get Attributes must encode the Attribute Reference tag selected by its catalog case.");
        }
        JsonNode report = mapper.readTree(outputDirectory.resolve("results.json").toFile());
        assertEquals(2, report.path("summary").path("passed").asInt(), report.toString());
        assertEquals(0, report.path("summary").path("failed").asInt(), report.toString());
        assertEquals(0, exitCode, standardError.toString(StandardCharsets.UTF_8));
    }

    @Test
    void cliRunRejectsGetAttributesSuccessWithIncorrectAttributeValue() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        Path connectionFile = temporaryDirectory.resolve("attribute-mismatch-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));

        Path sourceCatalog = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        ObjectMapper mapper = new ObjectMapper();
        ObjectNode catalog = (ObjectNode) mapper.readTree(sourceCatalog.toFile());
        JsonNode selectedCase = null;
        for (JsonNode testCase : catalog.path("test_cases")) {
            if ("test-attr-cryptographic-algorithm".equals(testCase.path("case_id").asText())) {
                selectedCase = testCase;
                break;
            }
        }
        assertTrue(selectedCase != null, "The Cryptographic Algorithm attribute case must exist.");
        ArrayNode selectedCases = mapper.createArrayNode().add(selectedCase);
        catalog.set("test_cases", selectedCases);
        ((ObjectNode) catalog.path("statistics")).put("total_cases", selectedCases.size());
        Path selectedCatalog = temporaryDirectory.resolve("attribute-mismatch-catalog.json");
        Files.writeString(selectedCatalog, mapper.writeValueAsString(catalog));

        long requestedAttributeTag = 0x420028L;
        long returnedAttributeTag = 0x420028L;
        long operation;
        try (ClientBatchItem request = TypedRequestFactory.create("Get Attributes", null, requestedAttributeTag)) {
            operation = request.operation();
        }
        FutureTask<List<byte[]>> peer = new FutureTask<>(
                () -> serveAttributeReadCases(listener, List.of(operation), List.of(requestedAttributeTag),
                        List.of(returnedAttributeTag), List.of(4L)));
        Thread peerThread = new Thread(peer, "java-validator-attribute-mismatch-peer");
        peerThread.setDaemon(true);
        peerThread.start();
        Path outputDirectory = temporaryDirectory.resolve("attribute-mismatch-run-report");
        var standardError = new ByteArrayOutputStream();
        int exitCode;
        try {
            exitCode = Main.execute(new String[] {"run", "--connection", connectionFile.toString(), "--catalog",
                    selectedCatalog.toString(), "--output-dir", outputDirectory.toString(), "--enable-mutations"},
                    new PrintStream(new ByteArrayOutputStream()), new PrintStream(standardError));
        } finally {
            listener.close();
        }
        List<byte[]> requests = peer.get(10, TimeUnit.SECONDS);
        assertEquals(3, requests.size());
        assertTrue(contains(requests.get(1), item(0x42013b, 5, intBytes(Math.toIntExact(requestedAttributeTag)))));
        JsonNode report = mapper.readTree(outputDirectory.resolve("results.json").toFile());
        assertEquals(1, report.path("summary").path("failed").asInt(), report.toString());
        assertEquals(0, report.path("summary").path("passed").asInt(), report.toString());
        assertEquals(1, exitCode, standardError.toString(StandardCharsets.UTF_8));
    }

    private String connectionJson(int port, byte[] serverCertificate, byte[] clientCertificate,
                                  byte[] clientPrivateKey) throws Exception {
        return connectionJson(port, serverCertificate, clientCertificate, clientPrivateKey, false);
    }

    private String connectionJson(int port, byte[] serverCertificate, byte[] clientCertificate,
                                  byte[] clientPrivateKey, boolean allowMutations) throws Exception {
        return connectionJson(port, serverCertificate, clientCertificate, clientPrivateKey,
                allowMutations, "2.1");
    }

    private String connectionJson(int port, byte[] serverCertificate, byte[] clientCertificate,
                                  byte[] clientPrivateKey, boolean allowMutations,
                                  String protocolVersion) throws Exception {
        Path serverCert = temporaryDirectory.resolve("server-cert.pem");
        Path clientCert = temporaryDirectory.resolve("client-cert.pem");
        Path clientKey = temporaryDirectory.resolve("client-key.pem");
        Files.write(serverCert, serverCertificate);
        Files.write(clientCert, clientCertificate);
        Files.write(clientKey, clientPrivateKey);
        ObjectNode root = new ObjectMapper().createObjectNode();
        root.put("server_id", "controlled-test-peer");
        root.put("server_configuration_fingerprint", "a".repeat(64));
        root.put("protocol_version", protocolVersion);
        root.put("transport", "raw_tls");
        root.put("encoding", "TTLV");
        root.put("api_level", "typed-protocol");
        ObjectNode endpoints = root.putObject("endpoints");
        endpoints.putObject("raw_tls").put("host", "127.0.0.1").put("port", port);
        ObjectNode tls = root.putObject("tls");
        tls.put("ca_cert_path", serverCert.toString());
        tls.put("client_cert_path", clientCert.toString());
        tls.put("client_key_path", clientKey.toString());
        tls.put("server_name_override", "localhost");
        tls.put("connect_timeout_seconds", 5);
        tls.put("read_timeout_seconds", 5);
        root.putObject("environment").put("kind", "dedicated_test")
                .put("dedicated_test_environment_confirmed", true);
        root.putObject("execution_policy").put("allow_mutations", allowMutations);
        return new ObjectMapper().writeValueAsString(root);
    }

    private static SSLServerSocket listener(X509Certificate serverCertificate, PrivateKey serverKey,
                                            X509Certificate clientCertificate) throws Exception {
        char[] password = "local-kmip-test".toCharArray();
        KeyStore serverKeys = KeyStore.getInstance("PKCS12");
        serverKeys.load(null, password);
        serverKeys.setKeyEntry("server", serverKey, password, new Certificate[] {serverCertificate});
        KeyManagerFactory keys = KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm());
        keys.init(serverKeys, password);
        KeyStore trust = KeyStore.getInstance("PKCS12");
        trust.load(null, password);
        trust.setCertificateEntry("client", clientCertificate);
        TrustManagerFactory trustManagers = TrustManagerFactory.getInstance(
                TrustManagerFactory.getDefaultAlgorithm());
        trustManagers.init(trust);
        SSLContext context = SSLContext.getInstance("TLSv1.3");
        context.init(keys.getKeyManagers(), trustManagers.getTrustManagers(), new SecureRandom());
        SSLServerSocket server = (SSLServerSocket) context.getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        server.setEnabledProtocols(new String[] {"TLSv1.3"});
        server.setNeedClientAuth(true);
        server.setSoTimeout(10_000);
        return server;
    }

    private static byte[] serveOneResponse(SSLServerSocket listener, byte[] response) throws Exception {
        try (SSLSocket socket = (SSLSocket) listener.accept()) {
            socket.setEnabledProtocols(new String[] {"TLSv1.3"});
            socket.setSoTimeout(10_000);
            socket.startHandshake();
            byte[] request = readTtlv(socket.getInputStream());
            OutputStream output = socket.getOutputStream();
            output.write(response);
            output.flush();
            return request;
        }
    }

    private static List<Long> serveUnsupportedResponses(SSLServerSocket listener, int count) throws Exception {
        return serveUnsupportedResponses(listener, count, 1);
    }

    private static List<Long> serveUnsupportedResponses(SSLServerSocket listener, int count,
                                                         int protocolMinorVersion) throws Exception {
        var operations = new ArrayList<Long>();
        for (int index = 0; index < count; index++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readTtlv(socket.getInputStream());
                long operation = requestOperation(request);
                operations.add(operation);
                socket.getOutputStream().write(unsupportedResponse(operation, protocolMinorVersion));
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(operations);
    }

    private static MatrixPlan operationMatrixPlan(
            java.util.Map<String, CatalogLoader.CatalogCase> casesByOperation, int protocolMinorVersion)
            throws Exception {
        var expectedOperations = new ArrayList<Long>();
        var responses = new ArrayList<byte[]>();
        for (var entry : casesByOperation.entrySet()) {
            String operationName = entry.getKey();
            CatalogLoader.CatalogCase testCase = entry.getValue();
            boolean fixture = requiresSymmetricKeyFixture(testCase);
            if ("Create".equals(operationName)) {
                expectedOperations.add(1L);
                responses.add(createResponse(protocolMinorVersion));
                expectedOperations.add(0x14L);
                responses.add(destroyResponse(protocolMinorVersion));
                continue;
            }
            if (fixture) {
                expectedOperations.add(1L);
                responses.add(createResponse(protocolMinorVersion));
            }
            try (ClientBatchItem request = TypedRequestFactory.create(operationName)) {
                expectedOperations.add(request.operation());
            }
            responses.add(unsupportedResponse(expectedOperations.get(expectedOperations.size() - 1),
                    protocolMinorVersion));
            if (fixture) {
                expectedOperations.add(0x14L);
                responses.add(destroyResponse(protocolMinorVersion));
            }
        }

        return new MatrixPlan(List.copyOf(expectedOperations), List.copyOf(responses));
    }

    private static List<Long> serveOperationMatrixResponses(SSLServerSocket listener,
            MatrixPlan plan, List<Long> peerTrace, List<byte[]> peerRequests) throws Exception {
        var observedOperations = new ArrayList<Long>();
        for (int index = 0; index < plan.expectedOperations().size(); index++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                peerTrace.add(-1L);
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readTtlv(socket.getInputStream());
                peerRequests.add(request);
                long operation = requestOperation(request);
                observedOperations.add(operation);
                peerTrace.add(operation);
                if (operation != plan.expectedOperations().get(index)) {
                    throw new AssertionError("The operation matrix sent an unexpected operation at exchange " + index);
                }
                socket.getOutputStream().write(plan.responses().get(index));
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(observedOperations);
    }

    private static boolean requiresSymmetricKeyFixture(CatalogLoader.CatalogCase testCase) {
        return testCase.isolationRecipe().setupSteps().stream()
                .anyMatch(step -> step.startsWith("Create unique SymmetricKey"))
                || ("Get Attributes".equals(testCase.operation())
                        && (testCase.attributeTag() == 0x420028L || testCase.attributeTag() == 0x420053L));
    }

    private static void assertPeerHealthy(FutureTask<?> peer) throws Exception {
        if (!peer.isDone()) return;
        try {
            peer.get();
        } catch (java.util.concurrent.ExecutionException exception) {
            throw new AssertionError("The controlled peer rejected the request sequence.", exception.getCause());
        }
    }

    private static long requestOperation(byte[] request) {
        byte[] header = new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4};
        for (int offset = 0; offset <= request.length - header.length - 4; offset++) {
            boolean matches = true;
            for (int index = 0; index < header.length; index++) {
                if (request[offset + index] != header[index]) {
                    matches = false;
                    break;
                }
            }
            if (matches) return Integer.toUnsignedLong(ByteBuffer.wrap(request, offset + header.length, 4).getInt());
        }
        throw new AssertionError("The outgoing KMIP request omitted its Operation field.");
    }

    private static void appendLocalQueryRouteEvents(ArrayNode events, String protocolVersion,
            String operationTreeHash) throws Exception {
        try (RawTag rawTag = RawTag.fromRaw(0x42005c);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.Enumeration(0x18);
                TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults())) {
            try (var view = item.view(); var valueView = view.value()) {
                assertEquals(0x42005c, view.tag().raw());
                assertEquals(TtlvItemType.Enumeration, view.itemType());
                assertEquals(0x18L, valueView.enumeration());
            }
            appendRouteEvent(events, "test-op-query", "java.generic-ttlv.ttlv_item.create",
                    "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.ttlv.TtlvItem.create", null, null,
                    "construct", protocolVersion, null, operationTreeHash, null);
            appendRouteEvent(events, "test-op-query", "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.ttlv.TtlvItem.view", null, null,
                    "decode", protocolVersion, null, null, operationTreeHash);
        }

        try (TtlvItem decoded = TtlvCodec.decode(queryOperationTtlv())) {
            assertTrue(decoded != null, "The installed TTLV decoder must return a value.");
        }
        appendRouteEvent(events, "test-op-query", "java.generic-ttlv.ttlv_codec.decode",
                "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null, null, operationTreeHash);
    }

    private static void appendTagRangeRouteEvents(ArrayNode events, String protocolVersion) throws Exception {
        for (TagRangeFixture fixture : TAG_RANGE_FIXTURES) {
            byte[] wire = item(fixture.rawTag(), 5, intBytes(0));
            String treeHash = normalizedTreeSha256(wire);
            if ("extension".equals(fixture.allocation())) {
                appendAllocatedTagRangeEvents(events, fixture, protocolVersion, wire, treeHash);
            } else {
                appendRejectedTagRangeEvent(events, fixture, protocolVersion, wire);
            }
        }
    }

    private static List<EnumerationFixture> genericEnumerationFixtures() throws Exception {
        JsonNode catalog = readRouteEvidenceCatalog();
        Map<String, JsonNode> targets = new HashMap<>();
        for (JsonNode target : catalog.path("normative_targets")) {
            targets.put(target.path("target_id").asText(), target);
        }

        var fixtures = new ArrayList<EnumerationFixture>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"enumeration_value".equals(requires.path("target_kind").asText())) continue;
            JsonNode target = targets.get(requires.path("normative_target_id").asText());
            if (target == null || !target.path("parent_element_ids").isArray()
                    || target.path("parent_element_ids").isEmpty()) continue;
            String parentId = target.path("parent_element_ids").get(0).asText();
            String parentName = parentId.replace("KMIPKIT-ELEM-ENUMERATION-", "");
            JsonNode tagTarget = targets.values().stream()
                    .filter(candidate -> "tag".equals(candidate.path("kind").asText()))
                    .filter(candidate -> normalizeTargetName(candidate.path("name").asText())
                            .equals(normalizeTargetName(parentName)))
                    .findFirst().orElse(null);
            if (tagTarget == null) continue;
            // OASIS KMIP 2.0/2.1 §11.34 Object Type Enumeration is distinct from §4.36 Object Type Attribute.
            JsonNode attributeTarget = "KMIPKIT-ELEM-ENUMERATION-OBJECT-TYPE".equals(parentId)
                    ? null : targets.values().stream()
                            .filter(candidate -> "attribute".equals(candidate.path("kind").asText()))
                            .filter(candidate -> normalizeTargetName(candidate.path("name").asText())
                                    .equals(normalizeTargetName(parentName)))
                            .findFirst().orElse(null);
            int rawTag = parseWireTag(tagTarget.path("wire_value").asText());
            StandardAttribute attribute = attributeTarget == null ? null : standardAttributeForTag(rawTag);
            if (attributeTarget != null && attribute == null) {
                throw new IllegalStateException("Catalog attribute has no installed StandardAttribute route: "
                        + parentName);
            }

            var versions = new ArrayList<String>();
            for (JsonNode version : testCase.path("execution").path("protocol_versions")) {
                versions.add(version.asText());
            }
            if (versions.isEmpty()) continue;
            String wireValue = target.path("wire_value").asText();
            long enumValue = parseRepresentativeEnumerationValue(
                    wireValue, target.path("allocation").asText());
            fixtures.add(new EnumerationFixture(testCase.path("case_id").asText(),
                    target.path("target_id").asText(), parentName, rawTag, enumValue,
                    attribute, List.copyOf(versions)));
        }
        return List.copyOf(fixtures);
    }

    private static void appendGenericEnumerationRouteEvents(ArrayNode events,
            List<EnumerationFixture> fixtures, String protocolVersion) throws Exception {
        for (EnumerationFixture fixture : fixtures) {
            if (!fixture.protocolVersions().contains(protocolVersion)) continue;
            long enumValue = installedEnumerationValue(fixture);
            assertEquals(fixture.value(), enumValue,
                    "The installed Java enum constant must match the catalog wire value.");
            byte[] wire = item(fixture.rawTag(), 5, intBytes((int) enumValue));
            String treeHash = normalizedTreeSha256(wire);
            try (TtlvValue value = TtlvValue.Enumeration(enumValue)) {
                appendRouteEvent(events, fixture.caseId(),
                        "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                        fixture.targetId(), "org.kmipkit.ttlv.TtlvValue.Enumeration", null, null,
                        "construct", protocolVersion, null, treeHash, null);
                appendLocalTtlvRoundtripEvents(events, fixture.caseId(), fixture.targetId(),
                        fixture.rawTag(), TtlvItemType.Enumeration, value, wire,
                        protocolVersion, fixture.value());
                appendTypedEnumerationRouteEvents(events, fixture, wire, protocolVersion, treeHash);
                appendCryptographicParameterBuilderRouteEvent(events, fixture, protocolVersion, treeHash);
                appendCredentialEnumerationRouteEvents(events, fixture, protocolVersion, treeHash, true);
                appendBatchErrorContinuationRouteEvent(events, fixture, protocolVersion);
            }
        }
    }

    private static List<EnumerationFixture> genericLocalEnumerationFixtures() throws Exception {
        JsonNode catalog = readRouteEvidenceCatalog();
        Set<String> localEnumerationCases = cataloguedLocalEnumerationCaseIds(catalog);
        List<JsonNode> targets = new ArrayList<>();
        for (JsonNode target : catalog.path("normative_targets")) targets.add(target);

        var fixtures = new ArrayList<EnumerationFixture>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String enumType = testCase.path("requires").path("enum_type").asText();
            if (!localEnumerationCases.contains(testCase.path("case_id").asText())
                    || enumType.isEmpty()
                    || !"local_model".equals(testCase.path("execution").path("mode").asText())) {
                continue;
            }
            String normalizedEnumType = normalizeTargetName(enumType);
            JsonNode tagTarget = targets.stream()
                    .filter(candidate -> "tag".equals(candidate.path("kind").asText()))
                    .filter(candidate -> normalizedEnumType.equals(
                            normalizeTargetName(candidate.path("name").asText())))
                    .findFirst().orElseThrow(() -> new IllegalStateException(
                            "Local enum has no same-named normative TTLV tag: " + enumType));
            int rawTag = parseWireTag(tagTarget.path("wire_value").asText());
            JsonNode attributeTarget = "OBJECTTYPE".equals(normalizedEnumType) ? null : targets.stream()
                    .filter(candidate -> "attribute".equals(candidate.path("kind").asText()))
                    .filter(candidate -> normalizedEnumType.equals(
                            normalizeTargetName(candidate.path("name").asText())))
                    .findFirst().orElse(null);
            StandardAttribute attribute = attributeTarget == null ? null : standardAttributeForTag(rawTag);
            if (attributeTarget != null && attribute == null) {
                throw new IllegalStateException("Catalog attribute has no installed StandardAttribute route: "
                        + enumType);
            }
            String parentName = enumType.toUpperCase(Locale.ROOT).replaceAll("[^A-Z0-9]+", "-");
            List<String> caseVersions = new ArrayList<>();
            for (JsonNode version : testCase.path("execution").path("protocol_versions")) {
                caseVersions.add(version.asText());
            }
            for (JsonNode target : targets) {
                String targetKind = target.path("kind").asText();
                if (!"enumeration_value".equals(targetKind)) continue;
                if (!belongsToEnumeration(target, enumType)) continue;
                String wireValue = target.path("wire_value").asText();
                long value = parseRepresentativeEnumerationValue(
                        wireValue, target.path("allocation").asText());
                List<String> versions = localEnumerationVersions(target, enumType, caseVersions);
                if (!versions.isEmpty()) {
                    fixtures.add(new EnumerationFixture(testCase.path("case_id").asText(),
                            target.path("target_id").asText(), parentName, rawTag, value,
                            attribute, versions));
                }
            }
        }
        return List.copyOf(fixtures);
    }

    private static Set<String> cataloguedLocalEnumerationCaseIds(JsonNode catalog) {
        Set<String> enumerationNames = new java.util.HashSet<>();
        Set<String> tagNames = new java.util.HashSet<>();
        for (JsonNode target : catalog.path("normative_targets")) {
            String kind = target.path("kind").asText();
            if ("tag".equals(kind)) tagNames.add(normalizeTargetName(target.path("name").asText()));
            if (!"enumeration_value".equals(kind)) continue;
            for (JsonNode parent : target.path("parent_element_ids")) {
                String parentId = parent.asText();
                String prefix = "KMIPKIT-ELEM-ENUMERATION-";
                if (parentId.startsWith(prefix)) {
                    enumerationNames.add(normalizeTargetName(parentId.substring(prefix.length())));
                }
            }
        }

        Set<String> caseIds = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String caseId = testCase.path("case_id").asText();
            String mode = testCase.path("execution").path("mode").asText();
            String enumType = testCase.path("requires").path("enum_type").asText();
            if (caseId.startsWith("test-enum-")
                    && "local_model".equals(mode)
                    && enumerationNames.contains(normalizeTargetName(enumType))
                    && tagNames.contains(normalizeTargetName(enumType))) {
                caseIds.add(caseId);
            }
        }
        return Set.copyOf(caseIds);
    }

    private static List<String> localEnumerationVersions(JsonNode target, String enumType,
            List<String> caseVersions) {
        boolean preserveUnknownValuesAcrossVersions = ENUMS_WITH_KMIP_2_1_ONLY_VALUES.contains(enumType);
        return caseVersions.stream()
                .filter(version -> preserveUnknownValuesAcrossVersions || target.path("protocol_versions").isEmpty()
                        || containsText(target.path("protocol_versions"), version))
                .toList();
    }

    private static boolean belongsToEnumeration(JsonNode target, String enumType) {
        String normalizedEnumType = normalizeTargetName(enumType);
        for (JsonNode parent : target.path("parent_element_ids")) {
            String parentId = parent.asText();
            String prefix = "KMIPKIT-ELEM-ENUMERATION-";
            if (parentId.startsWith(prefix)
                    && normalizedEnumType.equals(normalizeTargetName(parentId.substring(prefix.length())))) {
                return true;
            }
        }
        return false;
    }

    private static long parseRepresentativeEnumerationValue(String wireValue, String allocation) {
        if ("extension".equals(allocation) && wireValue.toUpperCase(Locale.ROOT).contains("X")) {
            return 0x8000_0000L;
        }
        // Reserved ranges are represented by their inclusive lower bound for the generic scalar probe.
        String representative = "reserved".equals(allocation) && wireValue.contains("-")
                ? wireValue.substring(0, wireValue.indexOf('-')) : wireValue;
        String digits = representative.replaceFirst("^0[xX]", "");
        if (!digits.matches("[0-9A-Fa-f]+")) {
            throw new IllegalStateException("Unsupported catalog enumeration wire value: " + wireValue);
        }
        return Long.parseUnsignedLong(digits, 16);
    }

    private static boolean containsText(JsonNode values, String expected) {
        for (JsonNode value : values) if (expected.equals(value.asText())) return true;
        return false;
    }

    private static List<String> referencedEnumerationTargetIds(JsonNode catalog, String parentName) {
        List<String> targetIds = new ArrayList<>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if ("referenced_enum_value".equals(requires.path("target_kind").asText())
                    && parentName.equals(requires.path("parent_name").asText())) {
                targetIds.add(requires.path("normative_target_id").asText());
            }
        }
        return List.copyOf(targetIds);
    }

    private static void appendGenericLocalEnumerationRouteEvents(ArrayNode events,
            List<EnumerationFixture> fixtures, String protocolVersion) throws Exception {
        for (EnumerationFixture fixture : fixtures) {
            if (!fixture.protocolVersions().contains(protocolVersion)) continue;
            byte[] wire = item(fixture.rawTag(), 5, intBytes((int) fixture.value()));
            try (TtlvValue value = TtlvValue.Enumeration(fixture.value())) {
                appendRouteEvent(events, fixture.caseId(),
                        "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                        fixture.targetId(), "org.kmipkit.ttlv.TtlvValue.Enumeration", null, null,
                        "construct", protocolVersion, null, normalizedTreeSha256(wire), null);
                appendLocalTtlvRoundtripEvents(events, fixture.caseId(), fixture.targetId(),
                        fixture.rawTag(), TtlvItemType.Enumeration, value, wire,
                        protocolVersion, fixture.value());
                appendTypedEnumerationRouteEvents(events, fixture, wire, protocolVersion,
                        normalizedTreeSha256(wire));
                appendCryptographicParameterBuilderRouteEvent(events, fixture, protocolVersion,
                        normalizedTreeSha256(wire));
                appendCredentialEnumerationRouteEvents(events, fixture, protocolVersion,
                        normalizedTreeSha256(wire), false);
                appendBatchErrorContinuationRouteEvent(events, fixture, protocolVersion);
            }
        }
    }

    private static void appendBatchErrorContinuationRouteEvent(ArrayNode events,
            EnumerationFixture fixture, String protocolVersion) throws Exception {
        if (!"BATCH-ERROR-CONTINUATION-OPTION".equals(fixture.parentName())) return;
        try (ClientBatch batch = ClientBatch.create()) {
            assertTrue(batch.addErrorContinuationValue(fixture.value()) == batch,
                    "The batch configuration API must retain the active ClientBatch handle.");
        }
        byte[] wire = item(fixture.rawTag(), 5, intBytes((int) fixture.value()));
        String treeHash = normalizedTreeSha256(wire);
        appendRouteEvent(events, fixture.caseId(),
                "java.typed-protocol.function.client_batch_add_error_continuation_value",
                fixture.targetId(), "org.kmipkit.extensions.ClientBatch.addErrorContinuationValue",
                null, null, "construct", protocolVersion, null, treeHash, null);
    }

    private static long installedEnumerationValue(EnumerationFixture fixture) {
        return switch (fixture.targetId()) {
            case ENDPOINT_ROLE_CLIENT_TARGET_ID -> KmipValues.EndpointRole.CLIENT;
            case ENDPOINT_ROLE_SERVER_TARGET_ID -> KmipValues.EndpointRole.SERVER;
            default -> fixture.value();
        };
    }

    private static void assertEndpointRoleCatalogValue(Set<String> actualCaseIds, String caseId,
            String targetId, long installedValue, String expectedName) throws Exception {
        assertTrue(actualCaseIds.contains(caseId), "The Endpoint Role enum target must have a generic TTLV fixture.");
        EnumerationFixture fixture = genericEnumerationFixtures().stream()
                .filter(candidate -> candidate.caseId().equals(caseId)
                        && candidate.targetId().equals(targetId))
                .findFirst().orElseThrow(() -> new AssertionError(
                        "The Endpoint Role enum target is missing from the generated fixture set."));
        assertEquals(installedValue, fixture.value(), "The Endpoint Role constant must match the catalog value.");
        assertEquals(expectedName, KmipValues.EndpointRole.name(installedValue));
    }

    private static void assertEndpointRoleRouteEvidence(JsonNode events, String caseId, String targetId,
            long installedValue, String expectedName) throws Exception {
        EnumerationFixture fixture = genericEnumerationFixtures().stream()
                .filter(candidate -> candidate.caseId().equals(caseId)
                        && candidate.targetId().equals(targetId))
                .findFirst().orElseThrow(() -> new AssertionError(
                        "The Endpoint Role enum target is missing from the generated fixture set."));
        assertEquals(installedValue, installedEnumerationValue(fixture));
        assertEquals(expectedName, KmipValues.EndpointRole.name(installedValue));
        for (String protocolVersion : KMIP_2_0_AND_2_1) {
            for (String routeId : ENDPOINT_ROLE_GENERIC_ROUTES) {
                JsonNode event = findRouteEvent(events, caseId, routeId, protocolVersion, targetId);
                assertEquals(targetId, event.path("target_id").asText());
                assertEquals("passed", event.path("status").asText());
                assertEquals(1, event.path("invocation_count").asInt());
            }
        }
    }

    private static void appendCredentialEnumerationRouteEvents(ArrayNode events,
            EnumerationFixture fixture, String protocolVersion, String enumTreeHash,
            boolean includeCredentialModels) throws Exception {
        if (!"CREDENTIAL-TYPE".equals(fixture.parentName())) return;
        CredentialType credentialType = CredentialType.fromRaw(fixture.value());
        assertEquals(fixture.value(), credentialType.raw());
        appendRouteEvent(events, fixture.caseId(), CREDENTIAL_TYPE_FROM_RAW_ROUTE, fixture.targetId(),
                "org.kmipkit.credentials.CredentialType.fromRaw", null, null,
                "construct", protocolVersion, null, enumTreeHash, null);

        if (!includeCredentialModels) return;
        String modelRoute = CREDENTIAL_TYPE_MODEL_ROUTES.get(fixture.targetId());
        if (modelRoute == null) return;
        String modelCaseId = CREDENTIAL_TYPE_MODEL_CASE_IDS.get(fixture.targetId());
        String modelTargetId = CREDENTIAL_TYPE_MODEL_TARGET_IDS.get(fixture.targetId());
        String modelSymbol;
        String credentialTreeHash = normalizedTreeSha256(credentialValueWire(modelRoute));
        try (TtlvStructure structure = credentialValueStructure(modelRoute)) {
            switch (modelRoute) {
                case "java.typed-protocol.model.org.kmipkit.credentials.attestationcredential.fromttlv" -> {
                    modelSymbol = "org.kmipkit.credentials.AttestationCredential.fromTtlv";
                    try (AttestationCredential credential = AttestationCredential.fromTtlv(structure)) {
                        assertTrue(Boolean.TRUE.equals(credential.withTtlv(view -> view.itemCount() > 0)));
                    }
                }
                case "java.typed-protocol.model.org.kmipkit.credentials.devicecredential.fromttlv" -> {
                    modelSymbol = "org.kmipkit.credentials.DeviceCredential.fromTtlv";
                    try (DeviceCredential credential = DeviceCredential.fromTtlv(structure)) {
                        assertTrue(Boolean.TRUE.equals(credential.withTtlv(view -> view.itemCount() > 0)));
                    }
                }
                case "java.typed-protocol.model.org.kmipkit.credentials.hashedpasswordcredential.fromttlv" -> {
                    modelSymbol = "org.kmipkit.credentials.HashedPasswordCredential.fromTtlv";
                    try (HashedPasswordCredential credential = HashedPasswordCredential.fromTtlv(structure)) {
                        assertTrue(Boolean.TRUE.equals(credential.withTtlv(view -> view.itemCount() > 0)));
                    }
                }
                case "java.typed-protocol.model.org.kmipkit.credentials.usernameandpasswordcredential.fromttlv" -> {
                    modelSymbol = "org.kmipkit.credentials.UsernameAndPasswordCredential.fromTtlv";
                    try (UsernameAndPasswordCredential credential =
                            UsernameAndPasswordCredential.fromTtlv(structure)) {
                        assertTrue(Boolean.TRUE.equals(credential.withTtlv(view -> view.itemCount() > 0)));
                    }
                }
                default -> throw new IllegalStateException("Unknown Credential Value model route: " + modelRoute);
            }
        }
        appendRouteEvent(events, modelCaseId, modelRoute, modelTargetId, modelSymbol,
                null, null, "construct", protocolVersion, null, credentialTreeHash, null);
        appendCredentialGenericStructureRouteEvents(events, modelCaseId, modelTargetId, modelRoute,
                protocolVersion, credentialTreeHash);
        appendCredentialEnvelopeRouteEvent(events, modelCaseId, modelTargetId, modelRoute,
                fixture.value(), protocolVersion);
    }

    private static void appendCredentialEnvelopeRouteEvent(ArrayNode events, String caseId,
            String targetId, String modelRoute, long credentialTypeRaw, String protocolVersion) throws Exception {
        String route = "java.typed-protocol.model.org.kmipkit.credentials.credential.fromttlv";
        String treeHash = normalizedTreeSha256(credentialEnvelopeWire(modelRoute, credentialTypeRaw));
        try (TtlvStructure structure = credentialEnvelopeStructure(modelRoute, credentialTypeRaw);
                Credential credential = Credential.fromTtlv(structure)) {
            assertEquals(credentialTypeRaw, credential.credentialTypeRaw());
            assertTrue(Boolean.TRUE.equals(credential.withTtlv(view -> view.itemCount() == 2)),
                    "The generic Credential decoder must retain both envelope fields.");
        }
        appendRouteEvent(events, caseId, route, targetId,
                "org.kmipkit.credentials.Credential.fromTtlv", null, null, "construct", protocolVersion,
                null, treeHash, null);
    }

    private static TtlvStructure credentialEnvelopeStructure(String modelRoute, long credentialTypeRaw)
            throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure envelope = TtlvStructure.create();
        try (RawTag rawTag = RawTag.fromRaw(0x420024);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.Enumeration(credentialTypeRaw);
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            envelope = TtlvStructure.withItem(envelope, item, limits);
        }
        try (TtlvStructure credentialValue = credentialValueStructure(modelRoute);
                TtlvValue value = TtlvValue.structure(credentialValue, limits);
                RawTag rawTag = RawTag.fromRaw(0x420025);
                Tag tag = rawTag.checked();
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            envelope = TtlvStructure.withItem(envelope, item, limits);
        }
        return envelope;
    }

    private static byte[] credentialEnvelopeWire(String modelRoute, long credentialTypeRaw) throws Exception {
        return structure(0x420023,
                item(0x420024, 5, intBytes((int) credentialTypeRaw)), credentialValueWire(modelRoute));
    }

    private static void appendCredentialGenericStructureRouteEvents(ArrayNode events, String caseId,
            String targetId, String modelRoute, String protocolVersion, String treeHash) throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvStructure structure = credentialValueStructure(modelRoute);
                TtlvValue value = TtlvValue.structure(structure, limits);
                RawTag rawTag = RawTag.fromRaw(0x420025);
                Tag tag = rawTag.checked();
                TtlvItem item = TtlvItem.create(tag, value, limits);
                var itemView = item.view();
                var itemValue = itemView.value();
                var valueStructureView = itemValue.structure()) {
            assertEquals(TtlvItemType.Structure, itemView.itemType());
            assertEquals(0x420025, itemView.tag().raw());
            assertTrue(valueStructureView.itemCount() > 0,
                    "The generic Structure constructor must preserve the Credential Value fields.");
        }
        appendRouteEvent(events, caseId,
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure", targetId,
                "org.kmipkit.ttlv.TtlvValue.structure", null, null, "construct", protocolVersion,
                null, treeHash, null);
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_item.create", targetId,
                "org.kmipkit.ttlv.TtlvItem.create", null, null, "construct", protocolVersion,
                null, treeHash, null);
        appendRouteEvent(events, caseId, "java.generic-ttlv.function.ttlv_item_view_from_item", targetId,
                "org.kmipkit.ttlv.TtlvItem.view", null, null, "decode", protocolVersion,
                null, null, treeHash);
        TtlvItem decodedItem;
        try {
            decodedItem = TtlvCodec.decode(credentialValueWire(modelRoute));
        } catch (RuntimeException error) {
            throw new AssertionError("The controlled generic Credential fixture must decode: " + modelRoute,
                    error);
        }
        try (TtlvItem decoded = decodedItem;
                var view = decoded.view();
                var decodedValue = view.value();
                var decodedStructureView = decodedValue.structure()) {
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertEquals(0x420025, view.tag().raw());
            assertTrue(decodedStructureView.itemCount() > 0,
                    "The generic decoder must preserve the Credential Value fields.");
        }
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_codec.decode", targetId,
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null, "decode", protocolVersion,
                null, null, treeHash);
    }

    private static byte[] credentialValueWire(String modelRoute) throws Exception {
        if (modelRoute.endsWith("usernameandpasswordcredential.fromttlv")) {
            return structure(0x420025,
                    textItem(0x420099, "validator-user"),
                    textItem(0x4200a1, "validator-password-fixture"));
        }
        if (modelRoute.endsWith("devicecredential.fromttlv")) {
            return structure(0x420025,
                    textItem(0x4200b0, "validator-serial"),
                    textItem(0x4200a1, "validator-password-fixture"),
                    textItem(0x4200a2, "validator-device-id"),
                    textItem(0x4200ab, "validator-network-id"),
                    textItem(0x4200a9, "validator-machine-id"),
                    textItem(0x4200aa, "validator-media-id"));
        }
        if (modelRoute.endsWith("attestationcredential.fromttlv")) {
            return structure(0x420025,
                    structure(0x4200c8,
                            item(0x4200c9, 8, new byte[] {0x01, 0x02}),
                            item(0x4200ca, 8, new byte[] {0x03, 0x04})),
                    item(0x4200c7, 5, intBytes(1)),
                    item(0x4200cb, 8, new byte[] {0x05, 0x06}));
        }
        if (modelRoute.endsWith("hashedpasswordcredential.fromttlv")) {
            return structure(0x420025,
                    textItem(0x420099, "validator-user"),
                    item(0x420092, 11, longBytes(1_800_000_000L)),
                    item(0x420038, 5, intBytes(6)),
                    item(0x420157, 8, new byte[] {0x11, 0x22, 0x33, 0x44}));
        }
        throw new IllegalArgumentException("No Credential Value wire fixture for the selected API route");
    }

    private static byte[] textItem(int tag, String value) throws Exception {
        return item(tag, 7, value.getBytes(StandardCharsets.UTF_8));
    }

    private static TtlvStructure credentialValueStructure(String modelRoute) throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        if (modelRoute.endsWith("usernameandpasswordcredential.fromttlv")) {
            TtlvStructure structure = TtlvStructure.create();
            structure = appendCredentialText(structure, 0x420099, "validator-user", limits);
            return appendCredentialText(structure, 0x4200a1, "validator-password-fixture", limits);
        }
        if (modelRoute.endsWith("devicecredential.fromttlv")) {
            TtlvStructure structure = TtlvStructure.create();
            structure = appendCredentialText(structure, 0x4200b0, "validator-serial", limits);
            structure = appendCredentialText(structure, 0x4200a1, "validator-password-fixture", limits);
            structure = appendCredentialText(structure, 0x4200a2, "validator-device-id", limits);
            structure = appendCredentialText(structure, 0x4200ab, "validator-network-id", limits);
            structure = appendCredentialText(structure, 0x4200a9, "validator-machine-id", limits);
            return appendCredentialText(structure, 0x4200aa, "validator-media-id", limits);
        }
        if (modelRoute.endsWith("attestationcredential.fromttlv")) {
            TtlvStructure nonce = TtlvStructure.create();
            nonce = appendCredentialValue(nonce, 0x4200c9,
                    TtlvValue.ByteString(new byte[] {0x01, 0x02}, limits), limits);
            nonce = appendCredentialValue(nonce, 0x4200ca,
                    TtlvValue.ByteString(new byte[] {0x03, 0x04}, limits), limits);
            TtlvStructure structure = TtlvStructure.create();
            structure = appendCredentialValue(structure, 0x4200c8, TtlvValue.structure(nonce, limits), limits);
            structure = appendCredentialValue(structure, 0x4200c7, TtlvValue.Enumeration(1), limits);
            return appendCredentialValue(structure, 0x4200cb,
                    TtlvValue.ByteString(new byte[] {0x05, 0x06}, limits), limits);
        }
        if (modelRoute.endsWith("hashedpasswordcredential.fromttlv")) {
            TtlvStructure structure = TtlvStructure.create();
            structure = appendCredentialText(structure, 0x420099, "validator-user", limits);
            structure = appendCredentialValue(structure, 0x420092,
                    TtlvValue.DateTimeExtended(1_800_000_000L), limits);
            structure = appendCredentialValue(structure, 0x420038, TtlvValue.Enumeration(6), limits);
            return appendCredentialValue(structure, 0x420157,
                    TtlvValue.ByteString(new byte[] {0x11, 0x22, 0x33, 0x44}, limits), limits);
        }
        throw new IllegalArgumentException("No Credential Value fixture for the selected API route");
    }

    private static TtlvStructure appendCredentialText(TtlvStructure structure, int rawTag,
            String fixtureValue, CodecLimits limits) {
        return appendCredentialValue(structure, rawTag,
                TtlvValue.TextString(fixtureValue.getBytes(StandardCharsets.UTF_8), limits), limits);
    }

    private static TtlvStructure appendCredentialValue(TtlvStructure structure, int rawTag,
            TtlvValue value, CodecLimits limits) {
        try (value; RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked();
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }

    private static void appendCryptographicParameterBuilderRouteEvent(ArrayNode events,
            EnumerationFixture fixture, String protocolVersion, String treeHash) {
        String routeId = CRYPTOGRAPHIC_PARAMETER_BUILDER_ROUTES.get(fixture.parentName());
        if (routeId == null) return;
        String method = switch (fixture.parentName()) {
            case "BLOCK-CIPHER-MODE" -> "blockCipherMode";
            case "CRYPTOGRAPHIC-ALGORITHM" -> "cryptographicAlgorithm";
            case "DIGITAL-SIGNATURE-ALGORITHM" -> "digitalSignatureAlgorithm";
            case "HASHING-ALGORITHM" -> "hashingAlgorithm";
            case "KEY-ROLE-TYPE" -> "keyRoleType";
            case "MASK-GENERATOR" -> "maskGenerator";
            case "PADDING-METHOD" -> "paddingMethod";
            default -> throw new IllegalStateException("Unknown Cryptographic Parameters route: "
                    + fixture.parentName());
        };
        try (CryptographicParameters.Builder builder = CryptographicParameters.builder()) {
            CryptographicParameters.Builder returned = switch (method) {
                case "blockCipherMode" -> builder.blockCipherMode(fixture.value());
                case "cryptographicAlgorithm" -> builder.cryptographicAlgorithm(fixture.value());
                case "digitalSignatureAlgorithm" -> builder.digitalSignatureAlgorithm(fixture.value());
                case "hashingAlgorithm" -> builder.hashingAlgorithm(fixture.value());
                case "keyRoleType" -> builder.keyRoleType(fixture.value());
                case "maskGenerator" -> builder.maskGenerator(fixture.value());
                case "paddingMethod" -> builder.paddingMethod(fixture.value());
                default -> throw new IllegalStateException("Unknown Cryptographic Parameters builder method: "
                        + method);
            };
            assertTrue(returned == builder, "The installed fluent setter must return its builder instance.");
        }
        appendRouteEvent(events, fixture.caseId(), routeId, fixture.targetId(),
                "org.kmipkit.protocol.CryptographicParameters.Builder." + method, null, null,
                "construct", protocolVersion, null, treeHash, null);
    }

    private static void appendTypedEnumerationRouteEvents(ArrayNode events, EnumerationFixture fixture,
            byte[] wire, String protocolVersion, String treeHash) throws Exception {
        if (fixture.attribute() == null) return;
        try (TtlvValue value = TtlvValue.Enumeration(fixture.value());
                TypedAttribute typed = TypedAttribute.create(fixture.attribute(), value);
                TtlvItem item = typed.asItem();
                TypedAttribute decoded = TypedAttribute.fromItem(fixture.attribute(), item)) {
            try (var view = decoded.view(); var decodedValue = view.value()) {
                assertEquals(fixture.rawTag(), view.tag().raw());
                assertEquals(fixture.value(), decodedValue.enumeration());
            }
            appendRouteEvent(events, fixture.caseId(),
                    "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                    fixture.targetId(), "org.kmipkit.ttlv.TypedAttribute.create", null, null,
                    "construct", protocolVersion, null, treeHash, null);
            appendRouteEvent(events, fixture.caseId(),
                    "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                    fixture.targetId(), "org.kmipkit.ttlv.TypedAttribute.fromItem", null, null,
                    "construct", protocolVersion, null, treeHash, null);
            try (TtlvStructure empty = TtlvStructure.create();
                    TtlvStructure attributeItems = TtlvStructure.withItem(empty, item, CodecLimits.defaults());
                    AttributeSet attributes = AttributeSet.tryNew(attributeItems)) {
                assertEquals(1, attributes.len());
                appendRouteEvent(events, fixture.caseId(),
                        "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew",
                        fixture.targetId(), "org.kmipkit.extensions.AttributeSet.tryNew", null, null,
                        "construct", protocolVersion, null,
                        normalizedTreeSha256(structure(0x420125, wire)), null);
            }
        }
    }

    private static List<BitmaskFixture> genericBitmaskFixtures() throws Exception {
        JsonNode catalog = readRouteEvidenceCatalog();
        var fixtures = new ArrayList<BitmaskFixture>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            String bitmaskName = testCase.path("requires").path("bitmask_type").asText();
            if (bitmaskName.isEmpty()) continue;
            String familyId = "KMIPKIT-ELEM-BITMASK-" + bitmaskName.toUpperCase(Locale.ROOT)
                    .replaceAll("[^A-Z0-9]+", "-");
            JsonNode tagTarget = null;
            JsonNode attributeTarget = null;
            for (JsonNode target : catalog.path("normative_targets")) {
                if ("tag".equals(target.path("kind").asText())
                        && normalizeTargetName(target.path("name").asText())
                                .equals(normalizeTargetName(bitmaskName))) tagTarget = target;
                if ("attribute".equals(target.path("kind").asText())
                        && normalizeTargetName(target.path("name").asText())
                                .equals(normalizeTargetName(bitmaskName))) attributeTarget = target;
            }
            if (tagTarget == null) continue;
            int rawTag = parseWireTag(tagTarget.path("wire_value").asText());
            StandardAttribute attribute = attributeTarget == null ? null : standardAttributeForTag(rawTag);
            if (attributeTarget != null && attribute == null) {
                throw new IllegalStateException("Catalog attribute has no installed StandardAttribute route: "
                        + bitmaskName);
            }
            var versions = new ArrayList<String>();
            for (JsonNode version : testCase.path("execution").path("protocol_versions")) {
                versions.add(version.asText());
            }
            for (JsonNode target : catalog.path("normative_targets")) {
                if (!"bitmask_value".equals(target.path("kind").asText())) continue;
                boolean belongsToFamily = false;
                for (JsonNode parent : target.path("parent_element_ids")) {
                    if (familyId.equals(parent.asText())) belongsToFamily = true;
                }
                if (!belongsToFamily) continue;
                String wireValue = target.path("wire_value").asText();
                if (wireValue.contains("-")) continue;
                long value = parseRepresentativeBitmaskValue(wireValue);
                fixtures.add(new BitmaskFixture(testCase.path("case_id").asText(),
                        target.path("target_id").asText(), rawTag, value,
                        attribute, List.copyOf(versions)));
            }
        }
        fixtures.addAll(genericPerTargetBitmaskFixtures(catalog));
        return List.copyOf(fixtures);
    }

    private static List<BitmaskFixture> genericPerTargetBitmaskFixtures(JsonNode catalog) throws Exception {
        Map<String, JsonNode> targets = new HashMap<>();
        for (JsonNode target : catalog.path("normative_targets")) {
            targets.put(target.path("target_id").asText(), target);
        }
        var fixtures = new ArrayList<BitmaskFixture>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"bitmask_value".equals(requires.path("target_kind").asText())
                    || !"local_model".equals(testCase.path("execution").path("mode").asText())) continue;
            JsonNode target = targets.get(requires.path("normative_target_id").asText());
            if (target == null || !target.path("parent_element_ids").isArray()
                    || target.path("parent_element_ids").isEmpty()) continue;
            String familyName = target.path("parent_element_ids").get(0).asText()
                    .replace("KMIPKIT-ELEM-BITMASK-", "");
            JsonNode tagTarget = targets.values().stream()
                    .filter(candidate -> "tag".equals(candidate.path("kind").asText()))
                    .filter(candidate -> normalizeTargetName(candidate.path("name").asText())
                            .equals(normalizeTargetName(familyName)))
                    .findFirst().orElse(null);
            if (tagTarget == null) continue;
            int rawTag = parseWireTag(tagTarget.path("wire_value").asText());
            JsonNode attributeTarget = targets.values().stream()
                    .filter(candidate -> "attribute".equals(candidate.path("kind").asText()))
                    .filter(candidate -> normalizeTargetName(candidate.path("name").asText())
                            .equals(normalizeTargetName(familyName)))
                    .findFirst().orElse(null);
            StandardAttribute attribute = attributeTarget == null ? null : standardAttributeForTag(rawTag);
            if (attributeTarget != null && attribute == null) {
                throw new IllegalStateException("Catalog bitmask attribute has no installed StandardAttribute route: "
                        + familyName);
            }
            var versions = new ArrayList<String>();
            for (JsonNode version : testCase.path("execution").path("protocol_versions")) {
                versions.add(version.asText());
            }
            fixtures.add(new BitmaskFixture(testCase.path("case_id").asText(),
                    target.path("target_id").asText(), rawTag,
                    parseRepresentativeBitmaskValue(target.path("wire_value").asText()),
                    attribute, List.copyOf(versions)));
        }
        return List.copyOf(fixtures);
    }

    private static List<DataTypeFixture> genericDataTypeFixtures() throws Exception {
        JsonNode catalog = readRouteEvidenceCatalog();
        Map<String, JsonNode> targets = new HashMap<>();
        for (JsonNode target : catalog.path("normative_targets")) {
            targets.put(target.path("target_id").asText(), target);
        }
        var fixtures = new ArrayList<DataTypeFixture>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"data_type".equals(requires.path("target_kind").asText())) continue;
            JsonNode target = targets.get(requires.path("normative_target_id").asText());
            if (target == null) continue;
            var versions = new ArrayList<String>();
            for (JsonNode version : testCase.path("execution").path("protocol_versions")) {
                versions.add(version.asText());
            }
            fixtures.add(new DataTypeFixture(testCase.path("case_id").asText(),
                    target.path("target_id").asText(), target.path("name").asText(),
                    parseWireTag(target.path("wire_value").asText()), List.copyOf(versions)));
        }
        return List.copyOf(fixtures);
    }

    private static void appendGenericDataTypeRouteEvents(ArrayNode events,
            List<DataTypeFixture> fixtures, Map<String, JsonNode> vectors,
            String protocolVersion) throws Exception {
        for (DataTypeFixture fixture : fixtures) {
            if (!fixture.protocolVersions().contains(protocolVersion)) continue;
            JsonNode vector = vectors.get(fixture.dataType());
            if (vector == null) throw new IllegalStateException("No TTLV boundary vectors for " + fixture.dataType());
            JsonNode representative = representativeDataTypeSample(vector, fixture.dataType());
            byte[] wire = HexFormat.of().parseHex(representative.path("hex").asText());
            String treeHash = normalizedTreeSha256(wire);
            for (JsonNode sample : vector.path("samples")) {
                if (!sample.path("valid").asBoolean()) continue;
                byte[] sampleWire = HexFormat.of().parseHex(sample.path("hex").asText());
                boolean primarySample = sample.path("branch").asText()
                        .equals(representative.path("branch").asText());
                String sampleHash = normalizedTreeSha256(sampleWire);
                try (TtlvValue value = createDataTypeValue(events, fixture, sampleWire,
                        protocolVersion, sampleHash, primarySample);
                        RawTag raw = RawTag.fromRaw(0x420001);
                        Tag tag = raw.checked();
                        TtlvItem created = TtlvItem.create(tag, value, CodecLimits.defaults());
                        var view = created.view();
                        var viewValue = view.value()) {
                    assertEquals(fixture.typeCode(), view.itemType().code());
                    assertTtlvDataTypeValue(fixture.dataType(), fixture.typeCode(), sampleWire, viewValue);
                }
                if (!primarySample) continue;
                appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_item.create",
                        fixture.targetId(), "org.kmipkit.ttlv.TtlvItem.create", null, null,
                        "construct", protocolVersion, null, treeHash, null);
                appendRouteEvent(events, fixture.caseId(),
                        "java.generic-ttlv.function.ttlv_item_view_from_item", fixture.targetId(),
                        "org.kmipkit.ttlv.TtlvItem.view", null, null,
                        "decode", protocolVersion, null, null, treeHash);
            }

            for (JsonNode sample : vector.path("samples")) {
                byte[] sampleWire = HexFormat.of().parseHex(sample.path("hex").asText());
                if (sample.path("valid").asBoolean()) {
                    try (TtlvItem decoded = TtlvCodec.decode(sampleWire);
                            var decodedView = decoded.view();
                            var decodedValue = decodedView.value()) {
                        assertEquals(fixture.typeCode(), decodedView.itemType().code());
                        assertTtlvDataTypeValue(fixture.dataType(), fixture.typeCode(), sampleWire, decodedValue);
                    }
                } else {
                    TtlvDecodeException failure = assertThrows(TtlvDecodeException.class,
                            () -> TtlvCodec.decode(sampleWire));
                    assertTrue(failure.byteOffset() >= 0);
                }
            }
            appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_codec.decode",
                    fixture.targetId(), "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                    "decode", protocolVersion, null, null, treeHash);
        }
    }

    private static Map<String, JsonNode> dataTypeBoundaryVectors() throws Exception {
        JsonNode source = new ObjectMapper().readTree(
                Path.of("../shared/vectors/ttlv-boundaries.json").toFile());
        Map<String, JsonNode> vectors = new HashMap<>();
        for (JsonNode vector : source.path("vectors")) {
            vectors.put(vector.path("data_type").asText(), vector);
        }
        return Map.copyOf(vectors);
    }

    private static JsonNode representativeDataTypeSample(JsonNode vector, String dataType) {
        String desiredBranch = "Structure".equals(dataType) ? "nested_integer" : null;
        JsonNode firstValid = null;
        for (JsonNode sample : vector.path("samples")) {
            if (!sample.path("valid").asBoolean()) continue;
            if (firstValid == null) firstValid = sample;
            if (desiredBranch != null && desiredBranch.equals(sample.path("branch").asText())) return sample;
        }
        if (firstValid == null) throw new IllegalStateException("TTLV boundary vector has no valid sample");
        return firstValid;
    }

    private static TtlvValue createDataTypeValue(ArrayNode events, DataTypeFixture fixture,
            byte[] wire, String protocolVersion, String treeHash, boolean recordEvidence) throws Exception {
        String dataType = fixture.dataType();
        byte[] bytes = dataTypeValueBytes(wire);
        CodecLimits limits = CodecLimits.defaults();
        TtlvValue value;
        switch (dataType) {
            case "Big Integer" -> value = TtlvValue.BigInteger(bytes, limits);
            case "Boolean" -> value = TtlvValue.Boolean(ByteBuffer.wrap(bytes).getLong() != 0);
            case "Byte String" -> value = TtlvValue.ByteString(bytes, limits);
            case "Date Time" -> value = TtlvValue.DateTime(ByteBuffer.wrap(bytes).getLong());
            case "Date Time Extended" -> value = TtlvValue.DateTimeExtended(ByteBuffer.wrap(bytes).getLong());
            case "Enumeration" -> value = TtlvValue.Enumeration(Integer.toUnsignedLong(ByteBuffer.wrap(bytes).getInt()));
            case "Integer" -> value = TtlvValue.Integer(ByteBuffer.wrap(bytes).getInt());
            case "Interval" -> value = TtlvValue.Interval(Integer.toUnsignedLong(ByteBuffer.wrap(bytes).getInt()));
            case "Long Integer" -> value = TtlvValue.LongInteger(ByteBuffer.wrap(bytes).getLong());
            case "Text String" -> value = TtlvValue.TextString(bytes, limits);
            case "Structure" -> {
                TtlvStructure structure = TtlvStructure.create();
                boolean withItemRecorded = false;
                try {
                    if (recordEvidence) {
                        appendRouteEvent(events, fixture.caseId(),
                                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.create",
                                fixture.targetId(), "org.kmipkit.ttlv.TtlvStructure.create", null, null,
                                "construct", protocolVersion, null, treeHash, null);
                    }
                    for (byte[] childWire : structureChildWires(wire)) {
                        try (TtlvItem child = TtlvCodec.decode(childWire)) {
                            structure = TtlvStructure.withItem(structure, child, limits);
                        }
                        if (recordEvidence && !withItemRecorded) {
                            appendRouteEvent(events, fixture.caseId(),
                                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.withitem",
                                    fixture.targetId(), "org.kmipkit.ttlv.TtlvStructure.withItem", null, null,
                                    "construct", protocolVersion, null, treeHash, null);
                            withItemRecorded = true;
                        }
                    }
                    value = TtlvValue.structure(structure, limits);
                } finally {
                    structure.close();
                }
            }
            default -> throw new IllegalArgumentException("Unsupported TTLV data type: " + dataType);
        }
        String suffix = dataType.replaceAll("[^A-Za-z]", "").toLowerCase(Locale.ROOT);
        if (recordEvidence) {
            appendRouteEvent(events, fixture.caseId(),
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue." + suffix,
                    fixture.targetId(), "org.kmipkit.ttlv.TtlvValue." + ttlvValueConstructorName(dataType),
                    null, null, "construct", protocolVersion, null, treeHash, null);
        }
        return value;
    }

    private static String ttlvValueConstructorName(String dataType) {
        return switch (dataType) {
            case "Big Integer" -> "BigInteger";
            case "Byte String" -> "ByteString";
            case "Date Time" -> "DateTime";
            case "Date Time Extended" -> "DateTimeExtended";
            case "Long Integer" -> "LongInteger";
            case "Structure" -> "structure";
            case "Text String" -> "TextString";
            default -> dataType.replace(" ", "");
        };
    }

    private static void assertTtlvDataTypeValue(String dataType, int typeCode, byte[] wire,
            org.kmipkit.ttlv.TtlvValueView value) throws Exception {
        byte[] bytes = dataTypeValueBytes(wire);
        switch (typeCode) {
            case 1 -> {
                List<Integer> tags = structureChildTags(wire);
                try (var structure = value.structure()) {
                    assertEquals(tags.size(), structure.itemCount());
                    for (int index = 0; index < tags.size(); index++) {
                        try (var child = structure.itemAt(index); Tag tag = child.tag()) {
                            assertEquals(tags.get(index), tag.raw());
                        }
                    }
                }
            }
            case 2 -> assertEquals(ByteBuffer.wrap(bytes).getInt(), value.integer());
            case 3, 9, 11 -> assertEquals(ByteBuffer.wrap(bytes).getLong(), switch (typeCode) {
                case 3 -> value.longInteger();
                case 9 -> value.dateTime();
                default -> value.dateTimeExtended();
            });
            case 4, 7, 8 -> assertArrayEquals(bytes, value.bytes());
            case 5 -> assertEquals(Integer.toUnsignedLong(ByteBuffer.wrap(bytes).getInt()), value.enumeration());
            case 6 -> assertEquals(ByteBuffer.wrap(bytes).getLong() != 0, value.booleanValue());
            case 10 -> assertEquals(Integer.toUnsignedLong(ByteBuffer.wrap(bytes).getInt()), value.interval());
            default -> throw new IllegalArgumentException("Unsupported TTLV type code " + typeCode + " for " + dataType);
        }
    }

    private static byte[] dataTypeValueBytes(byte[] wire) {
        int length = ByteBuffer.wrap(wire, 4, 4).getInt();
        return java.util.Arrays.copyOfRange(wire, 8, Math.addExact(8, length));
    }

    private static List<Integer> structureChildTags(byte[] wire) {
        int valueLength = ByteBuffer.wrap(wire, 4, 4).getInt();
        int end = Math.addExact(8, valueLength);
        int offset = 8;
        var tags = new ArrayList<Integer>();
        while (offset < end) {
            int tag = (Byte.toUnsignedInt(wire[offset]) << 16)
                    | (Byte.toUnsignedInt(wire[offset + 1]) << 8)
                    | Byte.toUnsignedInt(wire[offset + 2]);
            int childLength = ByteBuffer.wrap(wire, offset + 4, 4).getInt();
            tags.add(tag);
            offset = Math.addExact(offset, 8 + ((Math.addExact(childLength, 7)) & ~7));
        }
        if (offset != end) throw new IllegalArgumentException("TTLV structure fixture has a partial child");
        return List.copyOf(tags);
    }

    private static List<byte[]> structureChildWires(byte[] wire) {
        int valueLength = ByteBuffer.wrap(wire, 4, 4).getInt();
        int end = Math.addExact(8, valueLength);
        int offset = 8;
        var items = new ArrayList<byte[]>();
        while (offset < end) {
            int length = ByteBuffer.wrap(wire, offset + 4, 4).getInt();
            int itemEnd = Math.addExact(offset, 8 + ((Math.addExact(length, 7)) & ~7));
            if (itemEnd > end) throw new IllegalArgumentException("TTLV structure fixture has a partial child");
            items.add(java.util.Arrays.copyOfRange(wire, offset, itemEnd));
            offset = itemEnd;
        }
        if (offset != end) throw new IllegalArgumentException("TTLV structure fixture has a partial child");
        return List.copyOf(items);
    }

    private static StandardAttribute standardAttributeForTag(int rawTag) {
        for (StandardAttribute attribute : StandardAttribute.values()) {
            if (attribute.tag() == rawTag) return attribute;
        }
        return null;
    }

    private static int parseWireTag(String wireValue) {
        return Math.toIntExact(Long.parseUnsignedLong(wireValue.replaceFirst("^0[xX]", ""), 16));
    }

    private static Path routeEvidenceCatalogPath() {
        return Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
    }

    private static JsonNode readRouteEvidenceCatalog() throws Exception {
        return new ObjectMapper().readTree(routeEvidenceCatalogPath().toFile());
    }

    private static long parseRepresentativeBitmaskValue(String wireValue) {
        String digits = wireValue.replaceFirst("^0[xX]", "");
        StringBuilder representative = new StringBuilder(digits.length());
        boolean chosenExtensionBit = false;
        for (char digit : digits.toCharArray()) {
            if (digit == 'X' || digit == 'x') {
                representative.append(chosenExtensionBit ? '0' : '8');
                chosenExtensionBit = true;
            } else {
                representative.append(digit);
            }
        }
        return Long.parseUnsignedLong(representative.toString(), 16);
    }

    private static void appendGenericBitmaskRouteEvents(ArrayNode events,
            List<BitmaskFixture> fixtures, String protocolVersion) throws Exception {
        for (BitmaskFixture fixture : fixtures) {
            if (!fixture.protocolVersions().contains(protocolVersion)) continue;
            int rawValue = (int) fixture.value();
            byte[] wire = item(fixture.rawTag(), 2, intBytes(rawValue));
            String treeHash = normalizedTreeSha256(wire);
            try (TtlvValue value = TtlvValue.Integer(rawValue)) {
                appendRouteEvent(events, fixture.caseId(),
                        "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.integer",
                        fixture.targetId(), "org.kmipkit.ttlv.TtlvValue.Integer", null, null,
                        "construct", protocolVersion, null, treeHash, null);
                appendLocalTtlvRoundtripEvents(events, fixture.caseId(), fixture.targetId(),
                        fixture.rawTag(), TtlvItemType.Integer, value, wire,
                        protocolVersion, rawValue);
                appendTypedBitmaskRouteEvents(events, fixture, wire, protocolVersion, treeHash);
            }
        }
    }

    private static void appendTypedBitmaskRouteEvents(ArrayNode events, BitmaskFixture fixture,
            byte[] wire, String protocolVersion, String treeHash) throws Exception {
        if (fixture.attribute() == null) return;
        try (TtlvValue value = TtlvValue.Integer((int) fixture.value());
                TypedAttribute typed = TypedAttribute.create(fixture.attribute(), value);
                TtlvItem item = typed.asItem();
                TypedAttribute decoded = TypedAttribute.fromItem(fixture.attribute(), item)) {
            try (var view = decoded.view(); var decodedValue = view.value()) {
                assertEquals(fixture.rawTag(), view.tag().raw());
                assertEquals((int) fixture.value(), decodedValue.integer());
            }
            appendRouteEvent(events, fixture.caseId(),
                    "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                    fixture.targetId(), "org.kmipkit.ttlv.TypedAttribute.create", null, null,
                    "construct", protocolVersion, null, treeHash, null);
            appendRouteEvent(events, fixture.caseId(),
                    "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                    fixture.targetId(), "org.kmipkit.ttlv.TypedAttribute.fromItem", null, null,
                    "construct", protocolVersion, null, treeHash, null);
            try (TtlvStructure empty = TtlvStructure.create();
                    TtlvStructure attributeItems = TtlvStructure.withItem(empty, item, CodecLimits.defaults());
                    AttributeSet attributes = AttributeSet.tryNew(attributeItems)) {
                assertEquals(1, attributes.len());
                appendRouteEvent(events, fixture.caseId(),
                        "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew",
                        fixture.targetId(), "org.kmipkit.extensions.AttributeSet.tryNew", null, null,
                        "construct", protocolVersion, null,
                        normalizedTreeSha256(structure(0x420125, wire)), null);
            }
        }
    }

    private static String normalizeTargetName(String value) {
        return value.replaceAll("[^A-Za-z0-9]", "").toUpperCase(Locale.ROOT);
    }

    private static void appendActivationDateLocalRouteEvents(ArrayNode events, String protocolVersion)
            throws Exception {
        byte[] activationDateWire = item(0x420001, 9, longBytes(1));
        byte[] getAttributesOperationWire = item(0x42005c, 5, intBytes(0x0b));
        String activationDateHash = normalizedTreeSha256(activationDateWire);
        try (TtlvValue value = TtlvValue.DateTime(1)) {
            appendRouteEvent(events, "test-attr-activation-date",
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.datetime",
                    "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", "org.kmipkit.ttlv.TtlvValue.DateTime",
                    null, null, "construct", protocolVersion, null, activationDateHash, null);
            appendLocalTtlvRoundtripEvents(events, "test-attr-activation-date",
                    "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", 0x420001, TtlvItemType.DateTime,
                    value, activationDateWire, protocolVersion, 1);
        }
        try (TtlvValue value = TtlvValue.Enumeration(0x0b)) {
            appendLocalTtlvRoundtripEvents(events, "test-attr-activation-date",
                    "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES", 0x42005c, TtlvItemType.Enumeration,
                    value, getAttributesOperationWire, protocolVersion, 0x0b);
        }

        CodecLimits limits = CodecLimits.defaults();
        try (TtlvValue value = TtlvValue.DateTime(1);
                TypedAttribute typed = TypedAttribute.create(StandardAttribute.ACTIVATION_DATE, value);
                TtlvItem item = typed.asItem();
                TypedAttribute decoded = TypedAttribute.fromItem(StandardAttribute.ACTIVATION_DATE, item)) {
            assertEquals(0x420001, StandardAttribute.ACTIVATION_DATE.tag());
            try (var view = decoded.view(); var decodedValue = view.value()) {
                assertEquals(1L, decodedValue.dateTime());
            }
            appendRouteEvent(events, "test-attr-activation-date",
                    "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                    "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", "org.kmipkit.ttlv.TypedAttribute.create",
                    null, null, "construct", protocolVersion, null, activationDateHash, null);
            appendRouteEvent(events, "test-attr-activation-date",
                    "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                    "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", "org.kmipkit.ttlv.TypedAttribute.fromItem",
                    null, null, "construct", protocolVersion, null, activationDateHash, null);

            try (TtlvStructure empty = TtlvStructure.create();
                    TtlvStructure attributeItems = TtlvStructure.withItem(empty, item, limits);
                    AttributeSet attributes = AttributeSet.tryNew(attributeItems)) {
                assertEquals(1, attributes.len());
                appendRouteEvent(events, "test-attr-activation-date",
                        "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew",
                        "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", "org.kmipkit.extensions.AttributeSet.tryNew",
                        null, null, "construct", protocolVersion, null,
                        normalizedTreeSha256(structure(0x420125, activationDateWire)), null);
            }
        }

        UniqueIdentifier identifier = UniqueIdentifier.textString("object-7");
        byte[] requestPayloadWire = structure(0x42007c,
                item(0x420094, 7, "object-7".getBytes(StandardCharsets.UTF_8)),
                item(0x42013b, 5, intBytes(0x420001)));
        try (GetAttributesRequest request = GetAttributesRequest.create(identifier,
                        List.of(AttributeReference.tag(0x420001)));
                TtlvStructure payload = request.toTtlvPayload();
                var view = payload.view()) {
            assertEquals(2, view.itemCount());
            appendRouteEvent(events, "test-attr-activation-date",
                    "java.typed-protocol.function.attribute_request_get_attributes_to_ttlv_payload",
                    "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES",
                    "org.kmipkit.extensions.GetAttributesRequest.toTtlvPayload", null, null,
                    "encode", protocolVersion, null, normalizedTreeSha256(requestPayloadWire), null);
        }
    }

    private static void appendLocalTtlvRoundtripEvents(ArrayNode events, String caseId, String targetId,
            int rawTag, TtlvItemType expectedType, TtlvValue value, byte[] wire,
            String protocolVersion, long expectedValue) throws Exception {
        String wireHash = normalizedTreeSha256(wire);
        try (RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                TtlvItem created = TtlvItem.create(tag, value, CodecLimits.defaults());
                var view = created.view();
                var viewValue = view.value()) {
            assertEquals(rawTag, view.tag().raw());
            assertEquals(expectedType, view.itemType());
            if (expectedType == TtlvItemType.DateTime) assertEquals(expectedValue, viewValue.dateTime());
            else if (expectedType == TtlvItemType.Integer) assertEquals((int) expectedValue, viewValue.integer());
            else assertEquals(expectedValue, viewValue.enumeration());
        }
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_item.create", targetId,
                "org.kmipkit.ttlv.TtlvItem.create", null, null, "construct", protocolVersion,
                null, wireHash, null);
        appendRouteEvent(events, caseId, "java.generic-ttlv.function.ttlv_item_view_from_item", targetId,
                "org.kmipkit.ttlv.TtlvItem.view", null, null, "decode", protocolVersion,
                null, null, wireHash);
        try (TtlvItem decoded = TtlvCodec.decode(wire);
                var view = decoded.view();
                var decodedValue = view.value()) {
            assertEquals(rawTag, view.tag().raw());
            assertEquals(expectedType, view.itemType());
            if (expectedType == TtlvItemType.DateTime) assertEquals(expectedValue, decodedValue.dateTime());
            else if (expectedType == TtlvItemType.Integer) assertEquals((int) expectedValue, decodedValue.integer());
            else assertEquals(expectedValue, decodedValue.enumeration());
        }
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_codec.decode", targetId,
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null, "decode", protocolVersion,
                null, null, wireHash);
    }

    private void appendControlledActivationDateNetworkEvents(ArrayNode events, String protocolVersion)
            throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        int minor = "2.0".equals(protocolVersion) ? 0 : 1;
        SSLServerSocket listener = listener(certificate(serverCertificatePem),
                privateKey(serverPrivateKeyPem), certificate(clientCertificatePem));
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> serveActivationDateRequests(listener, minor));
        Thread peerThread = new Thread(peer, "java-validator-activation-date-peer-" + protocolVersion);
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("activation-date-" + protocolVersion + ".json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, false, protocolVersion));
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
            try (ClientBatchItem request = ClientBatchItem.getAttributes(
                    UniqueIdentifier.textString("object-7"), List.of(AttributeReference.tag(0x420001)))
                    .withUniqueBatchItemId("item-0".getBytes(StandardCharsets.US_ASCII))) {
                executeGetAttributesForEvidence(driver, request, true);
            }
            try (GetAttributesRequest request = GetAttributesRequest.create(
                    UniqueIdentifier.textString("object-7"), List.of(AttributeReference.tag(0x420001)));
                    ClientBatchItem batchItem = request.toBatchItem()
                            .withUniqueBatchItemId("item-0".getBytes(StandardCharsets.US_ASCII))) {
                executeGetAttributesForEvidence(driver, batchItem, false);
            }
        } finally {
            listener.close();
        }

        List<byte[]> requests = peer.get(15, TimeUnit.SECONDS);
        assertEquals(2, requests.size());
        for (int index = 0; index < requests.size(); index++) {
            byte[] observedRequest = requests.get(index);
            assertEquals(0x0bL, requestOperation(observedRequest));
            assertEquals(minor, requestProtocolMinorVersion(observedRequest));
            assertTrue(contains(observedRequest, item(0x42013b, 5, intBytes(0x420001))),
                    "The controlled peer must receive the requested Activation Date reference.");
            String routeId = index == 0 ? "java.typed-protocol.client_batch_item.get_attributes"
                    : "java.typed-protocol.function.attribute_request_get_attributes_to_batch_item";
            String symbol = index == 0 ? "org.kmipkit.extensions.ClientBatchItem.getAttributes"
                    : "org.kmipkit.extensions.GetAttributesRequest.toBatchItem";
            String requestHash = normalizedTreeSha256(observedRequest);
            String responseHash = normalizedTreeSha256(activationDateResponse(minor));
            appendTypedNetworkRouteEvents(events, "test-attr-activation-date", routeId,
                    "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES", symbol, protocolVersion,
                    requestHash, responseHash);
        }
        appendRouteEvent(events, "test-attr-activation-date",
                "java.typed-protocol.function.client_batch_item_response_get_attributes",
                "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES",
                "org.kmipkit.extensions.ClientBatchItemResult.getAttributesResponse", null, null,
                "decode", protocolVersion, null, null,
                normalizedTreeSha256(activationDateResponse(minor)));
    }

    private void appendControlledKeyFormatRawResponseRouteEvents(ArrayNode events, String protocolVersion)
            throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        int minor = "2.0".equals(protocolVersion) ? 0 : 1;
        SSLServerSocket listener = listener(certificate(serverCertificatePem),
                privateKey(serverPrivateKeyPem), certificate(clientCertificatePem));
        byte[] response = getResponse(minor);
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, response));
        Thread peerThread = new Thread(peer, "java-validator-key-format-peer-" + protocolVersion);
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("key-format-" + protocolVersion + ".json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, false, protocolVersion));
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile));
                ClientBatch batch = ClientBatch.create();
                ClientBatchItem request = ClientBatchItem.get(
                        UniqueIdentifier.textString(CREATED_OBJECT_ID), null, null, null, null)
                        .withUniqueBatchItemId("item-0".getBytes(StandardCharsets.US_ASCII))) {
            batch.add(request);
            try (ClientBatchResponse result = driver.executeBatchForEvidence(batch)) {
                assertEquals(1, result.itemCount());
                try (ClientBatchItemResult itemResult = result.itemAt(0);
                        TtlvStructure raw = itemResult.raw().orElseThrow();
                        TtlvStructureView view = raw.view()) {
                    assertEquals(0x0aL, itemResult.operation());
                    assertEquals(3L, view.itemCount());
                    String responseHash = normalizedTreeSha256(getResponsePayload());
                    appendRouteEvent(events,
                            KEY_FORMAT_RAW_CASE_ID,
                            CLIENT_BATCH_ITEM_RAW_TTLV_ROUTE,
                            KEY_FORMAT_RAW_TARGET_ID,
                            "org.kmipkit.extensions.ClientBatchItemResult.raw", null, null,
                            "decode", protocolVersion, null, null, responseHash);
                    appendRouteEvent(events,
                            KEY_FORMAT_RAW_AGGREGATE_CASE_ID,
                            CLIENT_BATCH_ITEM_RAW_TTLV_ROUTE,
                            KEY_FORMAT_RAW_TARGET_ID,
                            "org.kmipkit.extensions.ClientBatchItemResult.raw", null, null,
                            "decode", protocolVersion, null, null, responseHash);
                }
            }
        } finally {
            listener.close();
        }
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertEquals(0x0aL, requestOperation(request));
        assertEquals(minor, requestProtocolMinorVersion(request));
        assertTrue(contains(response, item(0x420042, 5, intBytes(1))),
                "The controlled Get response must contain the Raw Key Format Type value.");
    }

    private static void executeGetAttributesForEvidence(JavaKmipDriver driver, ClientBatchItem request,
            boolean readTypedResponse) throws Exception {
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(request);
            try (ClientBatchResponse response = driver.executeBatchForEvidence(batch);
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(0x0bL, result.operation());
                assertEquals(0L, result.resultStatus());
                if (readTypedResponse) {
                    try (var typed = result.getAttributesResponse().orElseThrow();
                            TtlvStructure attributes = typed.attributes().orElseThrow();
                            var view = attributes.view();
                            var attribute = view.itemAt(0);
                            var value = attribute.value()) {
                        assertEquals("object-7", typed.uniqueIdentifier());
                        assertEquals(1, view.itemCount());
                        assertEquals(0x420001, attribute.tag().raw());
                        assertEquals(TtlvItemType.DateTime, attribute.itemType());
                        assertEquals(1L, value.dateTime());
                    }
                }
            }
        }
    }

    private static List<byte[]> serveActivationDateRequests(SSLServerSocket listener, int minor)
            throws Exception {
        var observedRequests = new ArrayList<byte[]>();
        byte[] response = activationDateResponse(minor);
        for (int index = 0; index < 2; index++) {
            byte[] request = serveOneAndCapture(listener, response);
            if (requestOperation(request) != 0x0bL || requestProtocolMinorVersion(request) != minor) {
                throw new AssertionError("The controlled peer observed an unexpected Get Attributes request.");
            }
            observedRequests.add(request);
        }
        return List.copyOf(observedRequests);
    }

    private static byte[] activationDateResponse(int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] attributes = structure(0x420125, item(0x420001, 9, longBytes(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, "object-7".getBytes(StandardCharsets.UTF_8)), attributes);
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(0x0b)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static void appendAllocatedTagRangeEvents(ArrayNode events, TagRangeFixture fixture,
            String protocolVersion, byte[] wire, String treeHash) throws Exception {
        try (RawTag raw = RawTag.fromRaw(fixture.rawTag());
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Enumeration(0);
                TtlvItem created = TtlvItem.create(tag, value, CodecLimits.defaults());
                var view = created.view();
                var viewValue = view.value()) {
            assertEquals(fixture.rawTag(), view.tag().raw());
            assertEquals(TtlvItemType.Enumeration, view.itemType());
            assertEquals(0L, viewValue.enumeration());
            appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_item.create",
                    fixture.targetId(), "org.kmipkit.ttlv.TtlvItem.create", null, null,
                    "construct", protocolVersion, null, treeHash, null);
            appendRouteEvent(events, fixture.caseId(),
                    "java.generic-ttlv.function.ttlv_item_view_from_item", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvItem.view", null, null,
                    "decode", protocolVersion, null, null, treeHash);
        }
        try (TtlvItem decoded = TtlvCodec.decode(wire); var decodedView = decoded.view()) {
            assertEquals(fixture.rawTag(), decodedView.tag().raw());
            assertEquals(TtlvItemType.Enumeration, decodedView.itemType());
        }
        appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_codec.decode",
                fixture.targetId(), "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null, null, treeHash);
    }

    private static void appendRejectedTagRangeEvent(ArrayNode events, TagRangeFixture fixture,
            String protocolVersion, byte[] wire) throws Exception {
        TtlvDecodeException rejection = assertThrows(
                TtlvDecodeException.class, () -> TtlvCodec.decode(wire));
        int expectedKind = "reserved".equals(fixture.allocation())
                ? TtlvDecodeErrorKind.RESERVED_TAG : TtlvDecodeErrorKind.UNALLOCATED_TAG;
        assertEquals(expectedKind, rejection.kind());
        ObjectNode event = newRouteEvent(events, fixture.caseId(),
                "java.generic-ttlv.ttlv_codec.decode", fixture.targetId(),
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null);
        event.put("decode_outcome", "rejected");
        event.put("decode_rejection_code", "reserved".equals(fixture.allocation())
                ? "tag_reserved" : "tag_not_allocated");
        event.put("decode_input_sha256", sha256(wire));
    }

    private static void appendNetworkQueryRouteEvents(ArrayNode events, String protocolVersion,
            String requestTreeHash, String responseTreeHash) {
        String routeId = "java.typed-protocol.client_batch_item.query";
        String symbol = "org.kmipkit.extensions.ClientBatchItem.query";
        appendTypedNetworkRouteEvents(events, "test-op-query", routeId,
                "KMIPKIT-ELEM-OP-C2S-QUERY", symbol, protocolVersion, requestTreeHash, responseTreeHash);
        appendRouteEvent(events, "test-op-query", "java.typed-protocol.function.client_batch_item_response_query",
                "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.extensions.ClientBatchItemResult.queryResponse",
                null, null, "decode", protocolVersion, null, null, responseTreeHash);
        appendRouteEvent(events, "test-op-query", "java.typed-protocol.function.client_batch_item_response_query_field_at",
                "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.extensions.QueryResponse.fieldAt",
                null, null, "decode", protocolVersion, null, null, responseTreeHash);
        appendRouteEvent(events, "test-op-query", "java.typed-protocol.function.client_batch_item_response_query_field_count",
                "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.extensions.QueryResponse.fieldCount",
                null, null, "decode", protocolVersion, null, null, responseTreeHash);
    }

    private static void appendTypedNetworkRouteEvents(ArrayNode events, String caseId, String routeId,
            String targetId, String publicApiSymbol, String protocolVersion,
            String requestTreeHash, String responseTreeHash) {
        String executionRoute = "java.execute.typed-batch";
        String executionSymbol = "org.kmipkit.extensions.Client.execute";
        appendRouteEvent(events, caseId, routeId, targetId, publicApiSymbol,
                executionRoute, executionSymbol, "construct", protocolVersion, "raw_tls",
                requestTreeHash, null);
        ObjectNode sent = appendRouteEvent(events, caseId, routeId, targetId, publicApiSymbol,
                executionRoute, executionSymbol, "send", protocolVersion, "raw_tls",
                requestTreeHash, null);
        sent.put("request_delivery_state", "response_received");
        sent.put("transport_invocation_count", 1);
        appendRouteEvent(events, caseId, routeId, targetId, publicApiSymbol,
                executionRoute, executionSymbol, "decode", protocolVersion, "raw_tls",
                null, responseTreeHash);
    }

    private static ObjectNode appendRouteEvent(ArrayNode events, String caseId, String routeId,
            String targetId, String publicApiSymbol, String executionRouteId, String executionApiSymbol,
            String action, String protocolVersion, String transport, String requestHash,
            String responseHash) {
        ObjectNode event = newRouteEvent(events, caseId, routeId, targetId, publicApiSymbol,
                executionRouteId, executionApiSymbol, action, protocolVersion, transport);
        if (requestHash != null) event.put("request_tree_sha256", requestHash);
        if (responseHash != null) event.put("response_tree_sha256", responseHash);
        return event;
    }

    private static ObjectNode newRouteEvent(ArrayNode events, String caseId, String routeId,
            String targetId, String publicApiSymbol, String executionRouteId, String executionApiSymbol,
            String action, String protocolVersion, String transport) {
        ObjectNode event = events.addObject();
        event.put("case_id", caseId);
        event.put("route_id", routeId);
        event.put("target_id", targetId);
        event.put("public_api_symbol", publicApiSymbol);
        if (executionRouteId == null) event.putNull("execution_route_id");
        else event.put("execution_route_id", executionRouteId);
        if (executionApiSymbol == null) event.putNull("execution_api_symbol");
        else event.put("execution_api_symbol", executionApiSymbol);
        event.put("invocation_id", UUID.randomUUID().toString());
        event.put("action", action);
        event.put("protocol_version", protocolVersion);
        event.put("encoding", "TTLV");
        if (transport == null) event.putNull("transport");
        else event.put("transport", transport);
        event.put("status", "passed");
        event.put("invocation_count", 1);
        return event;
    }

    private static void writeRouteExecutionReport(Path reportPath, ObjectMapper mapper,
            ArrayNode events) throws Exception {
        Path catalogPath = routeEvidenceCatalogPath();
        Path manifestPath = Path.of("../shared/coverage/route-capabilities.json");
        Path pinPath = Path.of("library-pin.json");
        String libraryCommit = mapper.readTree(pinPath.toFile()).path("commit").asText();
        var artifact = ArtifactProvenance.load(Path.of(System.getProperty("kmipkit.artifactManifest")), libraryCommit);
        Path loadedJar = Path.of(ClientBatchItem.class.getProtectionDomain().getCodeSource()
                .getLocation().toURI()).toAbsolutePath().normalize();
        assertEquals(artifact.artifactPath(), loadedJar,
                "All route events must come from the independently installed package JAR.");

        ObjectNode report = mapper.createObjectNode();
        report.put("schema_version", "1.0");
        report.put("run_id", "java-controlled-query-" + UUID.randomUUID());
        report.put("language", "java");
        ObjectNode server = report.putObject("server");
        server.put("id", "java-controlled-tls-peer");
        server.put("product", "KMIPKit Java validator controlled fixture");
        server.put("version", "fixture-1");
        server.put("configuration_fingerprint", sha256("java-controlled-query-peer-fixture-v1"));
        report.put("library_commit", artifact.libraryCommit());
        report.put("artifact_sha256", artifact.artifactSha256());
        report.put("catalog_sha256", ArtifactProvenance.sha256(catalogPath));
        report.put("route_manifest_sha256", ArtifactProvenance.sha256(manifestPath));
        ArrayNode caseIds = report.putArray("case_ids").add("test-op-query");
        TAG_RANGE_FIXTURES.forEach(fixture -> caseIds.add(fixture.caseId()));
        caseIds.add("test-attr-activation-date");
        genericEnumerationFixtures().forEach(fixture -> caseIds.add(fixture.caseId()));
        genericLocalEnumerationFixtures().stream().map(EnumerationFixture::caseId).distinct()
                .forEach(caseIds::add);
        genericBitmaskFixtures().stream().map(BitmaskFixture::caseId).distinct().forEach(caseIds::add);
        genericDataTypeFixtures().forEach(fixture -> caseIds.add(fixture.caseId()));
        caseIds.add(RESPONSE_BATCH_ITEM_CASE_ID);
        caseIds.add("test-norm-message-field-message-field-8-5-batch-count");
        caseIds.add("test-norm-message-field-message-field-9-5-batch-count");
        caseIds.add("test-norm-message-field-message-field-8-6-batch-item");
        caseIds.add("test-norm-message-field-message-field-9-7-batch-item");
        caseIds.add("test-norm-message-field-message-field-8-6-message-extension");
        caseIds.add("test-norm-message-field-message-field-9-13-message-extension");
        caseIds.add("test-norm-tag-tag-420051");
        caseIds.add("test-norm-message-field-message-field-8-3-ephemeral");
        caseIds.add("test-norm-structure-member-structure-member-7-23-object-group");
        caseIds.add("test-norm-operation-structure-operation-structure-7-2-asynchronous-request");
        caseIds.add("test-norm-operation-structure-operation-structure-7-13-extension-information");
        caseIds.add(OBJECT_TYPE_MEMBER_CASE_ID);
        caseIds.add(CONSTRAINT_CASE_ID);
        caseIds.add(CONSTRAINTS_CASE_ID);
        PROFILE_INFORMATION_CASE_IDS.forEach(caseIds::add);
        OPERATION_STRUCTURE_FIXTURES.forEach(fixture -> caseIds.add(fixture.caseId()));
        SCALAR_OPERATION_STRUCTURE_FIXTURES.forEach(fixture -> caseIds.add(fixture.caseId()));
        STRUCTURE_MEMBER_STRUCTURE_FIXTURES.forEach(fixture -> caseIds.add(fixture.caseId()));
        STRUCTURE_MEMBER_BYTE_STRING_FIXTURES.forEach(fixture -> caseIds.add(fixture.caseId()));
        STRUCTURE_MEMBER_VALUE_FIXTURES.forEach(fixture -> caseIds.add(fixture.caseId()));
        caseIds.add("test-norm-payload-client-add-attribute-request-20-002");
        caseIds.add("test-norm-payload-client-add-attribute-request-21-002");
        caseIds.add("test-norm-payload-client-modify-attribute-request-20-003");
        caseIds.add("test-norm-payload-client-modify-attribute-request-21-003");
        caseIds.add("test-norm-payload-client-set-attribute-request-20-002");
        caseIds.add("test-norm-payload-client-set-attribute-request-21-002");
        genericOnlyTagCaseIds().forEach(caseIds::add);
        typedTagIdentityCaseIds().stream()
                .filter(caseId -> !"test-norm-tag-tag-420051".equals(caseId))
                .forEach(caseIds::add);
        for (JsonNode testCase : readRouteEvidenceCatalog().path("test_cases")) {
            if ("referenced_enum_value".equals(testCase.path("requires").path("target_kind").asText())) {
                caseIds.add(testCase.path("case_id").asText());
            }
        }
        caseIds.add("test-enum-pkcs#11-function");
        caseIds.add("test-enum-pkcs#11-return-code");
        CREDENTIAL_TYPE_MODEL_CASE_IDS.values().stream().sorted().forEach(caseIds::add);
        report.putArray("protocol_versions").add("2.0").add("2.1");
        ObjectNode combination = mapper.createObjectNode();
        combination.put("encoding", "TTLV");
        combination.put("transport", "raw_tls");
        report.putArray("network_combinations").add(combination);
        report.set("events", events);
        Files.createDirectories(reportPath.toAbsolutePath().normalize().getParent());
        mapper.writerWithDefaultPrettyPrinter().writeValue(reportPath.toFile(), report);
    }

    private static JsonNode findEvent(JsonNode events, String action, String protocolVersion) {
        for (JsonNode event : events) {
            if (action.equals(event.path("action").asText())
                    && protocolVersion.equals(event.path("protocol_version").asText())
                    && "test-op-query".equals(event.path("case_id").asText())
                    && event.hasNonNull("execution_api_symbol")) return event;
        }
        throw new AssertionError("The expected controlled Query route event is absent.");
    }

    private static void appendLocalCatalogInvocation(ArrayNode events, String version,
            LocalCatalogRouteChecks.Invocation invocation) throws Exception {
        if (invocation.rejectionCode() != null) {
            ObjectNode event = newRouteEvent(events, invocation.caseId(), invocation.routeId(),
                    invocation.targetId(), invocation.symbol(), null, null, invocation.action(), version, null);
            event.put("decode_outcome", "rejected");
            event.put("decode_rejection_code", invocation.rejectionCode());
            event.put("decode_input_sha256", sha256(invocation.wire()));
            return;
        }
        String treeHash = normalizedTreeSha256(invocation.wire());
        boolean construction = "construct".equals(invocation.action());
        appendRouteEvent(events, invocation.caseId(), invocation.routeId(), invocation.targetId(),
                invocation.symbol(), null, null, invocation.action(), version, null,
                construction ? treeHash : null, construction ? null : treeHash);
    }

    // OASIS KMIP 2.0 §11.54 and KMIP 2.1 §11.56: exact Tag assignments and reservations.
    private static void assertGenericOnlyTagInventoryEvidence(JsonNode events) throws Exception {
        Set<String> selected = genericOnlyTagCaseIds();
        for (JsonNode testCase : readRouteEvidenceCatalog().path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!selected.contains(testCase.path("case_id").asText())) continue;
            var routes = new ArrayList<>(List.of(
                    "java.generic-ttlv.model.org.kmipkit.ttlv.rawtag.fromraw",
                    "java.generic-ttlv.model.org.kmipkit.ttlv.rawtag.checked",
                    "java.generic-ttlv.ttlv_codec.decode"));
            if ("assigned".equals(requires.path("allocation").asText())) {
                routes.add("java.generic-ttlv.ttlv_item.create");
                routes.add("java.generic-ttlv.function.ttlv_item_view_from_item");
            }
            for (JsonNode version : testCase.path("execution").path("protocol_versions")) {
                for (String route : routes) {
                    JsonNode event = findRouteEvent(events, testCase.path("case_id").asText(), route,
                            version.asText(), requires.path("normative_target_id").asText());
                    assertEquals("passed", event.path("status").asText());
                    assertEquals(1, event.path("invocation_count").asInt());
                }
            }
        }
    }

    private static Set<String> genericOnlyTagCaseIds() throws Exception {
        JsonNode manifest = new ObjectMapper().readTree(
                Path.of("../shared/coverage/route-capabilities.json").toFile());
        Set<String> typedTargets = new java.util.HashSet<>();
        for (JsonNode route : manifest.path("routes")) {
            if (!"java".equals(route.path("language").asText())
                    || "generic-ttlv".equals(route.path("api_level").asText())) continue;
            for (JsonNode target : route.path("normative_target_ids")) typedTargets.add(target.asText());
        }
        assertTrue(!typedTargets.contains("*"), "A generic-only phase requires explicit typed route targets.");
        Set<String> selected = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : readRouteEvidenceCatalog().path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if ("tag".equals(requires.path("target_kind").asText())
                    && !typedTargets.contains(requires.path("normative_target_id").asText())) {
                selected.add(testCase.path("case_id").asText());
            }
        }
        assertTrue(!selected.isEmpty(), "The catalog must retain executable generic-only Tag cases.");
        return selected;
    }

    private static Set<String> missingTypedTagIdentityCaseIds(JsonNode events, String version) throws Exception {
        Set<String> typedCaseIds = typedTagIdentityCaseIds();
        Set<String> missing = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : readRouteEvidenceCatalog().path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            String caseId = testCase.path("case_id").asText();
            String targetId = requires.path("normative_target_id").asText();
            if (!"tag".equals(requires.path("target_kind").asText())
                    || !typedCaseIds.contains(caseId)
                    || !appliesToVersion(testCase, version)) continue;
            boolean fromRawPresent = hasRouteEvent(events, caseId,
                    "java.generic-ttlv.model.org.kmipkit.ttlv.rawtag.fromraw", version, targetId);
            boolean checkedPresent = hasRouteEvent(events, caseId,
                    "java.generic-ttlv.model.org.kmipkit.ttlv.rawtag.checked", version, targetId);
            assertEquals(fromRawPresent, checkedPresent,
                    "RawTag.fromRaw and RawTag.checked evidence must be recorded together for " + caseId);
            if (!fromRawPresent) missing.add(caseId);
        }
        return missing;
    }

    private static void assertTypedTagIdentityEvidence(JsonNode events) throws Exception {
        Set<String> typedCaseIds = typedTagIdentityCaseIds();
        for (JsonNode testCase : readRouteEvidenceCatalog().path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            String caseId = testCase.path("case_id").asText();
            String targetId = requires.path("normative_target_id").asText();
            if (!"tag".equals(requires.path("target_kind").asText()) || !typedCaseIds.contains(caseId)) continue;
            for (JsonNode version : testCase.path("execution").path("protocol_versions")) {
                for (String route : List.of(
                        "java.generic-ttlv.model.org.kmipkit.ttlv.rawtag.fromraw",
                        "java.generic-ttlv.model.org.kmipkit.ttlv.rawtag.checked",
                        "java.generic-ttlv.ttlv_item.create",
                        "java.generic-ttlv.function.ttlv_item_view_from_item",
                        "java.generic-ttlv.ttlv_codec.decode",
                        "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                        "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                        "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew")) {
                    JsonNode event = findRouteEvent(events, caseId, route, version.asText(), targetId);
                    assertEquals("passed", event.path("status").asText());
                    assertEquals(1, event.path("invocation_count").asInt());
                }
            }
        }
    }

    private static Set<String> typedTagIdentityCaseIds() throws Exception {
        JsonNode manifest = new ObjectMapper().readTree(
                Path.of("../shared/coverage/route-capabilities.json").toFile());
        Set<String> supportedTypedRoutes = Set.of(
                "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew",
                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem");
        Set<String> selected = new java.util.LinkedHashSet<>();
        for (JsonNode testCase : readRouteEvidenceCatalog().path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"tag".equals(requires.path("target_kind").asText())) continue;
            String targetId = requires.path("normative_target_id").asText();
            int tagCode = Integer.parseUnsignedInt(
                    requires.path("wire_value").asText().replaceFirst("^0[xX]", ""), 16);
            if (standardAttributeForTag(tagCode) == null) continue;

            Set<String> typedRouteIds = new java.util.HashSet<>();
            for (JsonNode route : manifest.path("routes")) {
                if (!"java".equals(route.path("language").asText())
                        || "generic-ttlv".equals(route.path("api_level").asText())
                        || !"local".equals(route.path("mode").asText())
                        || !routeSupportsCategory(route, testCase.path("category").asText())
                        || !routeSupportsVersion(route, testCase)
                        || !routeSupportsTarget(route, targetId)) continue;
                typedRouteIds.add(route.path("route_id").asText());
            }
            if (supportedTypedRoutes.equals(typedRouteIds)) {
                selected.add(testCase.path("case_id").asText());
            }
        }
        assertTrue(!selected.isEmpty(), "The Java typed attribute route fixtures must cover catalogued Tags.");
        return selected;
    }

    private static boolean routeSupportsVersion(JsonNode route, JsonNode testCase) {
        for (JsonNode version : testCase.path("execution").path("protocol_versions")) {
            for (JsonNode supported : route.path("protocol_versions")) {
                if (version.asText().equals(supported.asText())) return true;
            }
        }
        return false;
    }

    private static boolean routeSupportsCategory(JsonNode route, String category) {
        for (JsonNode supported : route.path("case_categories")) {
            if (category.equals(supported.asText())) return true;
        }
        return false;
    }

    private static boolean routeSupportsTarget(JsonNode route, String targetId) {
        for (JsonNode target : route.path("normative_target_ids")) {
            if ("*".equals(target.asText()) || targetId.equals(target.asText())) return true;
        }
        return false;
    }

    private static boolean appliesToVersion(JsonNode testCase, String version) {
        for (JsonNode candidate : testCase.path("execution").path("protocol_versions")) {
            if (version.equals(candidate.asText())) return true;
        }
        return false;
    }

    private static boolean hasRouteEvent(JsonNode events, String caseId, String routeId,
            String version, String targetId) {
        for (JsonNode event : events) {
            if (caseId.equals(event.path("case_id").asText())
                    && routeId.equals(event.path("route_id").asText())
                    && version.equals(event.path("protocol_version").asText())
                    && targetId.equals(event.path("target_id").asText())) return true;
        }
        return false;
    }

    // OASIS KMIP 2.0/2.1 §§11.38-11.39: exact referenced PKCS#11 function/return values.
    private static void assertReferencedPkcs11Evidence(JsonNode events) throws Exception {
        for (JsonNode testCase : readRouteEvidenceCatalog().path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"referenced_enum_value".equals(requires.path("target_kind").asText())) continue;
            String caseId = testCase.path("case_id").asText();
            String targetId = requires.path("normative_target_id").asText();
            for (JsonNode version : testCase.path("execution").path("protocol_versions")) {
                String constructedHash = null;
                for (String route : List.of(
                        "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                        "java.generic-ttlv.ttlv_item.create",
                        "java.generic-ttlv.function.ttlv_item_view_from_item",
                        "java.generic-ttlv.ttlv_codec.decode")) {
                    JsonNode event = findRouteEvent(events, caseId, route, version.asText(), targetId);
                    assertEquals("passed", event.path("status").asText());
                    String hash = event.path("construct".equals(event.path("action").asText())
                            ? "request_tree_sha256" : "response_tree_sha256").asText();
                    if (constructedHash == null) constructedHash = hash;
                    else assertEquals(constructedHash, hash,
                            "All PKCS#11 value routes must preserve the same exact tag and unsigned value.");
                }
            }
        }
    }

    private static long routeEventCount(JsonNode events, String caseId, String routeId) {
        long count = 0;
        for (JsonNode event : events) {
            if (caseId.equals(event.path("case_id").asText())
                    && routeId.equals(event.path("route_id").asText())) count++;
        }
        return count;
    }

    private static List<String> enumerationTargetIds(List<EnumerationFixture> fixtures, String parentName) {
        return fixtures.stream()
                .filter(fixture -> parentName.equals(fixture.parentName()))
                .map(EnumerationFixture::targetId).distinct().sorted().toList();
    }

    private static int expectedEnumerationRouteEvents(EnumerationFixture fixture) {
        int routesPerVersion = 4;
        if (fixture.attribute() != null) routesPerVersion += 3;
        if (CRYPTOGRAPHIC_PARAMETER_BUILDER_ROUTES.containsKey(fixture.parentName())) routesPerVersion++;
        if ("CREDENTIAL-TYPE".equals(fixture.parentName())) {
            routesPerVersion++;
            if (CREDENTIAL_TYPE_MODEL_ROUTES.containsKey(fixture.targetId())) routesPerVersion++;
        }
        return fixture.protocolVersions().size() * routesPerVersion;
    }

    private static List<String> concatRoutes(List<String> routes, String additionalRoute) {
        var combined = new ArrayList<>(routes);
        combined.add(additionalRoute);
        return List.copyOf(combined);
    }

    private static LocalEnumerationScope cryptographicParameterBuilderScope(String caseId,
            List<String> targetIds, String enumType) {
        return new LocalEnumerationScope(caseId, targetIds, KMIP_2_0_AND_2_1,
                List.of(CRYPTOGRAPHIC_PARAMETER_BUILDER_ROUTES.get(enumType)));
    }

    private static LocalEnumerationScope genericEnumerationScope(String caseId,
            List<String> targetIds, List<String> protocolVersions) {
        return new LocalEnumerationScope(caseId, targetIds, protocolVersions, List.of());
    }

    private static JsonNode findRouteEvent(JsonNode events, String routeId, String protocolVersion) {
        for (JsonNode event : events) {
            if (routeId.equals(event.path("route_id").asText())
                    && protocolVersion.equals(event.path("protocol_version").asText())) return event;
        }
        throw new AssertionError("The expected installed Java API route event is absent.");
    }

    private static JsonNode findRouteEvent(JsonNode events, String caseId, String routeId,
            String protocolVersion, String targetId) {
        int matches = 0;
        JsonNode found = null;
        for (JsonNode event : events) {
            if (caseId.equals(event.path("case_id").asText())
                    && routeId.equals(event.path("route_id").asText())
                    && protocolVersion.equals(event.path("protocol_version").asText())
                    && targetId.equals(event.path("target_id").asText())) {
                matches++;
                found = event;
            }
        }
        assertEquals(1, matches, "Each expected route must execute exactly once for its catalog case and version: "
                + caseId + " / " + routeId + " / " + protocolVersion + " / " + targetId);
        return found;
    }

    private static byte[] queryOperationTtlv() throws Exception {
        return item(0x42005c, 5, intBytes(0x18));
    }

    private static String normalizedTreeSha256(byte[] wire) throws Exception {
        StringBuilder normalized = new StringBuilder();
        int next = appendNormalizedItem(wire, 0, normalized);
        if (next != wire.length) throw new IllegalArgumentException("TTLV fixture has trailing bytes");
        return sha256(normalized.toString().getBytes(StandardCharsets.UTF_8));
    }

    private static int appendNormalizedItem(byte[] wire, int offset, StringBuilder normalized) {
        if (offset < 0 || wire.length - offset < 8) {
            throw new IllegalArgumentException("TTLV fixture item header is incomplete");
        }
        int tag = (Byte.toUnsignedInt(wire[offset]) << 16)
                | (Byte.toUnsignedInt(wire[offset + 1]) << 8)
                | Byte.toUnsignedInt(wire[offset + 2]);
        int type = Byte.toUnsignedInt(wire[offset + 3]);
        int length = ByteBuffer.wrap(wire, offset + 4, 4).getInt();
        if (length < 0) throw new IllegalArgumentException("TTLV fixture item length is invalid");
        int padded = Math.addExact(length, 7) & ~7;
        int valueOffset = Math.addExact(offset, 8);
        int valueEnd = Math.addExact(valueOffset, length);
        int itemEnd = Math.addExact(valueOffset, padded);
        if (itemEnd > wire.length || valueEnd > itemEnd) {
            throw new IllegalArgumentException("TTLV fixture item is incomplete");
        }
        if (tag != 0x420092) {
            normalized.append(String.format(java.util.Locale.ROOT, "%06x:%02x:", tag, type));
            if (type == 1) {
                int child = valueOffset;
                while (child < valueEnd) child = appendNormalizedItem(wire, child, normalized);
            } else {
                normalized.append(HexFormat.of().formatHex(wire, valueOffset, valueEnd));
            }
            normalized.append(';');
        }
        return itemEnd;
    }

    private static String sha256(String value) throws Exception {
        return sha256(value.getBytes(StandardCharsets.UTF_8));
    }

    private static String sha256(byte[] value) throws Exception {
        return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(value));
    }

    private static int requestProtocolMinorVersion(byte[] request) {
        byte[] minorHeader = new byte[] {0x42, 0x00, 0x6b, 2, 0, 0, 0, 4};
        for (int offset = 0; offset <= request.length - minorHeader.length - 4; offset++) {
            boolean matches = true;
            for (int index = 0; index < minorHeader.length; index++) {
                if (request[offset + index] != minorHeader[index]) {
                    matches = false;
                    break;
                }
            }
            if (matches) return ByteBuffer.wrap(request, offset + minorHeader.length, 4).getInt();
        }
        throw new AssertionError("The outgoing KMIP request omitted ProtocolVersionMinor.");
    }

    private static byte[] unsupportedResponse(long operation) throws Exception {
        return unsupportedResponse(operation, 1);
    }

    private static byte[] unsupportedResponse(long operation, int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(Math.toIntExact(operation))),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(1)),
                item(0x42007e, 5, intBytes(Math.toIntExact(KmipValues.ResultReason.OPERATION_NOT_SUPPORTED))));
        return structure(0x42007b, header, responseItem);
    }

    private static List<byte[]> serveAsynchronousResponses(SSLServerSocket listener, int count) throws Exception {
        return serveAsynchronousResponses(listener, count, 1);
    }

    private static List<byte[]> serveAsynchronousResponses(SSLServerSocket listener, int count,
            int protocolMinorVersion) throws Exception {
        var requests = new ArrayList<byte[]>();
        for (int index = 0; index < count; index++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readTtlv(socket.getInputStream());
                requests.add(request);
                socket.getOutputStream().write(asynchronousResponse(requestOperation(request),
                        requestCorrelation(request), protocolMinorVersion));
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(requests);
    }

    private static boolean acceptsConnection(SSLServerSocket listener) throws Exception {
        try (SSLSocket ignored = (SSLSocket) listener.accept()) {
            return true;
        } catch (java.net.SocketTimeoutException timeout) {
            return false;
        }
    }

    private static List<CatalogLoader.CatalogCase> casesById(CatalogLoader.Catalog catalog, String... caseIds) {
        return java.util.Arrays.stream(caseIds)
                .map(caseId -> catalog.cases().stream().filter(testCase -> testCase.caseId().equals(caseId))
                        .findFirst().orElseThrow())
                .toList();
    }

    private static byte[] asynchronousResponse(long operation, byte[] correlation, int protocolMinorVersion)
            throws Exception {
        byte[] version = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, version,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        List<byte[]> fields = new ArrayList<>();
        fields.add(item(0x42005c, 5, intBytes(Math.toIntExact(operation))));
        fields.add(item(0x42007f, 5, intBytes(operation == 0x19 ? 0 : 2)));
        if (operation == 0x19) {
            fields.add(structure(0x42007c, item(0x420006, 8, correlation),
                    item(0x420012, 5, intBytes(1))));
        } else {
            fields.add(item(0x420006, 8, correlation));
            if (operation != 0x1a) fields.add(structure(0x42007c));
        }
        return structure(0x42007b, header, structure(0x42000f, fields.toArray(byte[][]::new)));
    }

    private static byte[] requestCorrelation(byte[] request) {
        for (int offset = 0; offset <= request.length - 8; offset++) {
            if (request[offset] == 0x42 && request[offset + 1] == 0x00 && request[offset + 2] == 0x06
                    && request[offset + 3] == 8) {
                int length = ByteBuffer.wrap(request, offset + 4, 4).getInt();
                if (length < 1 || length > 1024 || offset + 8 + length > request.length) break;
                return java.util.Arrays.copyOfRange(request, offset + 8, offset + 8 + length);
            }
        }
        throw new AssertionError("The asynchronous request omitted its correlation value.");
    }

    private static byte[] readTtlv(InputStream input) throws Exception {
        DataInputStream data = new DataInputStream(input);
        byte[] header = data.readNBytes(8);
        if (header.length != 8) throw new IllegalStateException("TTLV request header is incomplete");
        long payloadLength = Integer.toUnsignedLong(ByteBuffer.wrap(header, 4, 4).getInt());
        if (payloadLength > 16 * 1024 * 1024L) throw new IllegalStateException("TTLV request is oversized");
        int paddedLength = Math.toIntExact((payloadLength + 7) & ~7L);
        byte[] payload = data.readNBytes(paddedLength);
        if (payload.length != paddedLength) throw new IllegalStateException("TTLV request is incomplete");
        byte[] frame = new byte[header.length + payload.length];
        System.arraycopy(header, 0, frame, 0, header.length);
        System.arraycopy(payload, 0, frame, header.length, payload.length);
        return frame;
    }

    private static byte[] discoverVersionsResponse() throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] payload = structure(0x42007c, protocolVersion);
        byte[] responseItem = structure(0x42000f, item(0x42005c, 5, intBytes(30)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] queryResponse() throws Exception {
        return queryResponse(1);
    }

    private static byte[] queryResponse(int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        return structure(0x42007b, header, queryResponseBatchItem());
    }

    private static byte[] queryResponseBatchItem() throws Exception {
        return structure(0x42000f,
                item(0x42005c, 5, intBytes(0x18)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)),
                structure(0x42007c,
                        item(0x42005c, 5, intBytes(0x3b)),
                        profileInformationWire(),
                        structure(0x42015f)),
                messageExtensionWire());
    }

    private static byte[] profileInformationWire() throws Exception {
        return structure(0x4200eb,
                item(0x4200ec, TtlvItemType.Enumeration.code(), intBytes(0x12a)),
                profileVersionWire(),
                textItem(0x4200ed, "https://kms.example.invalid"),
                item(0x4200ee, TtlvItemType.Integer.code(), intBytes(5696)));
    }

    private static byte[] profileVersionWire() throws Exception {
        return structure(0x4200ef,
                item(0x4200f0, TtlvItemType.Integer.code(), intBytes(1)),
                item(0x4200f1, TtlvItemType.Integer.code(), intBytes(0)));
    }

    private static byte[] messageExtensionWire() throws Exception {
        return structure(0x420051,
                textItem(0x42009d, "KMIPKit.fixture"),
                item(0x420026, 6, new byte[8]),
                structure(0x42009c, textItem(0x540001, "fixture-extension")));
    }

    private void appendPendingQueryCorrelationEvidence(ArrayNode events, String protocolVersion,
            byte[] serverCertificatePem, byte[] serverPrivateKeyPem,
            byte[] clientCertificatePem, byte[] clientPrivateKeyPem) throws Exception {
        byte[] correlation = {0x50, 0x45, 0x4e, 0x44, (byte) protocolVersion.charAt(2)};
        int protocolMinorVersion = "2.0".equals(protocolVersion) ? 0 : 1;
        SSLServerSocket listener = listener(certificate(serverCertificatePem),
                privateKey(serverPrivateKeyPem), certificate(clientCertificatePem));
        byte[] responseBytes = asynchronousResponse(0x18, correlation, protocolMinorVersion);
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, responseBytes));
        Thread peerThread = new Thread(peer, "java-validator-pending-query-peer-" + protocolVersion);
        peerThread.setDaemon(true);
        peerThread.start();
        Path connectionFile = temporaryDirectory.resolve("pending-query-evidence-" + protocolVersion + ".json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true, protocolVersion));
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile));
                ClientBatch batch = ClientBatch.create()) {
            batch.asynchronousIndicator(1);
            ClientBatchItem request = ClientBatchItem.query(new int[] {1}, null);
            batch.add(request);
            try (ClientBatchResponse response = driver.executeBatchForEvidence(batch);
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(2L, result.resultStatus());
                assertArrayEquals(correlation, result.pendingCorrelationValue().orElseThrow());
                appendRouteEvent(events, RESPONSE_BATCH_ITEM_CASE_ID,
                        "java.typed-protocol.function.client_batch_item_response_pending_correlation_value",
                        RESPONSE_BATCH_ITEM_TARGET_ID,
                        "org.kmipkit.extensions.ClientBatchItemResult.pendingCorrelationValue",
                        null, null, "decode", protocolVersion, null, null,
                        normalizedTreeSha256(responseBytes));
                assertArrayEquals(correlation, result.pendingCorrelationValue().orElseThrow());
                appendRouteEvent(events,
                        "test-norm-operation-structure-operation-structure-7-8-correlation-value",
                        "java.typed-protocol.function.client_batch_item_response_pending_correlation_value",
                        "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE",
                        "org.kmipkit.extensions.ClientBatchItemResult.pendingCorrelationValue",
                        null, null, "decode", protocolVersion, null, null,
                        normalizedTreeSha256(responseBytes));
                appendBatchItemFieldRouteEvents(events,
                        "java.typed-protocol.function.client_batch_item_response_pending_correlation_value",
                        "org.kmipkit.extensions.ClientBatchItemResult.pendingCorrelationValue",
                        protocolVersion, normalizedTreeSha256(responseBytes));
            }
        } finally {
            listener.close();
        }
        byte[] requestBytes = peer.get(15, TimeUnit.SECONDS);
        assertEquals(0x18L, requestOperation(requestBytes));
        assertEquals(protocolMinorVersion, requestProtocolMinorVersion(requestBytes));
    }

    private static void appendBatchItemGenericRouteEvents(ArrayNode events, String protocolVersion)
            throws Exception {
        String treeHash = normalizedTreeSha256(queryResponseBatchItem());
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvStructure structure = batchItemStructure();
                TtlvValue value = TtlvValue.structure(structure, limits);
                RawTag rawTag = RawTag.fromRaw(0x42000f);
                Tag tag = rawTag.checked();
                TtlvItem item = TtlvItem.create(tag, value, limits);
                var view = item.view();
                var itemValue = view.value();
                var structureView = itemValue.structure()) {
            assertEquals(0x42000f, view.tag().raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertEquals(5, structureView.itemCount());
        }
        appendRouteEvent(events, RESPONSE_BATCH_ITEM_CASE_ID,
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure",
                RESPONSE_BATCH_ITEM_TARGET_ID, "org.kmipkit.ttlv.TtlvValue.structure", null, null,
                "construct", protocolVersion, null, treeHash, null);
        appendRouteEvent(events, RESPONSE_BATCH_ITEM_CASE_ID, "java.generic-ttlv.ttlv_item.create",
                RESPONSE_BATCH_ITEM_TARGET_ID, "org.kmipkit.ttlv.TtlvItem.create", null, null,
                "construct", protocolVersion, null, treeHash, null);
        appendRouteEvent(events, RESPONSE_BATCH_ITEM_CASE_ID,
                "java.generic-ttlv.function.ttlv_item_view_from_item", RESPONSE_BATCH_ITEM_TARGET_ID,
                "org.kmipkit.ttlv.TtlvItem.view", null, null, "decode", protocolVersion,
                null, null, treeHash);
        try (TtlvItem decoded = TtlvCodec.decode(queryResponseBatchItem());
                var view = decoded.view();
                var value = view.value();
                var structureView = value.structure()) {
            assertEquals(0x42000f, view.tag().raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertEquals(5, structureView.itemCount());
        }
        appendRouteEvent(events, RESPONSE_BATCH_ITEM_CASE_ID, "java.generic-ttlv.ttlv_codec.decode",
                RESPONSE_BATCH_ITEM_TARGET_ID, "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null, null, treeHash);
    }

    private static void appendNormativeMessageFieldGenericRouteEvents(ArrayNode events,
            String protocolVersion) throws Exception {
        // OASIS KMIP 2.0/2.1 §8.3 defines the optional Boolean Ephemeral field in a Request Batch Item.
        appendMessageFieldBooleanRoutes(events, "test-norm-message-field-message-field-8-3-ephemeral",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-3-EPHEMERAL", 0x420154, protocolVersion);
        appendMessageFieldIntegerRoutes(events, "test-norm-message-field-message-field-8-5-batch-count",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-BATCH-COUNT", protocolVersion, false);
        appendMessageFieldIntegerRoutes(events, "test-norm-message-field-message-field-9-5-batch-count",
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT", protocolVersion, true);

        appendMessageFieldStructureRoutes(events, "test-norm-message-field-message-field-8-6-batch-item",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM", 0x42000f,
                queryResponseBatchItem(), batchItemStructure(), protocolVersion, true, false);
        appendMessageFieldStructureRoutes(events, "test-norm-message-field-message-field-9-7-batch-item",
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM", 0x42000f,
                queryResponseBatchItem(), batchItemStructure(), protocolVersion, true, false);
        appendMessageFieldStructureRoutes(events, "test-norm-message-field-message-field-8-6-message-extension",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-MESSAGE-EXTENSION", 0x420051,
                messageExtensionWire(), messageExtensionStructure(), protocolVersion, false, false);
        appendMessageFieldStructureRoutes(events, "test-norm-message-field-message-field-9-13-message-extension",
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION", 0x420051,
                messageExtensionWire(), messageExtensionStructure(), protocolVersion, true, false);
        appendMessageFieldStructureRoutes(events, "test-norm-tag-tag-420051",
                "KMIPKIT-ELEM-TAG-420051", 0x420051,
                messageExtensionWire(), messageExtensionStructure(), protocolVersion, false, true);
    }

    private static void appendMessageFieldBooleanRoutes(ArrayNode events, String caseId, String targetId,
            int rawTag, String protocolVersion) throws Exception {
        byte[] wire = item(rawTag, TtlvItemType.Boolean.code(), longBytes(1));
        String treeHash = normalizedTreeSha256(wire);
        try (TtlvValue value = TtlvValue.Boolean(true);
                RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                TtlvItem created = TtlvItem.create(tag, value, CodecLimits.defaults());
                var view = created.view();
                var viewValue = view.value()) {
            assertEquals(rawTag, view.tag().raw());
            assertEquals(TtlvItemType.Boolean, view.itemType());
            assertTrue(viewValue.booleanValue());
        }
        appendRouteEvent(events, caseId, "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.boolean",
                targetId, "org.kmipkit.ttlv.TtlvValue.Boolean", null, null,
                "construct", protocolVersion, null, treeHash, null);
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_item.create", targetId,
                "org.kmipkit.ttlv.TtlvItem.create", null, null,
                "construct", protocolVersion, null, treeHash, null);
        appendRouteEvent(events, caseId, "java.generic-ttlv.function.ttlv_item_view_from_item", targetId,
                "org.kmipkit.ttlv.TtlvItem.view", null, null,
                "decode", protocolVersion, null, null, treeHash);
        try (TtlvItem decoded = TtlvCodec.decode(wire);
                var decodedView = decoded.view();
                var decodedValue = decodedView.value()) {
            assertEquals(rawTag, decodedView.tag().raw());
            assertEquals(TtlvItemType.Boolean, decodedView.itemType());
            assertTrue(decodedValue.booleanValue());
        }
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_codec.decode", targetId,
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null, null, treeHash);
    }

    // OASIS KMIP 2.0/2.1 §5.7 Tables 148/163 define New Attribute as a Structure containing one
    // direct attribute Item. The operation fixtures follow §6.1.2 Tables 152/167, §6.1.33 Table 247 /
    // §6.1.34 Table 265, and §6.1.47 Table 296 / §6.1.51 Table 322, respectively.
    private void appendNewAttributeMutationEvents(ArrayNode events, String protocolVersion,
            byte[] serverCertificatePem, byte[] serverPrivateKeyPem,
            byte[] clientCertificatePem, byte[] clientPrivateKeyPem) throws Exception {
        int minor = "2.0".equals(protocolVersion) ? 0 : 1;
        for (NewAttributeMutationFixture fixture : NEW_ATTRIBUTE_MUTATION_FIXTURES) {
            String operation = fixture.methodName();
            String title = fixture.title();
            String suffix = title.toLowerCase(Locale.ROOT).replace(' ', '-') + "-request";
            String versionCode = "2.0".equals(protocolVersion) ? "20" : "21";
            String ordinal = fixture.ordinal();
            String routeOperation = title.toLowerCase(Locale.ROOT).replace(' ', '_');
            String caseId = "test-norm-payload-client-" + suffix + "-" + versionCode + "-" + ordinal;
            String payloadTarget = "KMIPKIT-PAYLOAD-CLIENT-" + suffix.toUpperCase(Locale.ROOT)
                    + "-" + versionCode + "-" + ordinal;
            String operationTarget = "KMIPKIT-ELEM-OP-C2S-" + title.toUpperCase(Locale.ROOT)
                    .replace(' ', '-');
            String clientItemRoute = "java.typed-protocol.client_batch_item." + routeOperation;
            String clientItemSymbol = "org.kmipkit.extensions.ClientBatchItem." + operation;
            String toBatchItemRoute = "java.typed-protocol.function.attribute_request_" + routeOperation
                    + "_to_batch_item";
            String requestClass = title.replace(" ", "") + "Request";
            String toBatchItemSymbol = "org.kmipkit.extensions." + requestClass + ".toBatchItem";
            String toPayloadRoute = "java.typed-protocol.function.attribute_request_" + routeOperation
                    + "_to_ttlv_payload";
            String toPayloadSymbol = "org.kmipkit.extensions." + requestClass + ".toTtlvPayload";
            String responseRoute = "java.typed-protocol.function.client_batch_item_response_" + routeOperation;
            String responseSymbol = "org.kmipkit.extensions.ClientBatchItemResult." + operation + "Response";
            int operationValue = fixture.operationValue();

            byte[] newAttributeWire = newAttributeStructureWire();
            String newAttributeHash = normalizedTreeSha256(newAttributeWire);
            appendNewAttributeGenericRouteEvents(events, caseId, payloadTarget, operationTarget,
                    protocolVersion, newAttributeWire, newAttributeHash);

            SSLServerSocket listener = listener(certificate(serverCertificatePem),
                    privateKey(serverPrivateKeyPem), certificate(clientCertificatePem));
            byte[] responseBytes = attributeMutationResponse(minor, operationValue);
            FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, responseBytes));
            Thread peerThread = new Thread(peer,
                    "java-validator-" + operation + "-new-attribute-peer-" + protocolVersion);
            peerThread.setDaemon(true);
            peerThread.start();

            Path connectionFile = temporaryDirectory.resolve(operation + "-new-attribute-"
                    + protocolVersion + ".json");
            Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                    clientCertificatePem, clientPrivateKeyPem, false, protocolVersion));
            try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile));
                    TtlvItem newAttributeItem = TtlvCodec.decode(newAttributeNameItemWire());
                    TtlvStructure genericPayload = attributeRequestPayload(operation, newAttributeItem);
                    ClientBatchItem directRequest = directAttributeMutationItem(operation, newAttributeItem)
                            .withUniqueBatchItemId("item-0".getBytes(StandardCharsets.US_ASCII));
                    ClientBatchItem typedRequest = attributeRequestBatchItem(operation, newAttributeItem)
                            .withUniqueBatchItemId("item-1".getBytes(StandardCharsets.US_ASCII));
                    ClientBatch batch = ClientBatch.create()) {
                try (var payloadView = genericPayload.view();
                        var newAttributeField = payloadView.itemAt(1);
                        var fieldTag = newAttributeField.tag();
                        var fieldValue = newAttributeField.value();
                        var fieldContents = fieldValue.structure()) {
                    assertEquals(2, payloadView.itemCount());
                    assertEquals(0x42013d, fieldTag.raw());
                    assertEquals(TtlvItemType.Structure, newAttributeField.itemType());
                    assertEquals(1, fieldContents.itemCount(),
                            "The typed payload must preserve its direct Name attribute Item.");
                }
                batch.add(directRequest);
                batch.add(typedRequest);
                String responseHash = normalizedTreeSha256(responseBytes);
                try (ClientBatchResponse response = driver.executeBatchForEvidence(batch)) {
                    assertEquals(2, response.itemCount());
                    for (int index = 0; index < response.itemCount(); index++) {
                        try (ClientBatchItemResult result = response.itemAt(index)) {
                            assertEquals((long) operationValue, result.operation());
                            assertEquals(0L, result.resultStatus());
                            String uniqueIdentifier = switch (operation) {
                                case "addAttribute" -> result.addAttributeResponse().orElseThrow()
                                        .uniqueIdentifier();
                                case "modifyAttribute" -> result.modifyAttributeResponse().orElseThrow()
                                        .uniqueIdentifier();
                                case "setAttribute" -> result.setAttributeResponse().orElseThrow()
                                        .uniqueIdentifier();
                                default -> throw new AssertionError("Unknown attribute mutation fixture.");
                            };
                            assertEquals("object-7", uniqueIdentifier);
                        }
                    }
                }
                byte[] requestBytes = peer.get(15, TimeUnit.SECONDS);
                assertEquals(operationValue, requestOperation(requestBytes));
                assertEquals(minor, requestProtocolMinorVersion(requestBytes));
                assertTrue(contains(requestBytes, newAttributeWire),
                        "The controlled peer must receive the exact New Attribute Structure.");
                String requestHash = normalizedTreeSha256(requestBytes);
                appendNewAttributeTypedMutationRouteEvents(events, caseId, payloadTarget, operationTarget,
                        clientItemRoute, clientItemSymbol, toBatchItemRoute, toBatchItemSymbol,
                        protocolVersion, requestHash, responseHash);
                appendRouteEvent(events, caseId, toPayloadRoute, payloadTarget, toPayloadSymbol,
                        null, null, "encode", protocolVersion, null, requestHash, null);
                appendRouteEvent(events, caseId, toPayloadRoute, operationTarget, toPayloadSymbol,
                        null, null, "encode", protocolVersion, null, requestHash, null);
                appendRouteEvent(events, caseId, responseRoute, operationTarget, responseSymbol,
                        null, null, "decode", protocolVersion, null, null, responseHash);
            } finally {
                listener.close();
            }
        }
    }

    private static void appendNewAttributeGenericRouteEvents(ArrayNode events, String caseId,
            String payloadTarget, String operationTarget, String protocolVersion,
            byte[] newAttributeWire, String treeHash) throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvStructure empty = TtlvStructure.create();
                TtlvItem directAttribute = TtlvCodec.decode(newAttributeNameItemWire());
                TtlvStructure content = TtlvStructure.withItem(empty, directAttribute, limits);
                TtlvValue structureValue = TtlvValue.structure(content, limits);
                RawTag raw = RawTag.fromRaw(0x42013d);
                Tag tag = raw.checked();
                TtlvItem item = TtlvItem.create(tag, structureValue, limits);
                var view = item.view();
                var value = view.value();
                var fields = value.structure()) {
            assertEquals(0x42013d, view.tag().raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertEquals(1, fields.itemCount(),
                    "New Attribute is a Structure containing exactly one direct attribute Item.");
        }
        appendRouteEvent(events, caseId,
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure", payloadTarget,
                "org.kmipkit.ttlv.TtlvValue.structure", null, null,
                "construct", protocolVersion, null, treeHash, null);
        for (String target : List.of(payloadTarget, operationTarget)) {
            appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_item.create", target,
                    "org.kmipkit.ttlv.TtlvItem.create", null, null,
                    "construct", protocolVersion, null, treeHash, null);
            appendRouteEvent(events, caseId, "java.generic-ttlv.function.ttlv_item_view_from_item", target,
                    "org.kmipkit.ttlv.TtlvItem.view", null, null,
                    "decode", protocolVersion, null, null, treeHash);
            try (TtlvItem decoded = TtlvCodec.decode(newAttributeWire);
                    var decodedView = decoded.view();
                    var decodedValue = decodedView.value();
                    var decodedFields = decodedValue.structure()) {
                assertEquals(0x42013d, decodedView.tag().raw());
                assertEquals(TtlvItemType.Structure, decodedView.itemType());
                assertEquals(1, decodedFields.itemCount());
            }
            appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_codec.decode", target,
                    "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                    "decode", protocolVersion, null, null, treeHash);
        }
    }

    private static void appendNewAttributeTypedMutationRouteEvents(ArrayNode events, String caseId,
            String payloadTarget, String operationTarget, String clientItemRoute, String clientItemSymbol,
            String toBatchItemRoute, String toBatchItemSymbol, String protocolVersion,
            String requestHash, String responseHash) {
        for (String target : List.of(payloadTarget, operationTarget)) {
            appendRouteEvent(events, caseId, clientItemRoute, target, clientItemSymbol,
                    "java.execute.typed-batch", "org.kmipkit.extensions.Client.execute",
                    "construct", protocolVersion, "raw_tls", requestHash, null);
            ObjectNode clientSent = appendRouteEvent(events, caseId, clientItemRoute, target, clientItemSymbol,
                    "java.execute.typed-batch", "org.kmipkit.extensions.Client.execute",
                    "send", protocolVersion, "raw_tls", requestHash, null);
            clientSent.put("request_delivery_state", "response_received");
            clientSent.put("transport_invocation_count", 1);
            appendRouteEvent(events, caseId, toBatchItemRoute, target, toBatchItemSymbol,
                    "java.execute.typed-batch", "org.kmipkit.extensions.Client.execute",
                    "construct", protocolVersion, "raw_tls", requestHash, null);
            ObjectNode convertedSent = appendRouteEvent(events, caseId, toBatchItemRoute, target,
                    toBatchItemSymbol, "java.execute.typed-batch", "org.kmipkit.extensions.Client.execute",
                    "send", protocolVersion, "raw_tls", requestHash, null);
            convertedSent.put("request_delivery_state", "response_received");
            convertedSent.put("transport_invocation_count", 1);
        }
        appendRouteEvent(events, caseId, clientItemRoute, operationTarget, clientItemSymbol,
                "java.execute.typed-batch", "org.kmipkit.extensions.Client.execute",
                "decode", protocolVersion, "raw_tls", null, responseHash);
        appendRouteEvent(events, caseId, toBatchItemRoute, operationTarget, toBatchItemSymbol,
                "java.execute.typed-batch", "org.kmipkit.extensions.Client.execute",
                "decode", protocolVersion, "raw_tls", null, responseHash);
    }

    private static ClientBatchItem directAttributeMutationItem(String operation, TtlvItem newAttribute) {
        UniqueIdentifier identifier = UniqueIdentifier.textString("object-7");
        return switch (operation) {
            case "addAttribute" -> ClientBatchItem.addAttribute(identifier, newAttribute);
            case "modifyAttribute" -> ClientBatchItem.modifyAttribute(identifier, null, newAttribute);
            case "setAttribute" -> ClientBatchItem.setAttribute(identifier, newAttribute);
            default -> throw new AssertionError("Unknown attribute mutation fixture.");
        };
    }

    private static TtlvStructure attributeRequestPayload(String operation, TtlvItem newAttribute) {
        UniqueIdentifier identifier = UniqueIdentifier.textString("object-7");
        return switch (operation) {
            case "addAttribute" -> {
                try (AddAttributeRequest request = AddAttributeRequest.create(identifier, newAttribute)) {
                    yield request.toTtlvPayload();
                }
            }
            case "modifyAttribute" -> {
                try (ModifyAttributeRequest request =
                        ModifyAttributeRequest.create(identifier, null, newAttribute)) {
                    yield request.toTtlvPayload();
                }
            }
            case "setAttribute" -> {
                try (SetAttributeRequest request = SetAttributeRequest.create(identifier, newAttribute)) {
                    yield request.toTtlvPayload();
                }
            }
            default -> throw new AssertionError("Unknown attribute mutation fixture.");
        };
    }

    private static ClientBatchItem attributeRequestBatchItem(String operation, TtlvItem newAttribute) {
        UniqueIdentifier identifier = UniqueIdentifier.textString("object-7");
        return switch (operation) {
            case "addAttribute" -> {
                try (AddAttributeRequest request = AddAttributeRequest.create(identifier, newAttribute)) {
                    yield request.toBatchItem();
                }
            }
            case "modifyAttribute" -> {
                try (ModifyAttributeRequest request =
                        ModifyAttributeRequest.create(identifier, null, newAttribute)) {
                    yield request.toBatchItem();
                }
            }
            case "setAttribute" -> {
                try (SetAttributeRequest request = SetAttributeRequest.create(identifier, newAttribute)) {
                    yield request.toBatchItem();
                }
            }
            default -> throw new AssertionError("Unknown attribute mutation fixture.");
        };
    }

    private static byte[] newAttributeNameItemWire() throws Exception {
        return structure(0x42000a,
                textItem(0x4200a5, "new name"), item(0x4200a6, 5, intBytes(1)));
    }

    private static byte[] newAttributeStructureWire() throws Exception {
        return structure(0x42013d, newAttributeNameItemWire());
    }

    private static byte[] attributeMutationResponse(int protocolMinorVersion, int operation)
            throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(2)));
        byte[] firstPayload = structure(0x42007c, textItem(0x420094, "object-7"));
        byte[] secondPayload = structure(0x42007c, textItem(0x420094, "object-7"));
        byte[] firstItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(operation)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), firstPayload);
        byte[] secondItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(operation)),
                item(0x420093, 8, "item-1".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), secondPayload);
        return structure(0x42007b, header, firstItem, secondItem);
    }

    private static void assertNewlyCataloguedLocalRoutesAreInvoked(ArrayNode events,
            String protocolVersion) {
        for (ScalarOperationStructureFixture fixture : SCALAR_OPERATION_STRUCTURE_FIXTURES) {
            assertScalarOperationStructureRouteEvents(events, fixture, protocolVersion);
        }
        // OASIS KMIP 2.0 §7.32 and KMIP 2.1 §7.38: Signature Data is a Byte String.
        assertScalarOperationStructureRouteEvents(events, SIGNATURE_DATA_FIXTURE, protocolVersion);
        // OASIS KMIP 2.0 §7.9 / §11.54 and KMIP 2.1 §7.13 / §11.56.
        assertGenericStructureRouteEvents(events,
                "test-norm-operation-structure-operation-structure-7-13-extension-information",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-13-EXTENSION-INFORMATION", protocolVersion);
        assertRouteEventRecorded(events,
                "test-norm-operation-structure-operation-structure-7-13-extension-information",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-13-EXTENSION-INFORMATION",
                "java.typed-protocol.function.extension_information_to_ttlv", "encode", protocolVersion);
        assertRouteEventRecorded(events,
                "test-norm-operation-structure-operation-structure-7-8-correlation-value",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE",
                "java.typed-protocol.function.client_batch_item_response_pending_correlation_value",
                "decode", protocolVersion);
        // OASIS KMIP 2.0 §§7.25–7.26, Tables 345–346; KMIP 2.1 §§7.31–7.32, Tables 383–384.
        assertGenericStructureRouteEvents(events, PROFILE_INFORMATION_CASE_ID,
                PROFILE_INFORMATION_TARGET_ID, protocolVersion);
        assertGenericStructureRouteEvents(events, PROFILE_VERSION_CASE_ID,
                PROFILE_VERSION_TARGET_ID, protocolVersion);
        for (ProfileInformationMemberFixture fixture : PROFILE_INFORMATION_MEMBER_FIXTURES) {
            String valueRoute = fixture.structure()
                    ? "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure"
                    : switch (fixture.itemType()) {
                        case Integer -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.integer";
                        case TextString -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring";
                        default -> throw new IllegalArgumentException("Unsupported Profile Information member type");
                    };
            for (String route : List.of(valueRoute, "java.generic-ttlv.ttlv_item.create",
                    "java.generic-ttlv.function.ttlv_item_view_from_item", "java.generic-ttlv.ttlv_codec.decode")) {
                String action = route.endsWith("ttlv_item.create") || route.equals(valueRoute)
                        ? "construct" : "decode";
                assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(), route,
                        action, protocolVersion);
            }
        }
        if ("2.1".equals(protocolVersion)) {
            assertGenericStructureRouteEvents(events,
                    "test-norm-operation-structure-operation-structure-7-2-asynchronous-request",
                    "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-2-ASYNCHRONOUS-REQUEST", protocolVersion);
            for (OperationStructureFixture fixture : OPERATION_STRUCTURE_FIXTURES) {
                assertGenericStructureRouteEvents(events, fixture.caseId(), fixture.targetId(), protocolVersion);
            }
            // OASIS KMIP 2.1 §§7.6–7.7, Tables 357–358.
            assertGenericStructureRouteEvents(events, CONSTRAINT_CASE_ID,
                    CONSTRAINT_TARGET_ID, protocolVersion);
            assertGenericStructureRouteEvents(events, CONSTRAINTS_CASE_ID,
                    CONSTRAINTS_TARGET_ID, protocolVersion);
            // OASIS KMIP 2.1 §7.24 Table 376; Object Type is optional and repeatable.
            for (String route : List.of(
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                    "java.generic-ttlv.ttlv_item.create",
                    "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "java.generic-ttlv.ttlv_codec.decode")) {
                String action = route.endsWith("ttlv_codec.decode")
                        || route.endsWith("ttlv_item_view_from_item") ? "decode" : "construct";
                assertRouteEventRecorded(events, OBJECT_TYPE_MEMBER_CASE_ID, OBJECT_TYPE_MEMBER_TARGET_ID,
                        route, action, protocolVersion);
            }
            String objectGroupCase = "test-norm-structure-member-structure-member-7-23-object-group";
            String objectGroupTarget = "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-23-OBJECT-GROUP";
            assertRouteEventRecorded(events, objectGroupCase, objectGroupTarget,
                    "java.generic-ttlv.ttlv_item.create", "construct", protocolVersion);
            assertRouteEventRecorded(events, objectGroupCase, objectGroupTarget,
                    "java.generic-ttlv.function.ttlv_item_view_from_item", "decode", protocolVersion);
            assertRouteEventRecorded(events, objectGroupCase, objectGroupTarget,
                    "java.generic-ttlv.ttlv_codec.decode", "decode", protocolVersion);

            for (StructureMemberStructureFixture fixture : STRUCTURE_MEMBER_STRUCTURE_FIXTURES) {
                assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(),
                        "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure",
                        "construct", protocolVersion);
                assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(),
                        "java.generic-ttlv.ttlv_item.create", "construct", protocolVersion);
                assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(),
                        "java.generic-ttlv.function.ttlv_item_view_from_item", "decode", protocolVersion);
                assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(),
                        "java.generic-ttlv.ttlv_codec.decode", "decode", protocolVersion);
            }
            for (StructureMemberByteStringFixture fixture : STRUCTURE_MEMBER_BYTE_STRING_FIXTURES) {
                for (String route : List.of(
                        "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.bytestring",
                        "java.generic-ttlv.ttlv_item.create",
                        "java.generic-ttlv.function.ttlv_item_view_from_item",
                        "java.generic-ttlv.ttlv_codec.decode")) {
                    String action = route.endsWith("ttlv_item.create") || route.endsWith("ttlvvalue.bytestring")
                            ? "construct" : "decode";
                    assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(), route,
                            action, protocolVersion);
                }
                if (fixture.typedQueryFilter()) {
                    assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(),
                            "java.typed-protocol.model.org.kmipkit.extensions.queryasyncrequestsrequest"
                                    + ".withcorrelationvalues", "construct", protocolVersion);
                }
            }
            for (StructureMemberValueFixture fixture : STRUCTURE_MEMBER_VALUE_FIXTURES) {
                String constructorRoute = fixture.itemType() == TtlvItemType.DateTimeExtended
                        ? "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.datetimeextended"
                        : "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration";
                assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(),
                        constructorRoute, "construct", protocolVersion);
                assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(),
                        "java.generic-ttlv.ttlv_item.create", "construct", protocolVersion);
                assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(),
                        "java.generic-ttlv.function.ttlv_item_view_from_item", "decode", protocolVersion);
                assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(),
                        "java.generic-ttlv.ttlv_codec.decode", "decode", protocolVersion);
            }
            StructureMemberValueFixture operationFixture = structureMemberValueFixture(0x42005c);
            assertRouteEventRecorded(events, operationFixture.caseId(), operationFixture.targetId(),
                    "java.typed-protocol.function.client_async_operation_outcome_response_payload_ttlv",
                    "decode", protocolVersion);
        }

        String ephemeralCase = "test-norm-message-field-message-field-8-3-ephemeral";
        String ephemeralTarget = "KMIPKIT-ELEM-MESSAGE-FIELD-8-3-EPHEMERAL";
        assertRouteEventRecorded(events, ephemeralCase, ephemeralTarget,
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.boolean", "construct", protocolVersion);
        assertRouteEventRecorded(events, ephemeralCase, ephemeralTarget,
                "java.generic-ttlv.ttlv_item.create", "construct", protocolVersion);
        assertRouteEventRecorded(events, ephemeralCase, ephemeralTarget,
                "java.generic-ttlv.function.ttlv_item_view_from_item", "decode", protocolVersion);
        assertRouteEventRecorded(events, ephemeralCase, ephemeralTarget,
                "java.generic-ttlv.ttlv_codec.decode", "decode", protocolVersion);
        assertRouteEventRecorded(events, ephemeralCase, ephemeralTarget,
                "java.typed-protocol.model.org.kmipkit.message.requestmessage.fromttlv",
                "construct", protocolVersion);

        String versionCode = "2.0".equals(protocolVersion) ? "20" : "21";
        for (NewAttributeMutationFixture fixture : NEW_ATTRIBUTE_MUTATION_FIXTURES) {
            String operation = fixture.methodName();
            String title = fixture.title();
            String suffix = title.toLowerCase(Locale.ROOT).replace(' ', '-') + "-request";
            String ordinal = fixture.ordinal();
            String routeOperation = title.toLowerCase(Locale.ROOT).replace(' ', '_');
            String caseId = "test-norm-payload-client-" + suffix + "-" + versionCode + "-" + ordinal;
            String payloadTarget = "KMIPKIT-PAYLOAD-CLIENT-" + suffix.toUpperCase(Locale.ROOT)
                    + "-" + versionCode + "-" + ordinal;
            String operationTarget = "KMIPKIT-ELEM-OP-C2S-" + title.toUpperCase(Locale.ROOT)
                    .replace(' ', '-');
            String clientRoute = "java.typed-protocol.client_batch_item." + routeOperation;
            String batchRoute = "java.typed-protocol.function.attribute_request_" + routeOperation
                    + "_to_batch_item";
            String payloadRoute = "java.typed-protocol.function.attribute_request_" + routeOperation
                    + "_to_ttlv_payload";
            String responseRoute = "java.typed-protocol.function.client_batch_item_response_" + routeOperation;
            assertRouteEventRecorded(events, caseId, payloadTarget,
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure", "construct", protocolVersion);
            for (String target : List.of(payloadTarget, operationTarget)) {
                assertRouteEventRecorded(events, caseId, target,
                        "java.generic-ttlv.ttlv_item.create", "construct", protocolVersion);
                assertRouteEventRecorded(events, caseId, target,
                        "java.generic-ttlv.function.ttlv_item_view_from_item", "decode", protocolVersion);
                assertRouteEventRecorded(events, caseId, target,
                        "java.generic-ttlv.ttlv_codec.decode", "decode", protocolVersion);
                assertRouteEventRecorded(events, caseId, target, clientRoute, "construct", protocolVersion);
                assertRouteEventRecorded(events, caseId, target, clientRoute, "send", protocolVersion);
                assertRouteEventRecorded(events, caseId, target, batchRoute, "construct", protocolVersion);
                assertRouteEventRecorded(events, caseId, target, batchRoute, "send", protocolVersion);
                assertRouteEventRecorded(events, caseId, target, payloadRoute, "encode", protocolVersion);
            }
            for (String route : List.of(clientRoute, batchRoute, responseRoute)) {
                assertRouteEventRecorded(events, caseId, operationTarget, route, "decode", protocolVersion);
            }
        }
    }

    // OASIS KMIP 2.1 §7.23 Table 375: Object Group is an optional, repeatable Attribute.
    private static void appendObjectGroupLocalRouteEvents(ArrayNode events) throws Exception {
        String caseId = "test-norm-structure-member-structure-member-7-23-object-group";
        String targetId = "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-23-OBJECT-GROUP";
        int rawTag = 0x420056;
        String fixture = "kmipkit-object-group";
        byte[] fixtureBytes = fixture.getBytes(StandardCharsets.UTF_8);
        byte[] wire = textItem(rawTag, fixture);
        String treeHash = normalizedTreeSha256(wire);
        CodecLimits limits = CodecLimits.defaults();
        assertObjectGroupsOptionalAndRepeatable(rawTag, fixture);
        try (RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.TextString(fixtureBytes, limits);
                TtlvItem created = TtlvItem.create(tag, value, limits)) {
            assertObjectGroupItem(created, rawTag, fixtureBytes);
        }
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_item.create", targetId,
                "org.kmipkit.ttlv.TtlvItem.create", null, null, "construct", "2.1",
                null, treeHash, null);
        appendRouteEvent(events, caseId, "java.generic-ttlv.function.ttlv_item_view_from_item", targetId,
                "org.kmipkit.ttlv.TtlvItem.view", null, null, "decode", "2.1",
                null, null, treeHash);
        try (TtlvItem decoded = TtlvCodec.decode(wire)) {
            assertObjectGroupItem(decoded, rawTag, fixtureBytes);
        }
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_codec.decode", targetId,
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null, "decode", "2.1",
                null, null, treeHash);
    }

    private static void assertObjectGroupsOptionalAndRepeatable(int objectGroupTag,
            String firstGroupText) throws Exception {
        byte[] firstGroupValue = firstGroupText.getBytes(StandardCharsets.UTF_8);
        byte[] secondGroupValue = "kmipkit-object-group-2".getBytes(StandardCharsets.UTF_8);
        byte[] firstGroup = textItem(objectGroupTag, firstGroupText);
        byte[] secondGroup = textItem(objectGroupTag, "kmipkit-object-group-2");
        try (TtlvItem absentGroups = TtlvCodec.decode(structure(0x420126));
                var absentView = absentGroups.view();
                var absentValue = absentView.value();
                var absentMembers = absentValue.structure();
                TtlvItem repeatedGroups = TtlvCodec.decode(structure(0x420126, firstGroup, secondGroup));
                var repeatedView = repeatedGroups.view();
                var repeatedValue = repeatedView.value();
                var repeatedMembers = repeatedValue.structure()) {
            try (Tag tag = absentView.tag()) {
                assertEquals(0x420126, tag.raw());
            }
            assertEquals(0, absentMembers.itemCount(), "Object Group is optional in Object Groups.");
            assertEquals(2, repeatedMembers.itemCount(), "Object Group may be repeated.");
            try (var first = repeatedMembers.itemAt(0); var second = repeatedMembers.itemAt(1)) {
                assertObjectGroupMember(first, objectGroupTag, firstGroupValue);
                assertObjectGroupMember(second, objectGroupTag, secondGroupValue);
            }
        }
    }

    private static void assertObjectGroupItem(TtlvItem item, int rawTag, byte[] expectedValue) {
        try (var view = item.view(); Tag tag = view.tag(); var value = view.value()) {
            assertEquals(rawTag, tag.raw());
            assertEquals(TtlvItemType.TextString, view.itemType());
            assertArrayEquals(expectedValue, value.bytes());
        }
    }

    private static void assertObjectGroupMember(TtlvItemView member, int rawTag, byte[] expectedValue) {
        try (Tag tag = member.tag(); var value = member.value()) {
            assertEquals(rawTag, tag.raw());
            assertEquals(TtlvItemType.TextString, member.itemType());
            assertArrayEquals(expectedValue, value.bytes());
        }
    }

    // OASIS KMIP 2.1 §§7.6–7.7 Tables 357–358: optional Structure members and a repeatable Constraint.
    private static void appendStructureMemberStructureRouteEvents(ArrayNode events) throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        for (StructureMemberStructureFixture fixture : STRUCTURE_MEMBER_STRUCTURE_FIXTURES) {
            byte[] wire = structure(fixture.rawTag());
            String treeHash = normalizedTreeSha256(wire);
            assertOptionalStructureMemberShape(fixture, wire);
            try (TtlvStructure content = TtlvStructure.create();
                    TtlvValue value = TtlvValue.structure(content, limits);
                    RawTag raw = RawTag.fromRaw(fixture.rawTag());
                    Tag tag = raw.checked();
                    TtlvItem created = TtlvItem.create(tag, value, limits);
                    var view = created.view();
                    Tag viewedTag = view.tag();
                    var viewedValue = view.value();
                    var structureView = viewedValue.structure()) {
                assertEquals(fixture.rawTag(), viewedTag.raw());
                assertEquals(TtlvItemType.Structure, view.itemType());
                assertEquals(0, structureView.itemCount());
            }
            appendRouteEvent(events, fixture.caseId(),
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvValue.structure", null, null, "construct", "2.1",
                    null, treeHash, null);
            appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_item.create", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvItem.create", null, null, "construct", "2.1",
                    null, treeHash, null);
            appendRouteEvent(events, fixture.caseId(),
                    "java.generic-ttlv.function.ttlv_item_view_from_item", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvItem.view", null, null, "decode", "2.1",
                    null, null, treeHash);
            try (TtlvItem decoded = TtlvCodec.decode(wire);
                    var view = decoded.view();
                    Tag tag = view.tag();
                    var value = view.value();
                    var structureView = value.structure()) {
                assertEquals(fixture.rawTag(), tag.raw());
                assertEquals(TtlvItemType.Structure, view.itemType());
                assertEquals(0, structureView.itemCount());
            }
            appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_codec.decode", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvCodec.decode", null, null, "decode", "2.1",
                    null, null, treeHash);
        }
    }

    // OASIS KMIP 2.1 §§7.1–7.2, Tables 352–353: correlation values are Byte Strings;
    // §7.1 is optional within its list and §7.2 requires the value. §11.56 Table 487 assigns their tags.
    private static void appendStructureMemberByteStringRouteEvents(ArrayNode events) throws Exception {
        int rawTag = 0x420006;
        int correlationValuesTag = 0x420176;
        byte[] firstValue = new byte[] {0x12, 0x34, (byte) 0xfe};
        byte[] secondValue = new byte[] {0x56, 0x00, (byte) 0x81};
        byte[] memberWire = item(rawTag, 8, firstValue);
        String treeHash = normalizedTreeSha256(memberWire);
        CodecLimits limits = CodecLimits.defaults();

        for (StructureMemberByteStringFixture fixture : STRUCTURE_MEMBER_BYTE_STRING_FIXTURES) {
            try (RawTag raw = RawTag.fromRaw(rawTag);
                    Tag tag = raw.checked();
                    TtlvValue value = TtlvValue.ByteString(firstValue, limits);
                    TtlvItem created = TtlvItem.create(tag, value, limits)) {
                assertTrue(created != null, "The installed Item constructor must return an owned Item.");
            }
            appendRouteEvent(events, fixture.caseId(),
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.bytestring", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvValue.ByteString", null, null, "construct", "2.1",
                    null, treeHash, null);
            appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_item.create", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvItem.create", null, null, "construct", "2.1",
                    null, treeHash, null);

            try (TtlvItem decoded = TtlvCodec.decode(memberWire);
                    TtlvItemView view = decoded.view()) {
                assertByteStringItemView(view, rawTag, firstValue);
            }
            appendRouteEvent(events, fixture.caseId(),
                    "java.generic-ttlv.function.ttlv_item_view_from_item", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvItem.view", null, null, "decode", "2.1",
                    null, null, treeHash);
            appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_codec.decode", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvCodec.decode", null, null, "decode", "2.1",
                    null, null, treeHash);
        }

        byte[] typedPayloadWire = structure(0x42007c,
                structure(correlationValuesTag, item(rawTag, 8, firstValue), item(rawTag, 8, secondValue)));
        String typedPayloadHash = normalizedTreeSha256(typedPayloadWire);
        try (QueryAsyncRequestsRequest unfiltered = QueryAsyncRequestsRequest.create();
                TtlvStructure unfilteredPayload = unfiltered.toTtlvPayload();
                TtlvStructureView unfilteredView = unfilteredPayload.view()) {
            assertEquals(0, unfilteredView.itemCount(),
                    "KMIP 2.1 §7.1 permits the optional asynchronous correlation filter to be absent.");
        }
        try (QueryAsyncRequestsRequest unfiltered = QueryAsyncRequestsRequest.create();
                QueryAsyncRequestsRequest filtered = unfiltered.withCorrelationValues(
                        List.of(firstValue, secondValue));
                TtlvStructure payload = filtered.toTtlvPayload();
                TtlvStructureView payloadView = payload.view();
                TtlvItemView valuesItem = payloadView.itemAt(0);
                Tag valuesTag = valuesItem.tag();
                var values = valuesItem.value();
                var valuesStructure = values.structure()) {
            assertEquals(1, payloadView.itemCount());
            assertEquals(correlationValuesTag, valuesTag.raw());
            assertEquals(TtlvItemType.Structure, valuesItem.itemType());
            assertEquals(2, valuesStructure.itemCount(),
                    "The §7.1 correlation list must preserve both supplied values in order.");
            try (TtlvItemView first = valuesStructure.itemAt(0);
                    TtlvItemView second = valuesStructure.itemAt(1)) {
                assertByteStringItemView(first, rawTag, firstValue);
                assertByteStringItemView(second, rawTag, secondValue);
            }
        }
        StructureMemberByteStringFixture queryFilterFixture = STRUCTURE_MEMBER_BYTE_STRING_FIXTURES.stream()
                .filter(StructureMemberByteStringFixture::typedQueryFilter).findFirst().orElseThrow();
        appendRouteEvent(events, queryFilterFixture.caseId(),
                "java.typed-protocol.model.org.kmipkit.extensions.queryasyncrequestsrequest"
                        + ".withcorrelationvalues", queryFilterFixture.targetId(),
                "org.kmipkit.extensions.QueryAsyncRequestsRequest.withCorrelationValues", null, null,
                "construct", "2.1", null, typedPayloadHash, null);
    }

    // OASIS KMIP 2.1 §7.2 Table 353: these required Asynchronous Request fields use their §11.56 tags.
    private void appendStructureMemberValueRouteEvents(ArrayNode events, byte[] serverCertificatePem,
            byte[] serverPrivateKeyPem, byte[] clientCertificatePem, byte[] clientPrivateKeyPem)
            throws Exception {
        for (StructureMemberValueFixture fixture : STRUCTURE_MEMBER_VALUE_FIXTURES) {
            byte[] memberWire = structureMemberValueWire(fixture);
            String treeHash = normalizedTreeSha256(memberWire);
            try (TtlvValue value = structureMemberValue(fixture);
                    RawTag rawTag = RawTag.fromRaw(fixture.rawTag());
                    Tag tag = rawTag.checked();
                    TtlvItem created = TtlvItem.create(tag, value, CodecLimits.defaults());
                    TtlvItemView view = created.view()) {
                assertStructureMemberValue(view, fixture);
            }
            String valueType = fixture.itemType() == TtlvItemType.DateTimeExtended
                    ? "datetimeextended" : "enumeration";
            String constructor = fixture.itemType() == TtlvItemType.DateTimeExtended
                    ? "DateTimeExtended" : "Enumeration";
            appendRouteEvent(events, fixture.caseId(),
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue." + valueType,
                    fixture.targetId(), "org.kmipkit.ttlv.TtlvValue." + constructor,
                    null, null, "construct", "2.1", null, treeHash, null);
            appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_item.create",
                    fixture.targetId(), "org.kmipkit.ttlv.TtlvItem.create", null, null,
                    "construct", "2.1", null, treeHash, null);
            appendRouteEvent(events, fixture.caseId(),
                    "java.generic-ttlv.function.ttlv_item_view_from_item", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvItem.view", null, null,
                    "decode", "2.1", null, null, treeHash);
            try (TtlvItem decoded = TtlvCodec.decode(memberWire);
                    TtlvItemView decodedView = decoded.view()) {
                assertStructureMemberValue(decodedView, fixture);
            }
            appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_codec.decode",
                    fixture.targetId(), "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                    "decode", "2.1", null, null, treeHash);
        }

        StructureMemberValueFixture operationFixture = structureMemberValueFixture(0x42005c);
        byte[] correlation = new byte[] {0x41, 0x53, 0x59, 0x4e, 0x43};
        SSLServerSocket listener = listener(certificate(serverCertificatePem),
                privateKey(serverPrivateKeyPem), certificate(clientCertificatePem));
        byte[] responseBytes = queryAsyncRequestsResponse(correlation, 0x18, 1_800_000_000L, 2);
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, responseBytes));
        Thread peerThread = new Thread(peer, "java-validator-query-async-response-payload-peer");
        peerThread.setDaemon(true);
        peerThread.start();
        Path connectionFile = temporaryDirectory.resolve("query-async-response-payload-2.1.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true, "2.1"));
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile));
                var outcome = driver.executeQueryAsyncRequestsForResponsePayloadEvidence(correlation);
                var payload = outcome.responsePayloadTtlv().orElseThrow();
                TtlvStructureView payloadView = payload.view()) {
            assertEquals(0x39L, outcome.metadataV1().operation());
            assertEquals(0L, outcome.metadataV1().resultStatus());
            assertEquals(1, payloadView.itemCount());
            try (TtlvItemView asynchronousRequest = payloadView.itemAt(0);
                    Tag asynchronousRequestTag = asynchronousRequest.tag();
                    var requestValue = asynchronousRequest.value();
                    TtlvStructureView requestFields = requestValue.structure()) {
                assertEquals(0x420173, asynchronousRequestTag.raw());
                assertEquals(TtlvItemType.Structure, asynchronousRequest.itemType());
                assertEquals(4, requestFields.itemCount());
                try (TtlvItemView correlationField = requestFields.itemAt(0);
                        Tag correlationTag = correlationField.tag();
                        var correlationValue = correlationField.value()) {
                    assertEquals(0x420006, correlationTag.raw());
                    assertEquals(TtlvItemType.ByteString, correlationField.itemType());
                    assertArrayEquals(correlation, correlationValue.bytes());
                }
                assertStructureMemberValue(requestFields, 1, operationFixture);
                assertStructureMemberValue(requestFields, 2, structureMemberValueFixture(0x420175));
                assertStructureMemberValue(requestFields, 3, structureMemberValueFixture(0x420174));
            }
            appendRouteEvent(events, operationFixture.caseId(),
                    "java.typed-protocol.function.client_async_operation_outcome_response_payload_ttlv",
                    operationFixture.targetId(), "org.kmipkit.extensions.AsyncOperationOutcome.responsePayloadTtlv",
                    null, null, "decode", "2.1", null, null, normalizedTreeSha256(responseBytes));
        } finally {
            listener.close();
        }
        byte[] requestBytes = peer.get(15, TimeUnit.SECONDS);
        assertEquals(0x39L, requestOperation(requestBytes));
        assertEquals(1, requestProtocolMinorVersion(requestBytes));
        assertTrue(contains(requestBytes, item(0x42005c, 5, intBytes(0x18))),
                "The typed Query Asynchronous Requests filter must reach the controlled KMIP 2.1 peer.");
    }

    private static TtlvValue structureMemberValue(StructureMemberValueFixture fixture) {
        return fixture.itemType() == TtlvItemType.DateTimeExtended
                ? TtlvValue.DateTimeExtended(fixture.value()) : TtlvValue.Enumeration(fixture.value());
    }

    private static byte[] structureMemberValueWire(StructureMemberValueFixture fixture) throws Exception {
        byte[] value = fixture.itemType() == TtlvItemType.DateTimeExtended
                ? longBytes(fixture.value()) : intBytes(Math.toIntExact(fixture.value()));
        return item(fixture.rawTag(), fixture.itemType().code(), value);
    }

    private static void assertStructureMemberValue(TtlvItemView view, StructureMemberValueFixture fixture)
            throws Exception {
        try (Tag tag = view.tag(); var value = view.value()) {
            assertEquals(fixture.rawTag(), tag.raw());
            assertEquals(fixture.itemType(), view.itemType());
            long actual = fixture.itemType() == TtlvItemType.DateTimeExtended
                    ? value.dateTimeExtended() : value.enumeration();
            assertEquals(fixture.value(), actual);
        }
    }

    private static void assertStructureMemberValue(TtlvStructureView fields, int index,
            StructureMemberValueFixture fixture) throws Exception {
        try (TtlvItemView view = fields.itemAt(index)) {
            assertStructureMemberValue(view, fixture);
        }
    }

    private static StructureMemberValueFixture structureMemberValueFixture(int rawTag) {
        return STRUCTURE_MEMBER_VALUE_FIXTURES.stream()
                .filter(fixture -> fixture.rawTag() == rawTag).findFirst().orElseThrow();
    }

    private static byte[] queryAsyncRequestsResponse(byte[] correlation, long operation,
            long submissionDate, long processingStage) throws Exception {
        byte[] version = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
        byte[] header = structure(0x42007a, version,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] asynchronousRequest = structure(0x420173,
                item(0x420006, 8, correlation),
                item(0x42005c, 5, intBytes(Math.toIntExact(operation))),
                item(0x420175, 5, intBytes(Math.toIntExact(processingStage))),
                item(0x420174, 11, longBytes(submissionDate)));
        byte[] payload = structure(0x42007c, asynchronousRequest);
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(0x39)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static void assertByteStringItemView(TtlvItemView view, int rawTag, byte[] expectedValue) {
        try (Tag tag = view.tag(); var value = view.value()) {
            assertEquals(rawTag, tag.raw());
            assertEquals(TtlvItemType.ByteString, view.itemType());
            assertArrayEquals(expectedValue, value.bytes());
        }
    }

    // OASIS KMIP 2.1 §7.2 Table 353: Asynchronous Request is a Structure with three required fields.
    private static void appendAsynchronousRequestOperationStructureRouteEvents(ArrayNode events) throws Exception {
        String caseId = "test-norm-operation-structure-operation-structure-7-2-asynchronous-request";
        String targetId = "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-2-ASYNCHRONOUS-REQUEST";
        int rawTag = 0x420173;
        long operation = 0x18;
        long processingStage = 2;
        long submissionDate = 1_800_000_000L;
        byte[] wire = structure(rawTag,
                item(0x42005c, 5, intBytes(Math.toIntExact(operation))),
                item(0x420175, 5, intBytes(Math.toIntExact(processingStage))),
                item(0x420174, 11, longBytes(submissionDate)));
        String treeHash = normalizedTreeSha256(wire);
        CodecLimits limits = CodecLimits.defaults();

        TtlvStructure fields = TtlvStructure.create();
        fields = appendTtlvItem(fields, 0x42005c, TtlvValue.Enumeration(operation), limits);
        fields = appendTtlvItem(fields, 0x420175, TtlvValue.Enumeration(processingStage), limits);
        fields = appendTtlvItem(fields, 0x420174, TtlvValue.DateTimeExtended(submissionDate), limits);
        TtlvStructure completedFields = fields;
        try (completedFields;
                TtlvValue structureValue = TtlvValue.structure(completedFields, limits);
                RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                TtlvItem created = TtlvItem.create(tag, structureValue, limits);
                var view = created.view();
                Tag viewedTag = view.tag();
                var value = view.value();
                var structureView = value.structure()) {
            assertEquals(rawTag, viewedTag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertAsynchronousRequestFields(structureView, operation, processingStage, submissionDate);
        }
        try (TtlvItem decoded = TtlvCodec.decode(wire);
                var view = decoded.view();
                var decodedTag = view.tag();
                var value = view.value();
                var structureView = value.structure()) {
            assertEquals(rawTag, decodedTag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertAsynchronousRequestFields(structureView, operation, processingStage, submissionDate);
        }
        appendGenericStructureRouteEvents(events, caseId, targetId, treeHash);
    }

    private static void assertAsynchronousRequestFields(TtlvStructureView fields, long operation,
            long processingStage, long submissionDate) throws Exception {
        assertEquals(3, fields.itemCount());
        try (TtlvItemView operationField = fields.itemAt(0);
                Tag operationTag = operationField.tag();
                var operationValue = operationField.value();
                TtlvItemView processingStageField = fields.itemAt(1);
                Tag processingStageTag = processingStageField.tag();
                var processingStageValue = processingStageField.value();
                TtlvItemView submissionDateField = fields.itemAt(2);
                Tag submissionDateTag = submissionDateField.tag();
                var submissionDateValue = submissionDateField.value()) {
            assertEquals(0x42005c, operationTag.raw());
            assertEquals(TtlvItemType.Enumeration, operationField.itemType());
            assertEquals(operation, operationValue.enumeration());
            assertEquals(0x420175, processingStageTag.raw());
            assertEquals(TtlvItemType.Enumeration, processingStageField.itemType());
            assertEquals(processingStage, processingStageValue.enumeration());
            assertEquals(0x420174, submissionDateTag.raw());
            assertEquals(TtlvItemType.DateTimeExtended, submissionDateField.itemType());
            assertEquals(submissionDate, submissionDateValue.dateTimeExtended());
        }
    }

    // OASIS KMIP 2.1 §§7.23–7.24, Tables 375–376: required Structures with repeatable members.
    private static void appendOperationStructureRouteEvents(ArrayNode events,
            OperationStructureFixture fixture) throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        byte[] memberValue = fixture.memberType() == TtlvItemType.TextString
                ? fixture.textValue().getBytes(StandardCharsets.UTF_8)
                : intBytes(Math.toIntExact(fixture.enumerationValue()));
        byte[] memberWire = item(fixture.memberTag(), fixture.memberType().code(), memberValue);
        byte[] wire = structure(fixture.structureTag(), memberWire);
        String treeHash = normalizedTreeSha256(wire);

        TtlvStructure fields = TtlvStructure.create();
        TtlvValue member = fixture.memberType() == TtlvItemType.TextString
                ? TtlvValue.TextString(memberValue, limits)
                : TtlvValue.Enumeration(fixture.enumerationValue());
        fields = appendTtlvItem(fields, fixture.memberTag(), member, limits);
        TtlvStructure completedFields = fields;
        try (completedFields;
                TtlvValue structureValue = TtlvValue.structure(completedFields, limits);
                RawTag raw = RawTag.fromRaw(fixture.structureTag());
                Tag tag = raw.checked();
                TtlvItem created = TtlvItem.create(tag, structureValue, limits);
                var view = created.view();
                Tag viewedTag = view.tag();
                var value = view.value();
                var structureView = value.structure()) {
            assertEquals(fixture.structureTag(), viewedTag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertOperationStructureMember(structureView, fixture, memberValue);
        }
        try (TtlvItem decoded = TtlvCodec.decode(wire);
                var view = decoded.view();
                var tag = view.tag();
                var value = view.value();
                var structureView = value.structure()) {
            assertEquals(fixture.structureTag(), tag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertOperationStructureMember(structureView, fixture, memberValue);
        }
        appendGenericStructureRouteEvents(events, fixture.caseId(), fixture.targetId(), treeHash);
    }

    // OASIS KMIP 2.1 §7.24 Table 376: Object Type is optional and may be repeated.
    private static void appendObjectTypeMemberRouteEvents(ArrayNode events) throws Exception {
        int objectTypesTag = 0x420125;
        int objectTypeTag = 0x420057;
        long firstObjectType = KmipValues.ObjectType.SYMMETRIC_KEY;
        long secondObjectType = KmipValues.ObjectType.PUBLIC_KEY;
        byte[] memberWire = item(objectTypeTag, TtlvItemType.Enumeration.code(),
                intBytes(Math.toIntExact(firstObjectType)));
        String treeHash = normalizedTreeSha256(memberWire);
        CodecLimits limits = CodecLimits.defaults();

        try (TtlvValue value = TtlvValue.Enumeration(firstObjectType);
                RawTag rawTag = RawTag.fromRaw(objectTypeTag);
                Tag tag = rawTag.checked();
                TtlvItem created = TtlvItem.create(tag, value, limits);
                TtlvItemView view = created.view();
                Tag viewedTag = view.tag();
                var viewedValue = view.value()) {
            assertEquals(objectTypeTag, viewedTag.raw());
            assertEquals(TtlvItemType.Enumeration, view.itemType());
            assertEquals(firstObjectType, viewedValue.enumeration());
        }
        try (TtlvItem decoded = TtlvCodec.decode(memberWire);
                TtlvItemView view = decoded.view();
                Tag viewedTag = view.tag();
                var viewedValue = view.value()) {
            assertEquals(objectTypeTag, viewedTag.raw());
            assertEquals(TtlvItemType.Enumeration, view.itemType());
            assertEquals(firstObjectType, viewedValue.enumeration());
        }

        byte[] absentWire = structure(objectTypesTag);
        try (TtlvItem absent = TtlvCodec.decode(absentWire);
                TtlvItemView absentView = absent.view();
                Tag absentTag = absentView.tag();
                var absentValue = absentView.value();
                var absentFields = absentValue.structure()) {
            assertEquals(objectTypesTag, absentTag.raw());
            assertEquals(0, absentFields.itemCount(),
                    "The optional Object Type member may be absent from Object Types.");
        }

        byte[] repeatedWire = structure(objectTypesTag,
                item(objectTypeTag, TtlvItemType.Enumeration.code(),
                        intBytes(Math.toIntExact(firstObjectType))),
                item(objectTypeTag, TtlvItemType.Enumeration.code(),
                        intBytes(Math.toIntExact(secondObjectType))));
        try (TtlvItem repeated = TtlvCodec.decode(repeatedWire);
                TtlvItemView repeatedView = repeated.view();
                Tag repeatedTag = repeatedView.tag();
                var repeatedValue = repeatedView.value();
                var repeatedFields = repeatedValue.structure()) {
            assertEquals(objectTypesTag, repeatedTag.raw());
            assertOperationStructureMemberValues(repeatedFields,
                    firstObjectType, secondObjectType);
        }

        appendRouteEvent(events, OBJECT_TYPE_MEMBER_CASE_ID,
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.enumeration",
                OBJECT_TYPE_MEMBER_TARGET_ID, "org.kmipkit.ttlv.TtlvValue.Enumeration",
                null, null, "construct", "2.1", null, treeHash, null);
        appendRouteEvent(events, OBJECT_TYPE_MEMBER_CASE_ID,
                "java.generic-ttlv.ttlv_item.create", OBJECT_TYPE_MEMBER_TARGET_ID,
                "org.kmipkit.ttlv.TtlvItem.create", null, null,
                "construct", "2.1", null, treeHash, null);
        appendRouteEvent(events, OBJECT_TYPE_MEMBER_CASE_ID,
                "java.generic-ttlv.function.ttlv_item_view_from_item", OBJECT_TYPE_MEMBER_TARGET_ID,
                "org.kmipkit.ttlv.TtlvItem.view", null, null,
                "decode", "2.1", null, null, treeHash);
        appendRouteEvent(events, OBJECT_TYPE_MEMBER_CASE_ID,
                "java.generic-ttlv.ttlv_codec.decode", OBJECT_TYPE_MEMBER_TARGET_ID,
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", "2.1", null, null, treeHash);
    }

    // OASIS KMIP 2.1 §§7.6–7.7, Tables 357–358: optional Constraint members and repeated Constraint values.
    private static void appendConstraintStructureRouteEvents(ArrayNode events) throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        byte[] constraintWire = structure(0x420169,
                structure(0x420167, item(0x420057, TtlvItemType.Enumeration.code(), intBytes(2))),
                structure(0x420166, textItem(0x420056, "kmipkit-object-group")));
        byte[] constraintsWire = structure(0x420168, constraintWire, constraintWire);
        String constraintHash = normalizedTreeSha256(constraintWire);
        String constraintsHash = normalizedTreeSha256(constraintsWire);

        try (TtlvStructure fields = constraintFields(limits);
                TtlvValue value = TtlvValue.structure(fields, limits);
                RawTag rawTag = RawTag.fromRaw(0x420169);
                Tag tag = rawTag.checked();
                TtlvItem created = TtlvItem.create(tag, value, limits);
                TtlvItemView view = created.view();
                Tag viewedTag = view.tag();
                var viewedValue = view.value();
                var viewedFields = viewedValue.structure()) {
            assertEquals(0x420169, viewedTag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertConstraintFields(viewedFields);
        }
        try (TtlvItem decoded = TtlvCodec.decode(constraintWire);
                TtlvItemView view = decoded.view();
                Tag viewedTag = view.tag();
                var viewedValue = view.value();
                var viewedFields = viewedValue.structure()) {
            assertEquals(0x420169, viewedTag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertConstraintFields(viewedFields);
        }

        try (TtlvItem emptyConstraint = TtlvCodec.decode(structure(0x420169));
                TtlvItemView view = emptyConstraint.view();
                var value = view.value();
                var fields = value.structure()) {
            assertEquals(0, fields.itemCount(),
                    "Object Types, Object Groups, and Attributes are optional Constraint members.");
        }
        try (TtlvItem emptyConstraints = TtlvCodec.decode(structure(0x420168));
                TtlvItemView view = emptyConstraints.view();
                var value = view.value();
                var fields = value.structure()) {
            assertEquals(0, fields.itemCount(), "The repeated Constraint member may be absent.");
        }

        try (TtlvStructure fields = constraintsFields(limits);
                TtlvValue value = TtlvValue.structure(fields, limits);
                RawTag rawTag = RawTag.fromRaw(0x420168);
                Tag tag = rawTag.checked();
                TtlvItem created = TtlvItem.create(tag, value, limits);
                TtlvItemView view = created.view();
                Tag viewedTag = view.tag();
                var viewedValue = view.value();
                var viewedFields = viewedValue.structure()) {
            assertEquals(0x420168, viewedTag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertConstraintsFields(viewedFields);
        }
        try (TtlvItem decoded = TtlvCodec.decode(constraintsWire);
                TtlvItemView view = decoded.view();
                Tag viewedTag = view.tag();
                var viewedValue = view.value();
                var viewedFields = viewedValue.structure()) {
            assertEquals(0x420168, viewedTag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertConstraintsFields(viewedFields);
        }

        appendGenericStructureRouteEvents(events, CONSTRAINT_CASE_ID,
                CONSTRAINT_TARGET_ID, "2.1", constraintHash);
        appendGenericStructureRouteEvents(events, CONSTRAINTS_CASE_ID,
                CONSTRAINTS_TARGET_ID, "2.1", constraintsHash);
    }

    private static TtlvStructure constraintFields(CodecLimits limits) {
        TtlvStructure fields = TtlvStructure.create();
        fields = appendTtlvItem(fields, 0x420167,
                TtlvValue.structure(objectTypesFields(limits), limits), limits);
        return appendTtlvItem(fields, 0x420166,
                TtlvValue.structure(objectGroupsFields(limits), limits), limits);
    }

    private static TtlvStructure objectTypesFields(CodecLimits limits) {
        return appendTtlvItem(TtlvStructure.create(), 0x420057,
                TtlvValue.Enumeration(KmipValues.ObjectType.SYMMETRIC_KEY), limits);
    }

    private static TtlvStructure objectGroupsFields(CodecLimits limits) {
        byte[] group = "kmipkit-object-group".getBytes(StandardCharsets.UTF_8);
        return appendTtlvItem(TtlvStructure.create(), 0x420056,
                TtlvValue.TextString(group, limits), limits);
    }

    private static TtlvStructure constraintsFields(CodecLimits limits) {
        TtlvStructure fields = TtlvStructure.create();
        fields = appendTtlvItem(fields, 0x420169,
                TtlvValue.structure(constraintFields(limits), limits), limits);
        return appendTtlvItem(fields, 0x420169,
                TtlvValue.structure(constraintFields(limits), limits), limits);
    }

    private static void assertConstraintFields(TtlvStructureView fields) throws Exception {
        int[] tags = {0x420167, 0x420166};
        assertEquals(tags.length, fields.itemCount());
        try (TtlvItemView objectTypes = fields.itemAt(0);
                Tag objectTypesTag = objectTypes.tag();
                var objectTypesValue = objectTypes.value();
                var objectTypesFields = objectTypesValue.structure()) {
            assertEquals(tags[0], objectTypesTag.raw());
            assertEquals(TtlvItemType.Structure, objectTypes.itemType());
            assertEquals(1, objectTypesFields.itemCount());
            try (TtlvItemView objectType = objectTypesFields.itemAt(0);
                    Tag objectTypeTag = objectType.tag();
                    var objectTypeValue = objectType.value()) {
                assertEquals(0x420057, objectTypeTag.raw());
                assertEquals(TtlvItemType.Enumeration, objectType.itemType());
                assertEquals(KmipValues.ObjectType.SYMMETRIC_KEY, objectTypeValue.enumeration());
            }
        }
        try (TtlvItemView objectGroups = fields.itemAt(1);
                Tag objectGroupsTag = objectGroups.tag();
                var objectGroupsValue = objectGroups.value();
                var objectGroupsFields = objectGroupsValue.structure()) {
            assertEquals(tags[1], objectGroupsTag.raw());
            assertEquals(TtlvItemType.Structure, objectGroups.itemType());
            assertEquals(1, objectGroupsFields.itemCount());
            try (TtlvItemView objectGroup = objectGroupsFields.itemAt(0);
                    Tag objectGroupTag = objectGroup.tag();
                    var objectGroupValue = objectGroup.value()) {
                assertEquals(0x420056, objectGroupTag.raw());
                assertEquals(TtlvItemType.TextString, objectGroup.itemType());
                assertArrayEquals("kmipkit-object-group".getBytes(StandardCharsets.UTF_8),
                        objectGroupValue.bytes());
            }
        }
    }

    private static void assertConstraintsFields(TtlvStructureView fields) throws Exception {
        assertEquals(2, fields.itemCount(), "The Constraint Structure member may be repeated.");
        for (int index = 0; index < fields.itemCount(); index++) {
            try (TtlvItemView constraint = fields.itemAt(index);
                    Tag tag = constraint.tag();
                    var value = constraint.value();
                    var constraintFields = value.structure()) {
                assertEquals(0x420169, tag.raw());
                assertEquals(TtlvItemType.Structure, constraint.itemType());
                assertConstraintFields(constraintFields);
            }
        }
    }

    private static void assertOperationStructureMemberValues(TtlvStructureView fields,
            long... expectedValues) throws Exception {
        assertEquals(expectedValues.length, fields.itemCount());
        for (int index = 0; index < expectedValues.length; index++) {
            try (TtlvItemView field = fields.itemAt(index);
                    Tag tag = field.tag();
                    var value = field.value()) {
                assertEquals(0x420057, tag.raw());
                assertEquals(TtlvItemType.Enumeration, field.itemType());
                assertEquals(expectedValues[index], value.enumeration());
            }
        }
    }

    // OASIS KMIP 2.0 §§7.1–7.2, 7.4, 7.6, 7.10, 7.12–7.13, and 7.15–7.16 plus KMIP 2.1
    // §§7.3–7.4, 7.8, 7.10, 7.14, 7.16–7.17, and 7.19–7.20 define these field value types.
    private static void appendScalarOperationStructureRouteEvents(ArrayNode events,
            ScalarOperationStructureFixture fixture, String protocolVersion) throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        byte[] scalarBytes = scalarOperationStructureBytes(fixture.valueType());
        byte[] wire = item(fixture.tag(), fixture.valueType().code(), scalarBytes);
        String scalarHash = normalizedTreeSha256(wire);

        if (fixture.optional()) {
            try (TtlvStructure absentFields = TtlvStructure.create();
                    var absentView = absentFields.view()) {
                assertEquals(0, absentView.itemCount(),
                        "An optional operation field must be representable as absent.");
            }
        }
        TtlvStructure fields = TtlvStructure.create();
        fields = appendTtlvItem(fields, fixture.tag(),
                scalarOperationStructureValue(fixture.valueType(), limits), limits);
        TtlvStructure completedFields = fields;
        try (completedFields;
                var structureView = completedFields.view();
                TtlvItemView field = structureView.itemAt(0);
                Tag fieldTag = field.tag();
                var fieldValue = field.value()) {
            assertEquals(1, structureView.itemCount());
            assertEquals(fixture.tag(), fieldTag.raw());
            assertEquals(fixture.valueType(), field.itemType());
            assertScalarOperationStructureValue(fieldValue, fixture.valueType(), scalarBytes);
        }
        appendRouteEvent(events, fixture.caseId(),
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.create", fixture.targetId(),
                "org.kmipkit.ttlv.TtlvStructure.create", null, null,
                "construct", protocolVersion, null, scalarHash, null);
        appendRouteEvent(events, fixture.caseId(),
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.withitem", fixture.targetId(),
                "org.kmipkit.ttlv.TtlvStructure.withItem", null, null,
                "construct", protocolVersion, null, scalarHash, null);
        appendRouteEvent(events, fixture.caseId(), scalarOperationStructureValueRoute(fixture.valueType()),
                fixture.targetId(), scalarOperationStructureValueSymbol(fixture.valueType()), null, null,
                "construct", protocolVersion, null, scalarHash, null);
        appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_item.create", fixture.targetId(),
                "org.kmipkit.ttlv.TtlvItem.create", null, null,
                "construct", protocolVersion, null, scalarHash, null);
        try (TtlvItem decoded = TtlvCodec.decode(wire);
                var view = decoded.view();
                Tag tag = view.tag();
                var value = view.value()) {
            assertEquals(fixture.tag(), tag.raw());
            assertEquals(fixture.valueType(), view.itemType());
            assertScalarOperationStructureValue(value, fixture.valueType(), scalarBytes);
        }
        appendRouteEvent(events, fixture.caseId(),
                "java.generic-ttlv.function.ttlv_item_view_from_item", fixture.targetId(),
                "org.kmipkit.ttlv.TtlvItem.view", null, null,
                "decode", protocolVersion, null, null, scalarHash);
        appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_codec.decode", fixture.targetId(),
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null, null, scalarHash);
    }

    private static TtlvValue scalarOperationStructureValue(TtlvItemType valueType, CodecLimits limits) {
        return switch (valueType) {
            case Integer -> TtlvValue.Integer(42);
            case Boolean -> TtlvValue.Boolean(true);
            case TextString -> TtlvValue.TextString(scalarOperationStructureBytes(valueType), limits);
            case ByteString -> TtlvValue.ByteString(scalarOperationStructureBytes(valueType), limits);
            default -> throw new IllegalArgumentException("Unsupported scalar operation structure type");
        };
    }

    private static byte[] scalarOperationStructureBytes(TtlvItemType valueType) {
        return switch (valueType) {
            case Integer -> intBytes(42);
            case Boolean -> longBytes(1);
            case TextString -> "kmipkit-java-fixture".getBytes(StandardCharsets.UTF_8);
            case ByteString -> new byte[] {0x01, 0x23, 0x45, 0x67};
            default -> throw new IllegalArgumentException("Unsupported scalar operation structure type");
        };
    }

    private static String scalarOperationStructureValueRoute(TtlvItemType valueType) {
        return switch (valueType) {
            case Integer -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.integer";
            case Boolean -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.boolean";
            case TextString -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring";
            case ByteString -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.bytestring";
            default -> throw new IllegalArgumentException("Unsupported scalar operation structure type");
        };
    }

    private static String scalarOperationStructureValueSymbol(TtlvItemType valueType) {
        return switch (valueType) {
            case Integer -> "org.kmipkit.ttlv.TtlvValue.Integer";
            case Boolean -> "org.kmipkit.ttlv.TtlvValue.Boolean";
            case TextString -> "org.kmipkit.ttlv.TtlvValue.TextString";
            case ByteString -> "org.kmipkit.ttlv.TtlvValue.ByteString";
            default -> throw new IllegalArgumentException("Unsupported scalar operation structure type");
        };
    }

    private static void assertScalarOperationStructureValue(org.kmipkit.ttlv.TtlvValueView value,
            TtlvItemType valueType, byte[] expectedBytes) throws Exception {
        switch (valueType) {
            case Integer -> assertEquals(42, value.integer());
            case Boolean -> assertTrue(value.booleanValue());
            case TextString, ByteString -> assertArrayEquals(expectedBytes, value.bytes());
            default -> throw new IllegalArgumentException("Unsupported scalar operation structure type");
        }
    }

    private static void assertOperationStructureMember(TtlvStructureView fields,
            OperationStructureFixture fixture, byte[] expectedTextValue) throws Exception {
        assertEquals(1, fields.itemCount());
        try (TtlvItemView member = fields.itemAt(0);
                Tag tag = member.tag();
                var value = member.value()) {
            assertEquals(fixture.memberTag(), tag.raw());
            assertEquals(fixture.memberType(), member.itemType());
            if (fixture.memberType() == TtlvItemType.TextString) {
                assertArrayEquals(expectedTextValue, value.bytes());
            } else {
                assertEquals(fixture.enumerationValue(), value.enumeration());
            }
        }
    }

    private static void assertOptionalStructureMemberShape(StructureMemberStructureFixture fixture,
            byte[] memberWire) throws Exception {
        byte[] absentWire = structure(fixture.parentTag());
        byte[] presentWire = fixture.repeatable()
                ? structure(fixture.parentTag(), memberWire, memberWire)
                : structure(fixture.parentTag(), memberWire);
        int expectedCount = fixture.repeatable() ? 2 : 1;
        try (TtlvItem absent = TtlvCodec.decode(absentWire);
                var absentView = absent.view();
                var absentValue = absentView.value();
                var absentMembers = absentValue.structure();
                TtlvItem present = TtlvCodec.decode(presentWire);
                var presentView = present.view();
                var presentValue = presentView.value();
                var presentMembers = presentValue.structure()) {
            try (Tag parentTag = absentView.tag()) {
                assertEquals(fixture.parentTag(), parentTag.raw());
            }
            assertEquals(0, absentMembers.itemCount(), "The normative Structure member is optional.");
            assertEquals(expectedCount, presentMembers.itemCount());
            for (int index = 0; index < expectedCount; index++) {
                try (var member = presentMembers.itemAt(index);
                        Tag tag = member.tag();
                        var value = member.value();
                        var memberStructure = value.structure()) {
                    assertEquals(fixture.rawTag(), tag.raw());
                    assertEquals(TtlvItemType.Structure, member.itemType());
                    assertEquals(0, memberStructure.itemCount());
                }
            }
        }
    }

    private static void assertRouteEventRecorded(ArrayNode events, String caseId, String targetId,
            String routeId, String action, String protocolVersion) {
        boolean recorded = false;
        for (JsonNode event : events) {
            if (caseId.equals(event.path("case_id").asText())
                    && targetId.equals(event.path("target_id").asText())
                    && routeId.equals(event.path("route_id").asText())
                    && action.equals(event.path("action").asText())
                    && protocolVersion.equals(event.path("protocol_version").asText())) {
                recorded = true;
                break;
            }
        }
        assertTrue(recorded, "Missing invoked Java route evidence for " + caseId + " / " + routeId
                + " / KMIP " + protocolVersion);
    }

    private static void assertGenericStructureRouteEvents(ArrayNode events, String caseId, String targetId,
            String protocolVersion) {
        for (String route : GENERIC_STRUCTURE_ROUTE_IDS) {
            String action = route.endsWith("ttlv_codec.decode")
                    || route.endsWith("ttlv_item_view_from_item") ? "decode" : "construct";
            assertRouteEventRecorded(events, caseId, targetId, route, action, protocolVersion);
        }
    }

    private static void assertScalarOperationStructureRouteEvents(ArrayNode events,
            ScalarOperationStructureFixture fixture, String protocolVersion) {
        String valueRoute = switch (fixture.valueType()) {
            case Integer -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.integer";
            case Boolean -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.boolean";
            case TextString -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring";
            case ByteString -> "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.bytestring";
            default -> throw new IllegalArgumentException("Unsupported scalar operation structure type");
        };
        for (String route : List.of(
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.create",
                "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvstructure.withitem",
                valueRoute,
                "java.generic-ttlv.ttlv_item.create",
                "java.generic-ttlv.function.ttlv_item_view_from_item",
                "java.generic-ttlv.ttlv_codec.decode")) {
            String action = route.endsWith("ttlv_codec.decode")
                    || route.endsWith("ttlv_item_view_from_item") ? "decode" : "construct";
            assertRouteEventRecorded(events, fixture.caseId(), fixture.targetId(), route,
                    action, protocolVersion);
        }
    }

    // OASIS KMIP 2.0 §§7.25–7.26, Tables 345–346; KMIP 2.1 §§7.31–7.32, Tables 383–384.
    private static void appendProfileInformationRouteEvents(ArrayNode events, String protocolVersion)
            throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        byte[] informationWire = profileInformationWire();
        byte[] versionWire = profileVersionWire();
        String informationHash = normalizedTreeSha256(informationWire);
        String versionHash = normalizedTreeSha256(versionWire);

        try (TtlvStructure informationFields = profileInformationFields(limits);
                TtlvValue informationValue = TtlvValue.structure(informationFields, limits);
                RawTag informationRawTag = RawTag.fromRaw(0x4200eb);
                Tag informationTag = informationRawTag.checked();
                TtlvItem informationItem = TtlvItem.create(informationTag, informationValue, limits);
                var informationView = informationItem.view();
                Tag viewedInformationTag = informationView.tag();
                var viewedInformationValue = informationView.value();
                var viewedInformationFields = viewedInformationValue.structure()) {
            assertEquals(0x4200eb, viewedInformationTag.raw());
            assertEquals(TtlvItemType.Structure, informationView.itemType());
            assertProfileInformationFields(viewedInformationFields);
        }
        try (TtlvItem decodedInformation = TtlvCodec.decode(informationWire);
                var informationView = decodedInformation.view();
                Tag viewedInformationTag = informationView.tag();
                var viewedInformationValue = informationView.value();
                var viewedInformationFields = viewedInformationValue.structure()) {
            assertEquals(0x4200eb, viewedInformationTag.raw());
            assertEquals(TtlvItemType.Structure, informationView.itemType());
            assertProfileInformationFields(viewedInformationFields);
        }

        TtlvStructure requiredOnlyFields = appendTtlvItem(TtlvStructure.create(), 0x4200ec,
                TtlvValue.Enumeration(0x12a), limits);
        try (TtlvValue requiredOnlyValue = TtlvValue.structure(requiredOnlyFields, limits);
                RawTag requiredOnlyRawTag = RawTag.fromRaw(0x4200eb);
                Tag requiredOnlyTag = requiredOnlyRawTag.checked();
                TtlvItem requiredOnlyItem = TtlvItem.create(requiredOnlyTag, requiredOnlyValue, limits);
                var requiredOnlyView = requiredOnlyItem.view();
                Tag requiredOnlyViewedTag = requiredOnlyView.tag();
                var requiredOnlyStructureValue = requiredOnlyView.value();
                var requiredOnlyStructure = requiredOnlyStructureValue.structure()) {
            assertEquals(0x4200eb, requiredOnlyViewedTag.raw());
            assertEquals(1, requiredOnlyStructure.itemCount(),
                    "Profile Version, Server URI, and Server Port are optional in Profile Information.");
        }

        try (TtlvStructure versionFields = profileVersionFields(limits);
                TtlvValue versionValue = TtlvValue.structure(versionFields, limits);
                RawTag versionRawTag = RawTag.fromRaw(0x4200ef);
                Tag versionTag = versionRawTag.checked();
                TtlvItem versionItem = TtlvItem.create(versionTag, versionValue, limits);
                var versionView = versionItem.view();
                Tag viewedVersionTag = versionView.tag();
                var viewedVersionValue = versionView.value();
                var viewedVersionFields = viewedVersionValue.structure()) {
            assertEquals(0x4200ef, viewedVersionTag.raw());
            assertEquals(TtlvItemType.Structure, versionView.itemType());
            assertProfileVersionFields(viewedVersionFields);
        }
        try (TtlvItem decodedVersion = TtlvCodec.decode(versionWire);
                var versionView = decodedVersion.view();
                Tag viewedVersionTag = versionView.tag();
                var viewedVersionValue = versionView.value();
                var viewedVersionFields = viewedVersionValue.structure()) {
            assertEquals(0x4200ef, viewedVersionTag.raw());
            assertEquals(TtlvItemType.Structure, versionView.itemType());
            assertProfileVersionFields(viewedVersionFields);
        }

        for (ProfileInformationMemberFixture fixture : PROFILE_INFORMATION_MEMBER_FIXTURES) {
            appendProfileInformationMemberRouteEvents(events, fixture, protocolVersion, limits);
        }
        appendGenericStructureRouteEvents(events, PROFILE_INFORMATION_CASE_ID,
                PROFILE_INFORMATION_TARGET_ID, protocolVersion, informationHash);
        appendGenericStructureRouteEvents(events, PROFILE_VERSION_CASE_ID,
                PROFILE_VERSION_TARGET_ID, protocolVersion, versionHash);
    }

    private static TtlvStructure profileInformationFields(CodecLimits limits) {
        TtlvStructure fields = TtlvStructure.create();
        fields = appendTtlvItem(fields, 0x4200ec, TtlvValue.Enumeration(0x12a), limits);
        fields = appendTtlvItem(fields, 0x4200ef,
                TtlvValue.structure(profileVersionFields(limits), limits), limits);
        fields = appendTtlvItem(fields, 0x4200ed,
                TtlvValue.TextString("https://kms.example.invalid".getBytes(StandardCharsets.UTF_8), limits), limits);
        return appendTtlvItem(fields, 0x4200ee, TtlvValue.Integer(5696), limits);
    }

    private static TtlvStructure profileVersionFields(CodecLimits limits) {
        TtlvStructure fields = TtlvStructure.create();
        fields = appendTtlvItem(fields, 0x4200f0, TtlvValue.Integer(1), limits);
        return appendTtlvItem(fields, 0x4200f1, TtlvValue.Integer(0), limits);
    }

    private static void appendProfileInformationMemberRouteEvents(ArrayNode events,
            ProfileInformationMemberFixture fixture, String protocolVersion, CodecLimits limits)
            throws Exception {
        byte[] wire;
        TtlvValue constructedValue;
        String constructorRoute;
        String constructorSymbol;
        if (fixture.structure()) {
            wire = profileVersionWire();
            constructedValue = TtlvValue.structure(profileVersionFields(limits), limits);
            constructorRoute = "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure";
            constructorSymbol = "org.kmipkit.ttlv.TtlvValue.structure";
        } else if (fixture.itemType() == TtlvItemType.Integer) {
            wire = item(fixture.rawTag(), TtlvItemType.Integer.code(), intBytes(fixture.integerValue()));
            constructedValue = TtlvValue.Integer(fixture.integerValue());
            constructorRoute = "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.integer";
            constructorSymbol = "org.kmipkit.ttlv.TtlvValue.Integer";
        } else {
            byte[] text = fixture.textValue().getBytes(StandardCharsets.UTF_8);
            wire = item(fixture.rawTag(), TtlvItemType.TextString.code(), text);
            constructedValue = TtlvValue.TextString(text, limits);
            constructorRoute = "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.textstring";
            constructorSymbol = "org.kmipkit.ttlv.TtlvValue.TextString";
        }
        String treeHash = normalizedTreeSha256(wire);
        try (TtlvValue value = constructedValue;
                RawTag rawTag = RawTag.fromRaw(fixture.rawTag());
                Tag tag = rawTag.checked();
                TtlvItem created = TtlvItem.create(tag, value, limits);
                var view = created.view();
                Tag viewedTag = view.tag()) {
            assertEquals(fixture.rawTag(), viewedTag.raw());
            assertEquals(fixture.itemType(), view.itemType());
            assertProfileInformationMemberValue(view, fixture);
        }
        try (TtlvItem decoded = TtlvCodec.decode(wire);
                var view = decoded.view();
                Tag viewedTag = view.tag()) {
            assertEquals(fixture.rawTag(), viewedTag.raw());
            assertEquals(fixture.itemType(), view.itemType());
            assertProfileInformationMemberValue(view, fixture);
        }
        appendRouteEvent(events, fixture.caseId(), constructorRoute, fixture.targetId(), constructorSymbol,
                null, null, "construct", protocolVersion, null, treeHash, null);
        appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_item.create", fixture.targetId(),
                "org.kmipkit.ttlv.TtlvItem.create", null, null,
                "construct", protocolVersion, null, treeHash, null);
        appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.function.ttlv_item_view_from_item",
                fixture.targetId(), "org.kmipkit.ttlv.TtlvItem.view", null, null,
                "decode", protocolVersion, null, null, treeHash);
        appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_codec.decode", fixture.targetId(),
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null, null, treeHash);
    }

    private static void assertProfileInformationMemberValue(TtlvItemView view,
            ProfileInformationMemberFixture fixture) throws Exception {
        try (var value = view.value()) {
            if (fixture.structure()) {
                assertProfileVersionFields(value.structure());
            } else if (fixture.itemType() == TtlvItemType.Integer) {
                assertEquals(fixture.integerValue(), value.integer());
            } else {
                assertArrayEquals(fixture.textValue().getBytes(StandardCharsets.UTF_8), value.bytes());
            }
        }
    }

    private static void assertProfileInformationFields(TtlvStructureView fields) throws Exception {
        int[] tags = {0x4200ec, 0x4200ef, 0x4200ed, 0x4200ee};
        TtlvItemType[] types = {TtlvItemType.Enumeration, TtlvItemType.Structure,
                TtlvItemType.TextString, TtlvItemType.Integer};
        assertEquals(tags.length, fields.itemCount());
        for (int index = 0; index < tags.length; index++) {
            try (TtlvItemView field = fields.itemAt(index);
                    Tag tag = field.tag()) {
                assertEquals(tags[index], tag.raw());
                assertEquals(types[index], field.itemType());
                try (var value = field.value()) {
                    switch (index) {
                        case 0 -> assertEquals(0x12a, value.enumeration());
                        case 1 -> assertProfileVersionFields(value.structure());
                        case 2 -> assertArrayEquals("https://kms.example.invalid".getBytes(StandardCharsets.UTF_8),
                                value.bytes());
                        case 3 -> assertEquals(5696, value.integer());
                        default -> throw new AssertionError("Unexpected Profile Information field index.");
                    }
                }
            }
        }
    }

    private static void assertProfileVersionFields(TtlvStructureView fields) throws Exception {
        int[] tags = {0x4200f0, 0x4200f1};
        int[] values = {1, 0};
        assertEquals(tags.length, fields.itemCount());
        for (int index = 0; index < tags.length; index++) {
            try (TtlvItemView field = fields.itemAt(index);
                    Tag tag = field.tag();
                    var value = field.value()) {
                assertEquals(tags[index], tag.raw());
                assertEquals(TtlvItemType.Integer, field.itemType());
                assertEquals(values[index], value.integer());
            }
        }
    }

    // OASIS KMIP 2.0 §7.9 / §11.54 and KMIP 2.1 §7.13 / §11.56.
    private static void appendExtensionInformationRouteEvents(ArrayNode events, String protocolVersion)
            throws Exception {
        String caseId = "test-norm-operation-structure-operation-structure-7-13-extension-information";
        String targetId = "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-13-EXTENSION-INFORMATION";
        int structureTag = 0x4200a4;
        int extensionTag = 0x540001;
        int extensionType = TtlvItemType.TextString.code();
        int extensionEnumeration = 7;
        int parentStructureTag = 0x420010;
        boolean extensionAttribute = true;
        String extensionName = "kmipkit-route-evidence";
        String extensionDescription = "Installed Extension Information route";
        byte[] extensionNameBytes = extensionName.getBytes(StandardCharsets.UTF_8);
        byte[] extensionDescriptionBytes = extensionDescription.getBytes(StandardCharsets.UTF_8);
        byte[] wire = structure(structureTag,
                item(0x4200a5, TtlvItemType.TextString.code(), extensionNameBytes),
                item(0x4200a6, TtlvItemType.Integer.code(), intBytes(extensionTag)),
                item(0x4200a7, TtlvItemType.Enumeration.code(), intBytes(extensionType)),
                item(0x420129, TtlvItemType.Integer.code(), intBytes(extensionEnumeration)),
                item(0x42012a, TtlvItemType.Boolean.code(), longBytes(1)),
                item(0x42012b, TtlvItemType.Integer.code(), intBytes(parentStructureTag)),
                item(0x42012c, TtlvItemType.TextString.code(), extensionDescriptionBytes));
        String treeHash = normalizedTreeSha256(wire);
        CodecLimits limits = CodecLimits.defaults();

        TtlvStructure fields = TtlvStructure.create();
        fields = appendTtlvItem(fields, 0x4200a5, TtlvValue.TextString(extensionNameBytes, limits), limits);
        fields = appendTtlvItem(fields, 0x4200a6, TtlvValue.Integer(extensionTag), limits);
        fields = appendTtlvItem(fields, 0x4200a7, TtlvValue.Enumeration(extensionType), limits);
        fields = appendTtlvItem(fields, 0x420129, TtlvValue.Integer(extensionEnumeration), limits);
        fields = appendTtlvItem(fields, 0x42012a, TtlvValue.Boolean(extensionAttribute), limits);
        fields = appendTtlvItem(fields, 0x42012b, TtlvValue.Integer(parentStructureTag), limits);
        fields = appendTtlvItem(fields, 0x42012c,
                TtlvValue.TextString(extensionDescriptionBytes, limits), limits);
        TtlvStructure completedFields = fields;
        try (completedFields;
                TtlvValue structureValue = TtlvValue.structure(completedFields, limits);
                RawTag raw = RawTag.fromRaw(structureTag);
                Tag tag = raw.checked();
                TtlvItem created = TtlvItem.create(tag, structureValue, limits);
                var view = created.view();
                Tag viewedTag = view.tag();
                var value = view.value();
                var structureView = value.structure()) {
            assertEquals(structureTag, viewedTag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertExtensionInformationFields(structureView, extensionNameBytes, extensionTag, extensionType,
                    extensionEnumeration, extensionAttribute, parentStructureTag, extensionDescriptionBytes);
        }
        try (TtlvItem decoded = TtlvCodec.decode(wire);
                var view = decoded.view();
                Tag tag = view.tag();
                var value = view.value();
                var structureView = value.structure()) {
            assertEquals(structureTag, tag.raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertExtensionInformationFields(structureView, extensionNameBytes, extensionTag, extensionType,
                    extensionEnumeration, extensionAttribute, parentStructureTag, extensionDescriptionBytes);
        }

        ExtensionInformation information = ExtensionInformation.create(extensionName);
        information = ExtensionInformation.with_tag(information, extensionTag);
        information = ExtensionInformation.with_type(information, TtlvItemType.TextString);
        information = ExtensionInformation.with_enumeration(information, extensionEnumeration);
        information = ExtensionInformation.with_attribute(information, extensionAttribute);
        information = ExtensionInformation.with_parent_structure_tag(information, parentStructureTag);
        information = ExtensionInformation.with_description(information, extensionDescription);
        try (ExtensionInformation completedInformation = information;
                TtlvStructure typedFields = ExtensionInformation.toTtlv(completedInformation);
                TtlvStructureView typedView = typedFields.view()) {
            assertExtensionInformationFields(typedView, extensionNameBytes, extensionTag, extensionType,
                    extensionEnumeration, extensionAttribute, parentStructureTag, extensionDescriptionBytes);
        }

        appendGenericStructureRouteEvents(events, caseId, targetId, protocolVersion, treeHash);
        appendRouteEvent(events, caseId, "java.typed-protocol.function.extension_information_to_ttlv",
                targetId, "org.kmipkit.extensions.ExtensionInformation.toTtlv", null, null,
                "encode", protocolVersion, null, treeHash, null);
    }

    private static void assertExtensionInformationFields(TtlvStructureView fields, byte[] extensionName,
            int extensionTag, int extensionType, int extensionEnumeration, boolean extensionAttribute,
            int parentStructureTag, byte[] extensionDescription) throws Exception {
        int[] tags = {0x4200a5, 0x4200a6, 0x4200a7, 0x420129, 0x42012a, 0x42012b, 0x42012c};
        TtlvItemType[] types = {TtlvItemType.TextString, TtlvItemType.Integer, TtlvItemType.Enumeration,
                TtlvItemType.Integer, TtlvItemType.Boolean, TtlvItemType.Integer, TtlvItemType.TextString};
        assertEquals(tags.length, fields.itemCount());
        for (int index = 0; index < tags.length; index++) {
            try (TtlvItemView field = fields.itemAt(index);
                    Tag tag = field.tag();
                    var value = field.value()) {
                assertEquals(tags[index], tag.raw());
                assertEquals(types[index], field.itemType());
                switch (index) {
                    case 0 -> assertArrayEquals(extensionName, value.bytes());
                    case 1 -> assertEquals(extensionTag, value.integer());
                    case 2 -> assertEquals(extensionType, value.enumeration());
                    case 3 -> assertEquals(extensionEnumeration, value.integer());
                    case 4 -> assertEquals(extensionAttribute, value.booleanValue());
                    case 5 -> assertEquals(parentStructureTag, value.integer());
                    case 6 -> assertArrayEquals(extensionDescription, value.bytes());
                    default -> throw new AssertionError("Unexpected Extension Information field index.");
                }
            }
        }
    }

    private static void appendGenericStructureRouteEvents(ArrayNode events, String caseId,
            String targetId, String treeHash) {
        appendGenericStructureRouteEvents(events, caseId, targetId, "2.1", treeHash);
    }

    private static void appendGenericStructureRouteEvents(ArrayNode events, String caseId,
            String targetId, String protocolVersion, String treeHash) {
        appendRouteEvent(events, caseId, GENERIC_STRUCTURE_ROUTE_IDS.get(0), targetId,
                "org.kmipkit.ttlv.TtlvStructure.create", null, null,
                "construct", protocolVersion, null, treeHash, null);
        appendRouteEvent(events, caseId, GENERIC_STRUCTURE_ROUTE_IDS.get(1), targetId,
                "org.kmipkit.ttlv.TtlvStructure.withItem", null, null,
                "construct", protocolVersion, null, treeHash, null);
        appendRouteEvent(events, caseId, GENERIC_STRUCTURE_ROUTE_IDS.get(2), targetId,
                "org.kmipkit.ttlv.TtlvValue.structure", null, null,
                "construct", protocolVersion, null, treeHash, null);
        appendRouteEvent(events, caseId, GENERIC_STRUCTURE_ROUTE_IDS.get(3), targetId,
                "org.kmipkit.ttlv.TtlvItem.create", null, null,
                "construct", protocolVersion, null, treeHash, null);
        appendRouteEvent(events, caseId, GENERIC_STRUCTURE_ROUTE_IDS.get(4), targetId,
                "org.kmipkit.ttlv.TtlvItem.view", null, null,
                "decode", protocolVersion, null, null, treeHash);
        appendRouteEvent(events, caseId, GENERIC_STRUCTURE_ROUTE_IDS.get(5), targetId,
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null, null, treeHash);
    }

    private static void appendMessageFieldIntegerRoutes(ArrayNode events, String caseId, String targetId,
            String protocolVersion, boolean recordValueConstructor) throws Exception {
        int rawTag = 0x42000d;
        int value = 1;
        byte[] wire = item(rawTag, 2, intBytes(value));
        String treeHash = normalizedTreeSha256(wire);
        try (TtlvValue ttlvValue = TtlvValue.Integer(value)) {
            if (recordValueConstructor) {
                appendRouteEvent(events, caseId,
                        "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.integer",
                        targetId, "org.kmipkit.ttlv.TtlvValue.Integer", null, null,
                        "construct", protocolVersion, null, treeHash, null);
            }
            appendLocalTtlvRoundtripEvents(events, caseId, targetId, rawTag,
                    TtlvItemType.Integer, ttlvValue, wire, protocolVersion, value);
        }
    }

    private static void appendMessageFieldStructureRoutes(ArrayNode events, String caseId, String targetId,
            int rawTag, byte[] wire, TtlvStructure content, String protocolVersion,
            boolean recordStructureConstructor, boolean recordRawTag) throws Exception {
        String treeHash = normalizedTreeSha256(wire);
        CodecLimits limits = CodecLimits.defaults();
        try (content;
                TtlvValue value = TtlvValue.structure(content, limits);
                RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                TtlvItem item = TtlvItem.create(tag, value, limits);
                var view = item.view();
                var itemValue = view.value();
                var structureView = itemValue.structure()) {
            assertEquals(rawTag, view.tag().raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertTrue(structureView.itemCount() > 0);
            if (recordRawTag) {
                appendRouteEvent(events, caseId,
                        "java.generic-ttlv.model.org.kmipkit.ttlv.rawtag.fromraw",
                        targetId, "org.kmipkit.ttlv.RawTag.fromRaw", null, null,
                        "construct", protocolVersion, null, treeHash, null);
                appendRouteEvent(events, caseId,
                        "java.generic-ttlv.model.org.kmipkit.ttlv.rawtag.checked",
                        targetId, "org.kmipkit.ttlv.RawTag.checked", null, null,
                        "construct", protocolVersion, null, treeHash, null);
            }
            if (recordStructureConstructor) {
                appendRouteEvent(events, caseId,
                        "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.structure",
                        targetId, "org.kmipkit.ttlv.TtlvValue.structure", null, null,
                        "construct", protocolVersion, null, treeHash, null);
            }
            appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_item.create",
                    targetId, "org.kmipkit.ttlv.TtlvItem.create", null, null,
                    "construct", protocolVersion, null, treeHash, null);
            appendRouteEvent(events, caseId, "java.generic-ttlv.function.ttlv_item_view_from_item",
                    targetId, "org.kmipkit.ttlv.TtlvItem.view", null, null,
                    "decode", protocolVersion, null, null, treeHash);
        }
        try (TtlvItem decoded = TtlvCodec.decode(wire);
                var view = decoded.view();
                var decodedValue = view.value();
                var structureView = decodedValue.structure()) {
            assertEquals(rawTag, view.tag().raw());
            assertEquals(TtlvItemType.Structure, view.itemType());
            assertTrue(structureView.itemCount() > 0);
        }
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_codec.decode",
                targetId, "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null, null, treeHash);
    }

    private static void appendNormativeRequestMessageParseEvents(ArrayNode events,
            String protocolVersion) throws Exception {
        int minorVersion = "2.0".equals(protocolVersion) ? 0 : 1;
        String treeHash = normalizedTreeSha256(requestMessageWire(minorVersion));
        try (RequestMessage message = RequestMessage.fromTtlv(requestMessageStructure(minorVersion));
                var header = message.header();
                var item = message.batchItemAt(0)) {
            assertEquals(1, header.batchCount());
            assertEquals(1, message.batchItemCount());
            assertEquals(0x18L, item.operation().orElseThrow());
            assertEquals(1L, item.extensionCount());
        }
        for (NormativeTarget field : REQUEST_MESSAGE_PARSE_TARGETS) {
            appendRouteEvent(events, field.caseId(),
                    "java.typed-protocol.model.org.kmipkit.message.requestmessage.fromttlv",
                    field.targetId(), "org.kmipkit.message.RequestMessage.fromTtlv", null, null,
                    "construct", protocolVersion, null, treeHash, null);
        }
        appendEphemeralRequestMessageParseEvent(events, protocolVersion);
    }

    private static void appendEphemeralRequestMessageParseEvent(ArrayNode events,
            String protocolVersion) throws Exception {
        int minorVersion = "2.0".equals(protocolVersion) ? 0 : 1;
        byte[] wire = ephemeralRequestMessageWire(minorVersion);
        try (RequestMessage message = RequestMessage.fromTtlv(ephemeralRequestMessageStructure(minorVersion));
                var item = message.batchItemAt(0)) {
            assertEquals(java.util.OptionalLong.of(0x0a), item.operation());
            assertEquals(java.util.Optional.of(true), item.ephemeral());
            assertEquals(java.util.Optional.of(1L), item.withRequestPayload(value -> value.itemCount()));
        }
        appendRouteEvent(events, "test-norm-message-field-message-field-8-3-ephemeral",
                "java.typed-protocol.model.org.kmipkit.message.requestmessage.fromttlv",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-3-EPHEMERAL",
                "org.kmipkit.message.RequestMessage.fromTtlv", null, null,
                "construct", protocolVersion, null, normalizedTreeSha256(wire), null);
    }

    private static TtlvStructure ephemeralRequestMessageStructure(int protocolMinorVersion) {
        CodecLimits limits = CodecLimits.defaults();
        byte[] uniqueIdentifier = "ephemeral-fixture".getBytes(StandardCharsets.US_ASCII);
        TtlvStructure version = TtlvStructure.create();
        version = appendTtlvItem(version, 0x42006a, TtlvValue.Integer(2), limits);
        version = appendTtlvItem(version, 0x42006b, TtlvValue.Integer(protocolMinorVersion), limits);
        TtlvStructure header = TtlvStructure.create();
        header = appendTtlvItem(header, 0x420069, TtlvValue.structure(version, limits), limits);
        header = appendTtlvItem(header, 0x420092, TtlvValue.DateTime(1), limits);
        header = appendTtlvItem(header, 0x42000d, TtlvValue.Integer(1), limits);

        TtlvStructure payload = TtlvStructure.create();
        payload = appendTtlvItem(payload, 0x420094,
                TtlvValue.TextString(uniqueIdentifier, limits), limits);
        TtlvStructure requestItem = TtlvStructure.create();
        requestItem = appendTtlvItem(requestItem, 0x42005c, TtlvValue.Enumeration(0x0a), limits);
        requestItem = appendTtlvItem(requestItem, 0x420154, TtlvValue.Boolean(true), limits);
        requestItem = appendTtlvItem(requestItem, 0x420093,
                TtlvValue.ByteString("item-0".getBytes(StandardCharsets.US_ASCII), limits), limits);
        requestItem = appendTtlvItem(requestItem, 0x420079, TtlvValue.structure(payload, limits), limits);

        TtlvStructure message = TtlvStructure.create();
        message = appendTtlvItem(message, 0x420077, TtlvValue.structure(header, limits), limits);
        return appendTtlvItem(message, 0x42000f, TtlvValue.structure(requestItem, limits), limits);
    }

    private static byte[] ephemeralRequestMessageWire(int protocolMinorVersion) throws Exception {
        byte[] uniqueIdentifier = "ephemeral-fixture".getBytes(StandardCharsets.US_ASCII);
        byte[] version = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x420077, version,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] payload = structure(0x420079, item(0x420094, 7, uniqueIdentifier));
        byte[] requestItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(0x0a)), item(0x420154, 6, longBytes(1)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)), payload);
        return structure(0x420078, header, requestItem);
    }

    private static TtlvStructure requestMessageStructure(int protocolMinorVersion) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure version = TtlvStructure.create();
        version = appendTtlvItem(version, 0x42006a, TtlvValue.Integer(2), limits);
        version = appendTtlvItem(version, 0x42006b, TtlvValue.Integer(protocolMinorVersion), limits);
        TtlvStructure header = TtlvStructure.create();
        header = appendTtlvItem(header, 0x420069, TtlvValue.structure(version, limits), limits);
        header = appendTtlvItem(header, 0x420092, TtlvValue.DateTime(1), limits);
        header = appendTtlvItem(header, 0x42000d, TtlvValue.Integer(1), limits);

        TtlvStructure payload = TtlvStructure.create();
        payload = appendTtlvItem(payload, 0x420074, TtlvValue.Enumeration(1), limits);
        TtlvStructure requestItem = TtlvStructure.create();
        requestItem = appendTtlvItem(requestItem, 0x42005c, TtlvValue.Enumeration(0x18), limits);
        requestItem = appendTtlvItem(requestItem, 0x420093,
                TtlvValue.ByteString("item-0".getBytes(StandardCharsets.US_ASCII), limits), limits);
        requestItem = appendTtlvItem(requestItem, 0x420079, TtlvValue.structure(payload, limits), limits);
        requestItem = appendTtlvItem(requestItem, 0x420051,
                TtlvValue.structure(messageExtensionStructure(), limits), limits);

        TtlvStructure message = TtlvStructure.create();
        message = appendTtlvItem(message, 0x420077, TtlvValue.structure(header, limits), limits);
        return appendTtlvItem(message, 0x42000f, TtlvValue.structure(requestItem, limits), limits);
    }

    private static byte[] requestMessageWire(int protocolMinorVersion) throws Exception {
        byte[] version = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x420077, version,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] requestPayload = structure(0x420079, item(0x420074, 5, intBytes(1)));
        byte[] requestItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(0x18)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                requestPayload, messageExtensionWire());
        return structure(0x420078, header, requestItem);
    }

    private static void assertBatchItemResponseRouteEvidence(ArrayNode events,
            ClientBatchItemResult result, String protocolVersion, String responseHash) throws Exception {
        try (TtlvStructure raw = result.raw().orElseThrow(); var view = raw.view()) {
            assertEquals(3, view.itemCount());
        }
        appendRouteEvent(events, "test-norm-message-field-message-field-8-6-batch-item",
                "java.typed-protocol.function.client_batch_item_response_raw_ttlv",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-BATCH-ITEM",
                "org.kmipkit.extensions.ClientBatchItemResult.raw", null, null, "decode",
                protocolVersion, null, null, responseHash);
        appendRouteEvent(events, "test-norm-message-field-message-field-9-7-batch-item",
                "java.typed-protocol.function.client_batch_item_response_raw_ttlv",
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM",
                "org.kmipkit.extensions.ClientBatchItemResult.raw", null, null, "decode",
                protocolVersion, null, null, responseHash);

        try (TtlvStructure extension = result.extensionTtlvAt(0); var view = extension.view()) {
            assertEquals(3, view.itemCount());
        }
        appendRouteEvent(events, "test-norm-message-field-message-field-8-6-message-extension",
                "java.typed-protocol.function.client_batch_item_response_extension_ttlv_at",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-MESSAGE-EXTENSION",
                "org.kmipkit.extensions.ClientBatchItemResult.extensionTtlvAt", null, null, "decode",
                protocolVersion, null, null, responseHash);
        appendRouteEvent(events, "test-norm-message-field-message-field-9-13-message-extension",
                "java.typed-protocol.function.client_batch_item_response_extension_ttlv_at",
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION",
                "org.kmipkit.extensions.ClientBatchItemResult.extensionTtlvAt", null, null, "decode",
                protocolVersion, null, null, responseHash);
        appendRouteEvent(events, "test-norm-tag-tag-420051",
                "java.typed-protocol.function.client_batch_item_response_extension_ttlv_at",
                "KMIPKIT-ELEM-TAG-420051",
                "org.kmipkit.extensions.ClientBatchItemResult.extensionTtlvAt", null, null, "decode",
                protocolVersion, null, null, responseHash);

        try (TtlvStructure header = result.responseHeaderTtlv(); var view = header.view()) {
            assertTrue(view.itemCount() >= 3);
        }
        appendRouteEvent(events, RESPONSE_BATCH_ITEM_CASE_ID,
                "java.typed-protocol.function.client_batch_item_response_header_ttlv",
                RESPONSE_BATCH_ITEM_TARGET_ID, "org.kmipkit.extensions.ClientBatchItemResult.responseHeaderTtlv",
                null, null, "decode", protocolVersion, null, null, responseHash);
        appendBatchItemFieldRouteEvents(events,
                "java.typed-protocol.function.client_batch_item_response_header_ttlv",
                "org.kmipkit.extensions.ClientBatchItemResult.responseHeaderTtlv", protocolVersion, responseHash);

        assertEquals(4, result.metadata().length);
        appendRouteEvent(events, RESPONSE_BATCH_ITEM_CASE_ID,
                "java.typed-protocol.function.client_batch_item_response_metadata_v1",
                RESPONSE_BATCH_ITEM_TARGET_ID, "org.kmipkit.extensions.ClientBatchItemResult.metadata",
                null, null, "decode", protocolVersion, null, null, responseHash);
        appendBatchItemFieldRouteEvents(events,
                "java.typed-protocol.function.client_batch_item_response_metadata_v1",
                "org.kmipkit.extensions.ClientBatchItemResult.metadata", protocolVersion, responseHash);
        assertEquals(0x18L, result.metadataV2().operation());
        appendRouteEvent(events, RESPONSE_BATCH_ITEM_CASE_ID,
                "java.typed-protocol.function.client_batch_item_response_metadata_v2",
                RESPONSE_BATCH_ITEM_TARGET_ID, "org.kmipkit.extensions.ClientBatchItemResult.metadataV2",
                null, null, "decode", protocolVersion, null, null, responseHash);
        appendBatchItemFieldRouteEvents(events,
                "java.typed-protocol.function.client_batch_item_response_metadata_v2",
                "org.kmipkit.extensions.ClientBatchItemResult.metadataV2", protocolVersion, responseHash);
        assertArrayEquals("item-0".getBytes(StandardCharsets.US_ASCII),
                result.uniqueBatchItemId().orElseThrow());
        appendRouteEvent(events, RESPONSE_BATCH_ITEM_CASE_ID,
                "java.typed-protocol.function.client_batch_item_response_unique_batch_item_id",
                RESPONSE_BATCH_ITEM_TARGET_ID, "org.kmipkit.extensions.ClientBatchItemResult.uniqueBatchItemId",
                null, null, "decode", protocolVersion, null, null, responseHash);
        appendBatchItemFieldRouteEvents(events,
                "java.typed-protocol.function.client_batch_item_response_unique_batch_item_id",
                "org.kmipkit.extensions.ClientBatchItemResult.uniqueBatchItemId", protocolVersion, responseHash);

        try (ResponseMessage message = ResponseMessage.fromTtlv(responseMessageStructure(
                "2.0".equals(protocolVersion) ? 0 : 1)); var item = message.batchItemAt(0)) {
            assertEquals(1, message.batchItemCount());
            assertEquals(0x18L, item.operation().orElseThrow());
        }
        appendRouteEvent(events, RESPONSE_BATCH_ITEM_CASE_ID,
                "java.typed-protocol.model.org.kmipkit.message.responsemessage.fromttlv",
                RESPONSE_BATCH_ITEM_TARGET_ID, "org.kmipkit.message.ResponseMessage.fromTtlv",
                null, null, "construct", protocolVersion, null, responseHash, null);
        for (NormativeTarget field : RESPONSE_MESSAGE_PARSE_TARGETS) {
            appendRouteEvent(events, field.caseId(),
                    "java.typed-protocol.model.org.kmipkit.message.responsemessage.fromttlv",
                    field.targetId(), "org.kmipkit.message.ResponseMessage.fromTtlv", null, null,
                    "construct", protocolVersion, null, responseHash, null);
        }
    }

    private static void appendBatchItemFieldRouteEvents(ArrayNode events, String routeId,
            String publicApiSymbol, String protocolVersion, String responseHash) {
        for (NormativeTarget field : BATCH_ITEM_FIELD_TARGETS) {
            appendRouteEvent(events, field.caseId(), routeId, field.targetId(), publicApiSymbol,
                    null, null, "decode", protocolVersion, null, null, responseHash);
        }
    }

    private static TtlvStructure batchItemStructure() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure payload = TtlvStructure.create();
        payload = appendTtlvItem(payload, 0x42005c, TtlvValue.Enumeration(0x3b), limits);
        payload = appendTtlvItem(payload, 0x42015f,
                TtlvValue.structure(TtlvStructure.create(), limits), limits);
        TtlvStructure item = TtlvStructure.create();
        item = appendTtlvItem(item, 0x42005c, TtlvValue.Enumeration(0x18), limits);
        item = appendTtlvItem(item, 0x420093,
                TtlvValue.ByteString("item-0".getBytes(StandardCharsets.US_ASCII), limits), limits);
        item = appendTtlvItem(item, 0x42007f, TtlvValue.Enumeration(0), limits);
        item = appendTtlvItem(item, 0x42007c, TtlvValue.structure(payload, limits), limits);
        return appendTtlvItem(item, 0x420051, TtlvValue.structure(messageExtensionStructure(), limits), limits);
    }

    private static TtlvStructure messageExtensionStructure() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure vendorExtension = TtlvStructure.create();
        vendorExtension = appendTtlvItem(vendorExtension, 0x540001,
                TtlvValue.TextString("fixture-extension".getBytes(StandardCharsets.UTF_8), limits), limits);
        TtlvStructure extension = TtlvStructure.create();
        extension = appendTtlvItem(extension, 0x42009d,
                TtlvValue.TextString("KMIPKit.fixture".getBytes(StandardCharsets.US_ASCII), limits), limits);
        extension = appendTtlvItem(extension, 0x420026, TtlvValue.Boolean(false), limits);
        return appendTtlvItem(extension, 0x42009c,
                TtlvValue.structure(vendorExtension, limits), limits);
    }

    private static TtlvStructure responseMessageStructure(int protocolMinorVersion) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure protocolVersion = TtlvStructure.create();
        protocolVersion = appendTtlvItem(protocolVersion, 0x42006a, TtlvValue.Integer(2), limits);
        protocolVersion = appendTtlvItem(protocolVersion, 0x42006b,
                TtlvValue.Integer(protocolMinorVersion), limits);
        TtlvStructure header = TtlvStructure.create();
        header = appendTtlvItem(header, 0x420069, TtlvValue.structure(protocolVersion, limits), limits);
        header = appendTtlvItem(header, 0x420092, TtlvValue.DateTime(1), limits);
        header = appendTtlvItem(header, 0x42000d, TtlvValue.Integer(1), limits);
        TtlvStructure message = TtlvStructure.create();
        message = appendTtlvItem(message, 0x42007a, TtlvValue.structure(header, limits), limits);
        return appendTtlvItem(message, 0x42000f, TtlvValue.structure(batchItemStructure(), limits), limits);
    }

    private static TtlvStructure appendTtlvItem(TtlvStructure structure, int rawTag,
            TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                value;
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }

    private static byte[] getResponse() throws Exception {
        return getResponse(1);
    }

    private static byte[] getResponse(int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] payload = getResponsePayload();
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(10)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] getResponsePayload() throws Exception {
        byte[] keyMaterial = new byte[32];
        java.util.Arrays.fill(keyMaterial, (byte) 0x5a);
        byte[] keyBlock = structure(0x420040,
                item(0x420042, 5, intBytes(1)),
                structure(0x420043, item(0x420044, 8, keyMaterial)),
                item(0x420028, 5, intBytes(3)),
                item(0x42002a, 2, intBytes(256)));
        byte[] payload = structure(0x42007c,
                item(0x420057, 5, intBytes(2)),
                item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8)),
                structure(0x42008f, keyBlock));
        return payload;
    }

    private static byte[] identifierResponse(long operation) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] identifier = item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8));
        byte[] payload = operation == 0x0c
                ? structure(0x42007c, identifier, item(0x42013b, 5, intBytes(0x42002f)))
                : structure(0x42007c, identifier);
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(Math.toIntExact(operation))),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] logResponse() throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(0x2c)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), structure(0x42007c));
        return structure(0x42007b, header, responseItem);
    }

    private static List<byte[]> serveCreateCleanup(SSLServerSocket listener) throws Exception {
        var requests = new ArrayList<byte[]>();
        for (int exchange = 0; exchange < 2; exchange++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readTtlv(socket.getInputStream());
                requests.add(request);
                byte[] operation = new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4,
                        0, 0, 0, (byte) (exchange == 0 ? 1 : 0x14)};
                if (!contains(request, operation)) {
                    throw new AssertionError("The controlled create recipe sent an unexpected operation.");
                }
                byte[] response = exchange == 0 ? createResponse() : destroyResponse();
                socket.getOutputStream().write(response);
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(requests);
    }

    private static List<byte[]> serveObjectFixtureAndGet(SSLServerSocket listener) throws Exception {
        var requests = new ArrayList<byte[]>();
        byte[][] responses = {createResponse(), getResponse(), destroyResponse()};
        for (byte[] response : responses) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                requests.add(readTtlv(socket.getInputStream()));
                socket.getOutputStream().write(response);
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(requests);
    }

    private static List<byte[]> serveAttributeReadCases(SSLServerSocket listener,
            List<Long> targetOperations, List<Long> targetAttributeTags) throws Exception {
        return serveAttributeReadCases(listener, targetOperations, targetAttributeTags, targetAttributeTags,
                java.util.Collections.nCopies(targetOperations.size(), 3L));
    }

    private static List<byte[]> serveAttributeReadCases(SSLServerSocket listener,
            List<Long> targetOperations, List<Long> targetAttributeTags, List<Long> responseAttributeTags)
            throws Exception {
        return serveAttributeReadCases(listener, targetOperations, targetAttributeTags, responseAttributeTags,
                java.util.Collections.nCopies(targetOperations.size(), 3L));
    }

    private static List<byte[]> serveAttributeReadCases(SSLServerSocket listener,
            List<Long> targetOperations, List<Long> targetAttributeTags, List<Long> responseAttributeTags,
            List<Long> responseAlgorithmValues) throws Exception {
        var requests = new ArrayList<byte[]>();
        for (int index = 0; index < targetOperations.size(); index++) {
            long targetOperation = targetOperations.get(index);
            byte[] createRequest = serveOneAndCapture(listener, createResponse());
            requests.add(createRequest);
            String fixtureName = requestTextString(createRequest, 0x420055);
            requests.add(serveOneAndCapture(listener, getAttributesResponse(targetOperation,
                    responseAttributeTags.get(index), responseAlgorithmValues.get(index), fixtureName)));
            requests.add(serveOneAndCapture(listener, destroyResponse()));
        }
        return List.copyOf(requests);
    }

    private static byte[] serveOneAndCapture(SSLServerSocket listener, byte[] response) throws Exception {
        try (SSLSocket socket = (SSLSocket) listener.accept()) {
            socket.setEnabledProtocols(new String[] {"TLSv1.3"});
            socket.setSoTimeout(10_000);
            socket.startHandshake();
            byte[] request = readTtlv(socket.getInputStream());
            socket.getOutputStream().write(response);
            socket.getOutputStream().flush();
            return request;
        }
    }

    private static String requestTextString(byte[] request, int tag) {
        byte[] header = new byte[] {(byte) ((tag >>> 16) & 0xff), (byte) ((tag >>> 8) & 0xff),
                (byte) (tag & 0xff), 7, 0, 0, 0, 0};
        for (int offset = 0; offset <= request.length - header.length; offset++) {
            boolean matches = true;
            for (int index = 0; index < 4; index++) {
                if (request[offset + index] != header[index]) {
                    matches = false;
                    break;
                }
            }
            if (!matches) continue;
            int length = ByteBuffer.wrap(request, offset + 4, 4).getInt();
            if (length < 0 || offset + 8 + length > request.length) {
                throw new AssertionError("The Create request contained an invalid Name Value length.");
            }
            return new String(request, offset + 8, length, StandardCharsets.UTF_8);
        }
        throw new AssertionError("The Create request omitted its Name Value.");
    }

    private static byte[] getAttributesResponse(long operation, long attributeTag, long algorithm,
            String fixtureName) throws Exception {
        byte[] value;
        if (attributeTag == 0x420028L) {
            value = item(0x420028, 5, intBytes(Math.toIntExact(algorithm)));
        } else if (attributeTag == 0x420053L) {
            value = structure(0x420053,
                    item(0x420055, 7, fixtureName.getBytes(StandardCharsets.UTF_8)),
                    item(0x420054, 5, intBytes(1)));
        } else {
            throw new IllegalArgumentException("No controlled attribute fixture is defined.");
        }
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] attributes = structure(0x420125, value);
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8)), attributes);
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(Math.toIntExact(operation))),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] createResponse() throws Exception {
        return createResponse(1);
    }

    private static byte[] createResponse(int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] payload = structure(0x42007c,
                item(0x420057, 5, intBytes(2)),
                item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8)));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(1)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] destroyResponse() throws Exception {
        return destroyResponse(1);
    }

    private static byte[] destroyResponse(int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8)));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(0x14)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] structure(int tag, byte[]... children) throws Exception {
        ByteArrayOutputStream body = new ByteArrayOutputStream();
        for (byte[] child : children) body.write(child);
        return item(tag, 1, body.toByteArray());
    }

    private static byte[] item(int tag, int type, byte[] value) throws Exception {
        ByteArrayOutputStream item = new ByteArrayOutputStream();
        item.write((tag >>> 16) & 0xff);
        item.write((tag >>> 8) & 0xff);
        item.write(tag & 0xff);
        item.write(type);
        item.write(ByteBuffer.allocate(4).putInt(value.length).array());
        item.write(value);
        int padding = (8 - value.length % 8) % 8;
        item.write(new byte[padding]);
        return item.toByteArray();
    }

    private static byte[] intBytes(int value) {
        return ByteBuffer.allocate(4).putInt(value).array();
    }

    private static byte[] longBytes(long value) {
        return ByteBuffer.allocate(8).putLong(value).array();
    }

    private static X509Certificate certificate(byte[] pem) throws Exception {
        return (X509Certificate) CertificateFactory.getInstance("X.509")
                .generateCertificate(new java.io.ByteArrayInputStream(pem));
    }

    private static PrivateKey privateKey(byte[] pem) throws Exception {
        String encoded = new String(pem, StandardCharsets.US_ASCII)
                .replace("-----BEGIN PRIVATE KEY-----", "")
                .replace("-----END PRIVATE KEY-----", "").replaceAll("\\s", "");
        return KeyFactory.getInstance("RSA").generatePrivate(
                new PKCS8EncodedKeySpec(Base64.getDecoder().decode(encoded)));
    }

    private static byte[] fixture(String name) throws Exception {
        Path path = Path.of("../../bindings/python/tests/fixtures/login-mtls", name);
        return Files.readAllBytes(path);
    }

    private static boolean contains(byte[] data, byte[] target) {
        outer: for (int index = 0; index <= data.length - target.length; index++) {
            for (int offset = 0; offset < target.length; offset++) {
                if (data[index + offset] != target[offset]) continue outer;
            }
            return true;
        }
        return false;
    }

    private static boolean containsTag(byte[] data, int tag) {
        byte[] tagBytes = new byte[] {(byte) ((tag >>> 16) & 0xff), (byte) ((tag >>> 8) & 0xff),
                (byte) (tag & 0xff)};
        outer: for (int offset = 0; offset <= data.length - 4; offset++) {
            for (int index = 0; index < tagBytes.length; index++) {
                if (data[offset + index] != tagBytes[index]) continue outer;
            }
            if (data[offset + 3] == 5) return true;
        }
        return false;
    }

    private record TagRangeFixture(String caseId, String targetId, int rawTag, String allocation) {}

    private record EnumerationFixture(String caseId, String targetId, String parentName, int rawTag, long value,
            StandardAttribute attribute, List<String> protocolVersions) {}

    private record LocalEnumerationScope(String caseId, List<String> targetIds, List<String> protocolVersions,
            List<String> additionalRoutes) {}

    private record BitmaskFixture(String caseId, String targetId, int rawTag, long value,
            StandardAttribute attribute, List<String> protocolVersions) {}

    private record DataTypeFixture(String caseId, String targetId, String dataType,
            int typeCode, List<String> protocolVersions) {}

    private record MatrixPlan(List<Long> expectedOperations, List<byte[]> responses) {}
}
