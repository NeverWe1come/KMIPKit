#include <stdint.h>

#include "kmipkit.h"

static const uint32_t CREDENTIAL_TYPE_TAG = 0x00420024;
static const uint32_t CREDENTIAL_VALUE_TAG = 0x00420025;
static const uint32_t CREDENTIAL_PAYLOAD_TAG = 0x0042002A;
static const int32_t CREDENTIAL_PAYLOAD_VALUE = 0x13579;

static int32_t append_item(
        kmipkit_ttlv_structure_t **structure, kmipkit_codec_limits_t *limits,
        uint32_t raw_tag, kmipkit_ttlv_value_t *value) {
    kmipkit_raw_tag_t *raw = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_item_t *item = 0;
    kmipkit_ttlv_structure_t *next = 0;
    int32_t status = kmipkit_ttlv_raw_tag_create(raw_tag, &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, &item);
    if (status == 0) {
        value = 0;
        tag = 0;
        status = kmipkit_ttlv_structure_with_item(*structure, item, limits, &next);
    }
    if (next != 0) {
        kmipkit_ttlv_structure_release(*structure);
        kmipkit_ttlv_item_release(item);
        *structure = next;
    }
    kmipkit_raw_tag_release(raw);
    kmipkit_tag_release(tag);
    kmipkit_ttlv_item_release(item);
    kmipkit_ttlv_value_release(value);
    return status;
}

static int32_t tags_match(kmipkit_ttlv_structure_view_t *view, uint64_t *out_count) {
    uint64_t count = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_count(view, &count);
    if (status != 0 || count != 2) return status == 0 ? 1 : status;
    const uint32_t expected[] = {CREDENTIAL_TYPE_TAG, CREDENTIAL_VALUE_TAG};
    for (uint64_t index = 0; index < count; index++) {
        kmipkit_ttlv_item_view_t *item = 0;
        kmipkit_tag_t *tag = 0;
        uint32_t actual = 0;
        status = kmipkit_ttlv_structure_view_item_at(view, index, &item);
        if (status == 0) status = kmipkit_ttlv_item_view_tag(item, &tag);
        if (status == 0) status = kmipkit_ttlv_tag_value(tag, &actual);
        if (status == 0 && actual != expected[index]) status = 1;
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
        if (status != 0) return status;
    }
    *out_count = count;
    return 0;
}

static int32_t payload_matches(kmipkit_ttlv_structure_view_t *view) {
    kmipkit_ttlv_item_view_t *outer_item = 0;
    kmipkit_ttlv_value_view_t *outer_value = 0;
    kmipkit_ttlv_structure_view_t *payload = 0;
    kmipkit_ttlv_item_view_t *payload_item = 0;
    kmipkit_tag_t *payload_tag = 0;
    kmipkit_ttlv_value_view_t *payload_value = 0;
    uint64_t count = 0;
    uint32_t raw_tag = 0;
    int32_t integer = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_at(view, 1, &outer_item);
    if (status == 0) status = kmipkit_ttlv_item_view_value(outer_item, &outer_value);
    if (status == 0) status = kmipkit_ttlv_value_view_structure(outer_value, &payload);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(payload, &count);
    if (status == 0 && count != 1) status = 1;
    if (status == 0) status = kmipkit_ttlv_structure_view_item_at(payload, 0, &payload_item);
    if (status == 0) status = kmipkit_ttlv_item_view_tag(payload_item, &payload_tag);
    if (status == 0) status = kmipkit_ttlv_tag_value(payload_tag, &raw_tag);
    if (status == 0 && raw_tag != CREDENTIAL_PAYLOAD_TAG) status = 1;
    if (status == 0) status = kmipkit_ttlv_item_view_value(payload_item, &payload_value);
    if (status == 0) status = kmipkit_ttlv_value_view_integer(payload_value, &integer);
    if (status == 0 && integer != CREDENTIAL_PAYLOAD_VALUE) status = 1;
    kmipkit_ttlv_value_view_release(payload_value);
    kmipkit_tag_release(payload_tag);
    kmipkit_ttlv_item_view_release(payload_item);
    kmipkit_ttlv_structure_view_release(payload);
    kmipkit_ttlv_value_view_release(outer_value);
    kmipkit_ttlv_item_view_release(outer_item);
    return status;
}

int32_t kmipkit_credential_c_consumer_round_trip(
        uint32_t raw_type, uint32_t *out_type, uint64_t *out_count) {
    if (out_type == 0 || out_count == 0) return 4;
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_structure_t *value_structure = 0;
    kmipkit_credential_t *credential = 0;
    kmipkit_ttlv_structure_view_t *source_view = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_structure_t *restored = 0;
    kmipkit_ttlv_structure_view_t *restored_view = 0;
    uint64_t count = 0;
    uint32_t typed_raw = 0;
    uint32_t field_raw = 0;
    int32_t status = kmipkit_codec_limits_create(4096, 8, 32, &limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&structure);
    kmipkit_ttlv_value_t *enum_value = 0;
    if (status == 0) status = kmipkit_ttlv_value_enumeration(raw_type, &enum_value);
    if (status == 0) {
        status = append_item(&structure, limits, CREDENTIAL_TYPE_TAG, enum_value);
        enum_value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_structure_create(&value_structure);
    kmipkit_ttlv_value_t *structure_value = 0;
    kmipkit_ttlv_value_t *payload_value = 0;
    if (status == 0) status = kmipkit_ttlv_value_integer(CREDENTIAL_PAYLOAD_VALUE, &payload_value);
    if (status == 0) {
        status = append_item(&value_structure, limits, CREDENTIAL_PAYLOAD_TAG, payload_value);
        payload_value = 0;
    }
    if (status == 0) {
        status = kmipkit_ttlv_value_structure(value_structure, limits, &structure_value);
        value_structure = 0;
    }
    if (status == 0) {
        status = append_item(&structure, limits, CREDENTIAL_VALUE_TAG, structure_value);
        structure_value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_structure_view(structure, &source_view);
    if (status == 0) {
        int32_t shared_owner_status = kmipkit_credential_from_ttlv(structure, &credential);
        if (shared_owner_status != KMIPKIT_ERROR_SHARED_OWNER) status = 1;
    }
    if (status == 0) status = tags_match(source_view, &count);
    kmipkit_ttlv_structure_view_release(source_view);
    source_view = 0;
    if (status == 0) status = kmipkit_credential_from_ttlv(structure, &credential);
    if (status == 0) {
        structure = 0;
        status = kmipkit_credential_type(credential, &typed_raw);
    }
    if (status == 0) status = kmipkit_credential_type_raw(credential, &field_raw);
    if (status == 0 && (typed_raw != raw_type || field_raw != raw_type)) status = 1;
    if (status == 0) status = kmipkit_credential_ttlv_view(credential, &view);
    if (status == 0) status = tags_match(view, &count);
    if (status == 0) status = payload_matches(view);
    kmipkit_ttlv_structure_view_release(view);
    view = 0;
    if (status == 0) status = kmipkit_credential_into_ttlv(credential, &restored);
    if (status == 0) {
        credential = 0;
        status = kmipkit_ttlv_structure_view(restored, &restored_view);
    }
    if (status == 0) status = tags_match(restored_view, &count);
    if (status == 0) status = payload_matches(restored_view);
    if (status == 0) {
        *out_type = typed_raw;
        *out_count = count;
    }
    kmipkit_codec_limits_release(limits);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_ttlv_structure_release(value_structure);
    kmipkit_ttlv_value_release(enum_value);
    kmipkit_ttlv_value_release(structure_value);
    kmipkit_ttlv_value_release(payload_value);
    kmipkit_credential_release(credential);
    kmipkit_ttlv_structure_view_release(source_view);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(restored);
    kmipkit_ttlv_structure_view_release(restored_view);
    return status;
}

static const uint32_t NONCE_TAG = 0x004200C8;
static const uint32_t NONCE_ID_TAG = 0x004200C9;
static const uint32_t NONCE_VALUE_TAG = 0x004200CA;
static const uint32_t ATTESTATION_TYPE_TAG = 0x004200C7;
static const uint32_t ATTESTATION_MEASUREMENT_TAG = 0x004200CB;
static const uint32_t ATTESTATION_ASSERTION_TAG = 0x004200CC;

static int32_t attestation_tags_match(
        kmipkit_ttlv_structure_view_t *view, uint64_t expected_count) {
    uint64_t count = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_count(view, &count);
    if (status != 0 || count != expected_count) return status == 0 ? 1 : status;
    const uint32_t expected[] = {
        NONCE_TAG, ATTESTATION_TYPE_TAG,
        ATTESTATION_MEASUREMENT_TAG, ATTESTATION_ASSERTION_TAG
    };
    for (uint64_t index = 0; index < count; index++) {
        kmipkit_ttlv_item_view_t *item = 0;
        kmipkit_tag_t *tag = 0;
        uint32_t actual = 0;
        status = kmipkit_ttlv_structure_view_item_at(view, index, &item);
        if (status == 0) status = kmipkit_ttlv_item_view_tag(item, &tag);
        if (status == 0) status = kmipkit_ttlv_tag_value(tag, &actual);
        if (status == 0 && actual != expected[index]) status = 1;
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
        if (status != 0) return status;
    }

    kmipkit_ttlv_item_view_t *nonce_item = 0;
    kmipkit_ttlv_value_view_t *nonce_value = 0;
    kmipkit_ttlv_structure_view_t *nonce = 0;
    uint64_t nonce_count = 0;
    status = kmipkit_ttlv_structure_view_item_at(view, 0, &nonce_item);
    if (status == 0) status = kmipkit_ttlv_item_view_value(nonce_item, &nonce_value);
    if (status == 0) status = kmipkit_ttlv_value_view_structure(nonce_value, &nonce);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(nonce, &nonce_count);
    if (status == 0 && nonce_count != 2) status = 1;
    kmipkit_ttlv_structure_view_release(nonce);
    kmipkit_ttlv_value_view_release(nonce_value);
    kmipkit_ttlv_item_view_release(nonce_item);
    return status;
}

static int32_t build_attestation_structure(
        int include_evidence, kmipkit_ttlv_structure_t **out_structure) {
    static const uint8_t nonce_id[] = {0x01, 0x02, 0xFE};
    static const uint8_t nonce_value[] = {0xA1, 0x00, 0xFF};
    static const uint8_t measurement[] = {0x10, 0x20, 0x30};
    static const uint8_t assertion[] = {0x40, 0x50, 0x60};
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_structure_t *nonce = 0;
    kmipkit_ttlv_value_t *value = 0;
    int32_t status = kmipkit_codec_limits_create(4096, 8, 64, &limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0) status = kmipkit_ttlv_structure_create(&nonce);
    if (status == 0) status = kmipkit_ttlv_value_byte_string(
            limits, nonce_id, sizeof(nonce_id), &value);
    if (status == 0) {
        status = append_item(&nonce, limits, NONCE_ID_TAG, value);
        value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_value_byte_string(
            limits, nonce_value, sizeof(nonce_value), &value);
    if (status == 0) {
        status = append_item(&nonce, limits, NONCE_VALUE_TAG, value);
        value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_value_structure(nonce, limits, &value);
    if (status == 0) {
        nonce = 0;
        status = append_item(&structure, limits, NONCE_TAG, value);
        value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_value_enumeration(0xF1234567, &value);
    if (status == 0) {
        status = append_item(&structure, limits, ATTESTATION_TYPE_TAG, value);
        value = 0;
    }
    if (status == 0 && include_evidence) status = kmipkit_ttlv_value_byte_string(
            limits, measurement, sizeof(measurement), &value);
    if (status == 0 && include_evidence) {
        status = append_item(&structure, limits, ATTESTATION_MEASUREMENT_TAG, value);
        value = 0;
    }
    if (status == 0 && include_evidence) status = kmipkit_ttlv_value_byte_string(
            limits, assertion, sizeof(assertion), &value);
    if (status == 0 && include_evidence) {
        status = append_item(&structure, limits, ATTESTATION_ASSERTION_TAG, value);
        value = 0;
    }
    if (status == 0) {
        *out_structure = structure;
        structure = 0;
    }
    kmipkit_codec_limits_release(limits);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_ttlv_structure_release(nonce);
    kmipkit_ttlv_value_release(value);
    return status;
}

int32_t kmipkit_attestation_credential_c_consumer_round_trip(uint64_t *out_count) {
    if (out_count == 0) return -1;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_attestation_credential_t *credential = 0;
    kmipkit_ttlv_structure_view_t *source_view = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_structure_t *restored = 0;
    kmipkit_ttlv_structure_view_t *restored_view = 0;
    uint64_t count = 0;
    int32_t status = build_attestation_structure(1, &structure);
    if (status == 0) status = kmipkit_ttlv_structure_view(structure, &source_view);
    if (status == 0 && kmipkit_attestation_credential_from_ttlv(
            structure, &credential) != KMIPKIT_ERROR_SHARED_OWNER) status = 1;
    if (status == 0) status = attestation_tags_match(source_view, 4);
    kmipkit_ttlv_structure_view_release(source_view);
    source_view = 0;
    if (status == 0) status = kmipkit_attestation_credential_from_ttlv(structure, &credential);
    if (status == 0) {
        structure = 0;
        status = kmipkit_attestation_credential_ttlv_view(credential, &view);
    }
    if (status == 0) status = attestation_tags_match(view, 4);
    if (status == 0) {
        int32_t view_conflict_status = kmipkit_attestation_credential_into_ttlv(
                credential, &restored);
        if (view_conflict_status == 0 || restored != 0) status = 1;
    }
    if (status == 0) status = attestation_tags_match(view, 4);
    kmipkit_ttlv_structure_view_release(view);
    view = 0;
    if (status == 0) status = kmipkit_attestation_credential_into_ttlv(credential, &restored);
    if (status == 0) {
        credential = 0;
        status = kmipkit_ttlv_structure_view(restored, &restored_view);
    }
    if (status == 0) status = attestation_tags_match(restored_view, 4);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(restored_view, &count);
    if (status == 0) *out_count = count;
    kmipkit_ttlv_structure_release(structure);
    kmipkit_attestation_credential_release(credential);
    kmipkit_ttlv_structure_view_release(source_view);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(restored);
    kmipkit_ttlv_structure_view_release(restored_view);
    return status;
}

int32_t kmipkit_attestation_credential_c_consumer_rejects_missing_evidence(void) {
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_attestation_credential_t *credential = 0;
    int32_t status = build_attestation_structure(0, &structure);
    if (status == 0) {
        status = kmipkit_attestation_credential_from_ttlv(structure, &credential);
        structure = 0;
        if (status == 0 || credential != 0) status = 1;
        else status = 0;
    }
    kmipkit_ttlv_structure_release(structure);
    kmipkit_attestation_credential_release(credential);
    return status;
}
