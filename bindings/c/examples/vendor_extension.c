/*
 * Register two data-only vendor extensions, inspect recognized and
 * schema-invalid inbound values as generic TTLV, then select each outbound
 * Message Extension Criticality Indicator explicitly and attach both to a
 * typed Discover Versions batch item in caller order.
 *
 * The receive path leaves unknown criticality actions to KMIPKIT-0007:
 * unrecognized critical response extensions are rejected and unrecognized
 * non-critical extensions remain generic TTLV. This sample inspects local
 * values and does not exercise transport response handling.
 *
 * Payload field Tags follow the shared extension fixture's known.alpha
 * allocation. It demonstrates typed construction only; the current C surface
 * does not expose a client send or message-encoding operation.
 */

#include "kmipkit.h"

#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define SUCCESS ((int32_t)0)
#define TAG_DISCRIMINATOR UINT32_C(0x420001)
#define TAG_VALUE UINT32_C(0x420004)

#define CHECK(call)                                                              \
    do {                                                                         \
        const int32_t check_status = (call);                                     \
        if (check_status != SUCCESS) {                                           \
            fprintf(stderr, "KMIPKit call failed at %s:%d (status=%" PRId32 ")\n", \
                    __FILE__, __LINE__, check_status);                           \
            goto cleanup;                                                        \
        }                                                                        \
    } while (0)

static int32_t append_value(kmipkit_ttlv_structure_t **structure,
                            uint32_t raw_tag,
                            kmipkit_ttlv_value_t **value,
                            kmipkit_codec_limits_t *codec_limits)
{
    kmipkit_raw_tag_t *raw = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_ttlv_structure_t *updated = NULL;
    int32_t status;

    status = kmipkit_ttlv_raw_tag_create(raw_tag, &raw);
    if (status == SUCCESS) {
        status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    }
    if (status == SUCCESS) {
        status = kmipkit_ttlv_item_create(tag, *value, codec_limits, &item);
        *value = NULL; /* Item creation consumes the value, including on error. */
    }
    if (status == SUCCESS) {
        status = kmipkit_ttlv_structure_with_item(*structure, item,
            codec_limits, &updated);
        *structure = NULL; /* Structure creation consumes both inputs. */
        item = NULL;
    }
    if (status == SUCCESS) {
        *structure = updated;
        updated = NULL;
    }

    kmipkit_ttlv_structure_release(updated);
    kmipkit_ttlv_item_release(item);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw);
    return status;
}

static int32_t make_payload(kmipkit_codec_limits_t *codec_limits,
                            const uint8_t *discriminator,
                            uint64_t discriminator_length,
                            int64_t value_number,
                            kmipkit_ttlv_structure_t **out_payload)
{
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    int32_t status = kmipkit_ttlv_structure_create(&payload);

    if (status == SUCCESS) {
        status = kmipkit_ttlv_value_text_string(codec_limits, discriminator,
            discriminator_length, &value);
    }
    if (status == SUCCESS) {
        status = append_value(&payload, TAG_DISCRIMINATOR, &value, codec_limits);
    }
    if (status == SUCCESS) {
        status = kmipkit_ttlv_value_long_integer(value_number, &value);
    }
    if (status == SUCCESS) {
        status = append_value(&payload, TAG_VALUE, &value, codec_limits);
    }
    if (status == SUCCESS) {
        *out_payload = payload;
        payload = NULL;
    }

    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_structure_release(payload);
    return status;
}

static int32_t make_definition(kmipkit_codec_limits_t *codec_limits,
                               const uint8_t *name,
                               uint64_t name_length,
                               const uint8_t *discriminator,
                               uint64_t discriminator_length,
                               kmipkit_extension_identity_t **out_identity,
                               kmipkit_extension_definition_t **out_definition)
{
    static const uint8_t vendor[] = "example.vendor";
    static const uint8_t version[] = "1";
    static const uint8_t compatible_minimum[] = "0.0.0";
    static const uint8_t compatible_maximum[] = "99.0.0";
    kmipkit_extension_identity_t *identity = NULL;
    kmipkit_extension_compatibility_t *compatibility = NULL;
    kmipkit_ttlv_path_t *path = NULL;
    kmipkit_ttlv_value_t *discriminator_value = NULL;
    kmipkit_extension_discriminator_t *matcher = NULL;
    kmipkit_extension_schema_t *text_schema = NULL;
    kmipkit_extension_schema_t *integer_schema = NULL;
    kmipkit_extension_schema_t *bounded_integer_schema = NULL;
    kmipkit_extension_child_rule_t *text_rule = NULL;
    kmipkit_extension_child_rule_t *integer_rule = NULL;
    kmipkit_extension_child_rule_t *children[2];
    kmipkit_extension_schema_t *root_schema = NULL;
    kmipkit_extension_definition_t *definition = NULL;
    int32_t status;

    *out_identity = NULL;
    *out_definition = NULL;
    status = kmipkit_extension_identity_create(vendor, sizeof(vendor) - 1U,
        name, name_length, version, sizeof(version) - 1U, &identity);
    if (status == SUCCESS) {
        status = kmipkit_extension_compatibility_create(2U, 1U, 2U, 1U,
            compatible_minimum, sizeof(compatible_minimum) - 1U,
            compatible_maximum, sizeof(compatible_maximum) - 1U,
            &compatibility);
    }
    if (status == SUCCESS) {
        status = kmipkit_ttlv_path_create(TAG_DISCRIMINATOR, &path);
    }
    if (status == SUCCESS) {
        status = kmipkit_ttlv_value_text_string(codec_limits, discriminator,
            discriminator_length, &discriminator_value);
    }
    if (status == SUCCESS) {
        status = kmipkit_extension_discriminator_create(path,
            discriminator_value, &matcher);
        path = NULL;
        discriminator_value = NULL; /* Discriminator creation consumes both. */
    }
    if (status == SUCCESS) {
        status = kmipkit_extension_schema_scalar(
            KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING, &text_schema);
    }
    if (status == SUCCESS) {
        status = kmipkit_extension_child_rule_required(TAG_DISCRIMINATOR,
            text_schema, &text_rule);
    }
    if (status == SUCCESS) {
        status = kmipkit_extension_schema_scalar(
            KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER, &integer_schema);
    }
    if (status == SUCCESS) {
        status = kmipkit_extension_schema_signed_numeric_range(integer_schema,
            1, 999, &bounded_integer_schema);
        integer_schema = NULL; /* Range construction consumes its source. */
    }
    if (status == SUCCESS) {
        status = kmipkit_extension_child_rule_required(TAG_VALUE,
            bounded_integer_schema, &integer_rule);
    }
    if (status == SUCCESS) {
        children[0] = text_rule;
        children[1] = integer_rule;
        status = kmipkit_extension_schema_structure(children, 2U, NULL, 0U,
            0U, &root_schema);
    }
    if (status == SUCCESS) {
        status = kmipkit_extension_definition_create(identity, compatibility,
            matcher, root_schema, &definition);
    }
    if (status == SUCCESS) {
        *out_identity = identity;
        *out_definition = definition;
        identity = NULL;
        definition = NULL;
    }

    kmipkit_extension_definition_release(definition);
    kmipkit_extension_schema_release(root_schema);
    kmipkit_extension_child_rule_release(integer_rule);
    kmipkit_extension_child_rule_release(text_rule);
    kmipkit_extension_schema_release(bounded_integer_schema);
    kmipkit_extension_schema_release(integer_schema);
    kmipkit_extension_schema_release(text_schema);
    kmipkit_extension_discriminator_release(matcher);
    kmipkit_ttlv_value_release(discriminator_value);
    kmipkit_ttlv_path_release(path);
    kmipkit_extension_compatibility_release(compatibility);
    kmipkit_extension_identity_release(identity);
    return status;
}

static bool identity_name_matches(kmipkit_extension_identity_t *identity,
                                  const uint8_t *expected,
                                  uint64_t expected_length)
{
    kmipkit_ttlv_value_t *name = NULL;
    kmipkit_ttlv_value_view_t *view = NULL;
    uint8_t item_type = 0U;
    uint64_t actual_length = 0U;
    uint64_t index;
    bool matches = false;

    if (kmipkit_extension_identity_name(identity, &name) != SUCCESS ||
        kmipkit_ttlv_value_view(name, &view) != SUCCESS ||
        kmipkit_ttlv_value_view_type(view, &item_type) != SUCCESS ||
        item_type != KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING ||
        kmipkit_ttlv_value_view_byte_length(view, &actual_length) != SUCCESS ||
        actual_length != expected_length) {
        goto cleanup;
    }
    matches = true;
    for (index = 0U; index < expected_length; ++index) {
        uint8_t actual = 0U;
        if (kmipkit_ttlv_value_view_byte_at(view, index, &actual) != SUCCESS ||
            actual != expected[index]) {
            matches = false;
            break;
        }
    }

cleanup:
    kmipkit_ttlv_value_view_release(view);
    kmipkit_ttlv_value_release(name);
    return matches;
}

static bool generic_payload_matches(kmipkit_ttlv_structure_view_t *payload,
                                    const uint8_t *expected_discriminator,
                                    uint64_t expected_discriminator_length,
                                    int64_t expected_value)
{
    /* View accessors copy into caller storage; clear such copies if sensitive. */
    uint64_t item_count = 0U;
    uint64_t index;
    bool matches = false;

    if (kmipkit_ttlv_structure_view_item_count(payload, &item_count) != SUCCESS ||
        item_count != 2U) {
        return false;
    }

    matches = true;
    for (index = 0U; index < item_count; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        kmipkit_tag_t *tag = NULL;
        kmipkit_ttlv_value_view_t *value = NULL;
        uint32_t tag_value = 0U;
        uint8_t item_type = 0U;

        if (kmipkit_ttlv_structure_view_item_at(payload, index, &item) != SUCCESS ||
            kmipkit_ttlv_item_view_tag(item, &tag) != SUCCESS ||
            kmipkit_ttlv_tag_value(tag, &tag_value) != SUCCESS ||
            kmipkit_ttlv_item_view_type(item, &item_type) != SUCCESS ||
            kmipkit_ttlv_item_view_value(item, &value) != SUCCESS) {
            matches = false;
        } else if (index == 0U) {
            uint64_t actual_length = 0U;
            uint64_t byte_index;

            matches = tag_value == TAG_DISCRIMINATOR &&
                item_type == KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING &&
                kmipkit_ttlv_value_view_byte_length(value, &actual_length) == SUCCESS &&
                actual_length == expected_discriminator_length;
            for (byte_index = 0U; matches && byte_index < actual_length;
                 ++byte_index) {
                uint8_t actual_byte = 0U;
                matches = kmipkit_ttlv_value_view_byte_at(value, byte_index,
                    &actual_byte) == SUCCESS &&
                    actual_byte == expected_discriminator[byte_index];
            }
        } else {
            int64_t actual_value = 0;

            matches = tag_value == TAG_VALUE &&
                item_type == KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER &&
                kmipkit_ttlv_value_view_long_integer(value, &actual_value) == SUCCESS &&
                actual_value == expected_value;
        }

        kmipkit_ttlv_value_view_release(value);
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
        if (!matches) {
            break;
        }
    }
    return matches;
}

static bool inspect_inbound_payload(kmipkit_client_extension_registry_t *registry,
                                    const uint8_t *vendor_identifier,
                                    uint64_t vendor_identifier_length,
                                    kmipkit_ttlv_structure_t *payload,
                                    kmipkit_codec_limits_t *codec_limits,
                                    bool expected_recognized,
                                    const uint8_t *expected_discriminator,
                                    uint64_t expected_discriminator_length,
                                    int64_t expected_value)
{
    kmipkit_extension_recognition_t *recognition = NULL;
    kmipkit_validated_extension_value_t *validated = NULL;
    kmipkit_ttlv_structure_view_t *generic = NULL;
    uint32_t recognized = 0U;
    bool matches = false;

    if (kmipkit_client_extension_registry_inspect(registry, vendor_identifier,
            vendor_identifier_length, payload, &recognition, codec_limits) != SUCCESS ||
        kmipkit_extension_recognition_is_recognized(recognition,
            &recognized) != SUCCESS ||
        (recognized != 0U) != expected_recognized ||
        kmipkit_extension_recognition_validated_value(recognition,
            &validated) != SUCCESS ||
        (validated != NULL) != expected_recognized ||
        kmipkit_extension_recognition_generic_value(recognition,
            &generic) != SUCCESS) {
        goto cleanup;
    }

    matches = generic_payload_matches(generic, expected_discriminator,
        expected_discriminator_length, expected_value);

cleanup:
    kmipkit_ttlv_structure_view_release(generic);
    kmipkit_validated_extension_value_release(validated);
    kmipkit_extension_recognition_release(recognition);
    return matches;
}

int main(void)
{
    static const uint8_t first_name[] = "accounting";
    static const uint8_t second_name[] = "audit";
    static const uint8_t first_discriminator[] = "accounting-v1";
    static const uint8_t second_discriminator[] = "audit-v1";
    static const uint8_t vendor_identifier[] = "example.vendor";
    kmipkit_codec_limits_t *codec_limits = NULL;
    kmipkit_extension_identity_t *first_identity = NULL;
    kmipkit_extension_identity_t *second_identity = NULL;
    kmipkit_extension_definition_t *first_definition = NULL;
    kmipkit_extension_definition_t *second_definition = NULL;
    kmipkit_extension_definition_t *definitions[2];
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_client_extension_registry_t *configured_registry = NULL;
    kmipkit_client_configuration_t *configuration = NULL;
    kmipkit_ttlv_structure_t *first_payload = NULL;
    kmipkit_ttlv_structure_t *second_payload = NULL;
    kmipkit_ttlv_structure_t *schema_invalid_payload = NULL;
    kmipkit_registered_extension_value_t *registered = NULL;
    kmipkit_client_request_message_extension_t *extension = NULL;
    kmipkit_client_batch_item_t *batch_item = NULL;
    kmipkit_client_batch_item_t *updated_batch_item = NULL;
    kmipkit_extension_registry_limits_t limits;
    kmipkit_extension_identity_t *attached_identity = NULL;
    uint64_t extension_count = 0U;
    uint8_t criticality = UINT8_C(0xFF);
    int32_t status;
    int exit_code = 1;

    CHECK(kmipkit_codec_limits_defaults(&codec_limits));
    CHECK(kmipkit_extension_registry_limits_default_values(
        &limits.max_definitions, &limits.max_schema_nodes,
        &limits.max_child_rules_per_structure, &limits.max_text_bytes_per_field,
        &limits.max_registry_text_bytes, &limits.max_discriminator_scalar_bytes,
        &limits.max_total_discriminator_scalar_bytes,
        &limits.max_constraint_members_per_rule, &limits.max_total_constraint_members,
        &limits.max_payload_index_records, &limits.max_lookup_comparisons,
        &limits.max_depth));
    CHECK(make_definition(codec_limits, first_name, sizeof(first_name) - 1U,
        first_discriminator, sizeof(first_discriminator) - 1U,
        &first_identity, &first_definition));
    CHECK(make_definition(codec_limits, second_name, sizeof(second_name) - 1U,
        second_discriminator, sizeof(second_discriminator) - 1U,
        &second_identity, &second_definition));

    definitions[0] = first_definition;
    definitions[1] = second_definition;
    CHECK(kmipkit_client_extension_registry_create(definitions, 2U,
        limits.max_definitions, limits.max_schema_nodes,
        limits.max_child_rules_per_structure, limits.max_text_bytes_per_field,
        limits.max_registry_text_bytes, limits.max_discriminator_scalar_bytes,
        limits.max_total_discriminator_scalar_bytes,
        limits.max_constraint_members_per_rule, limits.max_total_constraint_members,
        limits.max_payload_index_records, limits.max_lookup_comparisons,
        limits.max_depth, &registry));
    CHECK(kmipkit_client_configuration_create(registry, &configuration));
    registry = NULL; /* The configuration takes ownership of the registry. */
    CHECK(kmipkit_client_configuration_extension_registry(configuration,
        &configured_registry));

    CHECK(make_payload(codec_limits, first_discriminator,
        sizeof(first_discriminator) - 1U, 42, &first_payload));
    if (!inspect_inbound_payload(configured_registry, vendor_identifier,
            sizeof(vendor_identifier) - 1U, first_payload, codec_limits, true,
            first_discriminator, sizeof(first_discriminator) - 1U, 42)) {
        fprintf(stderr, "The valid inbound extension was not recognized with its generic TTLV intact\n");
        goto cleanup;
    }
    CHECK(make_payload(codec_limits, first_discriminator,
        sizeof(first_discriminator) - 1U, 1000, &schema_invalid_payload));
    if (!inspect_inbound_payload(configured_registry, vendor_identifier,
            sizeof(vendor_identifier) - 1U, schema_invalid_payload, codec_limits,
            false, first_discriminator, sizeof(first_discriminator) - 1U, 1000)) {
        fprintf(stderr, "The schema-invalid inbound extension was not preserved as generic TTLV\n");
        goto cleanup;
    }
    puts("Inbound accounting extension recognized; the original generic TTLV subtree was inspected.");
    puts("The out-of-range accounting value remained unrecognized and retained its generic TTLV tags and values.");

    CHECK(kmipkit_client_extension_registry_validate(configured_registry,
        first_identity, first_payload, codec_limits, &registered));
    status = kmipkit_client_request_message_extension_create(registered,
        UINT8_C(1), &extension); /* This caller selects Criticality Indicator=true. */
    registered = NULL;
    if (status != SUCCESS) {
        fprintf(stderr, "Failed to create the first request extension (status=%" PRId32 ")\n",
                status);
        goto cleanup;
    }
    CHECK(kmipkit_client_batch_item_discover_versions(&batch_item));
    status = kmipkit_client_batch_item_with_extension(batch_item, extension,
        &updated_batch_item);
    batch_item = NULL;
    extension = NULL;
    if (status != SUCCESS) {
        fprintf(stderr, "Failed to attach the first request extension (status=%" PRId32 ")\n",
                status);
        goto cleanup;
    }
    batch_item = updated_batch_item;
    updated_batch_item = NULL;

    CHECK(make_payload(codec_limits, second_discriminator,
        sizeof(second_discriminator) - 1U, 84, &second_payload));
    CHECK(kmipkit_client_extension_registry_validate(configured_registry,
        second_identity, second_payload, codec_limits, &registered));
    status = kmipkit_client_request_message_extension_create(registered,
        UINT8_C(0), &extension); /* This caller selects Criticality Indicator=false. */
    registered = NULL;
    if (status != SUCCESS) {
        fprintf(stderr, "Failed to create the second request extension (status=%" PRId32 ")\n",
                status);
        goto cleanup;
    }
    status = kmipkit_client_batch_item_with_extension(batch_item, extension,
        &updated_batch_item);
    batch_item = NULL;
    extension = NULL;
    if (status != SUCCESS) {
        fprintf(stderr, "Failed to attach the second request extension (status=%" PRId32 ")\n",
                status);
        goto cleanup;
    }
    batch_item = updated_batch_item;
    updated_batch_item = NULL;

    CHECK(kmipkit_client_batch_item_extension_count(batch_item, &extension_count));
    if (extension_count != 2U) {
        fprintf(stderr, "Expected two attached extensions; found %" PRIu64 "\n",
                extension_count);
        goto cleanup;
    }
    CHECK(kmipkit_client_batch_item_extension_identity_at(batch_item, 0U,
        &attached_identity));
    if (!identity_name_matches(attached_identity, first_name,
            sizeof(first_name) - 1U)) {
        fprintf(stderr, "The first attached extension is out of order\n");
        goto cleanup;
    }
    kmipkit_extension_identity_release(attached_identity);
    attached_identity = NULL;
    CHECK(kmipkit_client_batch_item_extension_criticality_indicator_at(
        batch_item, 0U, &criticality));
    if (criticality != 1U) {
        fprintf(stderr, "The first extension did not retain criticality=true\n");
        goto cleanup;
    }
    CHECK(kmipkit_client_batch_item_extension_identity_at(batch_item, 1U,
        &attached_identity));
    if (!identity_name_matches(attached_identity, second_name,
            sizeof(second_name) - 1U)) {
        fprintf(stderr, "The second attached extension is out of order\n");
        goto cleanup;
    }
    kmipkit_extension_identity_release(attached_identity);
    attached_identity = NULL;
    CHECK(kmipkit_client_batch_item_extension_criticality_indicator_at(
        batch_item, 1U, &criticality));
    if (criticality != 0U) {
        fprintf(stderr, "The second extension did not retain criticality=false\n");
        goto cleanup;
    }

    puts("Attached example.vendor/accounting then example.vendor/audit to one Discover Versions item.");
    puts("Criticality Indicators are true then false; typed attachment order verified.");
    puts("No network request was sent.");
    exit_code = 0;

cleanup:
    kmipkit_extension_identity_release(attached_identity);
    kmipkit_client_batch_item_release(updated_batch_item);
    kmipkit_client_batch_item_release(batch_item);
    kmipkit_client_request_message_extension_release(extension);
    kmipkit_registered_extension_value_release(registered);
    kmipkit_ttlv_structure_release(schema_invalid_payload);
    kmipkit_ttlv_structure_release(first_payload);
    kmipkit_ttlv_structure_release(second_payload);
    kmipkit_client_extension_registry_release(configured_registry);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
    kmipkit_extension_definition_release(first_definition);
    kmipkit_extension_definition_release(second_definition);
    kmipkit_extension_identity_release(first_identity);
    kmipkit_extension_identity_release(second_identity);
    kmipkit_codec_limits_release(codec_limits);
    return exit_code;
}
