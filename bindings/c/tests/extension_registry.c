/*
 * Red-stage public C consumer coverage for KMIPKIT-0012 User Story 3.
 *
 * This test checks that the shared corpus names and values remain present, then
 * constructs the C TTLV equivalent of `valid-recognized` from those values.
 * Its consumer assertions run through the public read-only view API.
 * KMIP Specification v2.1 §11.56 permits the 0x54 extension Tag range; the
 * catalog policy accepts those unknown Tags for generic preservation.
 *
 * The public C header exposes ClientBatchItem construction and inspection, but
 * does not execute or encode requests. The C ABI exposes only stable int32_t
 * error categories, not diagnostic strings, so payload redaction is constrained
 * to verifying stable category-only results.
 * Use-after-release, dangling, and foreign pointers are caller-precondition
 * violations and are deliberately not probed here.
 */

#include "kmipkit.h"
#include "extension_fixtures.generated.h"

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
    static const uint8_t compatibility_minimum[] = "0.0.0";
    static const uint8_t compatibility_maximum[] = "99.0.0";
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
            2U, 1U, 2U, 1U, compatibility_minimum,
            (uint64_t)(sizeof(compatibility_minimum) - 1U), compatibility_maximum,
            (uint64_t)(sizeof(compatibility_maximum) - 1U), &compatibility) != KMIPKIT_SUCCESS ||
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

static bool create_fixture_payload_with_discriminator(
    kmipkit_codec_limits_t *codec_limits,
    int64_t integer_value,
    uint32_t enumeration_value,
    bool include_synthetic_secret,
    const uint8_t *discriminator,
    uint64_t discriminator_length,
    kmipkit_ttlv_structure_t **out_structure)
{
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
            discriminator_length, &value));
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

static bool create_fixture_payload(kmipkit_codec_limits_t *codec_limits,
                                   int64_t integer_value,
                                   uint32_t enumeration_value,
                                   bool include_synthetic_secret,
                                   kmipkit_ttlv_structure_t **out_structure)
{
    static const uint8_t discriminator[] = "alpha-v1";
    return create_fixture_payload_with_discriminator(codec_limits, integer_value,
        enumeration_value, include_synthetic_secret, discriminator,
        (uint64_t)(sizeof(discriminator) - 1U), out_structure);
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
    {
        kmipkit_extension_information_t *consumed_information = information;
        information = NULL;
        REQUIRE_STATUS(kmipkit_extension_information_description(
            consumed_information, INVALID_INPUT_PTR, oversized_text_length,
            &updated_information), KMIPKIT_ERROR_RESOURCE_LIMIT);
    }
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

static int32_t create_fixture_schema(const kmipkit_fixture_schema_t *fixture,
                                     kmipkit_extension_schema_t **out_schema)
{
    kmipkit_extension_schema_t *base = NULL;
    kmipkit_extension_schema_t *bounded = NULL;
    kmipkit_extension_child_rule_t **rules = NULL;
    size_t index;
    int32_t status = KMIPKIT_ERROR_INVALID_INPUT;

    *out_schema = NULL;
    if (fixture->type != KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE) {
        status = kmipkit_extension_schema_scalar(fixture->type, &base);
        if (status == KMIPKIT_SUCCESS && fixture->has_range) {
            status = kmipkit_extension_schema_signed_numeric_range(
                base, fixture->minimum, fixture->maximum, &bounded);
            base = NULL; /* The range constructor consumes the source schema. */
            if (status == KMIPKIT_SUCCESS) {
                *out_schema = bounded;
                bounded = NULL;
            }
        } else if (status == KMIPKIT_SUCCESS) {
            *out_schema = base;
            base = NULL;
        }
        kmipkit_extension_schema_release(base);
        kmipkit_extension_schema_release(bounded);
        return status;
    }

    if (fixture->child_count != 0U) {
        rules = (kmipkit_extension_child_rule_t **)calloc(
            fixture->child_count, sizeof(*rules));
        if (rules == NULL) {
            return KMIPKIT_ERROR_RESOURCE_LIMIT;
        }
    }
    for (index = 0U; index < fixture->child_count; ++index) {
        kmipkit_extension_schema_t *child_schema = NULL;
        status = create_fixture_schema(fixture->children[index].nested, &child_schema);
        if (status == KMIPKIT_SUCCESS) {
            status = kmipkit_extension_child_rule_required(
                fixture->children[index].tag, child_schema, &rules[index]);
        }
        kmipkit_extension_schema_release(child_schema);
        if (status != KMIPKIT_SUCCESS) {
            break;
        }
    }
    if (status == KMIPKIT_SUCCESS) {
        status = kmipkit_extension_schema_structure(
            rules, fixture->child_count, NULL, 0U, 1U, out_schema);
    }
    for (index = 0U; index < fixture->child_count; ++index) {
        kmipkit_extension_child_rule_release(rules[index]);
    }
    free(rules);
    return status;
}

static int32_t create_fixture_discriminator_value(
    kmipkit_codec_limits_t *limits, uint8_t type, const char *text,
    int64_t number, kmipkit_ttlv_value_t **out_value)
{
    if (type == KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING) {
        return kmipkit_ttlv_value_text_string(limits,
            (const uint8_t *)text, (uint64_t)strlen(text), out_value);
    }
    if (type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION) {
        return kmipkit_ttlv_value_enumeration((uint32_t)number, out_value);
    }
    if (type == KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER) {
        return kmipkit_ttlv_value_long_integer(number, out_value);
    }
    return KMIPKIT_ERROR_INVALID_INPUT;
}

static bool create_shared_fixture_definition(
    kmipkit_codec_limits_t *limits,
    const kmipkit_fixture_definition_t *fixture,
    kmipkit_extension_definition_t **out_definition)
{
    static const uint8_t compatibility_minimum[] = "0.0.0";
    static const uint8_t compatibility_maximum[] = "99.0.0";
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_extension_compatibility_t *compatibility = NULL;
    kmipkit_ttlv_path_t *path = NULL;
    kmipkit_extension_discriminator_t *discriminator = NULL;
    kmipkit_ttlv_value_t *discriminator_value = NULL;
    kmipkit_extension_schema_t *schema = NULL;
    bool succeeded = false;
    size_t index;

    *out_definition = NULL;
    if (kmipkit_extension_identity_create(
            (const uint8_t *)fixture->vendor, (uint64_t)strlen(fixture->vendor),
            (const uint8_t *)fixture->name, (uint64_t)strlen(fixture->name),
            (const uint8_t *)fixture->version, (uint64_t)strlen(fixture->version),
            &identity) != KMIPKIT_SUCCESS ||
        kmipkit_extension_compatibility_create(
            2U, 1U, 2U, 1U, compatibility_minimum,
            (uint64_t)(sizeof(compatibility_minimum) - 1U), compatibility_maximum,
            (uint64_t)(sizeof(compatibility_maximum) - 1U), &compatibility) != KMIPKIT_SUCCESS ||
        kmipkit_ttlv_path_create(fixture->path[0], &path) != KMIPKIT_SUCCESS ||
        create_fixture_discriminator_value(limits, fixture->discriminator_type,
            fixture->discriminator_text, fixture->discriminator_number,
            &discriminator_value) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    for (index = 1U; index < fixture->path_count; ++index) {
        kmipkit_ttlv_path_t *updated = NULL;
        const int32_t status = kmipkit_ttlv_path_with_child_tag(
            path, fixture->path[index], &updated);
        path = NULL; /* Path extension consumes the previous path handle. */
        if (status != KMIPKIT_SUCCESS) {
            goto cleanup;
        }
        path = updated;
    }
    {
        const int32_t status = kmipkit_extension_discriminator_create(
            path, discriminator_value, &discriminator);
        path = NULL;
        discriminator_value = NULL;
        if (status != KMIPKIT_SUCCESS) {
            goto cleanup;
        }
    }
    if (create_fixture_schema(fixture->schema, &schema) != KMIPKIT_SUCCESS ||
        kmipkit_extension_definition_create(identity, compatibility, discriminator,
            schema, out_definition) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    succeeded = true;

cleanup:
    kmipkit_extension_identity_release(identity);
    kmipkit_extension_compatibility_release(compatibility);
    kmipkit_ttlv_path_release(path);
    kmipkit_extension_discriminator_release(discriminator);
    kmipkit_ttlv_value_release(discriminator_value);
    kmipkit_extension_schema_release(schema);
    if (!succeeded) {
        kmipkit_extension_definition_release(*out_definition);
        *out_definition = NULL;
    }
    return succeeded;
}

static int32_t create_fixture_item_value(
    kmipkit_codec_limits_t *limits, const kmipkit_fixture_item_t *fixture,
    kmipkit_ttlv_value_t **out_value);

static int32_t create_fixture_structure(
    kmipkit_codec_limits_t *limits, const kmipkit_fixture_item_t *items,
    size_t item_count, kmipkit_ttlv_structure_t **out_structure)
{
    kmipkit_ttlv_structure_t *structure = NULL;
    kmipkit_ttlv_structure_t *updated = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    size_t index;
    int32_t status = kmipkit_ttlv_structure_create(&structure);

    *out_structure = NULL;
    if (status != KMIPKIT_SUCCESS) {
        return status;
    }
    for (index = 0U; index < item_count; ++index) {
        status = create_fixture_item_value(limits, &items[index], &value);
        if (status != KMIPKIT_SUCCESS ||
            kmipkit_ttlv_raw_tag_create(items[index].tag, &raw_tag) != KMIPKIT_SUCCESS ||
            kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != KMIPKIT_SUCCESS) {
            goto cleanup;
        }
        status = kmipkit_ttlv_item_create(tag, value, limits, &item);
        value = NULL; /* Item construction consumes the value. */
        if (status != KMIPKIT_SUCCESS) {
            goto cleanup;
        }
        status = kmipkit_ttlv_structure_with_item(structure, item, limits, &updated);
        structure = NULL; /* Appending consumes the prior Structure and Item. */
        item = NULL;
        if (status != KMIPKIT_SUCCESS) {
            goto cleanup;
        }
        kmipkit_tag_release(tag);
        kmipkit_raw_tag_release(raw_tag);
        tag = NULL;
        raw_tag = NULL;
        structure = updated;
        updated = NULL;
    }
    *out_structure = structure;
    structure = NULL;

cleanup:
    kmipkit_ttlv_structure_release(structure);
    kmipkit_ttlv_structure_release(updated);
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_item_release(item);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    return status;
}

static int32_t create_fixture_item_value(
    kmipkit_codec_limits_t *limits, const kmipkit_fixture_item_t *fixture,
    kmipkit_ttlv_value_t **out_value)
{
    if (fixture->type == KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE) {
        kmipkit_ttlv_structure_t *nested = NULL;
        const int32_t status = create_fixture_structure(
            limits, fixture->children, fixture->child_count, &nested);
        if (status != KMIPKIT_SUCCESS) {
            return status;
        }
        return kmipkit_ttlv_value_structure(nested, limits, out_value);
    }
    if (fixture->type == KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING) {
        return kmipkit_ttlv_value_text_string(limits,
            (const uint8_t *)fixture->text, (uint64_t)strlen(fixture->text), out_value);
    }
    if (fixture->type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION) {
        return kmipkit_ttlv_value_enumeration(fixture->enum_value, out_value);
    }
    if (fixture->type == KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER) {
        return kmipkit_ttlv_value_long_integer(fixture->signed_value, out_value);
    }
    return KMIPKIT_ERROR_INVALID_INPUT;
}

static bool compare_fixture_structure(kmipkit_ttlv_structure_view_t *view,
                                      const kmipkit_fixture_item_t *expected,
                                      size_t expected_count);

static bool compare_fixture_value(kmipkit_ttlv_value_view_t *value,
                                  const kmipkit_fixture_item_t *expected)
{
    uint8_t actual_type = 0U;
    bool matches = kmipkit_ttlv_value_view_type(value, &actual_type) == KMIPKIT_SUCCESS &&
        actual_type == expected->type;
    if (!matches) {
        return false;
    }
    if (expected->type == KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE) {
        kmipkit_ttlv_structure_view_t *nested = NULL;
        matches = kmipkit_ttlv_value_view_structure(value, &nested) == KMIPKIT_SUCCESS &&
            compare_fixture_structure(nested, expected->children, expected->child_count);
        kmipkit_ttlv_structure_view_release(nested);
    } else if (expected->type == KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING) {
        uint64_t actual_length = 0U;
        const uint64_t expected_length = (uint64_t)strlen(expected->text);
        matches = kmipkit_ttlv_value_view_byte_length(value, &actual_length) == KMIPKIT_SUCCESS &&
            actual_length == expected_length;
        for (uint64_t index = 0U; matches && index < expected_length; ++index) {
            uint8_t actual = 0U;
            matches = kmipkit_ttlv_value_view_byte_at(value, index, &actual) == KMIPKIT_SUCCESS &&
                actual == (uint8_t)expected->text[index];
        }
    } else if (expected->type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION) {
        uint32_t actual = 0U;
        matches = kmipkit_ttlv_value_view_enumeration(value, &actual) == KMIPKIT_SUCCESS &&
            actual == expected->enum_value;
    } else if (expected->type == KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER) {
        int64_t actual = 0;
        matches = kmipkit_ttlv_value_view_long_integer(value, &actual) == KMIPKIT_SUCCESS &&
            actual == expected->signed_value;
    }
    return matches;
}

static bool compare_fixture_structure(kmipkit_ttlv_structure_view_t *view,
                                      const kmipkit_fixture_item_t *expected,
                                      size_t expected_count)
{
    uint64_t count = 0U;
    size_t index;
    if (kmipkit_ttlv_structure_view_item_count(view, &count) != KMIPKIT_SUCCESS ||
        count != (uint64_t)expected_count) {
        return false;
    }
    for (index = 0U; index < expected_count; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        kmipkit_tag_t *tag = NULL;
        kmipkit_ttlv_value_view_t *value = NULL;
        uint32_t actual_tag = 0U;
        uint8_t actual_type = 0U;
        bool matches = kmipkit_ttlv_structure_view_item_at(view, (uint64_t)index, &item) == KMIPKIT_SUCCESS &&
            kmipkit_ttlv_item_view_tag(item, &tag) == KMIPKIT_SUCCESS &&
            kmipkit_ttlv_tag_value(tag, &actual_tag) == KMIPKIT_SUCCESS &&
            kmipkit_ttlv_item_view_type(item, &actual_type) == KMIPKIT_SUCCESS &&
            actual_tag == expected[index].tag && actual_type == expected[index].type &&
            kmipkit_ttlv_item_view_value(item, &value) == KMIPKIT_SUCCESS &&
            compare_fixture_value(value, &expected[index]);
        kmipkit_ttlv_value_view_release(value);
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
        if (!matches) {
            return false;
        }
    }
    return true;
}

static bool ttlv_value_matches_text(kmipkit_ttlv_value_t *value,
                                    const uint8_t *expected,
                                    uint64_t expected_length);

static bool inspect_shared_fixture_case(kmipkit_client_extension_registry_t *registry,
                                        kmipkit_codec_limits_t *limits,
                                        const kmipkit_fixture_case_t *fixture)
{
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_extension_recognition_t *recognition = NULL;
    kmipkit_validated_extension_value_t *typed = NULL;
    kmipkit_ttlv_structure_view_t *generic = NULL;
    uint32_t recognized = 0U;
    bool expected_recognized = strcmp(fixture->outcome, "recognized") == 0;
    bool succeeded = false;

    if (create_fixture_structure(limits, fixture->payload,
            fixture->payload_count, &payload) != KMIPKIT_SUCCESS ||
        kmipkit_client_extension_registry_inspect(registry,
            (const uint8_t *)fixture->vendor, (uint64_t)strlen(fixture->vendor),
            payload, &recognition, limits) != KMIPKIT_SUCCESS ||
        kmipkit_extension_recognition_is_recognized(recognition, &recognized) != KMIPKIT_SUCCESS ||
        (recognized != 0U) != expected_recognized ||
        kmipkit_extension_recognition_validated_value(recognition, &typed) != KMIPKIT_SUCCESS ||
        (typed != NULL) != fixture->typed ||
        kmipkit_extension_recognition_generic_value(recognition, &generic) != KMIPKIT_SUCCESS ||
        !compare_fixture_structure(generic, fixture->payload, fixture->payload_count)) {
        fprintf(stderr, "FAIL shared fixture %s: public C adapter parity mismatch\n", fixture->id);
        goto cleanup;
    }
    succeeded = true;

cleanup:
    kmipkit_ttlv_structure_view_release(generic);
    kmipkit_validated_extension_value_release(typed);
    kmipkit_extension_recognition_release(recognition);
    kmipkit_ttlv_structure_release(payload);
    return succeeded;
}

static const kmipkit_fixture_case_t *shared_fixture_case(const char *fixture_id)
{
    size_t index;
    for (index = 0U; index < KMIPKIT_FIXTURE_CASE_COUNT; ++index) {
        if (strcmp(kmipkit_fixture_cases[index].id, fixture_id) == 0) {
            return &kmipkit_fixture_cases[index];
        }
    }
    return NULL;
}

static size_t shared_fixture_definition_index(const char *definition_id)
{
    size_t index;
    for (index = 0U; index < KMIPKIT_FIXTURE_DEFINITION_COUNT; ++index) {
        if (strcmp(kmipkit_fixture_definitions[index].id, definition_id) == 0) {
            return index;
        }
    }
    return KMIPKIT_FIXTURE_DEFINITION_COUNT;
}

static bool identity_matches_fixture(kmipkit_extension_identity_t *identity,
                                     const kmipkit_fixture_definition_t *fixture)
{
    kmipkit_ttlv_value_t *field = NULL;
    bool matches = kmipkit_extension_identity_vendor_identifier(identity, &field) == KMIPKIT_SUCCESS &&
        ttlv_value_matches_text(field, (const uint8_t *)fixture->vendor,
            (uint64_t)strlen(fixture->vendor));
    kmipkit_ttlv_value_release(field);
    field = NULL;
    matches = matches && kmipkit_extension_identity_name(identity, &field) == KMIPKIT_SUCCESS &&
        ttlv_value_matches_text(field, (const uint8_t *)fixture->name,
            (uint64_t)strlen(fixture->name));
    kmipkit_ttlv_value_release(field);
    field = NULL;
    matches = matches && kmipkit_extension_identity_version(identity, &field) == KMIPKIT_SUCCESS &&
        ttlv_value_matches_text(field, (const uint8_t *)fixture->version,
            (uint64_t)strlen(fixture->version));
    kmipkit_ttlv_value_release(field);
    return matches;
}

static bool inspect_shared_outbound_request(
    kmipkit_client_extension_registry_t *registry,
    kmipkit_codec_limits_t *limits,
    kmipkit_extension_definition_t *const *definitions,
    const kmipkit_fixture_outbound_request_t *request)
{
    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_client_batch_item_t *updated_item = NULL;
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_registered_extension_value_t *registered = NULL;
    kmipkit_client_request_message_extension_t *extension = NULL;
    kmipkit_fixture_case_t const *owner_case = shared_fixture_case(request->fixture_id);
    const char *failure = "request fixture has a recognized owner case";
    bool succeeded = false;
    size_t index;

    if (strcmp(request->outcome, "outbound.validated") != 0 || owner_case == NULL ||
        !owner_case->typed || strcmp(owner_case->outcome, "recognized") != 0 ||
        kmipkit_client_batch_item_discover_versions(&item) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    for (index = 0U; index < request->attachment_count; ++index) {
        const kmipkit_fixture_attachment_t *attachment = &request->attachments[index];
        const kmipkit_fixture_case_t *target = shared_fixture_case(attachment->fixture_id);
        size_t definition_index;
        uint64_t count = 0U;
        uint8_t criticality = UINT8_C(0xFF);
        failure = "outbound attachment resolves to one typed recognized case";
        if (target == NULL || !target->typed || target->matched_count != 1U ||
            strcmp(target->outcome, "recognized") != 0) {
            goto cleanup;
        }
        definition_index = shared_fixture_definition_index(target->matched_ids[0]);
        failure = "outbound fixture match resolves to a generated definition";
        if (definition_index >= KMIPKIT_FIXTURE_DEFINITION_COUNT ||
            kmipkit_extension_definition_identity(definitions[definition_index],
                &identity) != KMIPKIT_SUCCESS ||
            create_fixture_structure(limits, target->payload,
                target->payload_count, &payload) != KMIPKIT_SUCCESS ||
            kmipkit_client_extension_registry_validate(registry, identity,
                payload, limits, &registered) != KMIPKIT_SUCCESS) {
            goto cleanup;
        }
        failure = "public C adapter creates an explicit-criticality wrapper";
        if (kmipkit_client_request_message_extension_create(registered,
                attachment->criticality_indicator ? UINT8_C(1) : UINT8_C(0),
                &extension) != KMIPKIT_SUCCESS) {
            registered = NULL;
            goto cleanup;
        }
        registered = NULL;
        failure = "public C adapter appends the ordered outbound wrapper";
        if (kmipkit_client_batch_item_with_extension(item, extension,
                &updated_item) != KMIPKIT_SUCCESS) {
            item = NULL;
            extension = NULL;
            goto cleanup;
        }
        item = NULL;
        extension = NULL;
        item = updated_item;
        updated_item = NULL;
        kmipkit_extension_identity_release(identity);
        identity = NULL;
        kmipkit_ttlv_structure_release(payload);
        payload = NULL;
        failure = "public C adapter exposes generated outbound order and criticality";
        if (kmipkit_client_batch_item_extension_count(item, &count) != KMIPKIT_SUCCESS ||
            count != (uint64_t)(index + 1U) ||
            kmipkit_client_batch_item_extension_identity_at(item, (uint64_t)index,
                &identity) != KMIPKIT_SUCCESS || identity == NULL ||
            !identity_matches_fixture(identity, &kmipkit_fixture_definitions[definition_index]) ||
            kmipkit_client_batch_item_extension_criticality_indicator_at(item,
                (uint64_t)index, &criticality) != KMIPKIT_SUCCESS ||
            criticality != (attachment->criticality_indicator ? UINT8_C(1) : UINT8_C(0))) {
            goto cleanup;
        }
        kmipkit_extension_identity_release(identity);
        identity = NULL;
    }
    succeeded = true;

cleanup:
    if (!succeeded) {
        fprintf(stderr, "FAIL %s: %s (%s)\n", __func__, failure, request->fixture_id);
    }
    kmipkit_client_batch_item_release(updated_item);
    kmipkit_client_batch_item_release(item);
    kmipkit_extension_identity_release(identity);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_registered_extension_value_release(registered);
    kmipkit_client_request_message_extension_release(extension);
    return succeeded;
}

static bool inspect_shared_outbound_requests(
    kmipkit_client_extension_registry_t *registry,
    kmipkit_codec_limits_t *limits,
    kmipkit_extension_definition_t *const *definitions)
{
    size_t index;
    for (index = 0U; index < KMIPKIT_FIXTURE_OUTBOUND_REQUEST_COUNT; ++index) {
        if (!inspect_shared_outbound_request(registry, limits, definitions,
                &kmipkit_fixture_outbound_requests[index])) {
            return false;
        }
    }
    return true;
}

static bool test_inbound_inspection_and_generic_preservation(void)
{
    kmipkit_extension_definition_t *definitions[KMIPKIT_FIXTURE_DEFINITION_COUNT] = {NULL};
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_extension_registry_limits_t limits;
    bool succeeded = false;
    size_t index;
    unsigned int order;

    if (kmipkit_codec_limits_defaults(&codec_limits) != KMIPKIT_SUCCESS ||
        !load_default_limits(&limits)) {
        goto cleanup;
    }
    for (index = 0U; index < KMIPKIT_FIXTURE_DEFINITION_COUNT; ++index) {
        if (!create_shared_fixture_definition(codec_limits,
                &kmipkit_fixture_definitions[index], &definitions[index])) {
            goto cleanup;
        }
    }
    for (order = 0U; order < 2U; ++order) {
        kmipkit_extension_definition_t *ordered[KMIPKIT_FIXTURE_DEFINITION_COUNT];
        if (registry != NULL) {
            kmipkit_client_extension_registry_release(registry);
            registry = NULL;
        }
        for (index = 0U; index < KMIPKIT_FIXTURE_DEFINITION_COUNT; ++index) {
            const size_t source = order == 0U ? index : KMIPKIT_FIXTURE_DEFINITION_COUNT - index - 1U;
            ordered[index] = definitions[source];
        }
        if (!create_registry(ordered, KMIPKIT_FIXTURE_DEFINITION_COUNT,
                &limits, &registry)) {
            goto cleanup;
        }
        for (index = 0U; index < KMIPKIT_FIXTURE_CASE_COUNT; ++index) {
            if (!inspect_shared_fixture_case(registry, codec_limits,
                    &kmipkit_fixture_cases[index])) {
                goto cleanup;
            }
        }
        if (!inspect_shared_outbound_requests(registry, codec_limits, definitions)) {
            goto cleanup;
        }
    }
    succeeded = true;

cleanup:
    kmipkit_client_extension_registry_release(registry);
    for (index = 0U; index < KMIPKIT_FIXTURE_DEFINITION_COUNT; ++index) {
        kmipkit_extension_definition_release(definitions[index]);
    }
    kmipkit_codec_limits_release(codec_limits);
    return succeeded;
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

static bool ttlv_value_matches_text(kmipkit_ttlv_value_t *value,
                                    const uint8_t *expected,
                                    uint64_t expected_length)
{
    kmipkit_ttlv_value_view_t *view = NULL;
    uint8_t item_type = 0U;
    uint64_t actual_length = 0U;
    uint64_t index;
    bool matches = true;

    if (kmipkit_ttlv_value_view(value, &view) != KMIPKIT_SUCCESS ||
        kmipkit_ttlv_value_view_type(view, &item_type) != KMIPKIT_SUCCESS ||
        item_type != KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING ||
        kmipkit_ttlv_value_view_byte_length(view, &actual_length) != KMIPKIT_SUCCESS ||
        actual_length != expected_length) {
        matches = false;
        goto cleanup;
    }

    for (index = 0U; index < expected_length; ++index) {
        uint8_t actual_byte = 0U;
        if (kmipkit_ttlv_value_view_byte_at(view, index, &actual_byte) != KMIPKIT_SUCCESS ||
            actual_byte != expected[index]) {
            matches = false;
            break;
        }
    }

cleanup:
    kmipkit_ttlv_value_view_release(view);
    return matches;
}

static bool identity_name_matches(kmipkit_extension_identity_t *identity,
                                  const uint8_t *expected,
                                  uint64_t expected_length)
{
    kmipkit_ttlv_value_t *name = NULL;
    const int32_t status = kmipkit_extension_identity_name(identity, &name);
    const bool matches = status == KMIPKIT_SUCCESS && name != NULL &&
        ttlv_value_matches_text(name, expected, expected_length);
    kmipkit_ttlv_value_release(name);
    return matches;
}

static bool test_null_and_wrong_kind_release_are_noops(void)
{
    static const uint8_t vendor[] = "example.vendor";
    static const uint8_t name[] = "release-check";
    static const uint8_t version[] = "1";
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_ttlv_value_t *identity_name = NULL;

    REQUIRE_STATUS(kmipkit_extension_identity_create(vendor,
        (uint64_t)(sizeof(vendor) - 1U), name,
        (uint64_t)(sizeof(name) - 1U), version,
        (uint64_t)(sizeof(version) - 1U), &identity), KMIPKIT_SUCCESS);

    kmipkit_extension_identity_release(NULL);
    kmipkit_client_extension_registry_release(
        (kmipkit_client_extension_registry_t *)identity);
    REQUIRE_STATUS(kmipkit_extension_identity_name(identity, &identity_name),
        KMIPKIT_SUCCESS);
    REQUIRE(ttlv_value_matches_text(identity_name, name,
        (uint64_t)(sizeof(name) - 1U)));

    kmipkit_ttlv_value_release(identity_name);
    kmipkit_extension_identity_release(identity);
    return true;
}

static bool test_identity_fields_and_batch_extension_order(void)
{
    static const uint8_t alpha_name[] = "alpha";
    static const uint8_t beta_name[] = "beta";
    static const uint8_t vendor[] = "example.vendor";
    static const uint8_t version[] = "1";
    static const uint8_t alpha_discriminator[] = "alpha-v1";
    static const uint8_t beta_discriminator[] = "beta-v1";
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_extension_identity_t *alpha_identity = NULL;
    kmipkit_extension_identity_t *beta_identity = NULL;
    kmipkit_extension_identity_t *first_identity = NULL;
    kmipkit_extension_identity_t *second_identity = NULL;
    kmipkit_extension_definition_t *alpha_definition = NULL;
    kmipkit_extension_definition_t *beta_definition = NULL;
    kmipkit_extension_definition_t *definitions[2];
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_ttlv_structure_t *alpha_payload = NULL;
    kmipkit_ttlv_structure_t *beta_payload = NULL;
    kmipkit_registered_extension_value_t *registered = NULL;
    kmipkit_client_request_message_extension_t *extension = NULL;
    kmipkit_client_batch_item_t *batch_item = NULL;
    kmipkit_client_batch_item_t *updated_batch_item = NULL;
    kmipkit_ttlv_value_t *field_value = NULL;
    kmipkit_extension_registry_limits_t limits;
    uint64_t extension_count = 0U;
    uint8_t criticality = UINT8_C(0xFF);
    int32_t status;
    const char *failure_reason = "create codec limits and definitions";
    bool succeeded = false;

    if (kmipkit_codec_limits_defaults(&codec_limits) != KMIPKIT_SUCCESS ||
        !load_default_limits(&limits) ||
        !create_fixture_definition(codec_limits, alpha_name,
            (uint64_t)(sizeof(alpha_name) - 1U), KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR,
            alpha_discriminator, (uint64_t)(sizeof(alpha_discriminator) - 1U),
            &alpha_identity, &alpha_definition) ||
        !create_fixture_definition(codec_limits, beta_name,
            (uint64_t)(sizeof(beta_name) - 1U), KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR,
            beta_discriminator, (uint64_t)(sizeof(beta_discriminator) - 1U),
            &beta_identity, &beta_definition)) {
        goto cleanup;
    }

    failure_reason = "read the three identity fields";
    if (kmipkit_extension_identity_vendor_identifier(alpha_identity, &field_value) != KMIPKIT_SUCCESS ||
        !ttlv_value_matches_text(field_value, vendor, (uint64_t)(sizeof(vendor) - 1U))) {
        goto cleanup;
    }
    kmipkit_ttlv_value_release(field_value);
    field_value = NULL;
    if (kmipkit_extension_identity_name(alpha_identity, &field_value) != KMIPKIT_SUCCESS ||
        !ttlv_value_matches_text(field_value, alpha_name,
            (uint64_t)(sizeof(alpha_name) - 1U))) {
        goto cleanup;
    }
    kmipkit_ttlv_value_release(field_value);
    field_value = NULL;
    if (kmipkit_extension_identity_version(alpha_identity, &field_value) != KMIPKIT_SUCCESS ||
        !ttlv_value_matches_text(field_value, version,
            (uint64_t)(sizeof(version) - 1U))) {
        goto cleanup;
    }
    kmipkit_ttlv_value_release(field_value);
    field_value = NULL;

    failure_reason = "create registry and validate alpha extension";
    definitions[0] = alpha_definition;
    definitions[1] = beta_definition;
    if (!create_registry(definitions, UINT64_C(2), &limits, &registry) ||
        !create_fixture_payload_with_discriminator(codec_limits, 42, UINT32_MAX,
            false, alpha_discriminator,
            (uint64_t)(sizeof(alpha_discriminator) - 1U), &alpha_payload) ||
        kmipkit_client_extension_registry_validate(registry, alpha_identity,
            alpha_payload, codec_limits, &registered) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    status = kmipkit_client_request_message_extension_create(registered, UINT8_C(1),
        &extension);
    registered = NULL;
    if (status != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    failure_reason = "attach alpha extension";
    if (kmipkit_client_batch_item_discover_versions(&batch_item) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    status = kmipkit_client_batch_item_with_extension(batch_item, extension,
        &updated_batch_item);
    batch_item = NULL;
    extension = NULL;
    if (status != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    batch_item = updated_batch_item;
    updated_batch_item = NULL;

    failure_reason = "validate beta extension";
    if (!create_fixture_payload_with_discriminator(codec_limits, 43, UINT32_MAX,
            false, beta_discriminator,
            (uint64_t)(sizeof(beta_discriminator) - 1U), &beta_payload)) {
        goto cleanup;
    }
    if (kmipkit_client_extension_registry_validate(registry, beta_identity,
            beta_payload, codec_limits, &registered) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    status = kmipkit_client_request_message_extension_create(registered, UINT8_C(0),
        &extension);
    registered = NULL;
    if (status != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    failure_reason = "attach beta extension";
    updated_batch_item = NULL;
    status = kmipkit_client_batch_item_with_extension(batch_item, extension,
        &updated_batch_item);
    batch_item = NULL;
    extension = NULL;
    if (status != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    batch_item = updated_batch_item;
    updated_batch_item = NULL;

    failure_reason = "inspect extension order and first criticality";
    if (kmipkit_client_batch_item_extension_count(batch_item,
            &extension_count) != KMIPKIT_SUCCESS || extension_count != UINT64_C(2) ||
        kmipkit_client_batch_item_extension_identity_at(batch_item, UINT64_C(0),
            &first_identity) != KMIPKIT_SUCCESS || first_identity == NULL ||
        kmipkit_client_batch_item_extension_identity_at(batch_item, UINT64_C(1),
            &second_identity) != KMIPKIT_SUCCESS || second_identity == NULL ||
        !identity_name_matches(first_identity, alpha_name,
            (uint64_t)(sizeof(alpha_name) - 1U)) ||
        !identity_name_matches(second_identity, beta_name,
            (uint64_t)(sizeof(beta_name) - 1U)) ||
        kmipkit_client_batch_item_extension_criticality_indicator_at(batch_item,
            UINT64_C(0), &criticality) != KMIPKIT_SUCCESS || criticality != UINT8_C(1)) {
        goto cleanup;
    }
    criticality = UINT8_C(0xFF);
    failure_reason = "inspect second criticality";
    if (kmipkit_client_batch_item_extension_criticality_indicator_at(batch_item,
            UINT64_C(1), &criticality) != KMIPKIT_SUCCESS || criticality != UINT8_C(0)) {
        goto cleanup;
    }
    succeeded = true;

cleanup:
    if (!succeeded) {
        fprintf(stderr, "FAIL %s: %s\n", __func__, failure_reason);
    }
    kmipkit_ttlv_value_release(field_value);
    kmipkit_extension_identity_release(first_identity);
    kmipkit_extension_identity_release(second_identity);
    kmipkit_client_batch_item_release(updated_batch_item);
    kmipkit_client_batch_item_release(batch_item);
    kmipkit_client_request_message_extension_release(extension);
    kmipkit_registered_extension_value_release(registered);
    kmipkit_ttlv_structure_release(alpha_payload);
    kmipkit_ttlv_structure_release(beta_payload);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_extension_definition_release(alpha_definition);
    kmipkit_extension_definition_release(beta_definition);
    kmipkit_extension_identity_release(alpha_identity);
    kmipkit_extension_identity_release(beta_identity);
    kmipkit_codec_limits_release(codec_limits);
    return succeeded;
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
    kmipkit_extension_definition_t *indexed_definition = NULL;
    kmipkit_extension_definition_t *matched_definition = NULL;
    kmipkit_extension_registry_limits_t limits;
    uint64_t count = 0U;

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
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_at(accessor,
        0U, &indexed_definition), KMIPKIT_SUCCESS);
    REQUIRE(indexed_definition != NULL);
    REQUIRE_STATUS(kmipkit_client_extension_registry_definition_for_identity(
        accessor, identity, &matched_definition), KMIPKIT_SUCCESS);
    REQUIRE(matched_definition != NULL);

    kmipkit_extension_definition_release(matched_definition);
    kmipkit_extension_definition_release(indexed_definition);
    kmipkit_client_extension_registry_release(accessor);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_extension_definition_release(definition);
    kmipkit_extension_identity_release(identity);
    kmipkit_codec_limits_release(codec_limits);
    return true;
}

static bool value_view_has_type(kmipkit_ttlv_value_view_t *view,
                                kmipkit_ttlv_item_type_t expected)
{
    uint8_t observed = 0U;
    return kmipkit_ttlv_value_view_type(view, &observed) == KMIPKIT_SUCCESS &&
        observed == expected;
}

static bool test_ttlv_scalar_views_and_limits(void)
{
    static const uint8_t big_integer_bytes[] = {0x01U, 0x02U, 0x03U};
    static const uint8_t text_bytes[] = {'K', 'M', 'I', 'P'};
    static const uint8_t byte_string_bytes[] = {0U, 0x7FU, 0xFFU};
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_codec_limits_t *invalid_limits = NULL;
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_value_view_t *view = NULL;
    kmipkit_ttlv_structure_t *structure = NULL;
    kmipkit_ttlv_structure_t *updated_structure = NULL;
    kmipkit_ttlv_structure_view_t *structure_view = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_ttlv_item_view_t *item_view = NULL;
    kmipkit_ttlv_value_view_t *item_value_view = NULL;
    kmipkit_tag_t *item_tag = NULL;
    uint64_t observed = 0U;
    uint64_t item_count = UINT64_MAX;
    uint8_t item_type = 0U;
    uint8_t byte = 0U;
    int32_t integer = 0;
    uint32_t interval = 0U;
    uint32_t enumeration = 0U;
    uint8_t boolean = 0U;
    int64_t date_time = 0;
    uint32_t observed_raw_tag = 0U;

    REQUIRE_STATUS(kmipkit_codec_limits_create(4096U, 32U, 256U, &limits),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_codec_limits_max_message_bytes(limits, &observed),
        KMIPKIT_SUCCESS);
    REQUIRE(observed == UINT64_C(4096));
    REQUIRE_STATUS(kmipkit_codec_limits_max_structure_depth(limits, &observed),
        KMIPKIT_SUCCESS);
    REQUIRE(observed == UINT64_C(32));
    REQUIRE_STATUS(kmipkit_codec_limits_max_elements(limits, &observed),
        KMIPKIT_SUCCESS);
    REQUIRE(observed == UINT64_C(256));
    REQUIRE_STATUS(kmipkit_codec_limits_create(4096U, 65U, 256U, &invalid_limits),
        KMIPKIT_ERROR_RESOURCE_LIMIT);
    REQUIRE(invalid_limits == NULL);

    REQUIRE_STATUS(kmipkit_ttlv_raw_tag_create(KMIPKIT_PAYLOAD_TAG_VALUE, &raw_tag),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_raw_tag_value(raw_tag, &observed_raw_tag), KMIPKIT_SUCCESS);
    REQUIRE(observed_raw_tag == KMIPKIT_PAYLOAD_TAG_VALUE);
    REQUIRE_STATUS(kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_tag_value(tag, &observed_raw_tag), KMIPKIT_SUCCESS);
    REQUIRE(observed_raw_tag == KMIPKIT_PAYLOAD_TAG_VALUE);
    kmipkit_tag_release(tag);
    tag = NULL;
    kmipkit_raw_tag_release(raw_tag);
    raw_tag = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_value_integer(-17, &value), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view(value, &view), KMIPKIT_SUCCESS);
    REQUIRE(value_view_has_type(view, KMIPKIT_TTLV_ITEM_TYPE_INTEGER));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_type(view, &item_type), KMIPKIT_SUCCESS);
    REQUIRE(item_type == KMIPKIT_TTLV_ITEM_TYPE_INTEGER);
    REQUIRE_STATUS(kmipkit_ttlv_value_view_integer(view, &integer), KMIPKIT_SUCCESS);
    REQUIRE(integer == -17);
    REQUIRE_STATUS(kmipkit_ttlv_value_view_boolean(view, &boolean),
        KMIPKIT_ERROR_INVALID_INPUT);
    kmipkit_ttlv_value_view_release(view);
    view = NULL;
    kmipkit_ttlv_value_release(value);
    value = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_value_long_integer(INT64_C(-9000000001), &value),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view(value, &view), KMIPKIT_SUCCESS);
    REQUIRE(value_view_has_type(view, KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_long_integer(view, &date_time), KMIPKIT_SUCCESS);
    REQUIRE(date_time == INT64_C(-9000000001));
    kmipkit_ttlv_value_view_release(view);
    view = NULL;
    kmipkit_ttlv_value_release(value);
    value = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_value_enumeration(UINT32_C(314159), &value),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view(value, &view), KMIPKIT_SUCCESS);
    REQUIRE(value_view_has_type(view, KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_enumeration(view, &enumeration), KMIPKIT_SUCCESS);
    REQUIRE(enumeration == UINT32_C(314159));
    kmipkit_ttlv_value_view_release(view);
    view = NULL;
    kmipkit_ttlv_value_release(value);
    value = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_value_boolean(2U, &value), KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(value == NULL);
    REQUIRE_STATUS(kmipkit_ttlv_value_boolean(1U, &value), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view(value, &view), KMIPKIT_SUCCESS);
    REQUIRE(value_view_has_type(view, KMIPKIT_TTLV_ITEM_TYPE_BOOLEAN));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_boolean(view, &boolean), KMIPKIT_SUCCESS);
    REQUIRE(boolean == 1U);
    kmipkit_ttlv_value_view_release(view);
    view = NULL;
    kmipkit_ttlv_value_release(value);
    value = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_value_big_integer(limits, big_integer_bytes,
        (uint64_t)sizeof(big_integer_bytes), &value), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view(value, &view), KMIPKIT_SUCCESS);
    REQUIRE(value_view_has_type(view, KMIPKIT_TTLV_ITEM_TYPE_BIG_INTEGER));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_byte_length(view, &observed), KMIPKIT_SUCCESS);
    REQUIRE(observed == (uint64_t)sizeof(big_integer_bytes));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_byte_at(view, 2U, &byte), KMIPKIT_SUCCESS);
    REQUIRE(byte == big_integer_bytes[2]);
    REQUIRE_STATUS(kmipkit_ttlv_value_view_byte_at(view, 3U, &byte),
        KMIPKIT_ERROR_INVALID_INPUT);
    kmipkit_ttlv_value_view_release(view);
    view = NULL;
    kmipkit_ttlv_value_release(value);
    value = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_value_text_string(limits, text_bytes,
        (uint64_t)sizeof(text_bytes), &value), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view(value, &view), KMIPKIT_SUCCESS);
    REQUIRE(value_view_has_type(view, KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_byte_length(view, &observed), KMIPKIT_SUCCESS);
    REQUIRE(observed == (uint64_t)sizeof(text_bytes));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_byte_at(view, 0U, &byte), KMIPKIT_SUCCESS);
    REQUIRE(byte == (uint8_t)'K');
    kmipkit_ttlv_value_view_release(view);
    view = NULL;
    kmipkit_ttlv_value_release(value);
    value = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_value_byte_string(limits, byte_string_bytes,
        (uint64_t)sizeof(byte_string_bytes), &value), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view(value, &view), KMIPKIT_SUCCESS);
    REQUIRE(value_view_has_type(view, KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_byte_length(view, &observed), KMIPKIT_SUCCESS);
    REQUIRE(observed == (uint64_t)sizeof(byte_string_bytes));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_byte_at(view, 1U, &byte), KMIPKIT_SUCCESS);
    REQUIRE(byte == byte_string_bytes[1]);
    kmipkit_ttlv_value_view_release(view);
    view = NULL;
    kmipkit_ttlv_value_release(value);
    value = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_value_date_time(INT64_C(1700000000), &value),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view(value, &view), KMIPKIT_SUCCESS);
    REQUIRE(value_view_has_type(view, KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_date_time(view, &date_time), KMIPKIT_SUCCESS);
    REQUIRE(date_time == INT64_C(1700000000));
    kmipkit_ttlv_value_view_release(view);
    view = NULL;
    kmipkit_ttlv_value_release(value);
    value = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_value_interval(42U, &value), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view(value, &view), KMIPKIT_SUCCESS);
    REQUIRE(value_view_has_type(view, KMIPKIT_TTLV_ITEM_TYPE_INTERVAL));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_interval(view, &interval), KMIPKIT_SUCCESS);
    REQUIRE(interval == 42U);
    kmipkit_ttlv_value_view_release(view);
    view = NULL;
    kmipkit_ttlv_value_release(value);
    value = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_value_date_time_extended(INT64_C(1700000012), &value),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view(value, &view), KMIPKIT_SUCCESS);
    REQUIRE(value_view_has_type(view, KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME_EXTENDED));
    REQUIRE_STATUS(kmipkit_ttlv_value_view_date_time_extended(view, &date_time),
        KMIPKIT_SUCCESS);
    REQUIRE(date_time == INT64_C(1700000012));
    kmipkit_ttlv_value_view_release(view);
    view = NULL;
    kmipkit_ttlv_value_release(value);
    value = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_raw_tag_create(KMIPKIT_PAYLOAD_TAG_VALUE, &raw_tag),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_integer(11, &value), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_item_create(tag, value, limits, &item), KMIPKIT_SUCCESS);
    value = NULL; /* Item construction consumes the value handle. */
    REQUIRE_STATUS(kmipkit_ttlv_structure_create(&structure), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_structure_with_item(structure, item, limits,
        &updated_structure), KMIPKIT_SUCCESS);
    structure = NULL; /* Structure construction consumes both input handles. */
    item = NULL;
    structure = updated_structure;
    updated_structure = NULL;
    REQUIRE_STATUS(kmipkit_ttlv_structure_view(structure, &structure_view),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_structure_view_item_count(structure_view, &item_count),
        KMIPKIT_SUCCESS);
    REQUIRE(item_count == 1U);
    REQUIRE_STATUS(kmipkit_ttlv_structure_view_item_at(structure_view, 0U, &item_view),
        KMIPKIT_SUCCESS);
    REQUIRE(item_view != NULL);
    REQUIRE_STATUS(kmipkit_ttlv_item_view_tag(item_view, &item_tag), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_tag_value(item_tag, &observed_raw_tag), KMIPKIT_SUCCESS);
    REQUIRE(observed_raw_tag == KMIPKIT_PAYLOAD_TAG_VALUE);
    REQUIRE_STATUS(kmipkit_ttlv_item_view_type(item_view, &item_type), KMIPKIT_SUCCESS);
    REQUIRE(item_type == KMIPKIT_TTLV_ITEM_TYPE_INTEGER);
    REQUIRE_STATUS(kmipkit_ttlv_item_view_value(item_view, &item_value_view),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view_integer(item_value_view, &integer),
        KMIPKIT_SUCCESS);
    REQUIRE(integer == 11);
    kmipkit_ttlv_item_view_release(item_view);
    item_view = NULL;
    kmipkit_ttlv_value_view_release(item_value_view);
    item_value_view = NULL;
    item_count = UINT64_MAX;
    REQUIRE_STATUS(kmipkit_ttlv_structure_view_item_at(structure_view, 1U, &item_view),
        KMIPKIT_ERROR_INVALID_INPUT);
    REQUIRE(item_view == NULL);

    kmipkit_tag_release(item_tag);
    kmipkit_ttlv_item_view_release(item_view);
    kmipkit_ttlv_value_view_release(item_value_view);
    kmipkit_ttlv_structure_view_release(structure_view);
    kmipkit_ttlv_value_view_release(view);
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_item_release(item);
    kmipkit_ttlv_structure_release(updated_structure);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_codec_limits_release(invalid_limits);
    kmipkit_codec_limits_release(limits);
    return true;
}

static bool test_schema_path_and_order_builders(void)
{
    kmipkit_ttlv_path_t *path = NULL;
    kmipkit_ttlv_path_t *child_path = NULL;
    kmipkit_extension_schema_t *bytes_schema = NULL;
    kmipkit_extension_schema_t *updated_schema = NULL;
    kmipkit_extension_schema_t *integer_schema = NULL;
    kmipkit_extension_schema_t *masked_integer_schema = NULL;
    kmipkit_extension_schema_t *required_mask_schema = NULL;
    kmipkit_extension_schema_t *unsigned_schema = NULL;
    kmipkit_extension_schema_t *masked_schema = NULL;
    kmipkit_extension_schema_t *enum_schema = NULL;
    kmipkit_extension_schema_t *repeated_schema = NULL;
    kmipkit_extension_schema_t *scalar_schema = NULL;
    kmipkit_extension_schema_t *invalid_schema = NULL;
    kmipkit_extension_child_rule_t *optional_rule = NULL;
    kmipkit_extension_child_rule_t *required_rule = NULL;
    kmipkit_extension_child_rule_t *mask_rule = NULL;
    kmipkit_extension_child_rule_t *repeated_rule = NULL;
    kmipkit_extension_child_rule_t *children[4];
    kmipkit_extension_order_constraint_t *constraint = NULL;
    kmipkit_extension_schema_t *structure_schema = NULL;
    static const uint32_t scalar_types[] = {
        KMIPKIT_TTLV_ITEM_TYPE_INTEGER,
        KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER,
        KMIPKIT_TTLV_ITEM_TYPE_BIG_INTEGER,
        KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION,
        KMIPKIT_TTLV_ITEM_TYPE_BOOLEAN,
        KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
        KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING,
        KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME,
        KMIPKIT_TTLV_ITEM_TYPE_INTERVAL,
        KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME_EXTENDED
    };
    size_t index;

    for (index = 0U; index < sizeof(scalar_types) / sizeof(scalar_types[0]); ++index) {
        REQUIRE_STATUS(kmipkit_extension_schema_scalar(scalar_types[index],
            &scalar_schema), KMIPKIT_SUCCESS);
        kmipkit_extension_schema_release(scalar_schema);
        scalar_schema = NULL;
    }
    REQUIRE_STATUS(kmipkit_extension_schema_scalar(KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE,
        &invalid_schema), KMIPKIT_ERROR_INVALID_SCHEMA);
    REQUIRE(invalid_schema == NULL);
    REQUIRE_STATUS(kmipkit_extension_schema_scalar(0U, &invalid_schema),
        KMIPKIT_ERROR_INVALID_SCHEMA);
    REQUIRE(invalid_schema == NULL);

    REQUIRE_STATUS(kmipkit_ttlv_path_create(KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR, &path),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_path_with_child_tag(path, KMIPKIT_PAYLOAD_TAG_VALUE,
        &child_path), KMIPKIT_SUCCESS);
    path = NULL; /* Appending a path tag consumes the original path handle. */
    REQUIRE(child_path != NULL);

    REQUIRE_STATUS(kmipkit_extension_schema_scalar(KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING,
        &bytes_schema), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_schema_minimum_length(bytes_schema, 2U,
        &updated_schema), KMIPKIT_SUCCESS);
    bytes_schema = NULL; /* Schema transforms consume the source schema. */
    bytes_schema = updated_schema;
    updated_schema = NULL;
    REQUIRE_STATUS(kmipkit_extension_schema_maximum_length(bytes_schema, 16U,
        &updated_schema), KMIPKIT_SUCCESS);
    bytes_schema = NULL;
    bytes_schema = updated_schema;
    updated_schema = NULL;
    REQUIRE_STATUS(kmipkit_extension_schema_scalar(KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
        &invalid_schema), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_schema_minimum_length(invalid_schema, 4U,
        &updated_schema), KMIPKIT_SUCCESS);
    invalid_schema = NULL; /* Schema transforms consume their input handle. */
    invalid_schema = updated_schema;
    updated_schema = NULL;
    REQUIRE_STATUS(kmipkit_extension_schema_maximum_length(invalid_schema, 2U,
        &updated_schema), KMIPKIT_ERROR_INVALID_SCHEMA);
    invalid_schema = NULL; /* Failed transforms also consume their input handle. */
    REQUIRE(updated_schema == NULL);
    REQUIRE_STATUS(kmipkit_extension_child_rule_optional(KMIPKIT_PAYLOAD_TAG_VALUE,
        bytes_schema, &optional_rule), KMIPKIT_SUCCESS);

    REQUIRE_STATUS(kmipkit_extension_schema_scalar(KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION,
        &integer_schema), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_schema_unsigned_numeric_range(integer_schema,
        1U, 100U, &unsigned_schema), KMIPKIT_SUCCESS);
    integer_schema = NULL;
    REQUIRE_STATUS(kmipkit_extension_schema_allowed_enumeration(unsigned_schema,
        UINT32_C(7), &enum_schema), KMIPKIT_SUCCESS);
    unsigned_schema = NULL; /* Enumeration constraints consume their source schema. */
    REQUIRE_STATUS(kmipkit_extension_child_rule_required(
        KMIPKIT_PAYLOAD_TAG_ORDER_MARKER, enum_schema, &required_rule),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_schema_scalar(KMIPKIT_TTLV_ITEM_TYPE_INTEGER,
        &integer_schema), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_schema_allowed_bit_mask(integer_schema,
        UINT32_C(0x03), &masked_integer_schema), KMIPKIT_SUCCESS);
    integer_schema = NULL;
    REQUIRE_STATUS(kmipkit_extension_schema_required_bit_mask(masked_integer_schema,
        UINT32_C(0x01), &required_mask_schema), KMIPKIT_SUCCESS);
    masked_integer_schema = NULL;
    REQUIRE_STATUS(kmipkit_extension_child_rule_required(UINT32_C(0x420007),
        required_mask_schema, &mask_rule), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_schema_scalar(KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
        &repeated_schema), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_child_rule_repeated(UINT32_C(0x420008),
        repeated_schema, &repeated_rule), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_extension_order_constraint_create(
        KMIPKIT_PAYLOAD_TAG_VALUE, KMIPKIT_PAYLOAD_TAG_ORDER_MARKER, &constraint),
        KMIPKIT_SUCCESS);
    children[0] = optional_rule;
    children[1] = required_rule;
    children[2] = mask_rule;
    children[3] = repeated_rule;
    REQUIRE_STATUS(kmipkit_extension_schema_structure(children, 4U, &constraint, 1U,
        1U, &structure_schema), KMIPKIT_SUCCESS);
    REQUIRE(structure_schema != NULL);

    kmipkit_extension_schema_release(structure_schema);
    kmipkit_extension_order_constraint_release(constraint);
    kmipkit_extension_child_rule_release(repeated_rule);
    kmipkit_extension_child_rule_release(mask_rule);
    kmipkit_extension_child_rule_release(required_rule);
    kmipkit_extension_child_rule_release(optional_rule);
    kmipkit_extension_schema_release(required_mask_schema);
    kmipkit_extension_schema_release(masked_integer_schema);
    kmipkit_extension_schema_release(enum_schema);
    kmipkit_extension_schema_release(repeated_schema);
    kmipkit_extension_schema_release(scalar_schema);
    kmipkit_extension_schema_release(invalid_schema);
    kmipkit_extension_schema_release(masked_schema);
    kmipkit_extension_schema_release(unsigned_schema);
    kmipkit_extension_schema_release(integer_schema);
    kmipkit_extension_schema_release(updated_schema);
    kmipkit_extension_schema_release(bytes_schema);
    kmipkit_ttlv_path_release(child_path);
    kmipkit_ttlv_path_release(path);
    return true;
}

static bool test_extension_information_and_definition_accessors(void)
{
    static const uint8_t info_name[] = "extension-information";
    static const uint8_t description[] = "vendor metadata";
    static const uint8_t name[] = "alpha";
    static const uint8_t discriminator[] = "alpha-v1";
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_extension_identity_t *observed_identity = NULL;
    kmipkit_extension_definition_t *definition = NULL;
    kmipkit_extension_definition_t *updated_definition = NULL;
    kmipkit_extension_information_t *information = NULL;
    kmipkit_extension_information_t *updated_information = NULL;
    kmipkit_extension_information_t *observed_information = NULL;
    kmipkit_ttlv_structure_t *information_ttlv = NULL;
    kmipkit_ttlv_structure_view_t *information_view = NULL;
    uint64_t item_count = 0U;
    bool succeeded = false;

    if (kmipkit_codec_limits_defaults(&codec_limits) != KMIPKIT_SUCCESS ||
        !create_fixture_definition(codec_limits, name, sizeof(name) - 1U,
            KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR, discriminator,
            sizeof(discriminator) - 1U, &identity, &definition) ||
        kmipkit_extension_information_create(info_name, sizeof(info_name) - 1U,
            &information) != KMIPKIT_SUCCESS ||
        kmipkit_extension_information_tag(information, KMIPKIT_EXTENSION_TAG,
            &updated_information) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    information = updated_information;
    updated_information = NULL;
    if (kmipkit_extension_information_type(information,
            KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE, &updated_information) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    information = updated_information;
    updated_information = NULL;
    if (kmipkit_extension_information_enumeration(information, UINT32_C(7),
            &updated_information) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    information = updated_information;
    updated_information = NULL;
    if (kmipkit_extension_information_attribute(information, 1U,
            &updated_information) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    information = updated_information;
    updated_information = NULL;
    if (kmipkit_extension_information_parent_structure_tag(information,
            KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR, &updated_information) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    information = updated_information;
    updated_information = NULL;
    if (kmipkit_extension_information_description(information, description,
            sizeof(description) - 1U, &updated_information) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    information = updated_information;
    updated_information = NULL;
    if (kmipkit_extension_information_to_ttlv(information, &information_ttlv) !=
            KMIPKIT_SUCCESS ||
        kmipkit_ttlv_structure_view(information_ttlv, &information_view) !=
            KMIPKIT_SUCCESS ||
        kmipkit_ttlv_structure_view_item_count(information_view, &item_count) !=
            KMIPKIT_SUCCESS || item_count < UINT64_C(6)) {
        goto cleanup;
    }
    if (kmipkit_extension_definition_with_information(definition, information,
            &updated_definition) != KMIPKIT_SUCCESS) {
        goto cleanup;
    }
    definition = NULL;
    if (kmipkit_extension_definition_identity(updated_definition,
            &observed_identity) != KMIPKIT_SUCCESS ||
        kmipkit_extension_definition_information(updated_definition,
            &observed_information) != KMIPKIT_SUCCESS || observed_information == NULL) {
        goto cleanup;
    }
    succeeded = true;

cleanup:
    if (!succeeded) {
        fprintf(stderr, "FAIL %s: extension information round-trip failed\n", __func__);
    }
    kmipkit_ttlv_structure_view_release(information_view);
    kmipkit_ttlv_structure_release(information_ttlv);
    kmipkit_extension_information_release(observed_information);
    kmipkit_extension_information_release(updated_information);
    kmipkit_extension_information_release(information);
    kmipkit_extension_definition_release(updated_definition);
    kmipkit_extension_definition_release(definition);
    kmipkit_extension_identity_release(observed_identity);
    kmipkit_extension_identity_release(identity);
    kmipkit_codec_limits_release(codec_limits);
    return succeeded;
}

static bool test_definition_validation_and_ttlv_identity_readback(void)
{
    static const uint8_t name[] = "alpha";
    static const uint8_t discriminator[] = "alpha-v1";
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_extension_identity_t *validated_identity = NULL;
    kmipkit_extension_definition_t *definition = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_ttlv_structure_t *nested = NULL;
    kmipkit_ttlv_structure_t *updated = NULL;
    kmipkit_ttlv_structure_view_t *generic_value = NULL;
    kmipkit_validated_extension_value_t *validated = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_value_t *nested_value = NULL;
    kmipkit_ttlv_value_view_t *value_view = NULL;
    kmipkit_ttlv_path_t *path = NULL;
    kmipkit_ttlv_path_t *updated_path = NULL;
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    uint64_t generic_item_count = 0U;
    int64_t observed = 0;

    REQUIRE_STATUS(kmipkit_codec_limits_defaults(&codec_limits), KMIPKIT_SUCCESS);
    REQUIRE(create_fixture_definition(codec_limits, name, sizeof(name) - 1U,
        KMIPKIT_PAYLOAD_TAG_DISCRIMINATOR, discriminator,
        sizeof(discriminator) - 1U, &identity, &definition));
    REQUIRE(create_fixture_payload_with_discriminator(codec_limits, 42,
        UINT32_C(314159), false, discriminator, sizeof(discriminator) - 1U, &payload));

    /* Add a nested, unknown structure to exercise recursive generic path access. */
    REQUIRE_STATUS(kmipkit_ttlv_structure_create(&nested), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_raw_tag_create(UINT32_C(0x420008), &raw_tag),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_long_integer(73, &value), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_item_create(tag, value, codec_limits, &item),
        KMIPKIT_SUCCESS);
    value = NULL; /* Item construction consumes the value handle. */
    REQUIRE_STATUS(kmipkit_ttlv_structure_with_item(nested, item, codec_limits,
        &updated), KMIPKIT_SUCCESS);
    nested = NULL; /* Structure construction consumes both input handles. */
    item = NULL;
    nested = updated;
    updated = NULL;
    kmipkit_tag_release(tag);
    tag = NULL;
    kmipkit_raw_tag_release(raw_tag);
    raw_tag = NULL;

    REQUIRE_STATUS(kmipkit_ttlv_raw_tag_create(UINT32_C(0x420007), &raw_tag),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_structure(nested, codec_limits, &nested_value),
        KMIPKIT_SUCCESS);
    nested = NULL; /* Structure value construction consumes the structure handle. */
    REQUIRE_STATUS(kmipkit_ttlv_item_create(tag, nested_value, codec_limits, &item),
        KMIPKIT_SUCCESS);
    nested_value = NULL; /* Item construction consumes the value handle. */
    REQUIRE_STATUS(kmipkit_ttlv_structure_with_item(payload, item, codec_limits,
        &updated), KMIPKIT_SUCCESS);
    payload = NULL; /* Structure construction consumes both input handles. */
    item = NULL;
    payload = updated;
    updated = NULL;
    kmipkit_tag_release(tag);
    tag = NULL;
    kmipkit_raw_tag_release(raw_tag);
    raw_tag = NULL;

    REQUIRE_STATUS(kmipkit_extension_definition_validate(definition, payload,
        &validated, codec_limits), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_validated_extension_value_identity(validated,
        &validated_identity), KMIPKIT_SUCCESS);
    REQUIRE(identity_name_matches(validated_identity, name, sizeof(name) - 1U));
    REQUIRE_STATUS(kmipkit_validated_extension_value_generic_value(validated,
        &generic_value), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_structure_view_item_count(generic_value,
        &generic_item_count), KMIPKIT_SUCCESS);
    REQUIRE(generic_item_count == UINT64_C(6));

    REQUIRE_STATUS(kmipkit_ttlv_path_create(UINT32_C(0x420007), &path),
        KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_path_with_child_tag(path, UINT32_C(0x420008),
        &updated_path), KMIPKIT_SUCCESS);
    path = NULL; /* Appending the child tag consumes the original path handle. */
    path = updated_path;
    updated_path = NULL;
    REQUIRE_STATUS(kmipkit_validated_extension_value_value_at(validated, path,
        &value_view), KMIPKIT_SUCCESS);
    REQUIRE_STATUS(kmipkit_ttlv_value_view_long_integer(value_view, &observed),
        KMIPKIT_SUCCESS);
    REQUIRE(observed == 73);

    kmipkit_ttlv_structure_view_release(generic_value);
    kmipkit_ttlv_value_view_release(value_view);
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_value_release(nested_value);
    kmipkit_ttlv_item_release(item);
    kmipkit_ttlv_structure_release(updated);
    kmipkit_ttlv_structure_release(nested);
    kmipkit_ttlv_path_release(path);
    kmipkit_validated_extension_value_release(validated);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_extension_definition_release(definition);
    kmipkit_extension_identity_release(validated_identity);
    kmipkit_extension_identity_release(identity);
    kmipkit_codec_limits_release(codec_limits);
    return true;
}

int main(void)
{
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
        {"null and wrong-kind release no-ops", test_null_and_wrong_kind_release_are_noops},
        {"identity fields and batch extension order", test_identity_fields_and_batch_extension_order},
        {"handle release and null output semantics", test_handle_release_and_null_output_on_error},
        {"TTLV scalar views and limits", test_ttlv_scalar_views_and_limits},
        {"schema path and order builders", test_schema_path_and_order_builders},
        {"extension information and definition accessors", test_extension_information_and_definition_accessors},
        {"definition validation and TTLV identity readback", test_definition_validation_and_ttlv_identity_readback}
    };
    size_t index;
    size_t failures = 0U;

    for (index = 0; index < sizeof(tests_without_arguments) / sizeof(tests_without_arguments[0]); ++index) {
        if (!tests_without_arguments[index].run()) {
            ++failures;
        } else {
            printf("PASS %s\n", tests_without_arguments[index].name);
            (void)fflush(stdout);
        }
    }
    if (failures != 0U) {
        fprintf(stderr, "%lu C consumer test(s) failed\n", (unsigned long)failures);
        return EXIT_FAILURE;
    }
    puts("All KMIPKIT-0012 C consumer tests passed.");
    return EXIT_SUCCESS;
}
