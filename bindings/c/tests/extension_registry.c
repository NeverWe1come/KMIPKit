/*
 * Red-stage public C consumer coverage for KMIPKIT-0012 User Story 3.
 *
 * This test checks that the shared corpus names and values remain present, then
 * constructs the C TTLV equivalent of `valid-recognized` from those values.
 * Its consumer assertions run through the public read-only view API.
 * KMIP Specification v2.1 §11.56 permits the 0x54 extension Tag range; the
 * catalog policy accepts those unknown Tags for generic preservation.
 *
 * The current public C header has no ClientBatchItem constructor or request
 * encoder/getter, so a C consumer cannot observe emitted outbound order yet.
 * It can still test explicit Criticality Indicator validation. The C ABI
 * exposes only stable int32_t error categories, not diagnostic strings, so
 * payload redaction is constrained to verifying stable category-only results.
 * Use-after-release, dangling, and foreign pointers are caller-precondition
 * violations and are deliberately not probed here.
 */

#include "kmipkit.h"

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define KMIPKIT_SUCCESS ((int32_t)0)
#define KMIPKIT_EXTENSION_TAG UINT32_C(0x540001)
#define KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR UINT32_C(0x420001)
#define KMIPKIT_PAYLOAD_TAG_ENUMERATION UINT32_C(0x420002)
#define KMIPKIT_PAYLOAD_TAG_VALUE UINT32_C(0x420004)
#define KMIPKIT_PAYLOAD_TAG_ORDER_MARKER UINT32_C(0x420006)
#define INVALID_INPUT_PTR ((const uint8_t *)(uintptr_t)1)

_Static_assert(sizeof(uint64_t) == 8, "the C ABI requires 64-bit byte lengths");

#define REQUIRE(condition)                                                        \
    do {                                                                          \
        if (!(condition)) {                                                       \
            fprintf(stderr, "FAIL %s: %s\n", __func__, #condition);             \
            return false;                                                         \
        }                                                                         \
    } while (0)

#define REQUIRE_STATUS(expression, expected_status)                               \
    do {                                                                          \
        const int32_t actual_status = (expression);                              \
        if (actual_status != (expected_status)) {                                \
            fprintf(stderr, "FAIL %s: %s returned %ld, expected %ld\n",         \
                    __func__, #expression, (long)actual_status,                  \
                    (long)(expected_status));                                    \
            return false;                                                         \
        }                                                                         \
    } while (0)

static bool load_fixture_marker(const char *fixture_path, const char *marker)
{
    FILE *fixture = fopen(fixture_path, "rb");
    long file_length;
    char *contents;
    size_t bytes_read;
    bool found;

    if (fixture == NULL || fseek(fixture, 0, SEEK_END) != 0) {
        if (fixture != NULL) {
            fclose(fixture);
        }
        fprintf(stderr, "FAIL %s: cannot open shared fixture corpus\n", __func__);
        return false;
    }

    file_length = ftell(fixture);
    if (file_length < 0 || fseek(fixture, 0, SEEK_SET) != 0) {
        fclose(fixture);
        fprintf(stderr, "FAIL %s: cannot read shared fixture corpus\n", __func__);
        return false;
    }

    contents = (char *)malloc((size_t)file_length + 1U);
    if (contents == NULL) {
        fclose(fixture);
        fprintf(stderr, "FAIL %s: cannot allocate fixture buffer\n", __func__);
        return false;
    }
    bytes_read = fread(contents, 1U, (size_t)file_length, fixture);
    contents[bytes_read] = '\0';
    found = bytes_read == (size_t)file_length && strstr(contents, marker) != NULL;
    free(contents);
    fclose(fixture);

    if (!found) {
        fprintf(stderr, "FAIL %s: required shared fixture marker is missing\n", __func__);
    }
    return found;
}

static bool load_default_limits(kmipkit_extension_registry_limits_t *limits)
{
    return kmipkit_extension_registry_limits_default_values(
               &limits->max_definitions,
               &limits->max_schema_nodes,
               &limits->max_child_rules_per_structure,
               &limits->max_text_bytes_per_field,
               &limits->max_registry_text_bytes,
               &limits->max_discriminator_scalar_bytes,
               &limits->max_total_discriminator_scalar_bytes,
               &limits->max_constraint_members_per_rule,
               &limits->max_total_constraint_members,
               &limits->max_payload_index_records,
               &limits->max_lookup_comparisons,
               &limits->max_depth) == KMIPKIT_SUCCESS;
}

static int32_t validate_limits(const kmipkit_extension_registry_limits_t *limits)
{
    return kmipkit_extension_registry_limits_validate(
        limits->max_definitions,
        limits->max_schema_nodes,
        limits->max_child_rules_per_structure,
        limits->max_text_bytes_per_field,
        limits->max_registry_text_bytes,
        limits->max_discriminator_scalar_bytes,
        limits->max_total_discriminator_scalar_bytes,
        limits->max_constraint_members_per_rule,
        limits->max_total_constraint_members,
        limits->max_payload_index_records,
        limits->max_lookup_comparisons,
        limits->max_depth);
}

static void copy_limit_fields(const kmipkit_extension_registry_limits_t *limits,
                              uint64_t fields[12])
{
    fields[0] = limits->max_definitions;
    fields[1] = limits->max_schema_nodes;
    fields[2] = limits->max_child_rules_per_structure;
    fields[3] = limits->max_text_bytes_per_field;
    fields[4] = limits->max_registry_text_bytes;
    fields[5] = limits->max_discriminator_scalar_bytes;
    fields[6] = limits->max_total_discriminator_scalar_bytes;
    fields[7] = limits->max_constraint_members_per_rule;
    fields[8] = limits->max_total_constraint_members;
    fields[9] = limits->max_payload_index_records;
    fields[10] = limits->max_lookup_comparisons;
    fields[11] = limits->max_depth;
}

static void set_limit_field(kmipkit_extension_registry_limits_t *limits,
                            size_t index,
                            uint64_t value)
{
    switch (index) {
    case 0U: limits->max_definitions = value; break;
    case 1U: limits->max_schema_nodes = value; break;
    case 2U: limits->max_child_rules_per_structure = value; break;
    case 3U: limits->max_text_bytes_per_field = value; break;
    case 4U: limits->max_registry_text_bytes = value; break;
    case 5U: limits->max_discriminator_scalar_bytes = value; break;
    case 6U: limits->max_total_discriminator_scalar_bytes = value; break;
    case 7U: limits->max_constraint_members_per_rule = value; break;
    case 8U: limits->max_total_constraint_members = value; break;
    case 9U: limits->max_payload_index_records = value; break;
    case 10U: limits->max_lookup_comparisons = value; break;
    case 11U: limits->max_depth = value; break;
    default: break;
    }
}

static bool test_extension_registry_limits(void)
{
    static const uint64_t defaults[12] = {
        UINT64_C(256), UINT64_C(16384), UINT64_C(256), UINT64_C(4096),
        UINT64_C(1048576), UINT64_C(4096), UINT64_C(1048576), UINT64_C(256),
        UINT64_C(16384), UINT64_C(200000), UINT64_C(1048576), UINT64_C(64)
    };
    static const uint64_t hard_maxima[12] = {
        UINT64_C(1024), UINT64_C(100000), UINT64_C(4096), UINT64_C(4096),
        UINT64_C(16777216), UINT64_C(4096), UINT64_C(16777216), UINT64_C(4096),
        UINT64_C(100000), UINT64_C(200000), UINT64_C(4194304), UINT64_C(64)
    };
    kmipkit_extension_registry_limits_t limits;
    uint64_t observed_defaults[12];
    size_t index;

    REQUIRE(load_default_limits(&limits));
    REQUIRE_STATUS(validate_limits(&limits), KMIPKIT_SUCCESS);
    copy_limit_fields(&limits, observed_defaults);
    REQUIRE(memcmp(observed_defaults, defaults, sizeof(defaults)) == 0);

    for (index = 0; index < 12U; ++index) {
        uint64_t value;
        kmipkit_extension_registry_limits_t candidate;

        candidate = limits;
        value = defaults[index] - 1U;
        set_limit_field(&candidate, index, value);
        REQUIRE_STATUS(validate_limits(&candidate), KMIPKIT_SUCCESS);

        candidate = limits;
        set_limit_field(&candidate, index, hard_maxima[index]);
        REQUIRE_STATUS(validate_limits(&candidate), KMIPKIT_SUCCESS);

        candidate = limits;
        if (defaults[index] < hard_maxima[index]) {
            set_limit_field(&candidate, index, defaults[index] + 1U);
            REQUIRE_STATUS(validate_limits(&candidate), KMIPKIT_SUCCESS);
        } else {
            set_limit_field(&candidate, index, hard_maxima[index] + 1U);
            REQUIRE_STATUS(validate_limits(&candidate), KMIPKIT_ERROR_RESOURCE_LIMIT);
        }

        candidate = limits;
        set_limit_field(&candidate, index, hard_maxima[index] + 1U);
        REQUIRE_STATUS(validate_limits(&candidate), KMIPKIT_ERROR_RESOURCE_LIMIT);
    }
    return true;
}

static bool create_fixture_definition(kmipkit_codec_limits_t *codec_limits,
                                      const uint8_t *name,
                                      uint64_t name_length,
                                      uint32_t discriminator_tag,
                                      const uint8_t *discriminator_text,
                                      uint64_t discriminator_length,
                                      kmipkit_extension_identity_t **out_identity,
                                      kmipkit_extension_definition_t **out_definition)
{
    static const uint8_t vendor[] = "example.vendor";
    static const uint8_t version[] = "1";
    static const uint8_t compatibility_version[] = "1.0.0";
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_extension_compatibility_t *compatibility = NULL;
    kmipkit_ttlv_path_t *path = NULL;
    kmipkit_ttlv_value_t *discriminator_value = NULL;
    kmipkit_extension_discriminator_t *discriminator = NULL;
    kmipkit_extension_schema_t *text_schema = NULL;
    kmipkit_extension_schema_t *integer_schema = NULL;
    kmipkit_extension_schema_t *bounded_integer_schema = NULL;
    kmipkit_extension_schema_t *enumeration_schema = NULL;
    kmipkit_extension_schema_t *one_value_enumeration_schema = NULL;
    kmipkit_extension_schema_t *allowed_enumeration_schema = NULL;
    kmipkit_extension_child_rule_t *text_rule = NULL;
    kmipkit_extension_child_rule_t *integer_rule = NULL;
    kmipkit_extension_child_rule_t *enumeration_rule = NULL;
    kmipkit_extension_schema_t *root_schema = NULL;
    kmipkit_extension_definition_t *definition = NULL;
    kmipkit_extension_child_rule_t *children[3];
    bool succeeded = false;

    *out_identity = NULL;
    *out_definition = NULL;

    if (kmipkit_extension_identity_create(
            vendor, (uint64_t)(sizeof(vendor) - 1U), name, name_length,
            version, (uint64_t)(sizeof(version) - 1U), &identity) != KMIPKIT_SUCCESS ||
        kmipkit_extension_compatibility_create(
            2U, 1U, 2U, 1U, compatibility_version,
            (uint64_t)(sizeof(compatibility_version) - 1U), compatibility_version,
            (uint64_t)(sizeof(compatibility_version) - 1U), &compatibility) != KMIPKIT_SUCCESS ||
        kmipkit_ttlv_path_create(discriminator_tag, &path) != KMIPKIT_SUCCESS ||
        kmipkit_ttlv_value_text_string(codec_limits, discriminator_text,
            discriminator_length, &discriminator_value) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    {
        const int32_t status = kmipkit_extension_discriminator_create(path,
            discriminator_value, &discriminator);
        path = NULL;
        discriminator_value = NULL; /* Both handles are consumed by this call. */
        if (status != KMIPKIT_SUCCESS) {
            goto cleanup;
        }
    }
    if (kmipkit_extension_schema_scalar(KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
            &text_schema) != KMIPKIT_SUCCESS ||
        kmipkit_extension_child_rule_required(discriminator_tag, text_schema,
            &text_rule) != KMIPKIT_SUCCESS ||
        kmipkit_extension_schema_scalar(KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER,
            &integer_schema) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    {
        const int32_t status = kmipkit_extension_schema_signed_numeric_range(
            integer_schema, 0, 99, &bounded_integer_schema);
        integer_schema = NULL; /* The range builder consumes its source schema. */
        if (status != KMIPKIT_SUCCESS) {
            goto cleanup;
        }
    }
    if (kmipkit_extension_child_rule_required(KMIPKIT_PAYLOAD_TAG_VALUE,
            bounded_integer_schema, &integer_rule) != KMIPKIT_SUCCESS ||
        kmipkit_extension_schema_scalar(KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION,
            &enumeration_schema) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    {
        const int32_t status = kmipkit_extension_schema_allowed_enumeration(
            enumeration_schema, UINT32_MAX, &one_value_enumeration_schema);
        enumeration_schema = NULL; /* Enumeration constraints consume their schema. */
        if (status != KMIPKIT_SUCCESS) {
            goto cleanup;
        }
    }
    {
        const int32_t status = kmipkit_extension_schema_allowed_enumeration(
            one_value_enumeration_schema, UINT32_C(314159),
            &allowed_enumeration_schema);
        one_value_enumeration_schema = NULL;
        if (status != KMIPKIT_SUCCESS) {
            goto cleanup;
        }
    }
    if (
        kmipkit_extension_child_rule_required(KMIPKIT_PAYLOAD_TAG_ENUMERATION,
            allowed_enumeration_schema, &enumeration_rule) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }

    children[0] = text_rule;
    children[1] = enumeration_rule;
    children[2] = integer_rule;
    if (kmipkit_extension_schema_structure(children, 3U, NULL, 0U, 1U,
            &root_schema) != KMIPKIT_SUCCESS ||
        kmipkit_extension_definition_create(identity, compatibility, discriminator,
            root_schema, &definition) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }

    *out_identity = identity;
    *out_definition = definition;
    identity = NULL;
    definition = NULL;
    succeeded = true;

cleanup:
    if (identity != NULL) kmipkit_extension_identity_release(identity);
    if (compatibility != NULL) kmipkit_extension_compatibility_release(compatibility);
    if (path != NULL) kmipkit_ttlv_path_release(path);
    if (discriminator_value != NULL) kmipkit_ttlv_value_release(discriminator_value);
    if (discriminator != NULL) kmipkit_extension_discriminator_release(discriminator);
    if (text_schema != NULL) kmipkit_extension_schema_release(text_schema);
    if (integer_schema != NULL) kmipkit_extension_schema_release(integer_schema);
    if (bounded_integer_schema != NULL) kmipkit_extension_schema_release(bounded_integer_schema);
    if (enumeration_schema != NULL) kmipkit_extension_schema_release(enumeration_schema);
    if (one_value_enumeration_schema != NULL) kmipkit_extension_schema_release(one_value_enumeration_schema);
    if (allowed_enumeration_schema != NULL) kmipkit_extension_schema_release(allowed_enumeration_schema);
    if (text_rule != NULL) kmipkit_extension_child_rule_release(text_rule);
    if (integer_rule != NULL) kmipkit_extension_child_rule_release(integer_rule);
    if (enumeration_rule != NULL) kmipkit_extension_child_rule_release(enumeration_rule);
    if (root_schema != NULL) kmipkit_extension_schema_release(root_schema);
    if (definition != NULL) kmipkit_extension_definition_release(definition);
    return succeeded;
}

static int32_t create_registry_status(
    kmipkit_extension_definition_t **definitions,
    uint64_t definition_count,
    const kmipkit_extension_registry_limits_t *limits,
    kmipkit_client_extension_registry_t **out_registry)
{
    return kmipkit_client_extension_registry_create(
               definitions, definition_count,
               limits->max_definitions,
               limits->max_schema_nodes,
               limits->max_child_rules_per_structure,
               limits->max_text_bytes_per_field,
               limits->max_registry_text_bytes,
               limits->max_discriminator_scalar_bytes,
               limits->max_total_discriminator_scalar_bytes,
               limits->max_constraint_members_per_rule,
               limits->max_total_constraint_members,
               limits->max_payload_index_records,
               limits->max_lookup_comparisons,
               limits->max_depth,
               out_registry);
}

static bool create_registry(kmipkit_extension_definition_t **definitions,
                            uint64_t definition_count,
                            const kmipkit_extension_registry_limits_t *limits,
                            kmipkit_client_extension_registry_t **out_registry)
{
    return create_registry_status(definitions, definition_count, limits,
        out_registry) == KMIPKIT_SUCCESS;
}

static bool create_fixture_payload(kmipkit_codec_limits_t *codec_limits,
                                   int64_t integer_value,
                                   uint32_t enumeration_value,
                                   bool include_synthetic_secret,
                                   kmipkit_ttlv_structure_t **out_structure)
{
    static const uint8_t discriminator[] = "alpha-v1";
    static const uint8_t order_marker[] = "preserve-this-child-order";
    static const uint8_t vendor_range_marker[] = "preserve-vendor-range-tag";
    static const uint8_t synthetic_secret[] = "SYNTHETIC-ONLY-EXTENSION-SECRET-7C4A";
    kmipkit_ttlv_structure_t *structure = NULL;
    kmipkit_ttlv_structure_t *updated = NULL;
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    bool succeeded = false;

    *out_structure = NULL;
    if (kmipkit_ttlv_structure_create(&structure) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }

#define APPEND_VALUE(tag_number, constructor_call)                                \
    do {                                                                          \
        int32_t append_status;                                                    \
        if (kmipkit_ttlv_raw_tag_create((tag_number), &raw_tag) != KMIPKIT_SUCCESS || \
            kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != KMIPKIT_SUCCESS || \
            (constructor_call) != KMIPKIT_SUCCESS) {                             \
            goto cleanup;                                                         \
        }                                                                         \
        append_status = kmipkit_ttlv_item_create(tag, value, codec_limits, &item); \
        value = NULL; /* Item construction consumes the value on every result. */ \
        if (append_status != KMIPKIT_SUCCESS) {                                  \
            goto cleanup;                                                         \
        }                                                                         \
        append_status = kmipkit_ttlv_structure_with_item(structure, item, codec_limits, &updated); \
        structure = NULL;                                                         \
        item = NULL; /* Structure construction consumes both input handles. */   \
        if (append_status != KMIPKIT_SUCCESS) {                                  \
            goto cleanup;                                                         \
        }                                                                         \
        kmipkit_tag_release(tag);                                                 \
        kmipkit_raw_tag_release(raw_tag);                                         \
        structure = updated;                                                      \
        tag = NULL;                                                               \
        raw_tag = NULL;                                                           \
        updated = NULL;                                                           \
    } while (0)

    APPEND_VALUE(KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR,
        kmipkit_ttlv_value_text_string(codec_limits, discriminator,
            (uint64_t)(sizeof(discriminator) - 1U), &value));
    APPEND_VALUE(KMIPKIT_PAYLOAD_TAG_ENUMERATION,
        kmipkit_ttlv_value_enumeration(enumeration_value, &value));
    APPEND_VALUE(KMIPKIT_PAYLOAD_TAG_VALUE,
        kmipkit_ttlv_value_long_integer(integer_value, &value));
    APPEND_VALUE(KMIPKIT_PAYLOAD_TAG_ORDER_MARKER,
        kmipkit_ttlv_value_text_string(codec_limits, order_marker,
            (uint64_t)(sizeof(order_marker) - 1U), &value));
    APPEND_VALUE(KMIPKIT_EXTENSION_TAG,
        kmipkit_ttlv_value_text_string(codec_limits, vendor_range_marker,
            (uint64_t)(sizeof(vendor_range_marker) - 1U), &value));
    if (include_synthetic_secret) {
        APPEND_VALUE(UINT32_C(0x420005),
            kmipkit_ttlv_value_text_string(codec_limits, synthetic_secret,
                (uint64_t)(sizeof(synthetic_secret) - 1U), &value));
    }

#undef APPEND_VALUE
    *out_structure = structure;
    structure = NULL;
    succeeded = true;

cleanup:
    if (item != NULL) kmipkit_ttlv_item_release(item);
    if (value != NULL) kmipkit_ttlv_value_release(value);
    if (tag != NULL) kmipkit_tag_release(tag);
    if (raw_tag != NULL) kmipkit_raw_tag_release(raw_tag);
    if (updated != NULL) kmipkit_ttlv_structure_release(updated);
    if (structure != NULL) kmipkit_ttlv_structure_release(structure);
    return succeeded;
}

static bool test_shared_fixture_markers(const char *fixture_path)
{
    REQUIRE(load_fixture_marker(fixture_path, "valid-recognized"));
    REQUIRE(load_fixture_marker(fixture_path, "0x420006"));
    REQUIRE(load_fixture_marker(fixture_path, "0x540001"));
    REQUIRE(load_fixture_marker(fixture_path, "genericPayloadPreserved"));
    return true;
}

static bool test_typed_length_safety(void)
{
    static const uint8_t small[] = {'x', 'y'};
    static const uint8_t name[] = "alpha";
    static const uint8_t version[] = "1";
    static const uint8_t compatibility_text[] = "1.0.0";
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_extension_compatibility_t *compatibility = NULL;
    kmipkit_extension_information_t *information = NULL;
    kmipkit_extension_information_t *updated_information = NULL;
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    uint64_t oversized_text_length = UINT64_C(4097);

    /* Explicit byte lengths accept a non-NUL-terminated fixed-size span. */
    REQUIRE_STATUS(kmipkit_extension_identity_create(
        small, UINT64_C(2), name, (uint64_t)(sizeof(name) - 1U),
        version, (uint64_t)(sizeof(version) - 1U), &identity), KMIPKIT_SUCCESS);
    kmipkit_extension_identity_release(identity);
    identity = NULL;

    /* An over-limit length paired with an unreadable pointer must fail first. */
    REQUIRE_STATUS(kmipkit_extension_identity_create(
        INVALID_INPUT_PTR, oversized_text_length, name,
        (uint64_t)(sizeof(name) - 1U), version,
        (uint64_t)(sizeof(version) - 1U), &identity), KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(identity == NULL);
    REQUIRE_STATUS(kmipkit_extension_identity_create(
        small, UINT64_C(2), INVALID_INPUT_PTR, oversized_text_length,
        version, (uint64_t)(sizeof(version) - 1U), &identity), KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(identity == NULL);
    REQUIRE_STATUS(kmipkit_extension_identity_create(
        small, UINT64_C(2), name, (uint64_t)(sizeof(name) - 1U),
        INVALID_INPUT_PTR, oversized_text_length, &identity), KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(identity == NULL);

    REQUIRE_STATUS(kmipkit_extension_compatibility_create(
        2U, 1U, 2U, 1U, INVALID_INPUT_PTR, oversized_text_length,
        compatibility_text, (uint64_t)(sizeof(compatibility_text) - 1U),
        &compatibility), KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(compatibility == NULL);
    REQUIRE_STATUS(kmipkit_extension_compatibility_create(
        2U, 1U, 2U, 1U, compatibility_text,
        (uint64_t)(sizeof(compatibility_text) - 1U), INVALID_INPUT_PTR,
        oversized_text_length, &compatibility), KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(compatibility == NULL);

    REQUIRE_STATUS(kmipkit_extension_information_create(
        INVALID_INPUT_PTR, oversized_text_length, &information),
        KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(information == NULL);
    REQUIRE_STATUS(kmipkit_extension_information_create(
        name, (uint64_t)(sizeof(name) - 1U), &information), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_information_description(
        information, INVALID_INPUT_PTR, oversized_text_length,
        &updated_information), KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(updated_information == NULL);

    REQUIRE_STATUS(kmipkit_codec_limits_create(8U, 64U, 100U, &codec_limits),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_text_string(codec_limits,
        INVALID_INPUT_PTR, 9U, &value), KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(value == NULL);
    REQUIRE_STATUS(kmipkit_ttlv_value_byte_string(codec_limits,
        INVALID_INPUT_PTR, 9U, &value), KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(value == NULL);
    REQUIRE_STATUS(kmipkit_ttlv_value_big_integer(codec_limits,
        INVALID_INPUT_PTR, 9U, &value), KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(value == NULL);
    REQUIRE_STATUS(kmipkit_ttlv_value_text_string(codec_limits,
        NULL, 1U, &value), KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(value == NULL);

    kmipkit_extension_information_release(updated_information);
    kmipkit_extension_information_release(information);
    kmipkit_extension_compatibility_release(compatibility);
    kmipkit_codec_limits_release(codec_limits);
    return true;
}

static bool test_configuration_ownership_and_isolation(void)
{
    static const uint8_t alpha_name[] = "alpha";
    static const uint8_t beta_name[] = "beta";
    static const uint8_t alpha_text[] = "alpha-v1";
    static const uint8_t beta_text[] = "beta-v1";
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_extension_identity_t *alpha_identity = NULL;
    kmipkit_extension_identity_t *beta_identity = NULL;
    kmipkit_extension_definition_t *alpha_definition = NULL;
    kmipkit_extension_definition_t *beta_definition = NULL;
    kmipkit_client_extension_registry_t *alpha_registry = NULL;
    kmipkit_client_extension_registry_t *beta_registry = NULL;
    kmipkit_client_extension_registry_t *alpha_view = NULL;
    kmipkit_client_extension_registry_t *beta_view = NULL;
    kmipkit_client_configuration_t *alpha_configuration = NULL;
    kmipkit_client_configuration_t *beta_configuration = NULL;
    kmipkit_extension_definition_t *found_definition = NULL;
    kmipkit_extension_registry_limits_t limits;
    kmipkit_extension_definition_t *alpha_definitions[2];
    kmipkit_extension_definition_t *beta_definitions[1];
    uint64_t definition_count = 0U;

    REQUIRE_STATUS(kmipkit_codec_limits_defaults(&codec_limits), KMIPKIT_SUCCESS);
    REQUIRE(load_default_limits(&limits));
    REQUIRE(create_fixture_definition(codec_limits, alpha_name,
        (uint64_t)(sizeof(alpha_name) - 1U), KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR,
        alpha_text, (uint64_t)(sizeof(alpha_text) - 1U),
        &alpha_identity, &alpha_definition));
    REQUIRE(create_fixture_definition(codec_limits, beta_name,
        (uint64_t)(sizeof(beta_name) - 1U), UINT32_C(0x420010),
        beta_text, (uint64_t)(sizeof(beta_text) - 1U),
        &beta_identity, &beta_definition));

    alpha_definitions[0] = alpha_definition;
    alpha_definitions[1] = beta_definition;
    beta_definitions[0] = beta_definition;
    REQUIRE(create_registry(alpha_definitions, 2U, &limits, &alpha_registry));
    REQUIRE(create_registry(beta_definitions, 1U, &limits, &beta_registry));
    REQUIRE_STATUS(kmipkit_client_configuration_create(alpha_registry,
        &alpha_configuration), KMIPKIT_SUCCESS);
    alpha_registry = NULL; /* Ownership transfers to the configuration. */
    REQUIRE_STATUS(kmipkit_client_configuration_create(beta_registry,
        &beta_configuration), KMIPKIT_SUCCESS);
    beta_registry = NULL;

    /* Registration inputs are borrowed; the configuration-owned copies live on. */
    kmipkit_extension_definition_release(alpha_definition);
    alpha_definition = NULL;
    kmipkit_extension_definition_release(beta_definition);
    beta_definition = NULL;

    REQUIRE_STATUS(kmipkit_client_configuration_extension_registry(
        alpha_configuration, &alpha_view), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_client_configuration_extension_registry(
        beta_configuration, &beta_view), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_count(
        alpha_view, &definition_count), KMIPKIT_SUCCESS);
    REQUIRE(definition_count == 2U);
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_count(
        beta_view, &definition_count), KMIPKIT_SUCCESS);
    REQUIRE(definition_count == 1U);

    /* Alpha and beta definitions do not leak into the other configuration. */
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_for_identity(
        alpha_view, beta_identity, &found_definition), KMIPKIT_SUCCESS);
    REQUIRE(found_definition != NULL);
    kmipkit_extension_definition_release(found_definition);
    found_definition = NULL;
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_for_identity(
        alpha_view, alpha_identity, &found_definition), KMIPKIT_SUCCESS);
    REQUIRE(found_definition != NULL);
    kmipkit_extension_definition_release(found_definition);
    found_definition = NULL;
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_for_identity(
        beta_view, alpha_identity, &found_definition), KMIPKIT_SUCCESS);
    REQUIRE(found_definition == NULL);

    /* Releasing an accessor handle does not mutate or release its owner. */
    kmipkit_client_extension_registry_release(alpha_view);
    alpha_view = NULL;
    REQUIRE_STATUS(kmipkit_client_configuration_extension_registry(
        alpha_configuration, &alpha_view), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_for_identity(
        alpha_view, NULL, &found_definition), KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(found_definition == NULL);

    kmipkit_extension_identity_release(alpha_identity);
    alpha_identity = NULL;
    kmipkit_extension_identity_release(beta_identity);
    beta_identity = NULL;
    kmipkit_extension_definition_release(found_definition);
    kmipkit_client_extension_registry_release(alpha_view);
    kmipkit_client_extension_registry_release(beta_view);
    kmipkit_client_configuration_release(alpha_configuration);
    kmipkit_client_configuration_release(beta_configuration);
    kmipkit_extension_definition_release(alpha_definition);
    kmipkit_extension_definition_release(beta_definition);
    kmipkit_extension_identity_release(alpha_identity);
    kmipkit_extension_identity_release(beta_identity);
    kmipkit_client_extension_registry_release(alpha_registry);
    kmipkit_client_extension_registry_release(beta_registry);
    kmipkit_codec_limits_release(codec_limits);
    return true;
}

static bool compare_preserved_payload(kmipkit_ttlv_structure_view_t *view)
{
    static const uint8_t discriminator[] = "alpha-v1";
    static const uint8_t order_marker[] = "preserve-this-child-order";
    static const uint8_t vendor_range_marker[] = "preserve-vendor-range-tag";
    static const uint32_t expected_tags[5] = {
        KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR,
        KMIPKIT_PAYLOAD_TAG_ENUMERATION,
        KMIPKIT_PAYLOAD_TAG_VALUE,
        KMIPKIT_PAYLOAD_TAG_ORDER_MARKER,
        KMIPKIT_EXTENSION_TAG
    };
    static const uint8_t expected_types[5] = {
        KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
        KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION,
        KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER,
        KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
        KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING
    };
    uint64_t item_count = 0U;
    size_t index;

    if (kmipkit_ttlv_structure_view_item_count(view, &item_count) != KMIPKIT_SUCCESS ||
        item_count != 5U) {
        return false;
    }

    for (index = 0; index < 5U; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        kmipkit_tag_t *tag = NULL;
        kmipkit_ttlv_value_view_t *value = NULL;
        uint32_t raw_tag = 0U;
        uint8_t item_type = 0U;
        bool matches;

        if (kmipkit_ttlv_structure_view_item_at(view, (uint64_t)index, &item) != KMIPKIT_SUCCESS ||
            kmipkit_ttlv_item_view_tag(item, &tag) != KMIPKIT_SUCCESS ||
            kmipkit_ttlv_tag_value(tag, &raw_tag) != KMIPKIT_SUCCESS ||
            kmipkit_ttlv_item_view_type(item, &item_type) != KMIPKIT_SUCCESS ||
            kmipkit_ttlv_item_view_value(item, &value) != KMIPKIT_SUCCESS) {
            kmipkit_tag_release(tag);
            kmipkit_ttlv_value_view_release(value);
            kmipkit_ttlv_item_view_release(item);
            return false;
        }
        matches = raw_tag == expected_tags[index] && item_type == expected_types[index];
        if (matches && (index == 0U || index == 3U || index == 4U)) {
            const uint8_t *expected = index == 0U ? discriminator
                : (index == 3U ? order_marker : vendor_range_marker);
            const uint64_t expected_length = index == 0U
                ? (uint64_t)(sizeof(discriminator) - 1U)
                : (index == 3U ? (uint64_t)(sizeof(order_marker) - 1U)
                    : (uint64_t)(sizeof(vendor_range_marker) - 1U));
            uint64_t actual_length = 0U;
            uint64_t byte_index;
            matches = kmipkit_ttlv_value_view_byte_length(value,
                &actual_length) == KMIPKIT_SUCCESS && actual_length == expected_length;
            for (byte_index = 0U; matches && byte_index < expected_length; ++byte_index) {
                uint8_t actual_byte = 0U;
                matches = kmipkit_ttlv_value_view_byte_at(value, byte_index,
                    &actual_byte) == KMIPKIT_SUCCESS && actual_byte == expected[byte_index];
            }
        } else if (matches && index == 1U) {
            uint32_t actual_enumeration = 0U;
            const uint32_t expected_enumeration = UINT32_MAX;
            matches = kmipkit_ttlv_value_view_enumeration(value,
                &actual_enumeration) == KMIPKIT_SUCCESS &&
                actual_enumeration == expected_enumeration;
        } else if (matches && index == 2U) {
            int64_t actual_integer = 0;
            matches = kmipkit_ttlv_value_view_long_integer(value,
                &actual_integer) == KMIPKIT_SUCCESS && actual_integer == 42;
        }
        kmipkit_tag_release(tag);
        kmipkit_ttlv_value_view_release(value);
        kmipkit_ttlv_item_view_release(item);
        if (!matches) {
            return false;
        }
    }
    return true;
}

static bool test_inbound_inspection_and_generic_preservation(void)
{
    static const uint8_t name[] = "alpha";
    static const uint8_t discriminator[] = "alpha-v1";
    static const uint8_t vendor[] = "example.vendor";
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_extension_definition_t *definition = NULL;
    kmipkit_extension_definition_t *definitions[1];
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_extension_recognition_t *recognition = NULL;
    kmipkit_validated_extension_value_t *validated = NULL;
    kmipkit_ttlv_structure_view_t *generic = NULL;
    kmipkit_ttlv_path_t *value_path = NULL;
    kmipkit_ttlv_value_view_t *value_view = NULL;
    kmipkit_extension_registry_limits_t limits;
    uint32_t recognized = 0U;
    uint8_t item_type = 0U;
    int64_t integer_value = 0;

    REQUIRE_STATUS(kmipkit_codec_limits_defaults(&codec_limits), KMIPKIT_SUCCESS);
    REQUIRE(load_default_limits(&limits));
    REQUIRE(create_fixture_definition(codec_limits, name,
        (uint64_t)(sizeof(name) - 1U), KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR,
        discriminator, (uint64_t)(sizeof(discriminator) - 1U),
        &identity, &definition));
    definitions[0] = definition;
    REQUIRE(create_registry(definitions, 1U, &limits, &registry));
    REQUIRE(create_fixture_payload(codec_limits, 42, UINT32_MAX, false, &payload));

    REQUIRE_STATUS(kmipkit_client_extension_registry_inspect(registry,
        INVALID_INPUT_PTR, UINT64_C(4097), payload, &recognition,
        codec_limits), KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(recognition == NULL);

    REQUIRE_STATUS(kmipkit_client_extension_registry_inspect(registry,
        vendor, (uint64_t)(sizeof(vendor) - 1U), payload, &recognition,
        codec_limits), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_recognition_is_recognized(
        recognition, &recognized), KMIPKIT_SUCCESS);
    REQUIRE(recognized == 1U);
    REQUIRE_STATUS(kmipkit_extension_recognition_validated_value(
        recognition, &validated), KMIPKIT_SUCCESS);
    REQUIRE(validated != NULL);
    REQUIRE_STATUS(kmipkit_extension_recognition_generic_value(
        recognition, &generic), KMIPKIT_SUCCESS);
    REQUIRE(compare_preserved_payload(generic));

    REQUIRE_STATUS(kmipkit_ttlv_path_create(KMIPKIT_PAYLOAD_TAG_VALUE,
        &value_path), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_validated_extension_value_value_at(validated,
        value_path, &value_view), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view_type(value_view, &item_type),
        KMIPKIT_SUCCESS);
    REQUIRE(item_type == KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER);
    REQUIRE_STATUS(kmipkit_ttlv_value_view_long_integer(value_view,
        &integer_value), KMIPKIT_SUCCESS);
    REQUIRE(integer_value == 42);

    /* A deterministic, lowered lookup budget fails with no partial value. */
    {
        kmipkit_client_extension_registry_t *limited_registry = NULL;
        kmipkit_extension_recognition_t *limited_recognition = NULL;
        kmipkit_extension_registry_limits_t limited = limits;
        set_limit_field(&limited, 10U, 1U);
        REQUIRE(create_registry(definitions, 1U, &limited, &limited_registry));
        REQUIRE_STATUS(kmipkit_client_extension_registry_inspect(limited_registry,
            vendor, (uint64_t)(sizeof(vendor) - 1U), payload,
            &limited_recognition, codec_limits), KMIPKIT_ERROR_RESOURCE_LIMIT);
        REQUIRE(limited_recognition == NULL);
        kmipkit_client_extension_registry_release(limited_registry);
        kmipkit_extension_recognition_release(limited_recognition);
    }

    /* Lower per-Structure and per-rule constraint caps reject this schema. */
    {
        kmipkit_client_extension_registry_t *limited_registry = NULL;
        kmipkit_extension_registry_limits_t limited = limits;
        set_limit_field(&limited, 2U, 2U);
        REQUIRE_STATUS(create_registry_status(definitions, 1U, &limited,
            &limited_registry), KMIPKIT_ERROR_RESOURCE_LIMIT);
        REQUIRE(limited_registry == NULL);

        set_limit_field(&limited, 2U, limits.max_child_rules_per_structure);
        set_limit_field(&limited, 7U, 1U);
        REQUIRE_STATUS(create_registry_status(definitions, 1U, &limited,
            &limited_registry), KMIPKIT_ERROR_RESOURCE_LIMIT);
        REQUIRE(limited_registry == NULL);

        set_limit_field(&limited, 7U, limits.max_constraint_members_per_rule);
        set_limit_field(&limited, 8U, 1U);
        REQUIRE_STATUS(create_registry_status(definitions, 1U, &limited,
            &limited_registry), KMIPKIT_ERROR_RESOURCE_LIMIT);
        REQUIRE(limited_registry == NULL);
    }

    kmipkit_ttlv_value_view_release(value_view);
    kmipkit_ttlv_path_release(value_path);
    kmipkit_ttlv_structure_view_release(generic);
    kmipkit_validated_extension_value_release(validated);
    kmipkit_extension_recognition_release(recognition);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_extension_definition_release(definition);
    kmipkit_extension_identity_release(identity);
    kmipkit_codec_limits_release(codec_limits);
    return true;
}

static bool test_redacted_stable_errors_and_invalid_handles(void)
{
    static const uint8_t name[] = "alpha";
    static const uint8_t discriminator[] = "alpha-v1";
    static const uint8_t vendor[] = "example.vendor";
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_extension_definition_t *definition = NULL;
    kmipkit_extension_definition_t *definitions[1];
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_ttlv_structure_t *invalid_payload = NULL;
    kmipkit_registered_extension_value_t *registered_value = NULL;
    kmipkit_client_request_message_extension_t *message_extension = NULL;
    kmipkit_extension_recognition_t *recognition = NULL;
    kmipkit_client_extension_registry_t *configuration_registry = NULL;
    kmipkit_validated_extension_value_t *inspected_value = NULL;
    kmipkit_ttlv_structure_view_t *generic = NULL;
    kmipkit_extension_registry_limits_t limits;
    uint64_t definition_count = 0U;
    uint64_t generic_item_count = 0U;
    uint32_t recognized = 0U;
    int32_t first_error;
    int32_t second_error;

    REQUIRE_STATUS(kmipkit_codec_limits_defaults(&codec_limits), KMIPKIT_SUCCESS);
    REQUIRE(load_default_limits(&limits));
    REQUIRE(create_fixture_definition(codec_limits, name,
        (uint64_t)(sizeof(name) - 1U), KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR,
        discriminator, (uint64_t)(sizeof(discriminator) - 1U),
        &identity, &definition));
    definitions[0] = definition;
    REQUIRE(create_registry(definitions, 1U, &limits, &registry));
    REQUIRE(create_fixture_payload(codec_limits, 42, UINT32_MAX, false,
        &invalid_payload));

    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_count(
        NULL, &definition_count), KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE_STATUS(kmipkit_client_configuration_extension_registry(
        NULL, &configuration_registry), KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(configuration_registry == NULL);
    REQUIRE_STATUS(kmipkit_client_extension_registry_inspect(
        NULL, vendor, (uint64_t)(sizeof(vendor) - 1U), NULL,
        &recognition, codec_limits), KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(recognition == NULL);
    REQUIRE_STATUS(kmipkit_client_extension_registry_validate(
        NULL, identity, NULL, codec_limits, &registered_value),
        KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(registered_value == NULL);

    REQUIRE_STATUS(kmipkit_client_extension_registry_validate(
        (kmipkit_client_extension_registry_t *)identity, identity,
        invalid_payload, codec_limits, &registered_value),
        KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(registered_value == NULL);
    REQUIRE_STATUS(kmipkit_client_extension_registry_validate(registry,
        (kmipkit_extension_identity_t *)registry, invalid_payload,
        codec_limits, &registered_value), KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(registered_value == NULL);
    REQUIRE_STATUS(kmipkit_client_extension_registry_inspect(registry, vendor,
        (uint64_t)(sizeof(vendor) - 1U),
        (kmipkit_ttlv_structure_t *)identity, &recognition, codec_limits),
        KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(recognition == NULL);
    REQUIRE_STATUS(kmipkit_client_request_message_extension_create(
        (kmipkit_registered_extension_value_t *)identity, 0U,
        &message_extension), KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(message_extension == NULL);
    REQUIRE_STATUS(kmipkit_extension_recognition_is_recognized(
        (kmipkit_extension_recognition_t *)identity, &recognized),
        KMIPKIT_ERROR_INVALID_INPUT);

    /* A live identity handle passed in the registry position has the wrong kind. */
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_count(
        (kmipkit_client_extension_registry_t *)identity, &definition_count),
        KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE_STATUS(kmipkit_client_configuration_extension_registry(
        (kmipkit_client_configuration_t *)identity, &configuration_registry),
        KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(configuration_registry == NULL);

    /* Repeated failures expose the same category; this ABI has no text channel. */
    kmipkit_ttlv_structure_release(invalid_payload);
    invalid_payload = NULL;
    REQUIRE(create_fixture_payload(codec_limits, 100, UINT32_MAX, true,
        &invalid_payload));

    REQUIRE_STATUS(kmipkit_client_extension_registry_inspect(registry, vendor,
        (uint64_t)(sizeof(vendor) - 1U), invalid_payload, &recognition,
        codec_limits), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_recognition_is_recognized(recognition,
        &recognized), KMIPKIT_SUCCESS);
    REQUIRE(recognized == 0U);
    REQUIRE_STATUS(kmipkit_extension_recognition_validated_value(recognition,
        &inspected_value), KMIPKIT_SUCCESS);
    REQUIRE(inspected_value == NULL);
    REQUIRE_STATUS(kmipkit_extension_recognition_generic_value(recognition,
        &generic), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_structure_view_item_count(generic,
        &generic_item_count), KMIPKIT_SUCCESS);
    REQUIRE(generic_item_count == 6U);
    kmipkit_ttlv_structure_view_release(generic);
    generic = NULL;
    kmipkit_extension_recognition_release(recognition);
    recognition = NULL;

    first_error = kmipkit_client_extension_registry_validate(registry, identity,
        invalid_payload, codec_limits, &registered_value);
    REQUIRE(first_error == KMIPKIT_ERROR_INVALID_SCHEMA);
    REQUIRE(registered_value == NULL);
    second_error = kmipkit_client_extension_registry_validate(registry, identity,
        invalid_payload, codec_limits, &registered_value);
    REQUIRE(second_error == first_error);
    REQUIRE(registered_value == NULL);

    /* The request-use wrapper requires an explicit Boolean criticality. */
    REQUIRE_STATUS(kmipkit_client_extension_registry_validate(registry, identity,
        invalid_payload, codec_limits, &registered_value), KMIPKIT_ERROR_INVALID_SCHEMA);
    REQUIRE(registered_value == NULL);

    kmipkit_ttlv_structure_release(invalid_payload);
    kmipkit_extension_recognition_release(recognition);
    kmipkit_ttlv_structure_view_release(generic);
    kmipkit_validated_extension_value_release(inspected_value);
    kmipkit_registered_extension_value_release(registered_value);
    kmipkit_client_request_message_extension_release(message_extension);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_extension_definition_release(definition);
    kmipkit_extension_identity_release(identity);
    kmipkit_codec_limits_release(codec_limits);
    return true;
}

static bool test_explicit_outbound_criticality(void)
{
    static const uint8_t name[] = "alpha";
    static const uint8_t discriminator[] = "alpha-v1";
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_extension_definition_t *definition = NULL;
    kmipkit_extension_definition_t *definitions[1];
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_registered_extension_value_t *registered = NULL;
    kmipkit_client_request_message_extension_t *extension = NULL;
    kmipkit_extension_registry_limits_t limits;

    REQUIRE_STATUS(kmipkit_codec_limits_defaults(&codec_limits), KMIPKIT_SUCCESS);
    REQUIRE(load_default_limits(&limits));
    REQUIRE(create_fixture_definition(codec_limits, name,
        (uint64_t)(sizeof(name) - 1U), KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR,
        discriminator, (uint64_t)(sizeof(discriminator) - 1U),
        &identity, &definition));
    definitions[0] = definition;
    REQUIRE(create_registry(definitions, 1U, &limits, &registry));
    REQUIRE(create_fixture_payload(codec_limits, 42, UINT32_MAX, false, &payload));

    REQUIRE_STATUS(kmipkit_client_extension_registry_validate(registry, identity,
        payload, codec_limits, &registered), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_client_request_message_extension_create(
        registered, 0U, &extension), KMIPKIT_SUCCESS);
    registered = NULL; /* Ownership transfers to request-use creation. */
    REQUIRE(extension != NULL);
    kmipkit_client_request_message_extension_release(extension);
    extension = NULL;

    REQUIRE_STATUS(kmipkit_client_extension_registry_validate(registry, identity,
        payload, codec_limits, &registered), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_client_request_message_extension_create(
        registered, 1U, &extension), KMIPKIT_SUCCESS);
    registered = NULL;
    REQUIRE(extension != NULL);
    kmipkit_client_request_message_extension_release(extension);
    extension = NULL;

    REQUIRE_STATUS(kmipkit_client_extension_registry_validate(registry, identity,
        payload, codec_limits, &registered), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_client_request_message_extension_create(
        registered, 2U, &extension), KMIPKIT_ERROR_INVALID_INPUT);
    registered = NULL; /* The consumed input is not reused after failure. */
    REQUIRE(extension == NULL);

    kmipkit_client_request_message_extension_release(extension);
    kmipkit_registered_extension_value_release(registered);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_extension_definition_release(definition);
    kmipkit_extension_identity_release(identity);
    kmipkit_codec_limits_release(codec_limits);
    return true;
}

static bool test_handle_release_and_null_output_on_error(void)
{
    static const uint8_t name[] = "alpha";
    static const uint8_t discriminator[] = "alpha-v1";
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_extension_definition_t *definition = NULL;
    kmipkit_extension_definition_t *definitions[1];
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_client_configuration_t *configuration = NULL;
    kmipkit_client_extension_registry_t *accessor = NULL;
    kmipkit_extension_registry_limits_t limits;
    uint64_t count = 0U;
    kmipkit_extension_definition_t *indexed_definition = NULL;

    REQUIRE_STATUS(kmipkit_codec_limits_defaults(&codec_limits), KMIPKIT_SUCCESS);
    REQUIRE(load_default_limits(&limits));
    REQUIRE(create_fixture_definition(codec_limits, name,
        (uint64_t)(sizeof(name) - 1U), KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR,
        discriminator, (uint64_t)(sizeof(discriminator) - 1U),
        &identity, &definition));
    definitions[0] = definition;
    REQUIRE(create_registry(definitions, 1U, &limits, &registry));
    REQUIRE_STATUS(kmipkit_client_configuration_create(registry, &configuration),
        KMIPKIT_SUCCESS);
    registry = NULL;
    REQUIRE_STATUS(kmipkit_client_configuration_extension_registry(configuration,
        &accessor), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_count(accessor,
        &count), KMIPKIT_SUCCESS);
    REQUIRE(count == 1U);

    kmipkit_client_extension_registry_release(accessor);
    accessor = NULL;
    REQUIRE_STATUS(kmipkit_client_configuration_extension_registry(configuration,
        &accessor), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_count(accessor,
        &count), KMIPKIT_SUCCESS);
    REQUIRE(count == 1U);
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_at(accessor,
        UINT64_MAX, &indexed_definition), KMIPKIT_SUCCESS);
    REQUIRE(indexed_definition == NULL);

    kmipkit_extension_definition_release(indexed_definition);
    kmipkit_client_extension_registry_release(accessor);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_extension_definition_release(definition);
    kmipkit_extension_identity_release(identity);
    kmipkit_codec_limits_release(codec_limits);
    return true;
}

int main(int argc, char **argv)
{
    const char *fixture_path = argc > 1 ? argv[1] : "tests/fixtures/extensions/cases.json";
    struct test_case {
        const char *name;
        bool (*run)(void);
    } tests_without_arguments[] = {
        {"ExtensionRegistryLimits defaults and boundaries", test_extension_registry_limits},
        {"typed pointers and uint64_t length safety", test_typed_length_safety},
        {"client configuration ownership and isolation", test_configuration_ownership_and_isolation},
        {"inbound inspection and generic preservation", test_inbound_inspection_and_generic_preservation},
        {"stable redacted errors and invalid handles", test_redacted_stable_errors_and_invalid_handles},
        {"explicit outbound criticality", test_explicit_outbound_criticality},
        {"handle release and null output semantics", test_handle_release_and_null_output_on_error}
    };
    size_t index;
    size_t failures = 0U;

    if (!test_shared_fixture_markers(fixture_path)) {
        ++failures;
    }
    for (index = 0; index < sizeof(tests_without_arguments) / sizeof(tests_without_arguments[0]); ++index) {
        if (!tests_without_arguments[index].run()) {
            ++failures;
        } else {
            printf("PASS %s\n", tests_without_arguments[index].name);
        }
    }
    if (failures != 0U) {
        fprintf(stderr, "%lu C consumer test(s) failed\n", (unsigned long)failures);
        return EXIT_FAILURE;
    }
    puts("All KMIPKIT-0012 C consumer tests passed.");
    return EXIT_SUCCESS;
}
