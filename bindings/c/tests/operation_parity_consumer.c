#include "kmipkit.h"

#include <stdint.h>

#define KMIPKIT_OPERATION_INVALID_INPUT ((int32_t)4)

static const uint32_t EXPECTED_BATCH_OPERATIONS[] = {
    UINT32_C(0x12), UINT32_C(0x0d), UINT32_C(0x30), UINT32_C(0x15),
    UINT32_C(0x06), UINT32_C(0x09), UINT32_C(0x01), UINT32_C(0x02),
    UINT32_C(0x28), UINT32_C(0x20), UINT32_C(0x2f), UINT32_C(0x0f),
    UINT32_C(0x05), UINT32_C(0x14), UINT32_C(0x1e), UINT32_C(0x1f),
    UINT32_C(0x2b), UINT32_C(0x0a), UINT32_C(0x0b), UINT32_C(0x0c),
    UINT32_C(0x38), UINT32_C(0x11), UINT32_C(0x27), UINT32_C(0x2a),
    UINT32_C(0x08), UINT32_C(0x2c), UINT32_C(0x2d), UINT32_C(0x23),
    UINT32_C(0x24), UINT32_C(0x0e), UINT32_C(0x10), UINT32_C(0x3b),
    UINT32_C(0x18), UINT32_C(0x16), UINT32_C(0x03), UINT32_C(0x13),
    UINT32_C(0x07), UINT32_C(0x04), UINT32_C(0x1d), UINT32_C(0x35),
    UINT32_C(0x25), UINT32_C(0x26), UINT32_C(0x31), UINT32_C(0x37),
    UINT32_C(0x36), UINT32_C(0x21), UINT32_C(0x22), UINT32_C(0x17),
};

extern int kmipkit_client_abi_c_consumer_build_add_attribute_request(
    kmipkit_client_batch_item_t **out_item);
extern int kmipkit_client_abi_c_consumer_build_adjust_attribute(
    kmipkit_client_batch_item_t **out_item);
extern int kmipkit_client_abi_c_consumer_build_delete_attribute(
    kmipkit_client_batch_item_t **out_item);
extern int kmipkit_client_abi_c_consumer_build_modify_attribute(
    kmipkit_client_batch_item_t **out_item);
extern int kmipkit_client_abi_c_consumer_build_set_attribute_request(
    kmipkit_client_batch_item_t **out_item);
extern int kmipkit_client_abi_c_consumer_main(int argc, char **argv);

static int append_required_constructor_result(
        kmipkit_client_batch_t *batch,
        int32_t status,
        kmipkit_client_batch_item_t **item,
        uint64_t *created_count,
        uint64_t invoked_count) {
    uint32_t operation = UINT32_MAX;
    return status == 0 && item != NULL && *item != NULL &&
        invoked_count < sizeof(EXPECTED_BATCH_OPERATIONS) / sizeof(EXPECTED_BATCH_OPERATIONS[0]) &&
        kmipkit_client_batch_item_operation(*item, &operation) == 0 &&
        operation == EXPECTED_BATCH_OPERATIONS[invoked_count] &&
        kmipkit_client_batch_add_item(batch, item) == 0 && *item == NULL &&
        (++*created_count, 1);
}

static int create_crypto_request_inputs(
        kmipkit_codec_limits_t *limits,
        kmipkit_ttlv_structure_t **out_parameters,
        kmipkit_ttlv_value_t **out_data) {
    static const uint8_t data_bytes[] = "operation-matrix-data";
    kmipkit_ttlv_structure_t *empty = 0;
    kmipkit_raw_tag_t *raw_tag = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_t *algorithm = 0;
    kmipkit_ttlv_item_t *item = 0;
    int result = 0;

    if (limits == 0 || out_parameters == 0 || out_data == 0) {
        return 0;
    }
    *out_parameters = 0;
    *out_data = 0;
    if (kmipkit_ttlv_structure_create(&empty) != 0 ||
            kmipkit_ttlv_raw_tag_create(UINT32_C(0x420038), &raw_tag) != 0 ||
            kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag) != 0 ||
            kmipkit_ttlv_value_enumeration(UINT32_C(6), &algorithm) != 0 ||
            kmipkit_ttlv_item_create(tag, algorithm, limits, &item) != 0) {
        goto cleanup;
    }
    algorithm = 0;
    if (kmipkit_ttlv_structure_with_item(empty, item, limits, out_parameters) != 0) {
        goto cleanup;
    }
    empty = 0;
    item = 0;
    if (kmipkit_ttlv_value_byte_string(
            limits, data_bytes, (uint64_t)(sizeof(data_bytes) - 1), out_data) != 0) {
        goto cleanup;
    }
    result = 1;

cleanup:
    kmipkit_ttlv_value_release(algorithm);
    kmipkit_ttlv_item_release(item);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw_tag);
    kmipkit_ttlv_structure_release(empty);
    if (!result) {
        kmipkit_ttlv_structure_release(*out_parameters);
        kmipkit_ttlv_value_release(*out_data);
        *out_parameters = 0;
        *out_data = 0;
    }
    return result;
}

#define KMIPKIT_TRY_OPERATION(expression) \
    do { \
        item = 0; \
        status = (expression); \
        if (status != 0 || item == 0) { \
            result = -((int)((invoked_count + UINT64_C(1)) * UINT64_C(100) + \
                (uint64_t)(status < 0 ? -status : status))); \
            goto cleanup; \
        } \
        if (!append_required_constructor_result(batch, status, &item, &created_count, invoked_count)) { \
            result = -(int)((invoked_count + UINT64_C(1)) * UINT64_C(10) + UINT64_C(2)); \
            goto cleanup; \
        } \
        ++invoked_count; \
    } while (0)

#define KMIPKIT_TRY_EXISTING_C_BUILDER(expression) \
    do { \
        item = 0; \
        status = (expression); \
        if (status != 1 || item == 0) { \
            result = -(int)((invoked_count + UINT64_C(1)) * UINT64_C(10) + UINT64_C(1)); \
            goto cleanup; \
        } \
        if (!append_required_constructor_result(batch, 0, &item, &created_count, invoked_count)) { \
            result = -(int)((invoked_count + UINT64_C(1)) * UINT64_C(10) + UINT64_C(2)); \
            goto cleanup; \
        } \
        ++invoked_count; \
    } while (0)

static int async_operation_routes_are_callable(void) {
    kmipkit_client_async_operation_outcome_t *outcome =
        (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_cancel(0, 0, 0, 0, &outcome) !=
            KMIPKIT_OPERATION_INVALID_INPUT || outcome != 0) {
        return 0;
    }
    outcome = (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_poll(0, 0, 0, 0, &outcome) !=
            KMIPKIT_OPERATION_INVALID_INPUT || outcome != 0) {
        return 0;
    }
    outcome = (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_process(0, 0, 0, 0, 0, 0, &outcome) !=
            KMIPKIT_OPERATION_INVALID_INPUT || outcome != 0) {
        return 0;
    }
    outcome = (kmipkit_client_async_operation_outcome_t *)(uintptr_t)1;
    if (kmipkit_client_execute_query_async_requests(
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, &outcome) !=
            KMIPKIT_OPERATION_INVALID_INPUT || outcome != 0) {
        return 0;
    }
    return 1;
}

int kmipkit_operation_parity_0053_all_56(void) {
    kmipkit_client_batch_t *batch = 0;
    kmipkit_client_batch_item_t *item = 0;
    kmipkit_register_request_t *register_request = 0;
    kmipkit_revoke_request_t *revoke_request = 0;
    kmipkit_revocation_reason_t *revoke_reason = 0;
    kmipkit_ttlv_structure_t *register_attributes = 0;
    kmipkit_ttlv_structure_t *register_fields = 0;
    kmipkit_ttlv_structure_t *register_fields_with_type = 0;
    kmipkit_ttlv_structure_t *register_fields_with_type_and_value = 0;
    kmipkit_ttlv_item_t *register_object = 0;
    kmipkit_ttlv_item_t *register_type_item = 0;
    kmipkit_ttlv_item_t *register_value_item = 0;
    kmipkit_ttlv_value_t *register_type_value = 0;
    kmipkit_ttlv_value_t *register_data_value = 0;
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *empty_structure = 0;
    kmipkit_ttlv_structure_t *empty_masks = 0;
    kmipkit_ttlv_structure_t *object_fields = 0;
    kmipkit_ttlv_structure_t *any_object_fields = 0;
    kmipkit_ttlv_structure_t *any_object = 0;
    kmipkit_ttlv_structure_t *parameters = 0;
    kmipkit_ttlv_value_t *object_value = 0;
    kmipkit_ttlv_value_t *data = 0;
    kmipkit_raw_tag_t *object_raw_tag = 0;
    kmipkit_tag_t *object_tag = 0;
    kmipkit_ttlv_item_t *object_item = 0;
    kmipkit_unique_identifier_v1_t identifier = {0};
    kmipkit_unique_identifier_v1_t identifiers[2] = {{0}};
    static const uint8_t correlation_bytes[] = {0x11, 0x12};
    static const uint8_t mac_bytes[] = {0x21, 0x22};
    static const uint8_t digest_bytes[] = {0x31, 0x32};
    static const uint8_t signature_bytes[] = {0x41, 0x42};
    static const uint8_t log_message[] = "log-message-private-sentinel";
    static const uint32_t query_functions[] = {UINT32_C(5), UINT32_C(1)};
    static const uint8_t query_group_bytes[] = "operation-matrix-group";
    kmipkit_byte_span_v1_t query_groups[] = {
        {query_group_bytes, sizeof(query_group_bytes) - 1},
    };
    kmipkit_byte_span_v1_t correlation = {correlation_bytes, sizeof(correlation_bytes)};
    kmipkit_byte_span_v1_t mac = {mac_bytes, sizeof(mac_bytes)};
    kmipkit_byte_span_v1_t digest = {digest_bytes, sizeof(digest_bytes)};
    kmipkit_byte_span_v1_t signature = {signature_bytes, sizeof(signature_bytes)};
    static const uint8_t iv_bytes[] = {0x51, 0x52};
    static const uint8_t tag_bytes[] = {0x61, 0x62};
    static const uint8_t aad_bytes[] = {0x71, 0x72};
    kmipkit_byte_span_v1_t iv = {iv_bytes, sizeof(iv_bytes)};
    kmipkit_byte_span_v1_t authentication_tag = {tag_bytes, sizeof(tag_bytes)};
    kmipkit_byte_span_v1_t additional_data = {aad_bytes, sizeof(aad_bytes)};
    kmipkit_byte_span_v1_t rng_seed = {correlation_bytes, sizeof(correlation_bytes)};
    uint64_t created_count = 0;
    uint64_t invoked_count = 0;
    uint64_t batch_item_count = 0;
    int32_t status = 0;
    int result = 0;

    identifier.struct_size = (uint32_t)sizeof(identifier);
    identifier.abi_version = UINT32_C(1);
    identifier.kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = (const uint8_t *)"operation-matrix-id";
    identifier.text_length = UINT64_C(19);
    identifiers[0] = identifier;
    identifiers[1] = identifier;

    if (kmipkit_client_batch_create(&batch) != 0 || batch == 0) {
        result = -101;
        goto cleanup;
    }
    if (kmipkit_codec_limits_defaults(&limits) != 0 || limits == 0) {
        result = -102;
        goto cleanup;
    }
    if (kmipkit_ttlv_structure_create(&empty_structure) != 0 || empty_structure == 0 ||
            kmipkit_ttlv_structure_create(&empty_masks) != 0 || empty_masks == 0) {
        result = -103;
        goto cleanup;
    }
    if (kmipkit_ttlv_structure_create(&object_fields) != 0 || object_fields == 0 ||
            kmipkit_ttlv_value_structure(object_fields, limits, &object_value) != 0) {
        result = -104;
        goto cleanup;
    }
    object_fields = 0;
    if (kmipkit_ttlv_raw_tag_create(UINT32_C(0x42008f), &object_raw_tag) != 0 ||
            kmipkit_ttlv_raw_tag_try_checked(object_raw_tag, &object_tag) != 0 ||
            kmipkit_ttlv_item_create(object_tag, object_value, limits, &object_item) != 0) {
        result = -105;
        goto cleanup;
    }
    object_value = 0;
    if (kmipkit_ttlv_structure_create(&any_object_fields) != 0 ||
            kmipkit_ttlv_structure_with_item(
                any_object_fields, object_item, limits, &any_object) != 0) {
        result = -106;
        goto cleanup;
    }
    any_object_fields = 0;
    object_item = 0;
    if (!create_crypto_request_inputs(limits, &parameters, &data)) {
        result = -107;
        goto cleanup;
    }
    if (kmipkit_ttlv_structure_create(&register_attributes) != 0 ||
            kmipkit_ttlv_structure_create(&register_fields) != 0 ||
            kmipkit_ttlv_raw_tag_create(UINT32_C(0x00420059), &object_raw_tag) != 0 ||
            kmipkit_ttlv_raw_tag_try_checked(object_raw_tag, &object_tag) != 0 ||
            kmipkit_ttlv_value_enumeration(UINT32_C(0x80000001), &register_type_value) != 0 ||
            kmipkit_ttlv_item_create(
                object_tag, register_type_value, limits, &register_type_item) != 0) {
        result = -110;
        goto cleanup;
    }
    register_type_value = 0;
    kmipkit_tag_release(object_tag);
    object_tag = 0;
    kmipkit_raw_tag_release(object_raw_tag);
    object_raw_tag = 0;
    if (kmipkit_ttlv_structure_with_item(
            register_fields, register_type_item, limits, &register_fields_with_type) != 0) {
        result = -110;
        goto cleanup;
    }
    register_fields = 0;
    register_type_item = 0;
    if (kmipkit_ttlv_raw_tag_create(UINT32_C(0x0042005a), &object_raw_tag) != 0 ||
            kmipkit_ttlv_raw_tag_try_checked(object_raw_tag, &object_tag) != 0 ||
            kmipkit_ttlv_value_byte_string(
                limits, (const uint8_t *)"opaque-register-value", UINT64_C(21),
                &register_data_value) != 0 ||
            kmipkit_ttlv_item_create(
                object_tag, register_data_value, limits, &register_value_item) != 0 ||
            kmipkit_ttlv_structure_with_item(
                register_fields_with_type, register_value_item, limits,
                &register_fields_with_type_and_value) != 0 ||
            kmipkit_ttlv_value_structure(
                register_fields_with_type_and_value, limits, &object_value) != 0) {
        result = -111;
        goto cleanup;
    }
    register_data_value = 0;
    register_value_item = 0;
    register_fields_with_type = 0;
    register_fields_with_type_and_value = 0;
    kmipkit_tag_release(object_tag);
    object_tag = 0;
    kmipkit_raw_tag_release(object_raw_tag);
    object_raw_tag = 0;
    if (kmipkit_ttlv_raw_tag_create(UINT32_C(0x0042005b), &object_raw_tag) != 0 ||
            kmipkit_ttlv_raw_tag_try_checked(object_raw_tag, &object_tag) != 0 ||
            kmipkit_ttlv_item_create(object_tag, object_value, limits, &register_object) != 0) {
        result = -112;
        goto cleanup;
    }
    object_value = 0;
    if (kmipkit_register_request_create(
            UINT32_C(8), register_attributes, register_object, &register_request) != 0 ||
            register_request == 0) {
        result = -113;
        goto cleanup;
    }
    register_attributes = 0;
    register_object = 0;
    kmipkit_tag_release(object_tag);
    object_tag = 0;
    kmipkit_raw_tag_release(object_raw_tag);
    object_raw_tag = 0;
    if (kmipkit_revocation_reason_create(
            KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_KEY_COMPROMISE,
            0, 0, 0, &revoke_reason) != 0 || revoke_reason == 0 ||
            kmipkit_revoke_request_create(
                &identifier, revoke_reason, 0, 0, &revoke_request) != 0 ||
            revoke_request == 0) {
        result = -114;
        goto cleanup;
    }
    kmipkit_ttlv_item_release(object_item);
    object_item = 0;
    kmipkit_ttlv_value_release(object_value);
    object_value = 0;
    kmipkit_tag_release(object_tag);
    object_tag = 0;
    kmipkit_raw_tag_release(object_raw_tag);
    object_raw_tag = 0;

    /* 52 operation-specific batch item constructors, each called from C. */
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_activate(&identifier, &item));
    KMIPKIT_TRY_EXISTING_C_BUILDER(
        kmipkit_client_abi_c_consumer_build_add_attribute_request(&item));
    KMIPKIT_TRY_EXISTING_C_BUILDER(
        kmipkit_client_abi_c_consumer_build_adjust_attribute(&item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_archive(&identifier, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_certify(
        &identifier, 0, 0, 0, 0, 0, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_check(
        &identifier, 0, 0, 0, 0, 0, 0, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_create(
        KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY, empty_structure, empty_masks, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_create_key_pair(0, 0, 0, 0, 0, 0, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_create_split_key(
        UINT32_C(0xf0000000), &identifier, -3, 4, UINT32_C(0xdeadbeef),
        0, 0, 0, empty_structure, empty_masks, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_decrypt(
        &identifier, 0, data, &iv, &authentication_tag, 0, 1, 1, &additional_data, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_delegated_login(
        empty_structure, UINT32_C(2), INT64_MIN, 1, INT32_MIN, empty_structure, &item));
    KMIPKIT_TRY_EXISTING_C_BUILDER(
        kmipkit_client_abi_c_consumer_build_delete_attribute(&item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_derive_key(
        KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY, identifiers, UINT64_C(2),
        UINT32_C(0xfeedbeef), empty_structure, empty_structure, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_destroy(&identifier, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_discover_versions(&item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_encrypt(
        &identifier, parameters, data, &iv, 0, 1, 1, &additional_data, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_export(
        &identifier, 0, 0, 0, 0, 0, 0, 0, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_get(
        &identifier, 0, 0, 0, 0, 0, 0, 0, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_get_attributes(&identifier, 0, 0, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_get_attribute_list(&identifier, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_get_constraints(&item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_get_usage_allocation(
        &identifier, INT64_C(0), &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_hash(
        parameters, data, &correlation, 1, 2, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_import(
        &identifier, KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY,
        0, 0, 0, 0, empty_structure, any_object, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_locate(
        1, -4, 1, 0, 1, UINT32_C(0x80000001), 1, UINT32_C(0xf1234567),
        empty_structure, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_log(
        log_message, sizeof(log_message) - 1, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_login(
        UINT32_C(2), INT64_MIN, 1, INT32_MIN, empty_structure, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_mac(
        &identifier, parameters, data, &correlation, 1, 2, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_mac_verify(
        &identifier, parameters, data, &mac, &correlation, 1, 2, &item));
    KMIPKIT_TRY_EXISTING_C_BUILDER(
        kmipkit_client_abi_c_consumer_build_modify_attribute(&item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_obtain_lease(&identifier, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_ping(&item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_query(
        query_functions, sizeof(query_functions) / sizeof(query_functions[0]),
        1, query_groups, sizeof(query_groups) / sizeof(query_groups[0]), &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_recover(&identifier, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_register(register_request, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_revoke(revoke_request, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_recertify(
        &identifier, 0, 0, 0, 0, 0, 0, 0, 0, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_rekey(&identifier, 0, 0, 0, 0, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_rekey_key_pair(
        &identifier, 0, 0, 0, 0, 0, 0, 0, 0, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_reprovision(0, 0, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_rng_retrieve(-1, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_rng_seed(&rng_seed, &item));
    KMIPKIT_TRY_EXISTING_C_BUILDER(
        kmipkit_client_abi_c_consumer_build_set_attribute_request(&item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_set_constraints(empty_structure, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_set_defaults(empty_structure, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_sign(
        &identifier, parameters, data, &digest, &correlation, 1, 2, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_signature_verify(
        &identifier, parameters, data, &digest, &signature, &correlation, 1, 2, &item));
    KMIPKIT_TRY_OPERATION(kmipkit_client_batch_item_validate(
        0, 0, &identifier, UINT64_C(1), 0, 0, 0, 0, &item));

    {
        char executable[] = "operation-parity-consumer";
        char high_level[] = "high-level";
        char typed_protocol[] = "typed-protocol";
        char generic_ttlv[] = "generic-ttlv";
        char *arguments[] = {executable, high_level, typed_protocol, generic_ttlv};
        if (kmipkit_client_abi_c_consumer_main(4, arguments) != 0) {
            result = -108;
            goto cleanup;
        }
        invoked_count += UINT64_C(4); /* It also creates Interop, Join Split Key, Logout, and PKCS#11 items. */
    }

    if (invoked_count != UINT64_C(52) ||
            !async_operation_routes_are_callable() ||
            kmipkit_client_batch_item_count(batch, &batch_item_count) != 0 ||
            batch_item_count != created_count || created_count != UINT64_C(48)) {
        goto cleanup;
    }
    result = (int)created_count;

cleanup:
    kmipkit_client_batch_item_release(item);
    kmipkit_register_request_release(register_request);
    kmipkit_revoke_request_release(revoke_request);
    kmipkit_revocation_reason_release(revoke_reason);
    kmipkit_ttlv_structure_release(register_attributes);
    kmipkit_ttlv_structure_release(register_fields);
    kmipkit_ttlv_structure_release(register_fields_with_type);
    kmipkit_ttlv_structure_release(register_fields_with_type_and_value);
    kmipkit_ttlv_item_release(register_object);
    kmipkit_ttlv_item_release(register_type_item);
    kmipkit_ttlv_item_release(register_value_item);
    kmipkit_ttlv_value_release(register_type_value);
    kmipkit_ttlv_value_release(register_data_value);
    kmipkit_client_batch_release(batch);
    kmipkit_ttlv_item_release(object_item);
    kmipkit_ttlv_value_release(object_value);
    kmipkit_tag_release(object_tag);
    kmipkit_raw_tag_release(object_raw_tag);
    kmipkit_ttlv_structure_release(any_object);
    kmipkit_ttlv_structure_release(any_object_fields);
    kmipkit_ttlv_structure_release(object_fields);
    kmipkit_ttlv_value_release(data);
    kmipkit_ttlv_structure_release(parameters);
    kmipkit_ttlv_structure_release(empty_masks);
    kmipkit_ttlv_structure_release(empty_structure);
    kmipkit_codec_limits_release(limits);
    return result;
}

#undef KMIPKIT_TRY_OPERATION
#undef KMIPKIT_TRY_EXISTING_C_BUILDER
