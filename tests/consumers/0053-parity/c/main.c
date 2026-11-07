#include "kmipkit.h"

#include <stdint.h>
#include <string.h>

static int32_t append_value(kmipkit_ttlv_structure_t **structure, uint32_t raw_tag,
                            kmipkit_ttlv_value_t *value, kmipkit_codec_limits_t *limits) {
    kmipkit_raw_tag_t *raw = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_item_t *item = 0;
    kmipkit_ttlv_structure_t *next = 0;
    int32_t status = kmipkit_ttlv_raw_tag_create(raw_tag, &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, &item);
    if (status == 0) {
        value = 0;
        status = kmipkit_ttlv_structure_with_item(*structure, item, limits, &next);
    }
    if (status == 0) {
        *structure = next;
        next = 0;
        item = 0;
    }
    kmipkit_ttlv_structure_release(next);
    kmipkit_ttlv_item_release(item);
    kmipkit_ttlv_value_release(value);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw);
    return status;
}

static int32_t enum_item(uint32_t raw_tag, uint32_t raw_value,
                         kmipkit_codec_limits_t *limits, kmipkit_ttlv_item_t **out_item) {
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_raw_tag_t *raw = 0;
    kmipkit_tag_t *tag = 0;
    int32_t status = kmipkit_ttlv_value_enumeration(raw_value, &value);
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(raw_tag, &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, out_item);
    if (status == 0) value = 0;
    kmipkit_ttlv_value_release(value);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw);
    return status;
}

static int32_t build_synthetic_ping_response(kmipkit_codec_limits_t *limits,
                                            kmipkit_ttlv_structure_t **out_root) {
    kmipkit_ttlv_structure_t *version = 0;
    kmipkit_ttlv_structure_t *header = 0;
    kmipkit_ttlv_structure_t *batch_item = 0;
    kmipkit_ttlv_structure_t *payload = 0;
    kmipkit_ttlv_value_t *value = 0;
    int32_t status = kmipkit_ttlv_structure_create(&version);
    if (status == 0) status = kmipkit_ttlv_value_integer(2, &value);
    if (status == 0) { status = append_value(&version, 0x0042006a, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_value_integer(1, &value);
    if (status == 0) { status = append_value(&version, 0x0042006b, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_structure_create(&header);
    if (status == 0) status = kmipkit_ttlv_value_structure(version, limits, &value);
    if (status == 0) { version = 0; status = append_value(&header, 0x00420069, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_value_date_time(INT64_C(1751100123), &value);
    if (status == 0) { status = append_value(&header, 0x00420092, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_value_integer(1, &value);
    if (status == 0) { status = append_value(&header, 0x0042000d, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_structure_create(&batch_item);
    if (status == 0) status = kmipkit_ttlv_value_enumeration(0x3b, &value);
    if (status == 0) { status = append_value(&batch_item, 0x0042005c, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_value_enumeration(0, &value);
    if (status == 0) { status = append_value(&batch_item, 0x0042007f, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_structure_create(&payload);
    if (status == 0) status = kmipkit_ttlv_value_structure(payload, limits, &value);
    if (status == 0) { payload = 0; status = append_value(&batch_item, 0x0042007c, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_structure_create(out_root);
    if (status == 0) status = kmipkit_ttlv_value_structure(header, limits, &value);
    if (status == 0) { header = 0; status = append_value(out_root, 0x0042007a, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_value_structure(batch_item, limits, &value);
    if (status == 0) { batch_item = 0; status = append_value(out_root, 0x0042000f, value, limits); value = 0; }
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_structure_release(version);
    kmipkit_ttlv_structure_release(header);
    kmipkit_ttlv_structure_release(batch_item);
    kmipkit_ttlv_structure_release(payload);
    if (status != 0) { kmipkit_ttlv_structure_release(*out_root); *out_root = 0; }
    return status;
}

int main(void) {
    static const uint8_t comment[] = "local example";
    static const uint8_t username[] = "demo-user";
    static const uint8_t demo_bytes[] = "demo bytes";
    static const uint8_t key_bytes[] = "demo key bytes";
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_ttlv_item_t *typed_attribute = 0;
    kmipkit_ttlv_item_t *fields[2] = {0, 0};
    kmipkit_ttlv_item_t *opaque_object = 0;
    kmipkit_ttlv_item_t *key_material_source = 0;
    kmipkit_ttlv_item_t *key_material = 0;
    kmipkit_cryptographic_parameters_fields_v1_t parameter_fields = {0};
    kmipkit_cryptographic_parameters_t *parameters = 0;
    kmipkit_ttlv_structure_t *credential_value = 0;
    kmipkit_ttlv_structure_t *credential_tree = 0;
    kmipkit_credential_t *credential = 0;
    kmipkit_authentication_t *authentication = 0;
    kmipkit_client_batch_item_t *ping = 0;
    kmipkit_client_batch_t *batch = 0;
    kmipkit_validation_report_t *report = 0;
    kmipkit_ttlv_structure_t *response_tree = 0;
    kmipkit_response_message_t *response = 0;
    kmipkit_message_validation_error_t *validation_error = 0;
    kmipkit_response_batch_item_view_t *response_item = 0;
    kmipkit_ttlv_structure_t *response_item_ttlv = 0;
    kmipkit_ttlv_structure_view_t *response_view = 0;
    uint8_t valid = 0;
    uint8_t present = 0;
    uint32_t result_status = UINT32_MAX;
    uint64_t count = UINT64_MAX;
    int32_t status = kmipkit_codec_limits_defaults(&limits);

    if (status == 0) status = kmipkit_ttlv_value_text_string(limits, comment, sizeof(comment) - 1, &value);
    if (status == 0) status = kmipkit_typed_attribute_from_value(0x00420022, value, &typed_attribute);
    if (status == 0) value = 0;

    if (status == 0) status = enum_item(0x00420059, 1, limits, &fields[0]);
    if (status == 0) status = kmipkit_ttlv_value_byte_string(limits, demo_bytes, sizeof(demo_bytes) - 1, &value);
    if (status == 0) {
        kmipkit_raw_tag_t *raw = 0; kmipkit_tag_t *tag = 0;
        status = kmipkit_ttlv_raw_tag_create(0x0042005a, &raw);
        if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
        if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, &fields[1]);
        if (status == 0) value = 0;
        kmipkit_tag_release(tag); kmipkit_raw_tag_release(raw);
    }
    if (status == 0) status = kmipkit_opaque_object_from_fields(fields, 2, &opaque_object);
    if (status == 0) { fields[0] = 0; fields[1] = 0; }

    if (status == 0) status = kmipkit_ttlv_value_byte_string(limits, key_bytes, sizeof(key_bytes) - 1, &value);
    if (status == 0) {
        kmipkit_raw_tag_t *raw = 0; kmipkit_tag_t *tag = 0;
        status = kmipkit_ttlv_raw_tag_create(0x00420043, &raw);
        if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
        if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, &key_material_source);
        if (status == 0) value = 0;
        kmipkit_tag_release(tag); kmipkit_raw_tag_release(raw);
    }
    if (status == 0) status = kmipkit_key_material_from_item(1, key_material_source, &key_material);

    parameter_fields.struct_size = (uint32_t)sizeof(parameter_fields);
    parameter_fields.abi_version = 1;
    parameter_fields.cryptographic_algorithm_present = 1;
    parameter_fields.cryptographic_algorithm = UINT32_C(0x80000001);
    if (status == 0) status = kmipkit_cryptographic_parameters_from_fields_v1(&parameter_fields, &parameters);

    if (status == 0) status = kmipkit_ttlv_structure_create(&credential_value);
    if (status == 0) status = kmipkit_ttlv_value_text_string(limits, username, sizeof(username) - 1, &value);
    if (status == 0) { status = append_value(&credential_value, 0x004200bb, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_structure_create(&credential_tree);
    if (status == 0) status = kmipkit_ttlv_value_enumeration(1, &value);
    if (status == 0) { status = append_value(&credential_tree, 0x00420024, value, limits); value = 0; }
    if (status == 0) status = kmipkit_ttlv_value_structure(credential_value, limits, &value);
    if (status == 0) { credential_value = 0; status = append_value(&credential_tree, 0x00420025, value, limits); value = 0; }
    if (status == 0) status = kmipkit_credential_from_ttlv(credential_tree, &credential);
    if (status == 0) credential_tree = 0;
    if (status == 0) status = kmipkit_authentication_create(&credential, 1, &authentication);
    if (status == 0) credential = 0;
    if (status == 0) status = kmipkit_client_batch_item_ping(&ping);
    if (status == 0) status = kmipkit_client_batch_item_validation_report(ping, &report);
    if (status == 0) status = kmipkit_validation_report_is_valid(report, &valid);
    if (status == 0 && !valid) status = 4;
    if (status == 0) status = kmipkit_client_batch_create(&batch);
    if (status == 0) status = kmipkit_client_batch_add_item(batch, &ping);
    if (status == 0) ping = 0;
    if (status == 0) status = kmipkit_client_batch_set_authentication(batch, authentication);
    if (status == 0) authentication = 0;

    if (status == 0) status = build_synthetic_ping_response(limits, &response_tree);
    if (status == 0) status = kmipkit_response_message_from_ttlv(response_tree, &response, &validation_error);
    if (status == 0) response_tree = 0;
    if (status == 0) status = kmipkit_response_message_batch_item_count(response, &count);
    if (status == 0 && count != 1) status = 4;
    if (status == 0) status = kmipkit_response_message_batch_item_at(response, 0, &response_item);
    if (status == 0) status = kmipkit_response_batch_item_view_result_status(response_item, &present, &result_status);
    if (status == 0 && (!present || result_status != 0)) status = 4;
    if (status == 0) status = kmipkit_response_batch_item_view_ttlv(response_item, &response_item_ttlv);
    if (status == 0) status = kmipkit_ttlv_structure_view(response_item_ttlv, &response_view);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(response_view, &count);
    if (status == 0 && count != 3) status = 4;

    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_item_release(typed_attribute);
    kmipkit_ttlv_item_release(fields[0]); kmipkit_ttlv_item_release(fields[1]);
    kmipkit_ttlv_item_release(opaque_object);
    kmipkit_ttlv_item_release(key_material_source); kmipkit_ttlv_item_release(key_material);
    kmipkit_cryptographic_parameters_release(parameters);
    kmipkit_ttlv_structure_release(credential_value); kmipkit_ttlv_structure_release(credential_tree);
    kmipkit_credential_release(credential); kmipkit_authentication_release(authentication);
    kmipkit_client_batch_item_release(ping); kmipkit_client_batch_release(batch);
    kmipkit_validation_report_release(report);
    kmipkit_ttlv_structure_release(response_tree); kmipkit_response_message_release(response);
    kmipkit_message_validation_error_release(validation_error);
    kmipkit_response_batch_item_release(response_item);
    kmipkit_ttlv_structure_release(response_item_ttlv);
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_codec_limits_release(limits);
    return status == 0 ? 0 : 1;
}
