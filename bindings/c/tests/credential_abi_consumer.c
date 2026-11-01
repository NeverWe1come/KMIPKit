#include <stdint.h>

#include "kmipkit.h"

static const uint32_t CREDENTIAL_TYPE_TAG = 0x00420024;
static const uint32_t CREDENTIAL_VALUE_TAG = 0x00420025;
static const uint32_t CREDENTIAL_PAYLOAD_TAG = 0x0042002A;
static const int32_t CREDENTIAL_PAYLOAD_VALUE = 0x13579;
static const uint32_t DEVICE_SERIAL_NUMBER_TAG = 0x004200B0;
static const uint32_t USERNAME_TAG = 0x00420099;
static const uint32_t PASSWORD_TAG = 0x004200A1;
static const uint32_t DEVICE_IDENTIFIER_TAG = 0x004200A2;
static const uint32_t NETWORK_IDENTIFIER_TAG = 0x004200AB;
static const uint32_t MACHINE_IDENTIFIER_TAG = 0x004200A9;
static const uint32_t MEDIA_IDENTIFIER_TAG = 0x004200AA;
static const uint32_t DEVICE_EXTENSION_TAG = 0x00540001;
static const uint32_t HASHED_PASSWORD_USERNAME_TAG = 0x00420099;
static const uint32_t HASHED_PASSWORD_TIMESTAMP_TAG = 0x00420092;
static const uint32_t HASHING_ALGORITHM_TAG = 0x00420038;
static const uint32_t HASHED_PASSWORD_TAG = 0x00420157;
static const uint32_t ONE_TIME_PASSWORD_TAG = 0x00420156;
static const uint32_t UNKNOWN_HASHING_ALGORITHM = 0xe1234567;
static const uint8_t HASHED_PASSWORD_BYTES[] = {
    'h', 'a', 's', 'h', 'e', 'd', '-', 'p', 'a', 's', 's', 'w', 'o', 'r', 'd', 0, 0xff
};

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

static int32_t build_credential(uint32_t raw_type, kmipkit_credential_t **out_credential) {
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_structure_t *payload = 0;
    kmipkit_ttlv_value_t *value = 0;
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0) status = kmipkit_ttlv_value_enumeration(raw_type, &value);
    if (status == 0) {
        status = append_item(&structure, limits, CREDENTIAL_TYPE_TAG, value);
        value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_structure_create(&payload);
    if (status == 0) status = kmipkit_ttlv_value_integer(CREDENTIAL_PAYLOAD_VALUE, &value);
    if (status == 0) {
        status = append_item(&payload, limits, CREDENTIAL_PAYLOAD_TAG, value);
        value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_value_structure(payload, limits, &value);
    if (status == 0) {
        payload = 0;
        status = append_item(&structure, limits, CREDENTIAL_VALUE_TAG, value);
        value = 0;
    }
    if (status == 0) {
        status = kmipkit_credential_from_ttlv(structure, out_credential);
        structure = 0;
    }
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_structure_release(payload);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_codec_limits_release(limits);
    return status;
}

static int32_t authentication_tags_match(kmipkit_ttlv_structure_view_t *view) {
    uint64_t count = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_count(view, &count);
    if (status != 0 || count != 2) return status == 0 ? 1 : status;
    for (uint64_t index = 0; index < count; index++) {
        kmipkit_ttlv_item_view_t *item = 0;
        kmipkit_tag_t *tag = 0;
        uint32_t actual = 0;
        status = kmipkit_ttlv_structure_view_item_at(view, index, &item);
        if (status == 0) status = kmipkit_ttlv_item_view_tag(item, &tag);
        if (status == 0) status = kmipkit_ttlv_tag_value(tag, &actual);
        if (status == 0 && actual != 0x00420023) status = 1;
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
        if (status != 0) return status;
    }
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

static int32_t attestation_byte_string_matches(
        kmipkit_ttlv_structure_view_t *view, uint64_t index,
        uint32_t expected_tag, const uint8_t *expected_bytes, uint64_t expected_length) {
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_tag_t *tag = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint32_t actual_tag = 0;
    uint64_t actual_length = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_at(view, index, &item);
    if (status == 0) status = kmipkit_ttlv_item_view_tag(item, &tag);
    if (status == 0) status = kmipkit_ttlv_tag_value(tag, &actual_tag);
    if (status == 0 && actual_tag != expected_tag) status = 1;
    if (status == 0) status = kmipkit_ttlv_item_view_value(item, &value);
    if (status == 0) status = kmipkit_ttlv_value_view_byte_length(value, &actual_length);
    if (status == 0 && actual_length != expected_length) status = 1;
    for (uint64_t offset = 0; status == 0 && offset < expected_length; offset++) {
        uint8_t actual_byte = 0;
        status = kmipkit_ttlv_value_view_byte_at(value, offset, &actual_byte);
        if (status == 0 && actual_byte != expected_bytes[offset]) status = 1;
    }
    kmipkit_ttlv_value_view_release(value);
    kmipkit_tag_release(tag);
    kmipkit_ttlv_item_view_release(item);
    return status;
}

static int32_t attestation_values_match(kmipkit_ttlv_structure_view_t *view) {
    static const uint8_t expected_nonce_id[] = {0x01, 0x02, 0xFE};
    static const uint8_t expected_nonce_value[] = {0xA1, 0x00, 0xFF};
    static const uint8_t expected_measurement[] = {0x10, 0x20, 0x30};
    static const uint8_t expected_assertion[] = {0x40, 0x50, 0x60};
    kmipkit_ttlv_item_view_t *attestation_item = 0;
    kmipkit_ttlv_value_view_t *attestation_value = 0;
    kmipkit_ttlv_item_view_t *nonce_item = 0;
    kmipkit_ttlv_value_view_t *nonce_value = 0;
    kmipkit_ttlv_structure_view_t *nonce = 0;
    uint32_t attestation_type = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_at(view, 1, &attestation_item);
    if (status == 0) status = kmipkit_ttlv_item_view_value(attestation_item, &attestation_value);
    if (status == 0) status = kmipkit_ttlv_value_view_enumeration(
            attestation_value, &attestation_type);
    if (status == 0 && attestation_type != 0xF1234567u) status = 1;
    if (status == 0) status = kmipkit_ttlv_structure_view_item_at(view, 0, &nonce_item);
    if (status == 0) status = kmipkit_ttlv_item_view_value(nonce_item, &nonce_value);
    if (status == 0) status = kmipkit_ttlv_value_view_structure(nonce_value, &nonce);
    if (status == 0) status = attestation_byte_string_matches(
            nonce, 0, NONCE_ID_TAG, expected_nonce_id, sizeof(expected_nonce_id));
    if (status == 0) status = attestation_byte_string_matches(
            nonce, 1, NONCE_VALUE_TAG, expected_nonce_value, sizeof(expected_nonce_value));
    if (status == 0) status = attestation_byte_string_matches(
            view, 2, ATTESTATION_MEASUREMENT_TAG,
            expected_measurement, sizeof(expected_measurement));
    if (status == 0) status = attestation_byte_string_matches(
            view, 3, ATTESTATION_ASSERTION_TAG,
            expected_assertion, sizeof(expected_assertion));
    kmipkit_ttlv_structure_view_release(nonce);
    kmipkit_ttlv_value_view_release(nonce_value);
    kmipkit_ttlv_item_view_release(nonce_item);
    kmipkit_ttlv_value_view_release(attestation_value);
    kmipkit_ttlv_item_view_release(attestation_item);
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

static int32_t append_device_text(
        kmipkit_ttlv_structure_t **structure, kmipkit_codec_limits_t *limits,
        uint32_t raw_tag, const uint8_t *text, uint64_t text_length) {
    kmipkit_ttlv_value_t *value = 0;
    int32_t status = kmipkit_ttlv_value_text_string(
            limits, text, text_length, &value);
    if (status == 0) {
        status = append_item(structure, limits, raw_tag, value);
        value = 0;
    }
    kmipkit_ttlv_value_release(value);
    return status;
}

static int32_t build_device_structure(
        int include_identifier, kmipkit_ttlv_structure_t **out_structure) {
    static const uint8_t password[] = "sensitive-sentinel";
    static const uint8_t device_id[] = "device-id";
    static const uint8_t network_id[] = "network-id";
    static const uint8_t machine_id[] = "machine-id";
    static const uint8_t media_id[] = "media-id";
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_value_t *value = 0;
    int32_t status = kmipkit_codec_limits_create(4096, 8, 64, &limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0 && include_identifier) status = append_device_text(
            &structure, limits, DEVICE_SERIAL_NUMBER_TAG, (const uint8_t *)"", 0);
    if (status == 0) status = append_device_text(
            &structure, limits, PASSWORD_TAG, password, sizeof(password) - 1);
    if (status == 0) status = append_device_text(
            &structure, limits, DEVICE_IDENTIFIER_TAG, device_id, sizeof(device_id) - 1);
    if (status == 0 && include_identifier) status = append_device_text(
            &structure, limits, NETWORK_IDENTIFIER_TAG, network_id, sizeof(network_id) - 1);
    if (status == 0 && include_identifier) status = append_device_text(
            &structure, limits, MACHINE_IDENTIFIER_TAG, machine_id, sizeof(machine_id) - 1);
    if (status == 0 && include_identifier) status = append_device_text(
            &structure, limits, MEDIA_IDENTIFIER_TAG, media_id, sizeof(media_id) - 1);
    if (status == 0 && include_identifier) status = kmipkit_ttlv_value_integer(7, &value);
    if (status == 0 && include_identifier) {
        status = append_item(&structure, limits, DEVICE_EXTENSION_TAG, value);
        value = 0;
    }
    if (status == 0) {
        *out_structure = structure;
        structure = 0;
    }
    kmipkit_codec_limits_release(limits);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_ttlv_value_release(value);
    return status;
}

static int32_t device_tags_match(kmipkit_ttlv_structure_view_t *view) {
    static const uint32_t expected[] = {
        0x004200B0, 0x004200A1, 0x004200A2,
        0x004200AB, 0x004200A9, 0x004200AA, 0x00540001
    };
    uint64_t count = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_count(view, &count);
    if (status != 0 || count != sizeof(expected) / sizeof(expected[0]))
        return status == 0 ? 1 : status;
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
    return 0;
}

static int32_t device_text_value_matches(
        kmipkit_ttlv_structure_view_t *view, uint64_t index,
        const uint8_t *expected, uint64_t expected_length) {
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint8_t item_type = 0;
    uint64_t actual_length = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_at(view, index, &item);
    if (status == 0) status = kmipkit_ttlv_item_view_value(item, &value);
    if (status == 0) status = kmipkit_ttlv_value_view_type(value, &item_type);
    if (status == 0 && item_type != KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING) status = 1;
    if (status == 0) status = kmipkit_ttlv_value_view_byte_length(value, &actual_length);
    if (status == 0 && actual_length != expected_length) status = 1;
    for (uint64_t offset = 0; status == 0 && offset < expected_length; offset++) {
        uint8_t actual_byte = 0;
        status = kmipkit_ttlv_value_view_byte_at(value, offset, &actual_byte);
        if (status == 0 && actual_byte != expected[offset]) status = 1;
    }
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    return status;
}

static int32_t username_password_fields_match(kmipkit_ttlv_structure_view_t *view) {
    const uint32_t expected_tags[] = {USERNAME_TAG, PASSWORD_TAG};
    static const uint8_t username[] = "alice";
    static const uint8_t password[] = "";
    const uint8_t *expected_values[] = {username, password};
    const uint64_t expected_lengths[] = {
        sizeof(username) - 1, sizeof(password) - 1
    };
    for (uint64_t index = 0; index < 2; index++) {
        kmipkit_ttlv_item_view_t *item = 0;
        kmipkit_tag_t *tag = 0;
        uint32_t actual_tag = 0;
        int32_t status = kmipkit_ttlv_structure_view_item_at(view, index, &item);
        if (status == 0) status = kmipkit_ttlv_item_view_tag(item, &tag);
        if (status == 0) status = kmipkit_ttlv_tag_value(tag, &actual_tag);
        if (status == 0 && actual_tag != expected_tags[index]) status = 1;
        if (status == 0) status = device_text_value_matches(
                view, index, expected_values[index], expected_lengths[index]);
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
        if (status != 0) return status;
    }
    return 0;
}

static int32_t device_values_match(kmipkit_ttlv_structure_view_t *view) {
    static const uint8_t serial_number[] = "";
    static const uint8_t password[] = "sensitive-sentinel";
    static const uint8_t device_identifier[] = "device-id";
    static const uint8_t network_identifier[] = "network-id";
    static const uint8_t machine_identifier[] = "machine-id";
    static const uint8_t media_identifier[] = "media-id";
    const uint8_t *expected[] = {
        serial_number, password, device_identifier,
        network_identifier, machine_identifier, media_identifier
    };
    const uint64_t lengths[] = {
        sizeof(serial_number) - 1, sizeof(password) - 1, sizeof(device_identifier) - 1,
        sizeof(network_identifier) - 1, sizeof(machine_identifier) - 1,
        sizeof(media_identifier) - 1
    };
    for (uint64_t index = 0; index < 6; index++) {
        int32_t status = device_text_value_matches(view, index, expected[index], lengths[index]);
        if (status != 0) return status;
    }

    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint8_t item_type = 0;
    int32_t integer = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_at(view, 6, &item);
    if (status == 0) status = kmipkit_ttlv_item_view_value(item, &value);
    if (status == 0) status = kmipkit_ttlv_value_view_type(value, &item_type);
    if (status == 0 && item_type != KMIPKIT_TTLV_ITEM_TYPE_INTEGER) status = 1;
    if (status == 0) status = kmipkit_ttlv_value_view_integer(value, &integer);
    if (status == 0 && integer != 7) status = 1;
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    return status;
}

int32_t kmipkit_device_credential_c_consumer_round_trip(uint64_t *out_count) {
    if (out_count == 0) return -1;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_device_credential_t *credential = 0;
    kmipkit_ttlv_structure_view_t *source_view = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_structure_t *restored = 0;
    kmipkit_ttlv_structure_view_t *restored_view = 0;
    uint64_t count = 0;
    int32_t status = build_device_structure(1, &structure);
    if (status == 0) status = kmipkit_ttlv_structure_view(structure, &source_view);
    if (status == 0 && kmipkit_device_credential_from_ttlv(
            structure, &credential) != KMIPKIT_ERROR_SHARED_OWNER) status = 1;
    if (status == 0) status = device_tags_match(source_view);
    if (status == 0) status = device_values_match(source_view);
    kmipkit_ttlv_structure_view_release(source_view);
    source_view = 0;
    if (status == 0) status = kmipkit_device_credential_from_ttlv(structure, &credential);
    if (status == 0) {
        structure = 0;
        status = kmipkit_device_credential_ttlv_view(credential, &view);
    }
    if (status == 0) status = device_tags_match(view);
    if (status == 0) status = device_values_match(view);
    if (status == 0) {
        int32_t shared_status = kmipkit_device_credential_into_ttlv(credential, &restored);
        if (shared_status != 4 || restored != 0) status = 2;
    }
    kmipkit_ttlv_structure_view_release(view);
    view = 0;
    if (status == 0) status = kmipkit_device_credential_into_ttlv(credential, &restored);
    if (status == 0) {
        credential = 0;
        status = kmipkit_ttlv_structure_view(restored, &restored_view);
    }
    if (status == 0) status = device_tags_match(restored_view);
    if (status == 0) status = device_values_match(restored_view);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(restored_view, &count);
    if (status == 0) *out_count = count;
    kmipkit_ttlv_structure_release(structure);
    kmipkit_device_credential_release(credential);
    kmipkit_ttlv_structure_view_release(source_view);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(restored);
    kmipkit_ttlv_structure_view_release(restored_view);
    return status;
}

int32_t kmipkit_device_credential_c_consumer_rejects_missing_identifier(void) {
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_device_credential_t *credential = 0;
    kmipkit_credential_validation_error_v1_t error = {
        (uint32_t)sizeof(kmipkit_credential_validation_error_v1_t), 1, 0
    };
    int32_t status = build_device_structure(0, &structure);
    if (status == 0) {
        status = kmipkit_device_credential_from_ttlv(structure, &credential);
        structure = 0;
        if (status != 4 || credential != 0) status = 1;
        else status = 0;
    }
    if (status == 0) status = kmipkit_last_credential_validation_error_get_v1(&error);
    if (status == 0 && error.kind != KMIPKIT_CREDENTIAL_VALIDATION_ERROR_KIND_MISSING_FIELD)
        status = 1;
    if (status == 0) status = kmipkit_last_credential_validation_error_get_v1(&error);
    if (status == 0 && error.kind != KMIPKIT_CREDENTIAL_VALIDATION_ERROR_KIND_MISSING_FIELD)
        status = 1;
    if (status == 0) status = kmipkit_last_error_clear();
    error.kind = KMIPKIT_CREDENTIAL_VALIDATION_ERROR_KIND_MISSING_FIELD;
    if (status == 0) status = kmipkit_last_credential_validation_error_get_v1(&error);
    if (status == 0 && error.kind != 0) status = 1;
    kmipkit_ttlv_structure_release(structure);
    kmipkit_device_credential_release(credential);
    return status;
}

int32_t kmipkit_username_password_credential_c_consumer_round_trip(uint64_t *out_count) {
    static const uint8_t username[] = "alice";
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_structure_t *restored = 0;
    kmipkit_ttlv_structure_view_t *restored_view = 0;
    kmipkit_username_and_password_credential_t *credential = 0;
    uint64_t count = 0;
    int32_t status = out_count == 0 ? 4 : kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0) status = append_device_text(
            &structure, limits, USERNAME_TAG, username, sizeof(username) - 1);
    if (status == 0) status = append_device_text(
            &structure, limits, PASSWORD_TAG, (const uint8_t *)"", 0);
    if (status == 0) {
        status = kmipkit_username_and_password_credential_from_ttlv(structure, &credential);
        structure = 0;
    }
    if (status == 0) status = kmipkit_username_and_password_credential_ttlv_view(
            credential, &view);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(view, &count);
    if (status == 0 && count != 2) status = 1;
    if (status == 0) status = username_password_fields_match(view);
    if (status == 0) {
        int32_t shared_status = kmipkit_username_and_password_credential_into_ttlv(
                credential, &restored);
        if (shared_status != 4 || restored != 0) status = 2;
    }
    kmipkit_ttlv_structure_view_release(view);
    view = 0;
    if (status == 0) status = kmipkit_username_and_password_credential_into_ttlv(
            credential, &restored);
    if (status == 0) {
        credential = 0;
        status = kmipkit_ttlv_structure_view(restored, &restored_view);
    }
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(restored_view, &count);
    if (status == 0 && count != 2) status = 3;
    if (status == 0) status = username_password_fields_match(restored_view);
    if (status == 0) *out_count = count;
    kmipkit_codec_limits_release(limits);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(restored);
    kmipkit_ttlv_structure_view_release(restored_view);
    kmipkit_username_and_password_credential_release(credential);
    return status;
}

int32_t kmipkit_username_password_credential_c_consumer_rejects_missing_username(void) {
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_username_and_password_credential_t *credential = 0;
    kmipkit_credential_validation_error_v1_t error = {
        (uint32_t)sizeof(kmipkit_credential_validation_error_v1_t), 1, 0
    };
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0) status = append_device_text(
            &structure, limits, PASSWORD_TAG,
            (const uint8_t *)"sensitive-sentinel", 18);
    if (status == 0) {
        status = kmipkit_username_and_password_credential_from_ttlv(structure, &credential);
        structure = 0;
        if (status != 4 || credential != 0) status = 1;
        else status = 0;
    }
    if (status == 0) status = kmipkit_last_credential_validation_error_get_v1(&error);
    if (status == 0 && error.kind != KMIPKIT_CREDENTIAL_VALIDATION_ERROR_KIND_MISSING_FIELD)
        status = 2;
    kmipkit_codec_limits_release(limits);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_username_and_password_credential_release(credential);
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
    if (status == 0) status = attestation_values_match(view);
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
    if (status == 0) status = attestation_values_match(restored_view);
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
    kmipkit_credential_validation_error_v1_t error = {
        (uint32_t)sizeof(kmipkit_credential_validation_error_v1_t), 1, 0
    };
    int32_t status = build_attestation_structure(0, &structure);
    if (status == 0) {
        status = kmipkit_attestation_credential_from_ttlv(structure, &credential);
        structure = 0;
        if (status == 0 || credential != 0) status = 1;
        else status = 0;
    }
    if (status == 0) status = kmipkit_last_credential_validation_error_get_v1(&error);
    if (status == 0 && error.kind != KMIPKIT_CREDENTIAL_VALIDATION_ERROR_KIND_MISSING_FIELD)
        status = 1;
    kmipkit_ttlv_structure_release(structure);
    kmipkit_attestation_credential_release(credential);
    return status;
}

int32_t kmipkit_authentication_c_consumer_round_trip(
        uint32_t first_type, uint32_t second_type, uint64_t *out_count) {
    if (out_count == 0) return 4;
    kmipkit_credential_t *credentials[2] = {0, 0};
    kmipkit_authentication_t *authentication = 0;
    kmipkit_credential_view_t *first_view = 0;
    kmipkit_credential_view_t *second_view = 0;
    kmipkit_ttlv_structure_view_t *first_structure_view = 0;
    kmipkit_ttlv_structure_t *restored = 0;
    kmipkit_ttlv_structure_view_t *restored_view = 0;
    uint64_t count = 0;
    uint32_t observed_first = 0;
    uint32_t observed_second = 0;
    int32_t status = build_credential(first_type, &credentials[0]);
    if (status == 0) status = build_credential(second_type, &credentials[1]);
    if (status == 0) {
        status = kmipkit_authentication_create(credentials, 2, &authentication);
        if (status == 0) credentials[0] = credentials[1] = 0;
    }
    if (status == 0) {
        status = kmipkit_authentication_credential_count(authentication, &count);
        if (status == 0 && count != 2) status = 101;
    }
    if (status == 0) status = kmipkit_authentication_credential_at(
            authentication, 0, &first_view);
    if (status == 0) status = kmipkit_authentication_credential_at(
            authentication, 1, &second_view);
    if (status == 0) status = kmipkit_credential_view_type_raw(first_view, &observed_first);
    if (status == 0) status = kmipkit_credential_view_type_raw(second_view, &observed_second);
    if (status == 0 && (observed_first != first_type || observed_second != second_type)) status = 102;
    if (status == 0) status = kmipkit_credential_view_ttlv_view(
            first_view, &first_structure_view);
    if (status == 0) status = tags_match(first_structure_view, &count);
    if (status == 0) {
        int32_t shared_status = kmipkit_authentication_into_ttlv(authentication, &restored);
        if (shared_status != 4 || restored != 0) status = 103;
    }
    kmipkit_ttlv_structure_view_release(first_structure_view);
    first_structure_view = 0;
    kmipkit_credential_view_release(first_view);
    first_view = 0;
    kmipkit_credential_view_release(second_view);
    second_view = 0;
    if (status == 0) status = kmipkit_authentication_into_ttlv(authentication, &restored);
    if (status == 0) {
        authentication = 0;
        status = kmipkit_ttlv_structure_view(restored, &restored_view);
    }
    if (status == 0) {
        status = authentication_tags_match(restored_view);
        if (status == 1) status = 104;
    }
    if (status == 0) {
        status = kmipkit_ttlv_structure_view_item_count(restored_view, &count);
        if (status == 0 && count != 2) status = 105;
    }
    if (status == 0) *out_count = count;
    kmipkit_credential_release(credentials[0]);
    kmipkit_credential_release(credentials[1]);
    kmipkit_authentication_release(authentication);
    kmipkit_credential_view_release(first_view);
    kmipkit_credential_view_release(second_view);
    kmipkit_ttlv_structure_view_release(first_structure_view);
    kmipkit_ttlv_structure_release(restored);
    kmipkit_ttlv_structure_view_release(restored_view);
    return status;
}

int32_t kmipkit_authentication_c_consumer_rejects_empty_credentials(void) {
    kmipkit_authentication_t *authentication = 0;
    kmipkit_credential_validation_error_v1_t error = {
        (uint32_t)sizeof(kmipkit_credential_validation_error_v1_t), 1, 0
    };
    int32_t status = kmipkit_authentication_create(0, 0, &authentication);
    if (status != 4 || authentication != 0) {
        kmipkit_authentication_release(authentication);
        return 1;
    }
    status = kmipkit_last_credential_validation_error_get_v1(&error);
    if (status == 0 && error.kind != KMIPKIT_CREDENTIAL_VALIDATION_ERROR_KIND_EMPTY_AUTHENTICATION)
        status = 1;
    return status;
}

int32_t kmipkit_credential_c_consumer_rejects_missing_required_field(void) {
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_credential_t *credential = 0;
    kmipkit_credential_validation_error_v1_t error = {
        (uint32_t)sizeof(kmipkit_credential_validation_error_v1_t), 1, 0
    };
    int32_t status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0) {
        status = kmipkit_credential_from_ttlv(structure, &credential);
        structure = 0;
        if (status != 4 || credential != 0) status = 1;
        else status = 0;
    }
    if (status == 0) status = kmipkit_last_credential_validation_error_get_v1(&error);
    if (status == 0 && error.kind != KMIPKIT_CREDENTIAL_VALIDATION_ERROR_KIND_MISSING_FIELD)
        status = 1;
    kmipkit_ttlv_structure_release(structure);
    kmipkit_credential_release(credential);
    return status;
}

int32_t kmipkit_nonce_c_consumer_rejects_missing_required_value(void) {
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_nonce_t *nonce = 0;
    kmipkit_credential_validation_error_v1_t error = {
        (uint32_t)sizeof(kmipkit_credential_validation_error_v1_t), 1, 0
    };
    int32_t status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0) {
        status = kmipkit_nonce_from_ttlv(structure, &nonce);
        structure = 0;
        if (status != 4 || nonce != 0) status = 1;
        else status = 0;
    }
    if (status == 0) status = kmipkit_last_credential_validation_error_get_v1(&error);
    if (status == 0 && error.kind != KMIPKIT_CREDENTIAL_VALIDATION_ERROR_KIND_MISSING_FIELD)
        status = 1;
    kmipkit_ttlv_structure_release(structure);
    kmipkit_nonce_release(nonce);
    return status;
}

static int32_t hashed_password_tags_match(
        kmipkit_ttlv_structure_view_t *view, int include_algorithm) {
    const uint32_t with_algorithm[] = {
        HASHED_PASSWORD_USERNAME_TAG, HASHED_PASSWORD_TIMESTAMP_TAG,
        HASHING_ALGORITHM_TAG, HASHED_PASSWORD_TAG
    };
    const uint32_t without_algorithm[] = {
        HASHED_PASSWORD_USERNAME_TAG, HASHED_PASSWORD_TIMESTAMP_TAG, HASHED_PASSWORD_TAG
    };
    const uint32_t *expected = include_algorithm ? with_algorithm : without_algorithm;
    uint64_t expected_count = include_algorithm ? 4 : 3;
    uint64_t count = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_count(view, &count);
    if (status != 0 || count != expected_count) return status == 0 ? 1 : status;
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
    return 0;
}

static int32_t hashed_password_timestamp_matches(kmipkit_ttlv_structure_view_t *view) {
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint32_t item_type = 0;
    int64_t timestamp = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_at(view, 1, &item);
    if (status == 0) status = kmipkit_ttlv_item_view_value(item, &value);
    if (status == 0) status = kmipkit_ttlv_value_view_type(value, &item_type);
    if (status == 0 && item_type != KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME_EXTENDED) status = 1;
    if (status == 0) status = kmipkit_ttlv_value_view_date_time_extended(value, &timestamp);
    if (status == 0 && timestamp != 1800000000) status = 1;
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    return status;
}

static int32_t hashed_password_username_matches(kmipkit_ttlv_structure_view_t *view) {
    const uint8_t expected[] = {'a', 'l', 'i', 'c', 'e'};
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    uint32_t item_type = 0;
    uint64_t length = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_at(view, 0, &item);
    if (status == 0) status = kmipkit_ttlv_item_view_value(item, &value);
    if (status == 0) status = kmipkit_ttlv_value_view_type(value, &item_type);
    if (status == 0 && item_type != KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING) status = 1;
    if (status == 0) status = kmipkit_ttlv_value_view_byte_length(value, &length);
    if (status == 0 && length != sizeof(expected)) status = 1;
    for (uint64_t offset = 0; status == 0 && offset < length; offset++) {
        uint8_t actual = 0;
        status = kmipkit_ttlv_value_view_byte_at(value, offset, &actual);
        if (status == 0 && actual != expected[offset]) status = 1;
    }
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    return status;
}

static int32_t hashed_password_bytes_match(
        kmipkit_ttlv_structure_view_t *view, int include_algorithm) {
    uint64_t index = include_algorithm ? 3 : 2;
    uint64_t length = 0;
    kmipkit_ttlv_item_view_t *item = 0;
    kmipkit_ttlv_value_view_t *value = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_at(view, index, &item);
    if (status == 0) status = kmipkit_ttlv_item_view_value(item, &value);
    if (status == 0) status = kmipkit_ttlv_value_view_byte_length(value, &length);
    if (status == 0 && length != sizeof(HASHED_PASSWORD_BYTES)) status = 1;
    for (uint64_t offset = 0; status == 0 && offset < length; offset++) {
        uint8_t actual = 0;
        status = kmipkit_ttlv_value_view_byte_at(value, offset, &actual);
        if (status == 0 && actual != HASHED_PASSWORD_BYTES[offset]) status = 1;
    }
    kmipkit_ttlv_value_view_release(value);
    kmipkit_ttlv_item_view_release(item);
    return status;
}

static int32_t hashed_password_credential_case(int include_algorithm) {
    const uint8_t username[] = {'a', 'l', 'i', 'c', 'e'};
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_structure_t *restored = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_ttlv_structure_view_t *restored_view = 0;
    kmipkit_hashed_password_credential_t *credential = 0;
    uint8_t algorithm_present = 0;
    uint32_t algorithm_raw = 0;
    uint32_t effective_algorithm = 0;
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&structure);
    kmipkit_ttlv_value_t *value = 0;
    if (status == 0) status = kmipkit_ttlv_value_text_string(
            limits, username, sizeof(username), &value);
    if (status == 0) {
        status = append_item(&structure, limits, HASHED_PASSWORD_USERNAME_TAG, value);
        value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_value_date_time_extended(1800000000, &value);
    if (status == 0) {
        status = append_item(&structure, limits, HASHED_PASSWORD_TIMESTAMP_TAG, value);
        value = 0;
    }
    if (status == 0 && include_algorithm)
        status = kmipkit_ttlv_value_enumeration(UNKNOWN_HASHING_ALGORITHM, &value);
    if (status == 0 && include_algorithm) {
        status = append_item(&structure, limits, HASHING_ALGORITHM_TAG, value);
        value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_value_byte_string(
            limits, HASHED_PASSWORD_BYTES, sizeof(HASHED_PASSWORD_BYTES), &value);
    if (status == 0) {
        status = append_item(&structure, limits, HASHED_PASSWORD_TAG, value);
        value = 0;
    }
    if (status == 0) {
        status = kmipkit_hashed_password_credential_from_ttlv(structure, &credential);
        structure = 0;
    }
    if (status == 0) status = kmipkit_hashed_password_credential_hashing_algorithm_raw(
            credential, &algorithm_present, &algorithm_raw);
    if (status == 0 && algorithm_present != (uint8_t)include_algorithm) status = 1;
    if (status == 0 && include_algorithm && algorithm_raw != UNKNOWN_HASHING_ALGORITHM) status = 1;
    if (status == 0 && !include_algorithm && algorithm_raw != 0) status = 1;
    if (status == 0) status = kmipkit_hashed_password_credential_effective_hashing_algorithm_raw(
            credential, &effective_algorithm);
    if (status == 0 && effective_algorithm != (include_algorithm ? UNKNOWN_HASHING_ALGORITHM : 6))
        status = 1;
    if (status == 0) status = kmipkit_hashed_password_credential_ttlv_view(credential, &view);
    if (status == 0) status = hashed_password_tags_match(view, include_algorithm);
    if (status == 0) status = hashed_password_username_matches(view);
    if (status == 0) status = hashed_password_timestamp_matches(view);
    if (status == 0) status = hashed_password_bytes_match(view, include_algorithm);
    kmipkit_ttlv_structure_view_release(view);
    view = 0;
    if (status == 0) status = kmipkit_hashed_password_credential_into_ttlv(credential, &restored);
    if (status == 0) {
        credential = 0;
        status = kmipkit_ttlv_structure_view(restored, &restored_view);
    }
    if (status == 0) status = hashed_password_tags_match(restored_view, include_algorithm);
    if (status == 0) status = hashed_password_username_matches(restored_view);
    if (status == 0) status = hashed_password_timestamp_matches(restored_view);
    if (status == 0) status = hashed_password_bytes_match(restored_view, include_algorithm);
    kmipkit_codec_limits_release(limits);
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_ttlv_structure_release(restored);
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_view_release(restored_view);
    kmipkit_hashed_password_credential_release(credential);
    return status;
}

/* KMIP Specification v2.1 §9.11, Table 415 defines uniqueness and field order. */
static int32_t hashed_password_credential_rejects_field_case(int duplicate_username) {
    const uint8_t username[] = {'a', 'l', 'i', 'c', 'e'};
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_value_t *value = 0;
    kmipkit_hashed_password_credential_t *credential = 0;
    kmipkit_credential_validation_error_v1_t error = {
        (uint32_t)sizeof(kmipkit_credential_validation_error_v1_t), 1, 0
    };
    int32_t expected_kind = duplicate_username
            ? KMIPKIT_CREDENTIAL_VALIDATION_ERROR_KIND_DUPLICATE_FIELD
            : KMIPKIT_CREDENTIAL_VALIDATION_ERROR_KIND_FIELD_OUT_OF_ORDER;
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0) status = kmipkit_ttlv_value_text_string(
            limits, username, sizeof(username), &value);
    if (status == 0) {
        status = append_item(&structure, limits, HASHED_PASSWORD_USERNAME_TAG, value);
        value = 0;
    }
    if (status == 0 && duplicate_username) status = kmipkit_ttlv_value_text_string(
            limits, username, sizeof(username), &value);
    if (status == 0 && duplicate_username) {
        status = append_item(&structure, limits, HASHED_PASSWORD_USERNAME_TAG, value);
        value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_value_date_time_extended(1800000000, &value);
    if (status == 0) {
        status = append_item(&structure, limits, HASHED_PASSWORD_TIMESTAMP_TAG, value);
        value = 0;
    }
    if (status == 0) status = kmipkit_ttlv_value_byte_string(
            limits, HASHED_PASSWORD_BYTES, sizeof(HASHED_PASSWORD_BYTES), &value);
    if (status == 0) {
        status = append_item(&structure, limits, HASHED_PASSWORD_TAG, value);
        value = 0;
    }
    if (status == 0 && !duplicate_username)
        status = kmipkit_ttlv_value_enumeration(UNKNOWN_HASHING_ALGORITHM, &value);
    if (status == 0 && !duplicate_username) {
        status = append_item(&structure, limits, HASHING_ALGORITHM_TAG, value);
        value = 0;
    }
    if (status == 0) {
        status = kmipkit_hashed_password_credential_from_ttlv(structure, &credential);
        structure = 0;
        if (status != 4 || credential != 0) status = 1;
        else status = 0;
    }
    if (status == 0) status = kmipkit_last_credential_validation_error_get_v1(&error);
    if (status == 0 && error.kind != (uint32_t)expected_kind) status = 1;
    kmipkit_codec_limits_release(limits);
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_hashed_password_credential_release(credential);
    return status;
}

int32_t kmipkit_hashed_password_credential_c_consumer_contract(void) {
    int32_t status = hashed_password_credential_case(1);
    if (status == 0) status = hashed_password_credential_case(0);
    if (status == 0) status = hashed_password_credential_rejects_field_case(1);
    if (status == 0) status = hashed_password_credential_rejects_field_case(0);
    return status;
}

static int32_t build_credential_value_structure(
        uint32_t raw_type, kmipkit_ttlv_structure_t **out_structure) {
    const uint8_t username[] = {'a', 'l', 'i', 'c', 'e'};
    const uint8_t otp[] = {'o', 't', 'p', '-', 's', 'e', 'n', 't', 'i', 'n', 'e', 'l'};
    kmipkit_codec_limits_t *limits = 0;
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_value_t *value = 0;
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_structure_create(&structure);
    if (status == 0 && raw_type == 4) {
        status = kmipkit_ttlv_value_text_string(limits, username, sizeof(username), &value);
        if (status == 0) {
            status = append_item(&structure, limits, USERNAME_TAG, value);
            value = 0;
        }
        if (status == 0) {
            status = kmipkit_ttlv_value_text_string(limits, otp, sizeof(otp), &value);
        }
        if (status == 0) {
            status = append_item(&structure, limits, ONE_TIME_PASSWORD_TAG, value);
            value = 0;
        }
    } else if (status == 0) {
        status = kmipkit_ttlv_value_integer(CREDENTIAL_PAYLOAD_VALUE, &value);
        if (status == 0) {
            status = append_item(&structure, limits, DEVICE_EXTENSION_TAG, value);
            value = 0;
        }
    }
    if (status == 0) {
        *out_structure = structure;
        structure = 0;
    }
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_codec_limits_release(limits);
    return status;
}

static int32_t credential_value_round_trip(uint32_t expected_type, uint64_t expected_count) {
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_ttlv_structure_t *restored = 0;
    kmipkit_ttlv_structure_view_t *view = 0;
    kmipkit_credential_value_t *credential_value = 0;
    uint32_t observed_type = 0;
    uint64_t observed_count = 0;
    int32_t status = build_credential_value_structure(expected_type, &structure);
    if (status == 0) {
        status = kmipkit_credential_value_from_ttlv(expected_type, structure, &credential_value);
        structure = 0;
    }
    if (status == 0) status = kmipkit_credential_value_type_raw(credential_value, &observed_type);
    if (status == 0 && observed_type != expected_type) status = 1;
    if (status == 0) status = kmipkit_credential_value_ttlv_view(credential_value, &view);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(view, &observed_count);
    if (status == 0 && observed_count != expected_count) status = 1;
    kmipkit_ttlv_structure_view_release(view);
    view = 0;
    if (status == 0) status = kmipkit_credential_value_into_ttlv(credential_value, &restored);
    if (status == 0) credential_value = 0;
    if (status == 0) status = kmipkit_ttlv_structure_view(restored, &view);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(view, &observed_count);
    if (status == 0 && observed_count != expected_count) status = 1;
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(structure);
    kmipkit_ttlv_structure_release(restored);
    kmipkit_credential_value_release(credential_value);
    return status;
}

int32_t kmipkit_credential_value_c_consumer_contract(void) {
    // KMIP Specification v2.1 §9.11 Tables 414 and 442: validate the OTP shape and preserve raw
    // Extensions and unknown Credential Type discriminators through the generic value facade.
    int32_t status = credential_value_round_trip(4, 2);
    if (status == 0) status = credential_value_round_trip(0x80004321, 1);
    if (status == 0) status = credential_value_round_trip(0xf1234567, 1);
    return status;
}
