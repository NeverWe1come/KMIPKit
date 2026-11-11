package org.kmipkit.validator;

import com.fasterxml.jackson.core.StreamReadFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.IOException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

/** Loads and validates the shared, versioned interoperability catalog. */
public final class CatalogLoader {
    private static final Set<String> PROTOCOL_VERSIONS = Set.of("2.0", "2.1");
    private static final Set<String> NORMATIVE_DOCUMENTS = Set.of(
            "OASIS KMIP 2.0 Specification", "OASIS KMIP 2.1 Specification");
    private static final ObjectMapper JSON = new ObjectMapper(
            com.fasterxml.jackson.core.JsonFactory.builder()
                    .enable(StreamReadFeature.STRICT_DUPLICATE_DETECTION)
                    .build());

    private CatalogLoader() { }

    /** Reads a catalog and rejects incomplete or inconsistent cases. */
    public static Catalog load(Path path) throws IOException {
        JsonNode root = JSON.readTree(path.toFile());
        require(root != null && root.isObject(), "Catalog root must be an object");

        String version = requiredText(root, "version", "catalog");
        List<String> targetVersions = textArray(root, "target_versions", "catalog");
        require(!targetVersions.isEmpty(), "Catalog target_versions must not be empty");
        require(new HashSet<>(targetVersions).size() == targetVersions.size(),
                "Catalog target_versions must be unique");
        require(PROTOCOL_VERSIONS.containsAll(targetVersions),
                "Catalog contains a protocol version outside KMIP 2.0 and 2.1");
        require("probe_all_applicable".equals(requiredText(root, "capability_policy", "catalog")),
                "Catalog capability_policy must be probe_all_applicable");

        JsonNode policy = objectField(root, "execution_policy", "catalog");
        require(requiredBoolean(policy, "requires_dedicated_test_environment", "execution_policy"),
                "Catalog must require a dedicated test environment");
        require(requiredBoolean(policy, "mutating_cases_require_explicit_enablement", "execution_policy"),
                "Catalog must require explicit mutation enablement");
        require(requiredBoolean(policy, "never_retry_requests", "execution_policy"),
                "Catalog must prohibit automatic retries");

        JsonNode statistics = objectField(root, "statistics", "catalog");
        int declaredTotal = requiredInt(statistics, "total_cases", "statistics");
        JsonNode casesNode = root.get("test_cases");
        require(casesNode != null && casesNode.isArray(), "Catalog test_cases must be an array");
        require(declaredTotal == casesNode.size(), "Catalog statistics.total_cases does not match test_cases");
        require(!casesNode.isEmpty(), "Catalog must contain test cases");

        List<CatalogCase> cases = new ArrayList<>(casesNode.size());
        Set<String> caseIds = new HashSet<>();
        for (int index = 0; index < casesNode.size(); index++) {
            JsonNode caseNode = casesNode.get(index);
            String location = "test_cases[" + index + "]";
            require(caseNode != null && caseNode.isObject(), location + " must be an object");
            String id = requiredText(caseNode, "case_id", location);
            require(caseIds.add(id), "Duplicate case_id: " + id);
            String category = requiredText(caseNode, "category", location);
            String operation = requiredText(caseNode, "operation", location);
            String description = optionalText(caseNode, "description");

            JsonNode refsNode = caseNode.get("normative_refs");
            require(refsNode != null && refsNode.isArray() && !refsNode.isEmpty(),
                    location + " must have normative_refs");
            List<NormativeReference> refs = new ArrayList<>();
            for (int refIndex = 0; refIndex < refsNode.size(); refIndex++) {
                JsonNode ref = refsNode.get(refIndex);
                String refLocation = location + ".normative_refs[" + refIndex + "]";
                String document = requiredText(ref, "document", refLocation);
                String section = requiredText(ref, "section", refLocation);
                require(NORMATIVE_DOCUMENTS.contains(document), refLocation + " has an unsupported OASIS document");
                require(section.matches("[0-9]+(\\.[0-9]+)*"), refLocation + " has an invalid OASIS section");
                refs.add(new NormativeReference(document, section));
            }

            JsonNode execution = objectField(caseNode, "execution", location);
            String mode = requiredText(execution, "mode", location + ".execution");
            require(Set.of("server_exchange", "local_model").contains(mode),
                    location + ".execution.mode is invalid");
            List<String> versions = textArray(execution, "protocol_versions", location + ".execution");
            require(!versions.isEmpty(), location + ".execution.protocol_versions must not be empty");
            require(PROTOCOL_VERSIONS.containsAll(versions), location + " contains an unsupported protocol version");
            require(targetVersions.containsAll(versions), location + " references a version absent from target_versions");
            List<String> steps = textArray(execution, "steps", location + ".execution");
            List<String> assertions = textArray(execution, "assertions", location + ".execution");
            require(!steps.isEmpty(), location + ".execution.steps must not be empty");
            require(!assertions.isEmpty(), location + ".execution.assertions must not be empty");
            String mutation = requiredText(execution, "mutation", location + ".execution");
            require(Set.of("read_only", "mutating").contains(mutation), location + ".execution.mutation is invalid");
            String status = requiredText(execution, "status", location + ".execution");
            require(Set.of("planned", "implemented", "not_applicable").contains(status),
                    location + ".execution.status is invalid");
            String notApplicableReason = optionalText(execution, "not_applicable_reason");
            require(!"not_applicable".equals(status) || !notApplicableReason.isBlank(),
                    location + " must explain why it is not applicable");
            JsonNode requires = caseNode.get("requires");
            require(requires != null && requires.isObject(), location + ".requires must be an object");
            long attributeTag = resolveAttributeTag(root.path("normative_targets"), requires, location);
            JsonNode isolation = objectField(caseNode, "isolation_recipe", location);
            String uniquePrefix = requiredText(isolation, "unique_name_prefix", location + ".isolation_recipe");
            List<String> setupSteps = textArray(isolation, "setup_steps", location + ".isolation_recipe");
            String targetStep = requiredText(isolation, "target_step", location + ".isolation_recipe");
            List<String> teardownSteps = textArray(isolation, "teardown_steps", location + ".isolation_recipe");
            cases.add(new CatalogCase(id, category, operation, description, refs, requires.deepCopy(), attributeTag,
                    new IsolationRecipe(uniquePrefix, setupSteps, targetStep, teardownSteps),
                    new Execution(mode, versions, steps, assertions, mutation, status, notApplicableReason)));
        }

        Map<String, JsonNode> normativeTargets = new LinkedHashMap<>();
        JsonNode targetsNode = root.path("normative_targets");
        if (targetsNode.isArray()) {
            for (int index = 0; index < targetsNode.size(); index++) {
                JsonNode target = targetsNode.get(index);
                String targetId = requiredText(target, "target_id", "normative_targets[" + index + "]");
                require(normativeTargets.putIfAbsent(targetId, target.deepCopy()) == null,
                        "Duplicate normative target ID: " + targetId);
            }
        }

        return new Catalog(version, targetVersions, declaredTotal, cases, Map.copyOf(normativeTargets));
    }

    private static long resolveAttributeTag(JsonNode targets, JsonNode requires, String location) {
        JsonNode attributeName = requires.get("attribute");
        if (attributeName == null) return 0x42000aL;
        require(attributeName.isTextual() && !attributeName.textValue().isBlank(),
                location + ".requires.attribute must be a non-empty string");
        String attributeTargetId = requiredText(requires, "normative_target_id", location + ".requires");
        JsonNode attributeTarget = findTarget(targets, attributeTargetId);
        require(attributeTarget != null && "attribute".equals(optionalText(attributeTarget, "kind")),
                location + " references a non-attribute normative target");
        require(attributeName.textValue().equals(optionalText(attributeTarget, "name")),
                location + " attribute name differs from its normative target");
        JsonNode parents = attributeTarget.get("parent_element_ids");
        if (parents == null || !parents.isArray() || parents.isEmpty()) {
            // Some KMIP attribute definitions describe a group of certificate fields or
            // the name-based Vendor Attribute form and have no single Attribute Tag.
            return 0;
        }
        require(parents.get(0).isTextual(), location + " attribute parent ID must be text");
        JsonNode tagTarget = findTarget(targets, parents.get(0).textValue());
        require(tagTarget != null && "tag".equals(optionalText(tagTarget, "kind")),
                location + " attribute parent is not a KMIP tag");
        String wireValue = requiredText(tagTarget, "wire_value", location + ".attribute_tag");
        try {
            String hexadecimal = wireValue.startsWith("0x") || wireValue.startsWith("0X")
                    ? wireValue.substring(2) : wireValue;
            return Long.parseUnsignedLong(hexadecimal, 16);
        } catch (NumberFormatException exception) {
            throw new IllegalArgumentException(location + " attribute tag is not hexadecimal", exception);
        }
    }

    private static JsonNode findTarget(JsonNode targets, String targetId) {
        if (!targets.isArray()) return null;
        for (JsonNode target : targets) {
            if (targetId.equals(optionalText(target, "target_id"))) return target;
        }
        return null;
    }

    private static JsonNode objectField(JsonNode parent, String field, String location) {
        JsonNode value = parent.get(field);
        require(value != null && value.isObject(), location + "." + field + " must be an object");
        return value;
    }

    private static String requiredText(JsonNode parent, String field, String location) {
        JsonNode value = parent == null ? null : parent.get(field);
        require(value != null && value.isTextual() && !value.textValue().isBlank(),
                location + "." + field + " must be a non-empty string");
        return value.textValue();
    }

    private static String optionalText(JsonNode parent, String field) {
        JsonNode value = parent.get(field);
        return value != null && value.isTextual() ? value.textValue() : "";
    }

    private static boolean requiredBoolean(JsonNode parent, String field, String location) {
        JsonNode value = parent.get(field);
        require(value != null && value.isBoolean(), location + "." + field + " must be a boolean");
        return value.booleanValue();
    }

    private static int requiredInt(JsonNode parent, String field, String location) {
        JsonNode value = parent.get(field);
        require(value != null && value.canConvertToInt(), location + "." + field + " must be an integer");
        return value.intValue();
    }

    private static List<String> textArray(JsonNode parent, String field, String location) {
        JsonNode value = parent.get(field);
        require(value != null && value.isArray(), location + "." + field + " must be an array");
        List<String> result = new ArrayList<>();
        for (int index = 0; index < value.size(); index++) {
            JsonNode item = value.get(index);
            require(item != null && item.isTextual() && !item.textValue().isBlank(),
                    location + "." + field + "[" + index + "] must be a non-empty string");
            result.add(item.textValue());
        }
        return List.copyOf(result);
    }

    private static void require(boolean condition, String message) {
        if (!condition) {
            throw new IllegalArgumentException(message);
        }
    }

    public record Catalog(String version, List<String> targetVersions, int totalCases, List<CatalogCase> cases,
                          Map<String, JsonNode> normativeTargets) { }
    public record NormativeReference(String document, String section) { }
    public record IsolationRecipe(String uniqueNamePrefix, List<String> setupSteps, String targetStep,
                                 List<String> teardownSteps) { }
    public record Execution(String mode, List<String> protocolVersions, List<String> steps,
                            List<String> assertions, String mutation, String status,
                            String notApplicableReason) { }
    public record CatalogCase(String caseId, String category, String operation, String description,
                              List<NormativeReference> normativeRefs, JsonNode requires, long attributeTag,
                              IsolationRecipe isolationRecipe, Execution execution) { }
}
