/* Compile and exercise the client construction surface from the public header. */
#include "kmipkit.h"

#include <string.h>

int kmipkit_public_client_surface_probe(void) {
    kmipkit_client_batch_t *batch = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    int32_t status = kmipkit_client_batch_create(&batch);
    if (status != 0 || batch == NULL) {
        return 1;
    }

    status = kmipkit_client_batch_item_discover_versions(&item);
    if (status != 0 || item == NULL) {
        kmipkit_client_batch_release(batch);
        return 2;
    }
    status = kmipkit_client_batch_add_item(batch, &item);
    if (status != 0 || item != NULL) {
        if (item != NULL) {
            kmipkit_client_batch_item_release(item);
        }
        kmipkit_client_batch_release(batch);
        return 3;
    }
    kmipkit_client_batch_release(batch);

    /* A null client/batch is rejected before I/O; -1 exercises the no-override path. */
    kmipkit_client_batch_t *empty_batch = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    status = kmipkit_client_execute_with_options(NULL, &empty_batch, NULL, -1, &response);
    return status == 4 && response == NULL ? 0 : 4;
}

int kmipkit_public_fluent_builder_probe(void) {
    static const uint8_t name[] = "primary encryption key";
    kmipkit_symmetric_key_builder_t *builder = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    uint32_t operation = UINT32_C(0);
    int32_t status;

    status = kmipkit_symmetric_key_builder_create(&builder);
    if (status != INT32_C(0) || builder == NULL) {
        return 1;
    }
    if (kmipkit_symmetric_key_builder_set_algorithm(
            builder, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES) != INT32_C(0) ||
        kmipkit_symmetric_key_builder_set_length(builder, INT32_C(256)) != INT32_C(0) ||
        kmipkit_symmetric_key_builder_set_usage_mask(
            builder,
            (int32_t)(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
                      KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT)) != INT32_C(0) ||
        kmipkit_symmetric_key_builder_set_name(
            builder, name, (uint64_t)(sizeof(name) - 1)) != INT32_C(0)) {
        kmipkit_symmetric_key_builder_release(builder);
        return 2;
    }
    status = kmipkit_symmetric_key_builder_build_batch_item(builder, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_ENUMERATION_OPERATION_CREATE) {
        kmipkit_client_batch_item_release(item);
        kmipkit_symmetric_key_builder_release(builder);
        return 3;
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_symmetric_key_builder_release(builder);

    /* A larger struct_size is forward-compatible; null client fails before network I/O. */
    kmipkit_symmetric_key_request_v1_t request = {0};
    kmipkit_client_batch_response_t *response = NULL;
    request.struct_size = (uint32_t)sizeof(request) + UINT32_C(8);
    request.abi_version = UINT32_C(1);
    request.cryptographic_algorithm = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
    request.cryptographic_length = INT32_C(256);
    request.cryptographic_usage_mask =
        (int32_t)(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
                  KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT);
    status = kmipkit_client_create_symmetric_key_v1(NULL, &request, NULL, &response);
    if (status != KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        kmipkit_client_batch_response_release(response);
        return 4;
    }

    request.struct_size = (uint32_t)sizeof(request) - UINT32_C(1);
    response = NULL;
    status = kmipkit_client_create_symmetric_key_v1(NULL, &request, NULL, &response);
    if (status != KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        kmipkit_client_batch_response_release(response);
        return 5;
    }
    return 0;
}

static int kmipkit_public_key_pair_builder_attributes_probe(
    kmipkit_create_key_pair_builder_t *builder) {
    static const uint8_t description[] = "key pair description";
    static const uint8_t vendor[] = "Vendor_Name.1";
    static const uint8_t name[] = "profile";
    static const uint8_t invalid_utf8[] = {UINT8_C(0xFF)};
    const uint32_t expert_tag_raw = UINT32_C(0x54F001);
    const uint32_t description_tag_raw = UINT32_C(0x4200FC);
    const uint32_t algorithm_tag_raw = UINT32_C(0x420028);
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_raw_tag_t *expert_raw = NULL;
    kmipkit_raw_tag_t *description_raw = NULL;
    kmipkit_raw_tag_t *algorithm_raw = NULL;
    kmipkit_tag_t *expert_tag = NULL;
    kmipkit_tag_t *description_tag = NULL;
    kmipkit_tag_t *algorithm_tag = NULL;
    kmipkit_ttlv_value_t *expert_value = NULL;
    kmipkit_ttlv_value_t *description_value = NULL;
    kmipkit_ttlv_value_t *wrong_description_value = NULL;
    kmipkit_ttlv_value_t *bad_algorithm_value = NULL;
    kmipkit_ttlv_value_t *nested_value = NULL;
    kmipkit_ttlv_item_t *expert_item = NULL;
    kmipkit_ttlv_item_t *bad_algorithm_item = NULL;
    kmipkit_ttlv_item_t *nested_item = NULL;
    kmipkit_ttlv_structure_t *nested = NULL;
    int result = 1;

    if (kmipkit_codec_limits_defaults(&limits) != INT32_C(0) ||
        kmipkit_ttlv_raw_tag_create(expert_tag_raw, &expert_raw) != INT32_C(0) ||
        kmipkit_ttlv_raw_tag_try_checked(expert_raw, &expert_tag) != INT32_C(0) ||
        kmipkit_ttlv_value_integer(INT32_C(17), &expert_value) != INT32_C(0) ||
        kmipkit_ttlv_item_create(expert_tag, expert_value, limits, &expert_item) != INT32_C(0)) {
        goto cleanup;
    }
    expert_value = NULL;
    if (kmipkit_ttlv_raw_tag_create(description_tag_raw, &description_raw) != INT32_C(0) ||
        kmipkit_ttlv_raw_tag_try_checked(description_raw, &description_tag) != INT32_C(0) ||
        kmipkit_ttlv_value_text_string(
            limits, description, (uint64_t)(sizeof(description) - 1), &description_value) !=
            INT32_C(0) ||
        kmipkit_ttlv_value_integer(INT32_C(17), &wrong_description_value) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_add_common_catalogued_attribute(
            builder, description_tag_raw, wrong_description_value) !=
            KMIPKIT_ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH ||
        kmipkit_create_key_pair_builder_add_common_catalogued_attribute(
            builder, description_tag_raw, description_value) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_add_private_catalogued_attribute(
            builder, description_tag_raw, description_value) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_add_public_catalogued_attribute(
            builder, description_tag_raw, description_value) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_add_common_attribute_item(builder, expert_item) !=
            INT32_C(0) ||
        kmipkit_create_key_pair_builder_add_private_attribute_item(builder, expert_item) !=
            INT32_C(0) ||
        kmipkit_create_key_pair_builder_add_public_attribute_item(builder, expert_item) !=
            INT32_C(0) ||
        kmipkit_ttlv_raw_tag_create(algorithm_tag_raw, &algorithm_raw) != INT32_C(0) ||
        kmipkit_ttlv_raw_tag_try_checked(algorithm_raw, &algorithm_tag) != INT32_C(0) ||
        kmipkit_ttlv_value_text_string(
            limits, description, (uint64_t)(sizeof(description) - 1), &bad_algorithm_value) !=
            INT32_C(0) ||
        kmipkit_ttlv_item_create(
            algorithm_tag, bad_algorithm_value, limits, &bad_algorithm_item) != INT32_C(0)) {
        goto cleanup;
    }
    bad_algorithm_value = NULL;
    if (kmipkit_create_key_pair_builder_add_private_attribute_item(
            builder, bad_algorithm_item) != KMIPKIT_ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH ||
        kmipkit_create_key_pair_builder_add_common_vendor_attribute(
            builder, vendor, (uint64_t)(sizeof(vendor) - 1), name,
            (uint64_t)(sizeof(name) - 1), expert_item) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_add_private_vendor_attribute(
            builder, vendor, (uint64_t)(sizeof(vendor) - 1), name,
            (uint64_t)(sizeof(name) - 1), expert_item) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_add_public_vendor_attribute(
            builder, vendor, (uint64_t)(sizeof(vendor) - 1), name,
            (uint64_t)(sizeof(name) - 1), expert_item) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_add_public_vendor_attribute(
            builder, invalid_utf8, (uint64_t)sizeof(invalid_utf8), name,
            (uint64_t)(sizeof(name) - 1), expert_item) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_create_key_pair_builder_add_common_attribute_item(builder, NULL) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_create_key_pair_builder_set_asynchronous_indicator(builder, UINT32_MAX) !=
            INT32_C(0) ||
        kmipkit_ttlv_structure_create(&nested) != INT32_C(0) ||
        kmipkit_ttlv_value_structure(nested, limits, &nested_value) != INT32_C(0)) {
        goto cleanup;
    }
    nested = NULL;
    if (kmipkit_ttlv_item_create(expert_tag, nested_value, limits, &nested_item) != INT32_C(0)) {
        goto cleanup;
    }
    nested_value = NULL;
    if (kmipkit_create_key_pair_builder_add_public_vendor_attribute(
            builder, vendor, (uint64_t)(sizeof(vendor) - 1), name,
            (uint64_t)(sizeof(name) - 1), nested_item) != INT32_C(0)) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_ttlv_item_release(nested_item);
    kmipkit_ttlv_structure_release(nested);
    kmipkit_ttlv_value_release(nested_value);
    kmipkit_ttlv_item_release(bad_algorithm_item);
    kmipkit_ttlv_value_release(bad_algorithm_value);
    kmipkit_ttlv_value_release(wrong_description_value);
    kmipkit_ttlv_item_release(expert_item);
    kmipkit_ttlv_value_release(expert_value);
    kmipkit_ttlv_value_release(description_value);
    kmipkit_tag_release(algorithm_tag);
    kmipkit_raw_tag_release(algorithm_raw);
    kmipkit_tag_release(description_tag);
    kmipkit_raw_tag_release(description_raw);
    kmipkit_tag_release(expert_tag);
    kmipkit_raw_tag_release(expert_raw);
    kmipkit_codec_limits_release(limits);
    return result;
}

int kmipkit_public_symmetric_builder_execute_validation_probe(void) {
    kmipkit_symmetric_key_builder_t *builder = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_request_options_v1_t options = {0};
    if (kmipkit_symmetric_key_builder_create(&builder) != INT32_C(0) || builder == NULL) {
        return 1;
    }
    options.struct_size = (uint32_t)sizeof(options);
    options.abi_version = UINT32_C(1);
    if (kmipkit_symmetric_key_builder_execute_v1(builder, NULL, NULL, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT || response != NULL ||
        kmipkit_symmetric_key_builder_execute_with_options_v1(
            builder, NULL, NULL, NULL, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL ||
        kmipkit_symmetric_key_builder_execute_with_options_v1(
            builder, NULL, NULL, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL) {
        kmipkit_client_batch_response_release(response);
        kmipkit_symmetric_key_builder_release(builder);
        return 2;
    }
    kmipkit_symmetric_key_builder_release(builder);
    return 0;
}

int kmipkit_public_create_key_pair_builder_probe(void) {
    static const uint8_t private_name[] = "primary signing key";
    static const uint8_t public_name[] = "primary verification key";
    kmipkit_create_key_pair_builder_t *builder = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_request_options_v1_t options = {0};
    uint32_t operation = UINT32_C(0);
    int32_t status = kmipkit_create_key_pair_builder_create(&builder);
    if (status != INT32_C(0) || builder == NULL) {
        return 1;
    }
    if (kmipkit_create_key_pair_builder_validate(builder) != KMIPKIT_ERROR_INVALID_INPUT) {
        kmipkit_create_key_pair_builder_release(builder);
        return 2;
    }
    if (kmipkit_create_key_pair_builder_build_batch_item(builder, &item) !=
            KMIPKIT_ERROR_INVALID_INPUT || item != NULL ||
        kmipkit_create_key_pair_builder_execute_v1(builder, NULL, NULL, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        kmipkit_client_batch_response_release(response);
        kmipkit_create_key_pair_builder_release(builder);
        return 3;
    }
    if (kmipkit_create_key_pair_builder_set_algorithm(
            builder, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_ECDSA) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_set_length(builder, INT32_C(256)) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_set_private_usage_mask(
            builder, KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_SIGN) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_set_public_usage_mask(
            builder, KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_VERIFY) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_set_private_name(
            builder, private_name, (uint64_t)(sizeof(private_name) - 1)) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_set_public_name(
            builder, public_name, (uint64_t)(sizeof(public_name) - 1)) != INT32_C(0) ||
        kmipkit_create_key_pair_builder_set_private_name(builder, (const uint8_t *)"\xFF", 1) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_public_key_pair_builder_attributes_probe(builder) != 0) {
        kmipkit_create_key_pair_builder_release(builder);
        return 4;
    }
    if (kmipkit_create_key_pair_builder_validate(builder) != INT32_C(0)) {
        kmipkit_create_key_pair_builder_release(builder);
        return 5;
    }
    options.struct_size = (uint32_t)sizeof(options);
    options.abi_version = UINT32_C(1);
    if (kmipkit_create_key_pair_builder_execute_v1(builder, NULL, NULL, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT || response != NULL ||
        kmipkit_create_key_pair_builder_execute_with_options_v1(
            builder, NULL, NULL, NULL, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_create_key_pair_builder_execute_with_options_v1(
            builder, NULL, NULL, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL) {
        kmipkit_client_batch_response_release(response);
        kmipkit_create_key_pair_builder_release(builder);
        return 6;
    }
    status = kmipkit_create_key_pair_builder_build_batch_item(builder, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_ENUMERATION_OPERATION_CREATE_KEY_PAIR) {
        kmipkit_client_batch_item_release(item);
        kmipkit_create_key_pair_builder_release(builder);
        return 7;
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_create_key_pair_builder_release(builder);
    return 0;
}

int kmipkit_public_set_attribute_builder_probe(void) {
    static const uint8_t identifier[] = "key-42";
    static const uint8_t description[] = "managed by the fluent API";
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_raw_tag_t *vendor_raw_tag = NULL;
    kmipkit_tag_t *vendor_tag = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_value_t *wrong_type = NULL;
    kmipkit_ttlv_item_t *attribute_item = NULL;
    kmipkit_ttlv_item_t *vendor_value = NULL;
    kmipkit_set_attribute_builder_t *builder = NULL;
    kmipkit_set_attribute_builder_t *item_builder = NULL;
    kmipkit_set_attribute_builder_t *vendor_builder = NULL;
    kmipkit_set_attribute_request_t *request = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    uint32_t operation = UINT32_C(0);
    int32_t status;

    if (kmipkit_codec_limits_defaults(&limits) != INT32_C(0) ||
        kmipkit_ttlv_value_text_string(
            limits, description, (uint64_t)(sizeof(description) - 1), &value) != INT32_C(0) ||
        kmipkit_ttlv_value_integer(INT32_C(7), &wrong_type) != INT32_C(0) ||
        kmipkit_set_attribute_builder_create(&builder) != INT32_C(0) || builder == NULL) {
        status = 1;
        goto cleanup;
    }
    if (kmipkit_set_attribute_builder_validate(builder) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_set_attribute_builder_set_unique_identifier(
            builder, identifier, (uint64_t)(sizeof(identifier) - 1)) != INT32_C(0) ||
        kmipkit_set_attribute_builder_add_catalogued_attribute(
            builder, KMIPKIT_STANDARD_ATTRIBUTE_DESCRIPTION, wrong_type) !=
            KMIPKIT_ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH ||
        kmipkit_set_attribute_builder_add_catalogued_attribute(
            builder, KMIPKIT_STANDARD_ATTRIBUTE_DESCRIPTION, value) != INT32_C(0) ||
        kmipkit_set_attribute_builder_set_asynchronous_indicator(builder, UINT32_C(3)) !=
            INT32_C(0) ||
        kmipkit_set_attribute_builder_validate(builder) != INT32_C(0) ||
        kmipkit_set_attribute_builder_build_request(builder, &request) != INT32_C(0) ||
        request == NULL ||
        kmipkit_set_attribute_request_to_ttlv_payload(request, &payload) != INT32_C(0) ||
        payload == NULL) {
        status = 2;
        goto cleanup;
    }
    kmipkit_ttlv_structure_release(payload);
    payload = NULL;
    if (kmipkit_client_batch_item_set_attribute_from_request(request, &item) != INT32_C(0) ||
        item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_ENUMERATION_OPERATION_SET_ATTRIBUTE) {
        status = 2;
        goto cleanup;
    }
    kmipkit_client_batch_item_release(item);
    item = NULL;
    if (kmipkit_set_attribute_builder_build_batch_item(builder, &item) != INT32_C(0) ||
        item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_ENUMERATION_OPERATION_SET_ATTRIBUTE) {
        status = 2;
        goto cleanup;
    }
    kmipkit_client_batch_item_release(item);
    item = NULL;

    if (kmipkit_ttlv_raw_tag_create(UINT32_C(0x4200fc), &raw_tag) != INT32_C(0) ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != INT32_C(0) ||
        kmipkit_ttlv_item_create(tag, value, limits, &attribute_item) != INT32_C(0)) {
        status = 3;
        goto cleanup;
    }
    value = NULL;
    kmipkit_tag_release(tag);
    tag = NULL;
    kmipkit_raw_tag_release(raw_tag);
    raw_tag = NULL;
    if (kmipkit_set_attribute_builder_create(&item_builder) != INT32_C(0) ||
        kmipkit_set_attribute_builder_set_unique_identifier(
            item_builder, identifier, (uint64_t)(sizeof(identifier) - 1)) != INT32_C(0) ||
        kmipkit_set_attribute_builder_add_attribute_item(item_builder, attribute_item) != INT32_C(0) ||
        kmipkit_set_attribute_builder_validate(item_builder) != INT32_C(0) ||
        kmipkit_set_attribute_builder_build_request(item_builder, &request) != INT32_C(0)) {
        status = 4;
        goto cleanup;
    }
    kmipkit_set_attribute_request_release(request);
    request = NULL;

    if (kmipkit_ttlv_value_text_string(
            limits, description, (uint64_t)(sizeof(description) - 1), &value) != INT32_C(0) ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x540001), &vendor_raw_tag) != INT32_C(0) ||
        kmipkit_ttlv_raw_tag_try_checked(vendor_raw_tag, &vendor_tag) != INT32_C(0) ||
        kmipkit_ttlv_item_create(vendor_tag, value, limits, &vendor_value) != INT32_C(0)) {
        status = 5;
        goto cleanup;
    }
    value = NULL;
    kmipkit_tag_release(vendor_tag);
    vendor_tag = NULL;
    kmipkit_raw_tag_release(vendor_raw_tag);
    vendor_raw_tag = NULL;
    if (kmipkit_set_attribute_builder_create(&vendor_builder) != INT32_C(0) ||
        kmipkit_set_attribute_builder_set_unique_identifier(
            vendor_builder, identifier, (uint64_t)(sizeof(identifier) - 1)) != INT32_C(0) ||
        kmipkit_set_attribute_builder_add_vendor_attribute(
            vendor_builder, (const uint8_t *)"Vendor_Name.1", UINT64_C(13),
            (const uint8_t *)"region", UINT64_C(6), vendor_value) != INT32_C(0) ||
        kmipkit_set_attribute_builder_validate(vendor_builder) != INT32_C(0)) {
        status = 6;
        goto cleanup;
    }
    if (kmipkit_set_attribute_builder_execute_v1(builder, NULL, NULL, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT || response != NULL ||
        kmipkit_set_attribute_builder_execute_with_options_v1(
            builder, NULL, NULL, NULL, &response) != KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        status = 7;
        goto cleanup;
    }
    status = 0;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_client_batch_response_release(response);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_set_attribute_request_release(request);
    kmipkit_set_attribute_builder_release(vendor_builder);
    kmipkit_set_attribute_builder_release(item_builder);
    kmipkit_set_attribute_builder_release(builder);
    kmipkit_ttlv_item_release(vendor_value);
    kmipkit_ttlv_item_release(attribute_item);
    kmipkit_tag_release(vendor_tag);
    kmipkit_raw_tag_release(vendor_raw_tag);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_ttlv_value_release(wrong_type);
    kmipkit_ttlv_value_release(value);
    kmipkit_codec_limits_release(limits);
    return status;
}

int kmipkit_public_add_attribute_builder_probe(void) {
    static const uint8_t identifier[] = "key-42";
    static const uint8_t description[] = "managed by Add Attribute";
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_add_attribute_builder_t *builder = NULL;
    kmipkit_add_attribute_request_t *request = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    uint32_t operation = UINT32_C(0);
    int32_t status;

    if (kmipkit_codec_limits_defaults(&limits) != INT32_C(0) ||
        kmipkit_ttlv_value_text_string(
            limits, description, (uint64_t)(sizeof(description) - 1), &value) != INT32_C(0) ||
        kmipkit_add_attribute_builder_create(&builder) != INT32_C(0) || builder == NULL) {
        status = 1;
        goto cleanup;
    }
    if (kmipkit_add_attribute_builder_set_unique_identifier(
            builder, identifier, (uint64_t)(sizeof(identifier) - 1)) != INT32_C(0) ||
        kmipkit_add_attribute_builder_add_catalogued_attribute(
            builder, KMIPKIT_STANDARD_ATTRIBUTE_DESCRIPTION, value) != INT32_C(0) ||
        kmipkit_add_attribute_builder_validate(builder) != INT32_C(0) ||
        kmipkit_add_attribute_builder_build_request(builder, &request) != INT32_C(0) ||
        request == NULL ||
        kmipkit_add_attribute_request_to_ttlv_payload(request, &payload) != INT32_C(0) ||
        payload == NULL ||
        kmipkit_add_attribute_builder_build_batch_item(builder, &item) != INT32_C(0) ||
        item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_ENUMERATION_OPERATION_ADD_ATTRIBUTE) {
        status = 2;
        goto cleanup;
    }
    if (kmipkit_add_attribute_builder_execute_v1(
            builder, NULL, NULL, &response) != KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        status = 3;
        goto cleanup;
    }
    status = 0;

cleanup:
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_add_attribute_request_release(request);
    kmipkit_add_attribute_builder_release(builder);
    kmipkit_ttlv_value_release(value);
    kmipkit_codec_limits_release(limits);
    return status;
}

int kmipkit_public_async_batch_items_probe(void) {
    static const uint8_t correlation[] = "async-request-id";
    kmipkit_cancel_request_t *cancel_request = NULL;
    kmipkit_poll_request_t *poll_request = NULL;
    kmipkit_process_request_t *process_request = NULL;
    kmipkit_query_async_requests_request_t *query_request = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    uint32_t operation = UINT32_C(0);
    int32_t status;

    status = kmipkit_cancel_request_create(
        correlation, (uint64_t)(sizeof(correlation) - 1), &cancel_request);
    if (status != INT32_C(0) || cancel_request == NULL) {
        return 1;
    }
    status = kmipkit_client_batch_item_cancel_from_request(cancel_request, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_ENUMERATION_OPERATION_CANCEL ||
        kmipkit_cancel_request_to_ttlv_payload(cancel_request, &payload) != INT32_C(0) ||
        payload == NULL) {
        kmipkit_client_batch_item_release(item);
        kmipkit_ttlv_structure_release(payload);
        kmipkit_cancel_request_release(cancel_request);
        return 2;
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(payload);
    payload = NULL;
    kmipkit_cancel_request_release(cancel_request);

    status = kmipkit_poll_request_create(
        correlation, (uint64_t)(sizeof(correlation) - 1), &poll_request);
    if (status != INT32_C(0) || poll_request == NULL) {
        return 3;
    }
    status = kmipkit_client_batch_item_poll_from_request(poll_request, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_ENUMERATION_OPERATION_POLL ||
        kmipkit_poll_request_to_ttlv_payload(poll_request, &payload) != INT32_C(0) ||
        payload == NULL) {
        kmipkit_client_batch_item_release(item);
        kmipkit_ttlv_structure_release(payload);
        kmipkit_poll_request_release(poll_request);
        return 4;
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(payload);
    payload = NULL;
    kmipkit_poll_request_release(poll_request);

    status = kmipkit_process_request_create(
        correlation, (uint64_t)(sizeof(correlation) - 1), &process_request);
    if (status != INT32_C(0) || process_request == NULL) {
        return 5;
    }
    status = kmipkit_client_batch_item_process_from_request(process_request, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_ENUMERATION_OPERATION_PROCESS ||
        kmipkit_process_request_to_ttlv_payload(process_request, &payload) != INT32_C(0) ||
        payload == NULL) {
        kmipkit_client_batch_item_release(item);
        kmipkit_ttlv_structure_release(payload);
        kmipkit_process_request_release(process_request);
        return 6;
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(payload);
    payload = NULL;
    kmipkit_process_request_release(process_request);

    status = kmipkit_query_async_requests_request_create(&query_request);
    if (status != INT32_C(0) || query_request == NULL) {
        return 7;
    }
    status = kmipkit_client_batch_item_query_async_requests_from_request(query_request, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_ENUMERATION_OPERATION_QUERY_ASYNCHRONOUS_REQUESTS) {
        kmipkit_client_batch_item_release(item);
        kmipkit_ttlv_structure_release(payload);
        kmipkit_query_async_requests_request_release(query_request);
        return 8;
    }
    if (kmipkit_query_async_requests_request_to_ttlv_payload(query_request, &payload) != INT32_C(0) ||
        payload == NULL) {
        kmipkit_client_batch_item_release(item);
        kmipkit_ttlv_structure_release(payload);
        kmipkit_query_async_requests_request_release(query_request);
        return 9;
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_query_async_requests_request_release(query_request);
    return 0;
}

int kmipkit_public_kmip30_semantic_items_probe(void) {
    static const uint8_t identifier_text[] = "system-object";
    kmipkit_ttlv_structure_t *items = NULL;
    kmipkit_attribute_set_t *attributes = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint32_t operation = 0;
    uint64_t attribute_count = 0;
    int32_t status;

    status = kmipkit_ttlv_structure_create(&items);
    if (status != INT32_C(0) || items == NULL) {
        return 1;
    }
    status = kmipkit_attribute_set_create(items, &attributes);
    kmipkit_ttlv_structure_release(items);
    if (status != INT32_C(0) || attributes == NULL) {
        return 2;
    }

    status = kmipkit_client_batch_item_create_group_from_attributes(attributes, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_V3_0_OPERATION_CREATE_GROUP ||
        kmipkit_attribute_set_len(attributes, &attribute_count) != INT32_C(0) ||
        attribute_count != UINT64_C(0)) {
        kmipkit_client_batch_item_release(item);
        kmipkit_attribute_set_release(attributes);
        return 3;
    }
    kmipkit_attribute_set_release(attributes);
    attributes = NULL;
    if (kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_V3_0_OPERATION_CREATE_GROUP) {
        kmipkit_client_batch_item_release(item);
        return 4;
    }
    kmipkit_client_batch_item_release(item);
    item = NULL;

    status = kmipkit_ttlv_structure_create(&items);
    if (status != INT32_C(0) || items == NULL ||
        kmipkit_attribute_set_create(items, &attributes) != INT32_C(0) || attributes == NULL) {
        kmipkit_ttlv_structure_release(items);
        return 5;
    }
    kmipkit_ttlv_structure_release(items);
    status = kmipkit_client_batch_item_create_user_from_attributes(attributes, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_V3_0_OPERATION_CREATE_USER) {
        kmipkit_client_batch_item_release(item);
        kmipkit_attribute_set_release(attributes);
        return 6;
    }
    kmipkit_attribute_set_release(attributes);
    attributes = NULL;
    if (kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_V3_0_OPERATION_CREATE_USER) {
        kmipkit_client_batch_item_release(item);
        return 7;
    }
    kmipkit_client_batch_item_release(item);
    item = NULL;
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = (uint64_t)(sizeof(identifier_text) - 1);
    status = kmipkit_client_batch_item_obliterate_from_identifier(&identifier, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_V3_0_OPERATION_OBLITERATE) {
        kmipkit_client_batch_item_release(item);
        return 8;
    }
    kmipkit_client_batch_item_release(item);
    item = NULL;

    status = kmipkit_client_batch_item_obliterate_from_identifier(NULL, &item);
    if (status != KMIPKIT_ERROR_INVALID_INPUT || item != NULL) {
        kmipkit_client_batch_item_release(item);
        return 9;
    }

    status = kmipkit_client_batch_item_deactivate_from_fields(
        &identifier, NULL, UINT8_C(0), INT64_C(0), &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_V3_0_OPERATION_DEACTIVATE) {
        kmipkit_client_batch_item_release(item);
        return 10;
    }
    kmipkit_client_batch_item_release(item);
    item = NULL;
    status = kmipkit_client_batch_item_deactivate_from_fields(
        &identifier, NULL, UINT8_C(2), INT64_C(0), &item);
    if (status != KMIPKIT_ERROR_INVALID_INPUT || item != NULL) {
        kmipkit_client_batch_item_release(item);
        return 11;
    }
    return 0;
}

int kmipkit_public_kmip30_credential_kem_probe(void) {
    static const uint8_t identifier_text[] = "kem-key";
    static const uint8_t device_identifier[] = "device-id";
    static const uint8_t ciphertext[] = {UINT8_C(0x31), UINT8_C(0x32)};
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_ttlv_structure_t *fields = NULL;
    kmipkit_ttlv_structure_t *credential_fields = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_value_t *identifier_value = NULL;
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_raw_tag_t *raw_identifier_tag = NULL;
    kmipkit_tag_t *identifier_tag = NULL;
    kmipkit_ttlv_item_t *object = NULL;
    kmipkit_ttlv_item_t *identifier_item = NULL;
    kmipkit_ttlv_structure_t *attribute_items = NULL;
    kmipkit_attribute_set_t *attributes = NULL;
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_client_batch_item_t *item = NULL;
    uint32_t operation = 0;
    int32_t status;

    status = kmipkit_codec_limits_defaults(&limits);
    if (status != INT32_C(0) || limits == NULL ||
        kmipkit_ttlv_structure_create(&fields) != INT32_C(0) || fields == NULL ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x004200A2), &raw_identifier_tag) != INT32_C(0) ||
        raw_identifier_tag == NULL ||
        kmipkit_ttlv_raw_tag_try_checked(raw_identifier_tag, &identifier_tag) != INT32_C(0) ||
        identifier_tag == NULL ||
        kmipkit_ttlv_value_text_string(limits, device_identifier,
            (uint64_t)(sizeof(device_identifier) - 1), &identifier_value) != INT32_C(0) ||
        identifier_value == NULL ||
        kmipkit_ttlv_item_create(identifier_tag, identifier_value, limits,
            &identifier_item) != INT32_C(0) || identifier_item == NULL ||
        kmipkit_ttlv_structure_with_item(fields, identifier_item, limits,
            &credential_fields) != INT32_C(0) || credential_fields == NULL ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x004201A6), &raw_tag) != INT32_C(0) ||
        raw_tag == NULL || kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != INT32_C(0) ||
        tag == NULL || kmipkit_ttlv_value_structure(credential_fields, limits, &value) != INT32_C(0) ||
        value == NULL) {
        return 1;
    }
    fields = NULL;
    identifier_item = NULL;
    credential_fields = NULL;
    status = kmipkit_ttlv_item_create(tag, value, limits, &object);
    if (status != INT32_C(0) || object == NULL ||
        kmipkit_ttlv_structure_create(&attribute_items) != INT32_C(0) ||
        attribute_items == NULL ||
        kmipkit_attribute_set_create(attribute_items, &attributes) != INT32_C(0) ||
        attributes == NULL) {
        return 2;
    }
    value = NULL;
    tag = NULL;
    attribute_items = NULL;

    status = kmipkit_client_batch_item_create_credential_from_fields(
        KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_DEVICE,
        KMIPKIT_V3_0_OBJECT_TYPE_DEVICE_CREDENTIAL,
        attributes, object, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_V3_0_OPERATION_CREATE_CREDENTIAL) {
        return 3;
    }
    kmipkit_client_batch_item_release(item);
    item = NULL;
    status = kmipkit_client_batch_item_create_credential_from_fields(
        KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_DEVICE,
        KMIPKIT_V3_0_OBJECT_TYPE_PASSWORD_CREDENTIAL,
        attributes, object, &item);
    if (status != KMIPKIT_ERROR_INVALID_INPUT || item != NULL) {
        return 6;
    }

    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = 1;
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = (uint64_t)(sizeof(identifier_text) - 1);
    status = kmipkit_client_batch_item_encapsulate_from_fields(
        &identifier, NULL, attributes, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_V3_0_OPERATION_ENCAPSULATE) {
        return 4;
    }
    kmipkit_client_batch_item_release(item);
    item = NULL;

    status = kmipkit_client_batch_item_decapsulate_from_fields(
        &identifier, ciphertext, (uint64_t)sizeof(ciphertext), NULL, attributes, &item);
    if (status != INT32_C(0) || item == NULL ||
        kmipkit_client_batch_item_operation(item, &operation) != INT32_C(0) ||
        operation != KMIPKIT_V3_0_OPERATION_DECAPSULATE) {
        return 5;
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_attribute_set_release(attributes);
    kmipkit_ttlv_item_release(object);
    kmipkit_tag_release(identifier_tag);
    kmipkit_raw_tag_release(raw_identifier_tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_codec_limits_release(limits);
    return 0;
}

int kmipkit_public_fluent_validation_probe(void) {
    static const uint8_t description[] = "managed without TTLV tags";
    static const uint8_t unknown_value[] = "expert value";
    static const uint8_t invalid_utf8[] = {UINT8_C(0xff)};
    static const uint8_t vendor[] = "Vendor_Name.1";
    static const uint8_t vendor_name[] = "region";
    static const uint8_t identifier_text[] = "fluent-id";
    kmipkit_symmetric_key_builder_t *builder = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_value_t *wrong_value = NULL;
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_raw_tag_t *raw_vendor_tag = NULL;
    kmipkit_tag_t *vendor_tag = NULL;
    kmipkit_ttlv_value_t *vendor_value = NULL;
    kmipkit_ttlv_item_t *vendor_item = NULL;
    kmipkit_client_batch_item_t *built_item = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_symmetric_key_request_v1_t request = {0};
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_request_options_v1_t options = {0};
    uint32_t operation = 0;
    int32_t result = 1;

    if (kmipkit_symmetric_key_builder_set_algorithm(NULL, 3) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_set_length(NULL, 256) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_set_usage_mask(NULL, 12) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_set_name(NULL, description, sizeof(description) - 1) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_create(&builder) != 0 || builder == NULL) {
        result = 11;
        goto cleanup;
    }
    if (kmipkit_symmetric_key_builder_build_batch_item(builder, &built_item) !=
            KMIPKIT_ERROR_INVALID_INPUT || built_item != NULL) {
        result = 121;
        goto cleanup;
    }
    if (kmipkit_codec_limits_defaults(&limits) != 0 || limits == NULL) {
        result = 122;
        goto cleanup;
    }
    if (kmipkit_ttlv_value_text_string(limits, description, sizeof(description) - 1, &value) != 0) {
        result = 123;
        goto cleanup;
    }
    if (kmipkit_symmetric_key_builder_add_catalogued_attribute(
            builder, UINT32_C(0x4200fc), value) != 0) {
        result = 124;
        goto cleanup;
    }
    if (kmipkit_ttlv_value_integer(1, &wrong_value) != 0) {
        result = 125;
        goto cleanup;
    }
    if (kmipkit_symmetric_key_builder_add_catalogued_attribute(
            builder, UINT32_C(0x4200fc), wrong_value) !=
        KMIPKIT_ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH) {
        result = 126;
        goto cleanup;
    }
    kmipkit_ttlv_value_release(wrong_value);
    wrong_value = NULL;
    if (
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x4200fd), &raw_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
        kmipkit_ttlv_value_text_string(limits, unknown_value, sizeof(unknown_value) - 1, &wrong_value) != 0 ||
        kmipkit_ttlv_item_create(tag, wrong_value, limits, &item) != 0) {
        result = 13;
        goto cleanup;
    }
    wrong_value = NULL; /* The successful Item constructor consumes its Value. */
    if (kmipkit_symmetric_key_builder_add_attribute_item(builder, item) != 0 ||
        kmipkit_symmetric_key_builder_add_vendor_attribute(
            builder, vendor, sizeof(vendor) - 1, vendor_name, sizeof(vendor_name) - 1, item) != 0 ||
        kmipkit_ttlv_raw_tag_create(UINT32_C(0x540001), &raw_vendor_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_vendor_tag, &vendor_tag) != 0 ||
        kmipkit_ttlv_value_text_string(limits, unknown_value, sizeof(unknown_value) - 1,
                                       &vendor_value) != 0 ||
        kmipkit_ttlv_item_create(vendor_tag, vendor_value, limits, &vendor_item) != 0) {
        result = 14;
        goto cleanup;
    }
    vendor_value = NULL;
    if (kmipkit_symmetric_key_builder_add_attribute_item(builder, vendor_item) != 0) {
        result = 141;
        goto cleanup;
    }
    kmipkit_ttlv_item_release(vendor_item);
    vendor_item = NULL;
    kmipkit_tag_release(vendor_tag);
    vendor_tag = NULL;
    kmipkit_raw_tag_release(raw_vendor_tag);
    raw_vendor_tag = NULL;
    if (kmipkit_ttlv_raw_tag_create(UINT32_C(0x420008), &raw_vendor_tag) != 0 ||
        kmipkit_ttlv_raw_tag_try_checked(raw_vendor_tag, &vendor_tag) != 0 ||
        kmipkit_ttlv_value_text_string(limits, unknown_value, sizeof(unknown_value) - 1,
                                       &vendor_value) != 0 ||
        kmipkit_ttlv_item_create(vendor_tag, vendor_value, limits, &vendor_item) != 0) {
        result = 142;
        goto cleanup;
    }
    vendor_value = NULL;
    if (kmipkit_symmetric_key_builder_add_attribute_item(builder, vendor_item) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_set_name(builder, invalid_utf8, sizeof(invalid_utf8)) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_symmetric_key_builder_set_algorithm(
            builder, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES) != 0 ||
        kmipkit_symmetric_key_builder_set_length(builder, 256) != 0 ||
        kmipkit_symmetric_key_builder_set_usage_mask(
            builder, KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT) != 0 ||
        kmipkit_symmetric_key_builder_build_batch_item(builder, &built_item) != 0 ||
        built_item == NULL ||
        kmipkit_client_batch_item_operation(built_item, &operation) != 0 ||
        operation != KMIPKIT_ENUMERATION_OPERATION_CREATE) {
        result = 15;
        goto cleanup;
    }

    options.struct_size = (uint32_t)sizeof(options);
    options.abi_version = UINT32_C(1);
    if (kmipkit_symmetric_key_builder_execute_v1(builder, NULL, limits, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT || response != NULL ||
        kmipkit_symmetric_key_builder_execute_with_options_v1(
            builder, NULL, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL) {
        result = 16;
        goto cleanup;
    }
    request.struct_size = (uint32_t)sizeof(request);
    request.abi_version = UINT32_C(1);
    if (kmipkit_client_create_symmetric_key_v1(NULL, &request, limits, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT || response != NULL) {
        result = 171;
        goto cleanup;
    }
    request.cryptographic_algorithm = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
    request.cryptographic_length = INT32_C(256);
    request.cryptographic_usage_mask = KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT;
    if (kmipkit_client_create_symmetric_key_v1(NULL, &request, limits, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_create_symmetric_key_with_options_v1(
            NULL, &request, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT) {
        result = 17;
        goto cleanup;
    }
    if (kmipkit_client_create_symmetric_key_with_options_v1(
            NULL, &request, limits, NULL, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL ||
        kmipkit_client_get_by_id_with_options_v1(
            NULL, NULL, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL ||
        kmipkit_client_destroy_by_id_with_options_v1(
            NULL, NULL, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        response != NULL) {
        result = 172;
        goto cleanup;
    }
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_text;
    identifier.text_length = sizeof(identifier_text) - 1;
    if (kmipkit_client_get_by_id_v1(NULL, &identifier, limits, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_get_by_id_with_options_v1(
            NULL, &identifier, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_destroy_by_id_v1(NULL, &identifier, limits, &response) !=
            KMIPKIT_ERROR_INVALID_INPUT ||
        kmipkit_client_destroy_by_id_with_options_v1(
            NULL, &identifier, limits, &options, &response) != KMIPKIT_ERROR_INVALID_INPUT) {
        result = 18;
        goto cleanup;
    }
    options.abi_version = UINT32_C(2);
    result = kmipkit_client_get_by_id_with_options_v1(
        NULL, &identifier, limits, &options, &response) == KMIPKIT_ERROR_INVALID_INPUT ? 0 : 2;

cleanup:
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_item_release(built_item);
    kmipkit_ttlv_item_release(vendor_item);
    kmipkit_ttlv_value_release(vendor_value);
    kmipkit_tag_release(vendor_tag);
    kmipkit_raw_tag_release(raw_vendor_tag);
    kmipkit_ttlv_item_release(item);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_ttlv_value_release(wrong_value);
    kmipkit_ttlv_value_release(value);
    kmipkit_codec_limits_release(limits);
    kmipkit_symmetric_key_builder_release(builder);
    return result;
}

int kmipkit_public_client_execute_json_override(
    uint16_t port,
    const uint8_t *leaf_certificate,
    uint64_t leaf_certificate_length,
    const uint8_t *intermediate_certificate,
    uint64_t intermediate_certificate_length,
    const uint8_t *private_key,
    uint64_t private_key_length,
    const uint8_t *trust_root,
    uint64_t trust_root_length) {
    static const uint8_t endpoint[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_client_configuration_t *configuration = NULL;
    kmipkit_transport_config_t *transport_configuration = NULL;
    kmipkit_client_t *client = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_client_batch_t *batch = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_transport_input_v1_t client_certificates[2] = {{0}};
    kmipkit_transport_input_v1_t trust_certificates[1] = {{0}};
    kmipkit_transport_config_v1_t transport = {0};
    uint64_t response_count = 0;
    int32_t result = 1;
    int32_t execute_status;

#define REQUIRE_C_API(call, error_code) \
    do { \
        if ((call) != INT32_C(0)) { \
            result = (error_code); \
            goto cleanup; \
        } \
    } while (0)

    if (leaf_certificate == NULL || leaf_certificate_length == 0 ||
        intermediate_certificate == NULL || intermediate_certificate_length == 0 ||
        private_key == NULL || private_key_length == 0 || trust_root == NULL ||
        trust_root_length == 0 || port == 0) {
        result = 1;
        goto cleanup;
    }

    registry_limits.struct_size = (uint32_t)sizeof(registry_limits);
    registry_limits.abi_version = UINT32_C(1);
    REQUIRE_C_API(
        kmipkit_extension_registry_limits_v1_default_values(&registry_limits), 2);
    REQUIRE_C_API(
        kmipkit_client_extension_registry_create_v1(NULL, UINT64_C(0),
                                                    &registry_limits, &registry),
        3);
    REQUIRE_C_API(kmipkit_client_configuration_create(registry, &configuration), 4);
    registry = NULL; /* The configuration constructor consumes this handle. */

    client_certificates[0].source_kind = KMIPKIT_TRANSPORT_SOURCE_MEMORY;
    client_certificates[0].encoding = KMIPKIT_TRANSPORT_ENCODING_DER;
    client_certificates[0].data.data = leaf_certificate;
    client_certificates[0].data.length = leaf_certificate_length;
    client_certificates[1].source_kind = KMIPKIT_TRANSPORT_SOURCE_MEMORY;
    client_certificates[1].encoding = KMIPKIT_TRANSPORT_ENCODING_DER;
    client_certificates[1].data.data = intermediate_certificate;
    client_certificates[1].data.length = intermediate_certificate_length;
    trust_certificates[0].source_kind = KMIPKIT_TRANSPORT_SOURCE_MEMORY;
    trust_certificates[0].encoding = KMIPKIT_TRANSPORT_ENCODING_DER;
    trust_certificates[0].data.data = trust_root;
    trust_certificates[0].data.length = trust_root_length;

    transport.struct_size = (uint32_t)sizeof(transport);
    transport.abi_version = UINT32_C(1);
    transport.endpoint_kind = KMIPKIT_TRANSPORT_ENDPOINT_RAW_TLS;
    transport.trust_kind = KMIPKIT_TRANSPORT_TRUST_CERTIFICATES;
    transport.flags = KMIPKIT_TRANSPORT_CONFIG_FLAG_TLS_SERVER_NAME;
    transport.endpoint.data = endpoint;
    transport.endpoint.length = (uint64_t)(sizeof(endpoint) - 1);
    transport.endpoint_port = port;
    transport.client_certificates = client_certificates;
    transport.client_certificate_count = UINT64_C(2);
    transport.client_private_key.source_kind = KMIPKIT_TRANSPORT_SOURCE_MEMORY;
    transport.client_private_key.encoding = KMIPKIT_TRANSPORT_ENCODING_DER;
    transport.client_private_key.data.data = private_key;
    transport.client_private_key.data.length = private_key_length;
    transport.trust_certificates = trust_certificates;
    transport.trust_certificate_count = UINT64_C(1);
    transport.tls_server_name.data = server_name;
    transport.tls_server_name.length = (uint64_t)(sizeof(server_name) - 1);
    transport.connect_timeout.kind = KMIPKIT_TRANSPORT_TIMEOUT_DEFAULT;
    transport.write_timeout.kind = KMIPKIT_TRANSPORT_TIMEOUT_DEFAULT;
    transport.read_timeout.kind = KMIPKIT_TRANSPORT_TIMEOUT_DEFAULT;
    transport.total_timeout.kind = KMIPKIT_TRANSPORT_TIMEOUT_DEFAULT;
    transport.max_request_bytes = UINT64_C(1048576);

    REQUIRE_C_API(kmipkit_transport_config_create_v1(&transport, &transport_configuration), 5);
    REQUIRE_C_API(kmipkit_client_create(configuration, &transport_configuration, &client), 6);
    REQUIRE_C_API(kmipkit_codec_limits_defaults(&limits), 7);
    REQUIRE_C_API(kmipkit_client_batch_create(&batch), 8);
    REQUIRE_C_API(kmipkit_client_batch_item_discover_versions(&item), 9);
    REQUIRE_C_API(kmipkit_client_batch_add_item(batch, &item), 10);
    execute_status = kmipkit_client_execute_with_options(
        client, &batch, limits, KMIPKIT_WIRE_ENCODING_JSON, &response);
    if (execute_status != INT32_C(0)) {
        result = INT32_C(100) + execute_status;
        goto cleanup;
    }
    if (response == NULL) {
        result = 12;
        goto cleanup;
    }
    REQUIRE_C_API(kmipkit_client_batch_response_item_count(response, &response_count), 13);
    if (response_count != UINT64_C(1)) {
        result = 14;
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    kmipkit_client_batch_item_release(item);
    kmipkit_client_release(client);
    kmipkit_codec_limits_release(limits);
    kmipkit_client_configuration_release(configuration);
    kmipkit_transport_config_release(transport_configuration);
#undef REQUIRE_C_API
    return result;
}

int kmipkit_public_fluent_lifecycle_probe(
    uint16_t port,
    const uint8_t *leaf_certificate,
    uint64_t leaf_certificate_length,
    const uint8_t *intermediate_certificate,
    uint64_t intermediate_certificate_length,
    const uint8_t *private_key,
    uint64_t private_key_length,
    const uint8_t *trust_root,
    uint64_t trust_root_length) {
    static const uint8_t endpoint[] = "127.0.0.1";
    static const uint8_t server_name[] = "server.kmipkit.test";
    static const uint8_t expected_identifier[] = "fluent-key-42";
    kmipkit_extension_registry_limits_v1_t registry_limits = {0};
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_client_configuration_t *configuration = NULL;
    kmipkit_client_t *client = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_client_batch_response_t *create_response = NULL;
    kmipkit_client_batch_item_response_t *create_item = NULL;
    kmipkit_client_batch_response_t *get_response = NULL;
    kmipkit_client_batch_item_response_t *get_item = NULL;
    kmipkit_client_batch_response_t *destroy_response = NULL;
    kmipkit_client_batch_item_response_t *destroy_item = NULL;
    kmipkit_ttlv_structure_t *get_object = NULL;
    kmipkit_byte_span_v1_t client_certificates[2] = {{0}};
    kmipkit_byte_span_v1_t trust_certificates[1] = {{0}};
    kmipkit_mtls_client_config_v1_t tls = {0};
    kmipkit_symmetric_key_request_v1_t request = {0};
    kmipkit_request_options_v1_t request_options = {0};
    kmipkit_unique_identifier_v1_t identifier = {0};
    uint8_t create_present = 0;
    uint32_t create_object_type = 0;
    uint8_t get_present = 0;
    uint8_t get_payload_present = 0;
    uint32_t get_object_type = 0;
    uint8_t destroy_present = 0;
    int32_t result = 1;

#define REQUIRE_LIFECYCLE_C_API(call, error_code) \
    do { \
        if ((call) != INT32_C(0)) { \
            result = (error_code); \
            goto lifecycle_cleanup; \
        } \
    } while (0)

    if (port == 0 || leaf_certificate == NULL || leaf_certificate_length == 0 ||
        intermediate_certificate == NULL || intermediate_certificate_length == 0 ||
        private_key == NULL || private_key_length == 0 || trust_root == NULL ||
        trust_root_length == 0) {
        goto lifecycle_cleanup;
    }

    registry_limits.struct_size = (uint32_t)sizeof(registry_limits);
    registry_limits.abi_version = UINT32_C(1);
    REQUIRE_LIFECYCLE_C_API(
        kmipkit_extension_registry_limits_v1_default_values(&registry_limits), 2);
    REQUIRE_LIFECYCLE_C_API(
        kmipkit_client_extension_registry_create_v1(
            NULL, UINT64_C(0), &registry_limits, &registry), 3);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_configuration_create(registry, &configuration), 4);
    registry = NULL;

    client_certificates[0].data = leaf_certificate;
    client_certificates[0].length = leaf_certificate_length;
    client_certificates[1].data = intermediate_certificate;
    client_certificates[1].length = intermediate_certificate_length;
    trust_certificates[0].data = trust_root;
    trust_certificates[0].length = trust_root_length;

    tls.struct_size = (uint32_t)sizeof(tls);
    tls.abi_version = UINT32_C(1);
    tls.endpoint_host.data = endpoint;
    tls.endpoint_host.length = (uint64_t)(sizeof(endpoint) - 1);
    tls.endpoint_port = port;
    tls.tls_server_name.data = server_name;
    tls.tls_server_name.length = (uint64_t)(sizeof(server_name) - 1);
    tls.client_certificates = client_certificates;
    tls.client_certificate_count = UINT64_C(2);
    tls.client_private_key.data = private_key;
    tls.client_private_key.length = private_key_length;
    tls.trust_certificates = trust_certificates;
    tls.trust_certificate_count = UINT64_C(1);
    tls.max_request_bytes = UINT64_C(1024 * 1024);
    tls.abi_version = UINT32_C(2);
    if (kmipkit_client_create_mtls_v1(configuration, &tls, &client) !=
            KMIPKIT_ERROR_INVALID_INPUT || client != NULL) {
        result = 21;
        goto lifecycle_cleanup;
    }
    tls.abi_version = UINT32_C(1);
    tls.struct_size -= UINT32_C(1);
    if (kmipkit_client_create_mtls_v1(configuration, &tls, &client) !=
            KMIPKIT_ERROR_INVALID_INPUT || client != NULL) {
        result = 22;
        goto lifecycle_cleanup;
    }
    tls.struct_size = (uint32_t)sizeof(tls);
    tls.client_certificates = NULL;
    tls.client_certificate_count = UINT64_C(0);
    if (kmipkit_client_create_mtls_v1(configuration, &tls, &client) !=
            KMIPKIT_ERROR_INVALID_INPUT || client != NULL) {
        result = 23;
        goto lifecycle_cleanup;
    }
    tls.client_certificates = client_certificates;
    tls.client_certificate_count = UINT64_C(2);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_create_mtls_v1(configuration, &tls, &client), 6);
    REQUIRE_LIFECYCLE_C_API(kmipkit_codec_limits_defaults(&limits), 7);

    request.struct_size = (uint32_t)sizeof(request);
    request.abi_version = UINT32_C(1);
    request.cryptographic_algorithm = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
    request.cryptographic_length = INT32_C(256);
    request.cryptographic_usage_mask = (int32_t)(
        KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT |
        KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT);
    request.name_present = UINT8_C(1);
    request.name.data = expected_identifier;
    request.name.length = (uint64_t)(sizeof(expected_identifier) - 1);
    request_options.struct_size = (uint32_t)sizeof(request_options);
    request_options.abi_version = UINT32_C(1);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_create_symmetric_key_v1(
        client, &request, limits, &create_response), 8);
    if (create_response == NULL) {
        result = 9;
        goto lifecycle_cleanup;
    }
    REQUIRE_LIFECYCLE_C_API(
        kmipkit_client_batch_response_item_at(create_response, UINT64_C(0), &create_item), 10);
    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_batch_item_response_create(
        create_item, &create_present, &create_object_type, &identifier), 11);
    if (create_present != UINT8_C(1) ||
        create_object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY ||
        identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        identifier.text_length != (uint64_t)(sizeof(expected_identifier) - 1) ||
        memcmp(identifier.text_data, expected_identifier, sizeof(expected_identifier) - 1) != 0) {
        result = 12;
        goto lifecycle_cleanup;
    }

    REQUIRE_LIFECYCLE_C_API(kmipkit_client_get_by_id_with_options_v1(
        client, &identifier, limits, &request_options, &get_response), 13);
    REQUIRE_LIFECYCLE_C_API(
        kmipkit_client_batch_response_item_at(get_response, UINT64_C(0), &get_item), 14);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_batch_item_response_get(
        get_item, &get_present, &get_payload_present, &get_object_type, &identifier,
        &get_object), 15);
    if (get_present != UINT8_C(1) || get_payload_present != UINT8_C(1) ||
        get_object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY || get_object == NULL) {
        result = 16;
        goto lifecycle_cleanup;
    }

    REQUIRE_LIFECYCLE_C_API(kmipkit_client_destroy_by_id_with_options_v1(
        client, &identifier, limits, &request_options, &destroy_response), 17);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_batch_response_item_at(
        destroy_response, UINT64_C(0), &destroy_item), 18);
    REQUIRE_LIFECYCLE_C_API(kmipkit_client_batch_item_response_destroy(
        destroy_item, &destroy_present, &identifier), 19);
    result = destroy_present == UINT8_C(1) ? 0 : 20;

lifecycle_cleanup:
    kmipkit_ttlv_structure_release(get_object);
    kmipkit_client_batch_item_response_release(destroy_item);
    kmipkit_client_batch_response_release(destroy_response);
    kmipkit_client_batch_item_response_release(get_item);
    kmipkit_client_batch_response_release(get_response);
    kmipkit_client_batch_item_response_release(create_item);
    kmipkit_client_batch_response_release(create_response);
    kmipkit_client_release(client);
    kmipkit_codec_limits_release(limits);
    kmipkit_client_configuration_release(configuration);
    kmipkit_client_extension_registry_release(registry);
#undef REQUIRE_LIFECYCLE_C_API
    return result;
}
