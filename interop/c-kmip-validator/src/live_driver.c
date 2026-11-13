#include "live_driver.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifndef KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE
#define KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE 0
#endif

#define KMIP_MAX_REQUEST_BYTES UINT64_C(16777216)

static kmipkit_byte_span_v1_t borrowed_span(const char *text) {
    kmipkit_byte_span_v1_t span = {0};
    if (text) {
        span.data = (const uint8_t *)text;
        span.length = (uint64_t)strlen(text);
    }
    return span;
}

static kmipkit_timeout_limit_v1_t bounded_timeout(uint64_t seconds) {
    kmipkit_timeout_limit_v1_t timeout = {
        .kind = KMIPKIT_TRANSPORT_TIMEOUT_BOUNDED,
        .reserved = 0,
        .seconds = seconds,
        .nanoseconds = 0,
        .reserved2 = 0
    };
    return timeout;
}

static bool parse_version(const char *text, int32_t *major, int32_t *minor) {
    if (strcmp(text, "2.0") == 0) {
        *major = 2;
        *minor = 0;
        return true;
    }
    if (strcmp(text, "2.1") == 0) {
        *major = 2;
        *minor = 1;
        return true;
    }
    return false;
}

static bool parse_attribute_reference_tag(const char *text, uint32_t *out_tag) {
    if (!text || text[0] == '\0' || !out_tag) return false;
    char *end = NULL;
    unsigned long value = strtoul(text, &end, 16);
    if (end == text || *end != '\0' || value == 0 || value > UINT32_C(0xFFFFFF)) return false;
    *out_tag = (uint32_t)value;
    return true;
}

static bool parse_attribute_response_type(const char *text, uint8_t *out_type) {
    if (!text || text[0] == '\0' || !out_type) return false;
    char *end = NULL;
    unsigned long value = strtoul(text, &end, 10);
    if (end == text || *end != '\0' || value < KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE ||
        value > KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME_EXTENDED) return false;
    *out_type = (uint8_t)value;
    return true;
}

static bool parse_wire_encoding(const char *text, uint32_t *encoding) {
    if (strcmp(text, "TTLV") == 0) *encoding = KMIPKIT_WIRE_ENCODING_TTLV;
    else if (strcmp(text, "JSON") == 0) *encoding = KMIPKIT_WIRE_ENCODING_JSON;
    else if (strcmp(text, "XML") == 0) *encoding = KMIPKIT_WIRE_ENCODING_XML;
    else return false;
    return true;
}

static void set_error(char *error, size_t capacity, const char *message, int32_t status) {
    if (error && capacity > 0) {
        if (status == 0) snprintf(error, capacity, "%s", message);
        else snprintf(error, capacity, "%s (KMIPKit status %d)", message, status);
    }
}

static bool append_ttlv_value(kmip_live_client_t *client,
                              kmipkit_ttlv_structure_t **structure,
                              uint32_t raw_tag_value,
                              kmipkit_ttlv_value_t **value) {
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_ttlv_structure_t *updated = NULL;
    int32_t status = kmipkit_ttlv_raw_tag_create(raw_tag_value, &raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag);
    if (status == 0) {
        status = kmipkit_ttlv_item_create(tag, *value, client->limits, &item);
        *value = NULL;
    }
    if (status == 0) {
        status = kmipkit_ttlv_structure_with_item(*structure, item, client->limits, &updated);
        *structure = NULL;
        item = NULL;
        if (status == 0) {
            *structure = updated;
            updated = NULL;
        }
    }
    if (item) kmipkit_ttlv_item_release(item);
    if (updated) kmipkit_ttlv_structure_release(updated);
    if (tag) kmipkit_tag_release(tag);
    if (raw_tag) kmipkit_raw_tag_release(raw_tag);
    return status == 0;
}

static bool append_secret_text(kmip_live_client_t *client,
                               kmipkit_ttlv_structure_t **structure,
                               uint32_t raw_tag,
                               const char *text) {
    kmipkit_secret_text_t *secret = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    int32_t status = kmipkit_secret_text_create((const uint8_t *)text,
                                                (uint64_t)strlen(text), &secret);
    if (status == 0) {
        status = kmipkit_secret_text_into_ttlv_value(secret, &value);
        secret = NULL;
    }
    bool appended = status == 0 && append_ttlv_value(client, structure, raw_tag, &value);
    if (secret) kmipkit_secret_text_release(secret);
    if (value) kmipkit_ttlv_value_release(value);
    return appended;
}

static bool structure_into_value(kmip_live_client_t *client,
                                 kmipkit_ttlv_structure_t **structure,
                                 kmipkit_ttlv_value_t **value) {
    int32_t status = kmipkit_ttlv_value_structure(*structure, client->limits, value);
    *structure = NULL;
    return status == 0;
}

static bool create_cryptographic_key_attributes(kmip_live_client_t *client,
                                               uint32_t algorithm,
                                               int32_t length,
                                               kmipkit_ttlv_structure_t **out_attributes) {
    int32_t status = kmipkit_ttlv_structure_create(out_attributes);
    kmipkit_ttlv_value_t *value = NULL;
    bool valid = status == 0 && kmipkit_ttlv_value_enumeration(algorithm, &value) == 0 &&
        append_ttlv_value(client, out_attributes, UINT32_C(0x420028), &value) &&
        kmipkit_ttlv_value_integer(length, &value) == 0 &&
        append_ttlv_value(client, out_attributes, UINT32_C(0x42002A), &value);
    if (value) kmipkit_ttlv_value_release(value);
    if (!valid && *out_attributes) {
        kmipkit_ttlv_structure_release(*out_attributes);
        *out_attributes = NULL;
    }
    return valid;
}

static bool create_symmetric_key_attributes(kmip_live_client_t *client,
                                            kmipkit_ttlv_structure_t **out_attributes) {
    return create_cryptographic_key_attributes(
        client, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES, 128, out_attributes);
}

static bool create_rsa_key_attributes(kmip_live_client_t *client,
                                      kmipkit_ttlv_structure_t **out_attributes) {
    return create_cryptographic_key_attributes(
        client, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_RSA, 2048, out_attributes);
}

static bool item_view_has_tag(kmipkit_ttlv_item_view_t *item,
                              uint32_t expected_tag,
                              uint8_t expected_type,
                              kmipkit_ttlv_value_view_t **out_value) {
    kmipkit_tag_t *tag = NULL;
    uint32_t raw_tag = 0;
    uint8_t item_type = 0;
    bool valid = kmipkit_ttlv_item_view_tag(item, &tag) == 0 &&
        kmipkit_ttlv_tag_value(tag, &raw_tag) == 0 &&
        kmipkit_ttlv_item_view_type(item, &item_type) == 0 &&
        raw_tag == expected_tag && item_type == expected_type;
    if (tag) kmipkit_tag_release(tag);
    if (valid && out_value) valid = kmipkit_ttlv_item_view_value(item, out_value) == 0;
    return valid;
}

static int32_t register_payload_identifier_status(
    kmipkit_ttlv_structure_view_t *payload_view,
    const kmipkit_unique_identifier_v1_t *expected_identifier) {
    if (!payload_view || !expected_identifier ||
        expected_identifier->kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        !expected_identifier->text_data || expected_identifier->text_length == 0) {
        return KMIPKIT_ERROR_INVALID_INPUT;
    }
    uint64_t item_count = 0;
    int32_t status = kmipkit_ttlv_structure_view_item_count(payload_view, &item_count);
    bool identifier_found = false;
    bool identifier_matches = false;
    for (uint64_t index = 0; status == 0 && index < item_count; ++index) {
        kmipkit_ttlv_item_view_t *item_view = NULL;
        kmipkit_ttlv_value_view_t *value_view = NULL;
        status = kmipkit_ttlv_structure_view_item_at(payload_view, index, &item_view);
        if (status == 0 && item_view_has_tag(item_view, UINT32_C(0x420094),
                                              KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING, &value_view)) {
            if (identifier_found) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            } else {
                identifier_found = true;
                uint64_t value_length = 0;
                status = kmipkit_ttlv_value_view_byte_length(value_view, &value_length);
                identifier_matches = status == 0 &&
                    value_length == expected_identifier->text_length;
                if (status == 0 && !identifier_matches) status = KMIPKIT_ERROR_INVALID_INPUT;
                for (uint64_t byte_index = 0; status == 0 && byte_index < value_length; ++byte_index) {
                    uint8_t byte = 0;
                    status = kmipkit_ttlv_value_view_byte_at(value_view, byte_index, &byte);
                    identifier_matches = status == 0 &&
                        byte == expected_identifier->text_data[byte_index];
                    if (status == 0 && !identifier_matches) status = KMIPKIT_ERROR_INVALID_INPUT;
                }
            }
        }
        if (value_view) kmipkit_ttlv_value_view_release(value_view);
        if (item_view) kmipkit_ttlv_item_view_release(item_view);
    }
    if (status == 0 && (!identifier_found || !identifier_matches)) {
        status = KMIPKIT_ERROR_INVALID_INPUT;
    }
    return status;
}

typedef int32_t (*client_response_ttlv_accessor_t)(
    kmipkit_client_batch_item_response_t *, uint8_t *, kmipkit_ttlv_structure_t **);

static bool response_ttlv_matches_identifier(
    kmipkit_client_batch_item_response_t *response_item,
    client_response_ttlv_accessor_t accessor,
    const kmipkit_unique_identifier_v1_t *expected_identifier,
    int32_t *out_status) {
    if (!out_status) return false;
    *out_status = KMIPKIT_ERROR_INVALID_INPUT;
    if (!response_item || !expected_identifier ||
        expected_identifier->kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        !expected_identifier->text_data || expected_identifier->text_length == 0) {
        return false;
    }
    uint8_t present = 0;
    kmipkit_ttlv_structure_t *structure = NULL;
    kmipkit_ttlv_structure_view_t *structure_view = NULL;
    int32_t status = accessor(response_item, &present, &structure);
    if (status == 0 && (!present || !structure)) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_structure_view(structure, &structure_view);
    uint64_t item_count = 0;
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(structure_view, &item_count);
    bool payload_found = false;
    for (uint64_t index = 0; status == 0 && index < item_count; ++index) {
        kmipkit_ttlv_item_view_t *item_view = NULL;
        kmipkit_ttlv_value_view_t *value_view = NULL;
        kmipkit_ttlv_structure_view_t *payload_view = NULL;
        status = kmipkit_ttlv_structure_view_item_at(structure_view, index, &item_view);
        if (status == 0 && item_view_has_tag(item_view, UINT32_C(0x42007C),
                                              KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE, &value_view)) {
            if (payload_found) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            } else {
                payload_found = true;
                status = kmipkit_ttlv_value_view_structure(value_view, &payload_view);
                if (status == 0) status = register_payload_identifier_status(
                    payload_view, expected_identifier);
            }
        }
        if (payload_view) kmipkit_ttlv_structure_view_release(payload_view);
        if (value_view) kmipkit_ttlv_value_view_release(value_view);
        if (item_view) kmipkit_ttlv_item_view_release(item_view);
    }
    if (status == 0 && !payload_found) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (structure_view) kmipkit_ttlv_structure_view_release(structure_view);
    if (structure) kmipkit_ttlv_structure_release(structure);
    *out_status = status;
    return status == 0;
}

static bool attributes_match_created_symmetric_key(kmipkit_ttlv_structure_t *attributes) {
    kmipkit_ttlv_structure_view_t *view = NULL;
    uint64_t count = 0;
    bool valid = attributes && kmipkit_ttlv_structure_view(attributes, &view) == 0 &&
        kmipkit_ttlv_structure_view_item_count(view, &count) == 0;
    bool algorithm_found = false;
    bool length_found = false;
    for (uint64_t index = 0; valid && index < count; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        if (kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0) {
            valid = false;
            break;
        }
        kmipkit_ttlv_value_view_t *value = NULL;
        if (item_view_has_tag(item, UINT32_C(0x420028),
                              KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION, &value)) {
            uint32_t actual = 0;
            algorithm_found = kmipkit_ttlv_value_view_enumeration(value, &actual) == 0 &&
                actual == KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
        } else if (item_view_has_tag(item, UINT32_C(0x42002A),
                                     KMIPKIT_TTLV_ITEM_TYPE_INTEGER, &value)) {
            int32_t actual = 0;
            length_found = kmipkit_ttlv_value_view_integer(value, &actual) == 0 && actual == 128;
        }
        if (value) kmipkit_ttlv_value_view_release(value);
        kmipkit_ttlv_item_view_release(item);
    }
    if (view) kmipkit_ttlv_structure_view_release(view);
    return valid && algorithm_found && length_found;
}

static bool attributes_contain_value(kmipkit_ttlv_structure_t *attributes,
                                     uint32_t expected_tag,
                                     uint8_t expected_type,
                                     uint32_t expected_enumeration,
                                     int32_t expected_integer) {
    kmipkit_ttlv_structure_view_t *view = NULL;
    uint64_t count = 0;
    bool valid = attributes && kmipkit_ttlv_structure_view(attributes, &view) == 0 &&
        kmipkit_ttlv_structure_view_item_count(view, &count) == 0;
    bool expected_value_found = false;
    for (uint64_t index = 0; valid && index < count; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        if (kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0) {
            valid = false;
            break;
        }
        kmipkit_ttlv_value_view_t *value = NULL;
        if (item_view_has_tag(item, expected_tag, expected_type, &value)) {
            if (expected_type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION) {
                uint32_t actual = 0;
                expected_value_found = kmipkit_ttlv_value_view_enumeration(value, &actual) == 0 &&
                    actual == expected_enumeration;
            } else if (expected_type == KMIPKIT_TTLV_ITEM_TYPE_INTEGER) {
                int32_t actual = 0;
                expected_value_found = kmipkit_ttlv_value_view_integer(value, &actual) == 0 &&
                    actual == expected_integer;
            }
        }
        if (value) kmipkit_ttlv_value_view_release(value);
        kmipkit_ttlv_item_view_release(item);
    }
    if (view) kmipkit_ttlv_structure_view_release(view);
    return valid && expected_value_found;
}

static bool attributes_contain_typed_tag(kmipkit_ttlv_structure_t *attributes,
                                         uint32_t expected_tag,
                                         uint8_t expected_type) {
    kmipkit_ttlv_structure_view_t *view = NULL;
    uint64_t count = 0;
    bool valid = attributes && kmipkit_ttlv_structure_view(attributes, &view) == 0 &&
        kmipkit_ttlv_structure_view_item_count(view, &count) == 0;
    bool found = false;
    for (uint64_t index = 0; valid && index < count; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        kmipkit_ttlv_value_view_t *value = NULL;
        if (kmipkit_ttlv_structure_view_item_at(view, index, &item) != 0) {
            valid = false;
            break;
        }
        found = item_view_has_tag(item, expected_tag, expected_type, &value);
        if (value) kmipkit_ttlv_value_view_release(value);
        kmipkit_ttlv_item_view_release(item);
        if (found) break;
    }
    if (view) kmipkit_ttlv_structure_view_release(view);
    return valid && found;
}

static bool attributes_contain_field_list(kmipkit_ttlv_structure_t *attributes,
                                          const char *field_list) {
    if (!field_list || field_list[0] == '\0' ||
        strlen(field_list) >= KMIP_MAX_NAME_LEN) return false;
    char remaining[KMIP_MAX_NAME_LEN];
    memcpy(remaining, field_list, strlen(field_list) + 1);
    size_t field_count = 0;
    char *field = remaining;
    while (field && *field) {
        char *next = strchr(field, ',');
        if (next) *next++ = '\0';
        char *separator = strchr(field, ':');
        if (!separator || separator == field || separator[1] == '\0') return false;
        *separator++ = '\0';
        uint32_t tag = 0;
        uint8_t item_type = 0;
        if (!parse_attribute_reference_tag(field, &tag) ||
            !parse_attribute_response_type(separator, &item_type) ||
            !attributes_contain_typed_tag(attributes, tag, item_type)) return false;
        ++field_count;
        field = next;
    }
    return field_count > 0;
}

static bool text_value_matches(kmipkit_ttlv_value_view_t *value,
                               const uint8_t *expected,
                               uint64_t expected_length) {
    uint8_t item_type = 0;
    uint64_t actual_length = 0;
    if (!value || (!expected && expected_length != 0) ||
        kmipkit_ttlv_value_view_type(value, &item_type) != 0 ||
        item_type != KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING ||
        kmipkit_ttlv_value_view_byte_length(value, &actual_length) != 0 ||
        actual_length != expected_length) return false;
    for (uint64_t index = 0; index < actual_length; ++index) {
        uint8_t actual_byte = 0;
        if (kmipkit_ttlv_value_view_byte_at(value, index, &actual_byte) != 0 ||
            actual_byte != expected[index]) return false;
    }
    return true;
}

static bool structure_view_has_text(kmipkit_ttlv_structure_view_t *structure,
                                    uint32_t tag,
                                    const uint8_t *expected,
                                    uint64_t expected_length) {
    uint64_t count = 0;
    if (!structure || kmipkit_ttlv_structure_view_item_count(structure, &count) != 0) return false;
    for (uint64_t index = 0; index < count; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        kmipkit_ttlv_value_view_t *value = NULL;
        if (kmipkit_ttlv_structure_view_item_at(structure, index, &item) != 0) return false;
        bool matches = item_view_has_tag(item, tag, KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING, &value) &&
            text_value_matches(value, expected, expected_length);
        if (value) kmipkit_ttlv_value_view_release(value);
        kmipkit_ttlv_item_view_release(item);
        if (matches) return true;
    }
    return false;
}

static bool attribute_reference_name_matches(kmipkit_ttlv_item_view_t *item,
                                             const kmipkit_attribute_reference_v1_t *reference) {
    kmipkit_ttlv_value_view_t *value = NULL;
    kmipkit_ttlv_structure_view_t *structure = NULL;
    bool matches = reference && reference->kind == KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME &&
        reference->vendor_identification_data && reference->attribute_name_data &&
        item_view_has_tag(item, UINT32_C(0x42013B), KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE, &value) &&
        kmipkit_ttlv_value_view_structure(value, &structure) == 0 &&
        structure_view_has_text(structure, UINT32_C(0x42009D),
                                reference->vendor_identification_data,
                                reference->vendor_identification_length) &&
        structure_view_has_text(structure, UINT32_C(0x42000A),
                                reference->attribute_name_data,
                                reference->attribute_name_length);
    if (structure) kmipkit_ttlv_structure_view_release(structure);
    if (value) kmipkit_ttlv_value_view_release(value);
    return matches;
}

static bool same_text_identifier(const kmipkit_unique_identifier_v1_t *left,
                                 const kmipkit_unique_identifier_v1_t *right) {
    return left->kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        right->kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
        left->text_length == right->text_length &&
        left->text_data != NULL && right->text_data != NULL &&
        memcmp(left->text_data, right->text_data, (size_t)left->text_length) == 0;
}

static int32_t set_batch_item_id(kmipkit_client_batch_item_t **item, const char *id) {
    kmipkit_client_batch_item_t *identified_item = NULL;
    int32_t status = kmipkit_client_batch_item_with_unique_batch_item_id(
        *item, (const uint8_t *)id, (uint64_t)strlen(id), &identified_item);
    if (status == 0) *item = identified_item;
    return status;
}

static bool request_item_is_valid(kmipkit_client_batch_item_t *item,
                                  char *diagnostic,
                                  size_t diagnostic_capacity) {
    kmipkit_validation_report_t *report = NULL;
    uint8_t is_valid = 0;
    int32_t status = kmipkit_client_batch_item_validation_report(item, &report);
    if (status == 0) status = kmipkit_validation_report_is_valid(report, &is_valid);
    if (status != 0) {
        snprintf(diagnostic, diagnostic_capacity,
                 "KMIPKit request validation failed with status %d", status);
        if (report) kmipkit_validation_report_release(report);
        return false;
    }
    if (is_valid) {
        kmipkit_validation_report_release(report);
        return true;
    }
    uint64_t issue_count = 0;
    if (kmipkit_validation_report_issue_count(report, &issue_count) == 0 && issue_count > 0) {
        kmipkit_validation_issue_view_v1_t issue = {
            .struct_size = (uint32_t)sizeof(kmipkit_validation_issue_view_v1_t),
            .abi_version = 1
        };
        if (kmipkit_validation_report_issue_at_v1(report, 0, &issue) == 0 &&
            issue.code.data && issue.code.length <= 120 &&
            issue.path.data && issue.path.length <= 160) {
            snprintf(diagnostic, diagnostic_capacity,
                     "KMIPKit rejected request schema code=%.*s path=%.*s",
                     (int)issue.code.length, (const char *)issue.code.data,
                     (int)issue.path.length, (const char *)issue.path.data);
        } else {
            snprintf(diagnostic, diagnostic_capacity,
                     "KMIPKit rejected request schema with %llu finding(s)",
                     (unsigned long long)issue_count);
        }
    } else {
        snprintf(diagnostic, diagnostic_capacity,
                 "KMIPKit rejected request schema without a readable finding");
    }
    kmipkit_validation_report_release(report);
    return false;
}

static bool response_ttlv_matches_field_count(
    kmipkit_client_batch_item_response_t *response_item,
    client_response_ttlv_accessor_t accessor,
    uint64_t expected_field_count,
    int32_t *out_status) {
    uint8_t present = 0;
    kmipkit_ttlv_structure_t *structure = NULL;
    kmipkit_ttlv_structure_view_t *view = NULL;
    uint64_t field_count = 0;
    int32_t status = accessor(response_item, &present, &structure);
    if (status == 0 && present && structure) {
        status = kmipkit_ttlv_structure_view(structure, &view);
    }
    if (status == 0 && present && view) {
        status = kmipkit_ttlv_structure_view_item_count(view, &field_count);
    }
    bool matches = status == 0 && present && structure && view &&
        field_count == expected_field_count;
    if (view) kmipkit_ttlv_structure_view_release(view);
    if (structure) kmipkit_ttlv_structure_release(structure);
    *out_status = status;
    return matches;
}

static bool get_attribute_list_response_matches_key(
    kmipkit_client_batch_item_response_t *response_item,
    const kmipkit_unique_identifier_v1_t *created_identifier,
    int32_t *out_status) {
    uint8_t present = 0;
    kmipkit_unique_identifier_v1_t response_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    uint64_t reference_count = 0;
    int32_t status = kmipkit_client_batch_item_response_get_attribute_list(
        response_item, &present, &response_identifier, NULL, 0, &reference_count);
    bool valid = status == 0 && present && reference_count > 0 &&
        reference_count <= UINT64_C(4096) &&
        same_text_identifier(created_identifier, &response_identifier);
    kmipkit_attribute_reference_v1_t *references = NULL;
    if (valid) {
        references = calloc((size_t)reference_count, sizeof(*references));
        valid = references != NULL;
        if (!valid) status = KMIPKIT_ERROR_RESOURCE_LIMIT;
    }
    for (uint64_t index = 0; valid && index < reference_count; ++index) {
        references[index].struct_size = (uint32_t)sizeof(kmipkit_attribute_reference_v1_t);
        references[index].abi_version = 1;
    }
    uint64_t copied_reference_count = 0;
    if (valid) {
        status = kmipkit_client_batch_item_response_get_attribute_list(
            response_item, &present, &response_identifier, references,
            reference_count, &copied_reference_count);
        valid = status == 0 && present && copied_reference_count == reference_count &&
            same_text_identifier(created_identifier, &response_identifier);
    }
    bool algorithm_found = false;
    bool length_found = false;
    for (uint64_t index = 0; valid && index < copied_reference_count; ++index) {
        if (references[index].kind == KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG &&
            references[index].tag_value == UINT32_C(0x420028)) algorithm_found = true;
        if (references[index].kind == KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG &&
            references[index].tag_value == UINT32_C(0x42002A)) length_found = true;
    }
    free(references);
    if (valid) valid = response_ttlv_matches_field_count(
        response_item, kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv,
        copied_reference_count + 1, &status);
    *out_status = status;
    return valid && algorithm_found && length_found;
}

static bool destroy_response_matches_key(
    kmipkit_client_batch_item_response_t *response_item,
    const kmipkit_unique_identifier_v1_t *created_identifier,
    int32_t *out_status) {
    uint8_t present = 0;
    kmipkit_unique_identifier_v1_t destroyed_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    *out_status = kmipkit_client_batch_item_response_destroy(
        response_item, &present, &destroyed_identifier);
    return *out_status == 0 && present &&
        same_text_identifier(created_identifier, &destroyed_identifier);
}

static bool structure_view_find_value(kmipkit_ttlv_structure_view_t *structure,
                                      uint32_t expected_tag,
                                      uint8_t expected_type,
                                      kmipkit_ttlv_value_view_t **out_value) {
    uint64_t count = 0;
    if (kmipkit_ttlv_structure_view_item_count(structure, &count) != 0) return false;
    for (uint64_t index = 0; index < count; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        if (kmipkit_ttlv_structure_view_item_at(structure, index, &item) != 0) return false;
        bool found = item_view_has_tag(item, expected_tag, expected_type, out_value);
        kmipkit_ttlv_item_view_release(item);
        if (found) return true;
    }
    return false;
}

static bool get_response_matches_created_key(
    kmipkit_client_batch_item_response_t *response_item,
    const kmipkit_unique_identifier_v1_t *created_identifier,
    int32_t *out_status) {
    uint8_t present = 0;
    uint8_t payload_present = 0;
    uint32_t object_type = 0;
    kmipkit_unique_identifier_v1_t response_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    kmipkit_ttlv_structure_t *any_object = NULL;
    int32_t status = kmipkit_client_batch_item_response_get(
        response_item, &present, &payload_present, &object_type,
        &response_identifier, &any_object);
    bool valid = status == 0 && present && payload_present &&
        object_type == KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY &&
        same_text_identifier(created_identifier, &response_identifier) && any_object;
    kmipkit_ttlv_structure_view_t *any_object_view = NULL;
    kmipkit_ttlv_value_view_t *symmetric_key_value = NULL;
    kmipkit_ttlv_structure_view_t *symmetric_key = NULL;
    kmipkit_ttlv_value_view_t *key_block_value = NULL;
    kmipkit_ttlv_structure_view_t *key_block = NULL;
    kmipkit_ttlv_value_view_t *key_value_value = NULL;
    kmipkit_ttlv_structure_view_t *key_value = NULL;
    kmipkit_ttlv_value_view_t *algorithm_value = NULL;
    kmipkit_ttlv_value_view_t *key_material = NULL;
    if (valid) valid = kmipkit_ttlv_structure_view(any_object, &any_object_view) == 0 &&
        structure_view_find_value(any_object_view, UINT32_C(0x42008F),
                                  KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE, &symmetric_key_value) &&
        kmipkit_ttlv_value_view_structure(symmetric_key_value, &symmetric_key) == 0 &&
        structure_view_find_value(symmetric_key, UINT32_C(0x420040),
                                  KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE, &key_block_value) &&
        kmipkit_ttlv_value_view_structure(key_block_value, &key_block) == 0 &&
        structure_view_find_value(key_block, UINT32_C(0x420028),
                                  KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION, &algorithm_value);
    if (valid) {
        uint32_t algorithm = 0;
        valid = algorithm_value &&
            kmipkit_ttlv_value_view_enumeration(algorithm_value, &algorithm) == 0 &&
            algorithm == KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
    }
    if (algorithm_value) kmipkit_ttlv_value_view_release(algorithm_value);
    if (valid) valid = structure_view_find_value(key_block, UINT32_C(0x420045),
                                                KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE,
                                                &key_value_value) &&
        kmipkit_ttlv_value_view_structure(key_value_value, &key_value) == 0 &&
        structure_view_find_value(key_value, UINT32_C(0x420043),
                                  KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING, &key_material);
    if (valid) {
        uint64_t key_length = 0;
        valid = kmipkit_ttlv_value_view_byte_length(key_material, &key_length) == 0 &&
            key_length > 0;
    }
    if (key_material) kmipkit_ttlv_value_view_release(key_material);
    if (key_value) kmipkit_ttlv_structure_view_release(key_value);
    if (key_value_value) kmipkit_ttlv_value_view_release(key_value_value);
    if (key_block) kmipkit_ttlv_structure_view_release(key_block);
    if (key_block_value) kmipkit_ttlv_value_view_release(key_block_value);
    if (symmetric_key) kmipkit_ttlv_structure_view_release(symmetric_key);
    if (symmetric_key_value) kmipkit_ttlv_value_view_release(symmetric_key_value);
    if (any_object_view) kmipkit_ttlv_structure_view_release(any_object_view);
    if (any_object) kmipkit_ttlv_structure_release(any_object);
    *out_status = status;
    return valid;
}

static bool check_response_matches_created_key(
    kmipkit_client_batch_item_response_t *response_item,
    const kmipkit_unique_identifier_v1_t *created_identifier,
    int32_t *out_status) {
    uint8_t present = 0;
    uint8_t unique_identifier_present = 0;
    uint8_t usage_limits_count_present = 0;
    uint8_t cryptographic_usage_mask_present = 0;
    uint8_t lease_time_present = 0;
    int64_t usage_limits_count = 0;
    int32_t cryptographic_usage_mask = 0;
    uint32_t lease_time = 0;
    kmipkit_unique_identifier_v1_t response_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    int32_t status = kmipkit_client_batch_item_response_check(
        response_item, &present, &unique_identifier_present, &response_identifier,
        &usage_limits_count_present, &usage_limits_count,
        &cryptographic_usage_mask_present, &cryptographic_usage_mask,
        &lease_time_present, &lease_time);
    bool valid = status == 0 && present && unique_identifier_present &&
        same_text_identifier(created_identifier, &response_identifier) &&
        !usage_limits_count_present && !cryptographic_usage_mask_present &&
        !lease_time_present;
    if (valid) valid = response_ttlv_matches_field_count(
        response_item, kmipkit_client_batch_item_response_check_ttlv, 4, &status);
    *out_status = status;
    return valid;
}

static bool revoke_response_matches_created_key(
    kmipkit_client_batch_item_response_t *response_item,
    const kmipkit_unique_identifier_v1_t *created_identifier,
    int32_t *out_status) {
    uint8_t present = 0;
    kmipkit_unique_identifier_v1_t response_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    int32_t status = kmipkit_client_batch_item_response_revoke(
        response_item, &present, &response_identifier);
    bool valid = status == 0 && present &&
        same_text_identifier(created_identifier, &response_identifier);
    if (valid) valid = response_ttlv_matches_field_count(
        response_item, kmipkit_client_batch_item_response_revoke_ttlv, 4, &status);
    *out_status = status;
    return valid;
}

static bool activate_response_matches_created_key(
    kmipkit_client_batch_item_response_t *response_item,
    const kmipkit_unique_identifier_v1_t *created_identifier,
    int32_t *out_status) {
    uint8_t present = 0;
    kmipkit_unique_identifier_v1_t response_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    int32_t status = kmipkit_client_batch_item_response_activate(
        response_item, &present, &response_identifier);
    *out_status = status;
    return status == 0 && present &&
        same_text_identifier(created_identifier, &response_identifier);
}

static bool get_constraints_response_is_typed(
    kmipkit_client_batch_item_response_t *response_item,
    int32_t *out_status) {
    uint8_t present = 0;
    kmipkit_ttlv_structure_t *constraints = NULL;
    int32_t status = kmipkit_client_batch_item_response_get_constraints_ttlv(
        response_item, &present, &constraints);
    bool valid = status == 0 && present && constraints;
    if (valid) {
        kmipkit_ttlv_structure_view_t *view = NULL;
        uint64_t field_count = 0;
        valid = kmipkit_ttlv_structure_view(constraints, &view) == 0 && view &&
            kmipkit_ttlv_structure_view_item_count(view, &field_count) == 0;
        if (view) kmipkit_ttlv_structure_view_release(view);
    }
    if (constraints) kmipkit_ttlv_structure_release(constraints);
    *out_status = status;
    return valid;
}

static bool get_usage_allocation_response_matches_identifier(
    kmipkit_client_batch_item_response_t *response_item,
    const kmipkit_unique_identifier_v1_t *created_identifier,
    int32_t *out_status) {
    uint8_t present = 0;
    kmipkit_unique_identifier_v1_t allocation_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    int32_t status = kmipkit_client_batch_item_response_get_usage_allocation(
        response_item, &present, &allocation_identifier);
    *out_status = status;
    return status == 0 && present &&
        same_text_identifier(created_identifier, &allocation_identifier);
}

static bool get_usage_allocation_response_ttlv_matches_identifier(
    kmipkit_client_batch_item_response_t *response_item,
    const kmipkit_unique_identifier_v1_t *created_identifier,
    int32_t *out_status) {
    return response_ttlv_matches_identifier(
        response_item, kmipkit_client_batch_item_response_get_usage_allocation_ttlv,
        created_identifier, out_status);
}

static int32_t create_get_usage_allocation_variant_item(
    bool include_identifier,
    kmipkit_client_batch_item_t **out_item) {
    static const uint8_t fixture_identifier[] = "fixture-key-1";
    const kmipkit_unique_identifier_v1_t identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1,
        .kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        .text_data = fixture_identifier,
        .text_length = sizeof(fixture_identifier) - 1
    };
    return kmipkit_client_batch_item_get_usage_allocation(
        include_identifier ? &identifier : NULL, 1, out_item);
}

static bool query_response_ttlv_matches_field_count(
    kmipkit_client_batch_item_response_t *response_item,
    uint64_t expected_field_count,
    int32_t *out_status) {
    return response_ttlv_matches_field_count(
        response_item, kmipkit_client_batch_item_response_query_ttlv,
        expected_field_count, out_status);
}

static int32_t create_revocation_item(
    const kmipkit_unique_identifier_v1_t *unique_identifier,
    kmipkit_client_batch_item_t **out_item) {
    kmipkit_revocation_reason_t *reason = NULL;
    kmipkit_revoke_request_t *request = NULL;
    int32_t status = kmipkit_revocation_reason_create(
        KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_CESSATION_OF_OPERATION,
        0, NULL, 0, &reason);
    if (status == 0) {
        status = kmipkit_revoke_request_create(
            unique_identifier, reason, 0, 0, &request);
        reason = NULL;
    }
    if (status == 0) {
        status = kmipkit_client_batch_item_revoke(request, out_item);
        request = NULL;
    }
    if (request) kmipkit_revoke_request_release(request);
    if (reason) kmipkit_revocation_reason_release(reason);
    return status;
}

static bool create_authentication(kmip_live_client_t *client,
                                  kmipkit_authentication_t **out_authentication);

static bool cleanup_placeholder_chain_object(
    kmip_live_client_t *client,
    const kmipkit_unique_identifier_v1_t *unique_identifier,
    bool placeholder_batch_ids,
    int32_t *out_status) {
    kmipkit_client_batch_t *batch = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_client_batch_item_t *revoke_item = NULL;
    kmipkit_client_batch_item_t *destroy_item = NULL;
    kmipkit_client_batch_item_response_t *revoke_response = NULL;
    kmipkit_client_batch_item_response_t *destroy_response = NULL;
    kmipkit_client_batch_item_metadata_v2_t revoke_metadata = {
        .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
        .abi_version = 2
    };
    kmipkit_client_batch_item_metadata_v2_t destroy_metadata = {
        .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
        .abi_version = 2
    };
    uint64_t response_count = 0;
    int32_t status = kmipkit_client_batch_create(&batch);
    if (status == 0 && client->username_env[0] != '\0') {
        kmipkit_authentication_t *authentication = NULL;
        if (!create_authentication(client, &authentication)) {
            status = KMIPKIT_ERROR_INVALID_INPUT;
        } else {
            status = kmipkit_client_batch_set_authentication(batch, authentication);
            if (status != 0) kmipkit_authentication_release(authentication);
        }
    }
    if (status == 0) status = create_revocation_item(unique_identifier, &revoke_item);
    if (status == 0) status = kmipkit_client_batch_item_destroy(unique_identifier, &destroy_item);
    if (status == 0) status = set_batch_item_id(&revoke_item,
        placeholder_batch_ids ? "placeholder-revoke-cleanup" : "interop-revoke");
    if (status == 0) status = set_batch_item_id(&destroy_item,
        placeholder_batch_ids ? "placeholder-destroy-cleanup" : "interop-destroy");
    if (status == 0) status = kmipkit_client_batch_set_order_option(batch, 1);
    if (status == 0) status = kmipkit_client_batch_add_error_continuation_value(
        batch, KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_CONTINUE);
    if (status == 0) status = kmipkit_client_batch_add_item(batch, &revoke_item);
    if (status == 0) status = kmipkit_client_batch_add_item(batch, &destroy_item);
    if (status == 0) status = kmipkit_client_execute(client->client, &batch, client->limits, &response);
    if (status == 0) status = kmipkit_client_batch_response_item_count(response, &response_count);
    if (status == 0 && response_count == 2) {
        status = kmipkit_client_batch_response_item_at(response, 0, &revoke_response);
    } else if (status == 0) {
        status = KMIPKIT_ERROR_INVALID_INPUT;
    }
    if (status == 0) status = kmipkit_client_batch_response_item_at(response, 1, &destroy_response);
    if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
        revoke_response, &revoke_metadata);
    if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
        destroy_response, &destroy_metadata);
    bool valid = status == 0 &&
        revoke_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS &&
        destroy_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS &&
        revoke_response_matches_created_key(revoke_response, unique_identifier, &status) &&
        destroy_response_matches_key(destroy_response, unique_identifier, &status);
    if (revoke_response) kmipkit_client_batch_item_response_release(revoke_response);
    if (destroy_response) kmipkit_client_batch_item_response_release(destroy_response);
    if (response) kmipkit_client_batch_response_release(response);
    if (batch) kmipkit_client_batch_release(batch);
    if (revoke_item) kmipkit_client_batch_item_release(revoke_item);
    if (destroy_item) kmipkit_client_batch_item_release(destroy_item);
    *out_status = status;
    return valid;
}

static bool create_authentication(kmip_live_client_t *client,
                                  kmipkit_authentication_t **out_authentication) {
    const char *username = getenv(client->username_env);
    const char *password = getenv(client->password_env);
    if (!username || username[0] == '\0' || !password) return false;

    kmipkit_ttlv_structure_t *credential_value = NULL;
    kmipkit_ttlv_structure_t *credential = NULL;
    kmipkit_ttlv_structure_t *authentication = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    int32_t status = kmipkit_ttlv_structure_create(&credential_value);
    bool valid = status == 0 &&
        append_secret_text(client, &credential_value, UINT32_C(0x420099), username) &&
        append_secret_text(client, &credential_value, UINT32_C(0x4200A1), password) &&
        kmipkit_ttlv_structure_create(&credential) == 0 &&
        kmipkit_ttlv_value_enumeration(
            KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_USERNAME_AND_PASSWORD, &value) == 0 &&
        append_ttlv_value(client, &credential, UINT32_C(0x420024), &value) &&
        structure_into_value(client, &credential_value, &value) &&
        append_ttlv_value(client, &credential, UINT32_C(0x420025), &value);
    if (valid) {
        valid = kmipkit_ttlv_structure_create(&authentication) == 0 &&
            structure_into_value(client, &credential, &value) &&
            append_ttlv_value(client, &authentication, UINT32_C(0x420023), &value);
    }
    if (valid) {
        status = kmipkit_authentication_from_ttlv(authentication, out_authentication);
        authentication = NULL;
        valid = status == 0;
    }
    if (value) kmipkit_ttlv_value_release(value);
    if (credential_value) kmipkit_ttlv_structure_release(credential_value);
    if (credential) kmipkit_ttlv_structure_release(credential);
    if (authentication) kmipkit_ttlv_structure_release(authentication);
    return valid;
}

static bool has_tls_material(const kmip_connection_config_t *configuration) {
    return configuration->ca_cert_path[0] != '\0' &&
           configuration->client_cert_path[0] != '\0' &&
           configuration->client_key_path[0] != '\0';
}

static bool create_client(const kmip_connection_config_t *configuration,
                          kmip_live_client_t *client, char *error,
                          size_t error_capacity) {
    kmipkit_extension_registry_limits_v1_t registry_limits = {
        .struct_size = (uint32_t)sizeof(kmipkit_extension_registry_limits_v1_t),
        .abi_version = 1
    };
    kmipkit_client_extension_registry_t *registry = NULL;
    kmipkit_client_configuration_t *default_configuration = NULL;
    kmipkit_client_configuration_t *version_configuration = NULL;
    kmipkit_client_configuration_t *encoding_configuration = NULL;
    kmipkit_transport_config_t *transport_configuration = NULL;
    kmipkit_transport_input_v1_t certificate = {0};
    kmipkit_transport_input_v1_t trust_certificate = {0};
    kmipkit_transport_config_v1_t transport = {0};
    int32_t status = kmipkit_extension_registry_limits_v1_default_values(&registry_limits);
    if (status != 0) {
        set_error(error, error_capacity, "could not initialize extension registry limits", status);
        goto fail;
    }
    status = kmipkit_client_extension_registry_create_v1(NULL, 0, &registry_limits, &registry);
    if (status != 0) {
        set_error(error, error_capacity, "could not create the standard-only extension registry", status);
        goto fail;
    }
    status = kmipkit_client_configuration_create(registry, &default_configuration);
    registry = NULL;
    if (status != 0) {
        set_error(error, error_capacity, "could not create client configuration", status);
        goto fail;
    }
    status = kmipkit_client_configuration_with_protocol_version(
        default_configuration, client->protocol_major, client->protocol_minor,
        &version_configuration);
    if (status != 0) {
        set_error(error, error_capacity, "could not set the requested KMIP version", status);
        goto fail;
    }
    status = kmipkit_client_configuration_with_wire_encoding(
        version_configuration, client->wire_encoding, &encoding_configuration);
    if (status != 0) {
        set_error(error, error_capacity, "could not set the requested wire encoding", status);
        goto fail;
    }

    certificate.source_kind = KMIPKIT_TRANSPORT_SOURCE_FILE;
    certificate.encoding = KMIPKIT_TRANSPORT_ENCODING_PEM;
    certificate.data = borrowed_span(configuration->client_cert_path);
    trust_certificate.source_kind = KMIPKIT_TRANSPORT_SOURCE_FILE;
    trust_certificate.encoding = KMIPKIT_TRANSPORT_ENCODING_PEM;
    trust_certificate.data = borrowed_span(configuration->ca_cert_path);
    transport.struct_size = (uint32_t)sizeof(transport);
    transport.abi_version = 1;
    transport.trust_kind = KMIPKIT_TRANSPORT_TRUST_CERTIFICATES;
    transport.client_certificates = &certificate;
    transport.client_certificate_count = 1;
    transport.client_private_key.source_kind = KMIPKIT_TRANSPORT_SOURCE_FILE;
    transport.client_private_key.encoding = KMIPKIT_TRANSPORT_ENCODING_PEM;
    transport.client_private_key.data = borrowed_span(configuration->client_key_path);
    transport.trust_certificates = &trust_certificate;
    transport.trust_certificate_count = 1;
    transport.connect_timeout = bounded_timeout(10);
    transport.write_timeout = bounded_timeout(30);
    transport.read_timeout = bounded_timeout(30);
    transport.total_timeout = bounded_timeout(60);
    transport.max_request_bytes = KMIP_MAX_REQUEST_BYTES;
    if (configuration->server_name_override[0] != '\0') {
        transport.flags |= KMIPKIT_TRANSPORT_CONFIG_FLAG_TLS_SERVER_NAME;
        transport.tls_server_name = borrowed_span(configuration->server_name_override);
    }
    if (strcmp(configuration->transport, "raw_tls") == 0) {
        transport.endpoint_kind = KMIPKIT_TRANSPORT_ENDPOINT_RAW_TLS;
        transport.endpoint = borrowed_span(configuration->host);
        transport.endpoint_port = configuration->port;
    } else if (strcmp(configuration->transport, "https") == 0) {
        transport.endpoint_kind = KMIPKIT_TRANSPORT_ENDPOINT_HTTPS;
        transport.endpoint = borrowed_span(configuration->https_url);
        transport.endpoint_port = 0;
    } else {
        set_error(error, error_capacity, "unsupported configured transport", 0);
        goto fail;
    }
    status = kmipkit_transport_config_create_v1(&transport, &transport_configuration);
    if (status != 0) {
        set_error(error, error_capacity, "could not construct the configured mutual-TLS transport", status);
        goto fail;
    }
    status = kmipkit_client_create(encoding_configuration, &transport_configuration, &client->client);
    if (status != 0) {
        set_error(error, error_capacity, "could not construct the KMIPKit client", status);
        goto fail;
    }
    kmipkit_client_configuration_release(encoding_configuration);
    kmipkit_client_configuration_release(version_configuration);
    kmipkit_client_configuration_release(default_configuration);
    return true;

fail:
    if (client->limits) kmipkit_codec_limits_release(client->limits);
    if (client->client) kmipkit_client_release(client->client);
    if (transport_configuration) kmipkit_transport_config_release(transport_configuration);
    if (encoding_configuration) kmipkit_client_configuration_release(encoding_configuration);
    if (version_configuration) kmipkit_client_configuration_release(version_configuration);
    if (default_configuration) kmipkit_client_configuration_release(default_configuration);
    if (registry) kmipkit_client_extension_registry_release(registry);
    return false;
}

bool kmip_live_client_open(const kmip_connection_config_t *configuration,
                           kmip_live_client_t *out_client, char *error,
                           size_t error_capacity) {
    if (!configuration || !out_client) {
        set_error(error, error_capacity, "missing live client configuration", 0);
        return false;
    }
    memset(out_client, 0, sizeof(*out_client));
    if (!parse_version(configuration->protocol_version, &out_client->protocol_major,
                       &out_client->protocol_minor) ||
        !parse_wire_encoding(configuration->encoding, &out_client->wire_encoding)) {
        set_error(error, error_capacity, "unsupported KMIP version or encoding", 0);
        return false;
    }
    if (strcmp(configuration->api_level, "typed-protocol") == 0 &&
        !has_tls_material(configuration)) {
        set_error(error, error_capacity, "CA certificate, client certificate and client key are required", 0);
        return false;
    }
    if (strcmp(configuration->api_level, "typed-protocol") == 0 &&
        (configuration->username_env[0] != '\0' || configuration->password_env[0] != '\0')) {
        if (configuration->username_env[0] == '\0' || configuration->password_env[0] == '\0' ||
            !getenv(configuration->username_env) || getenv(configuration->username_env)[0] == '\0' ||
            !getenv(configuration->password_env)) {
            set_error(error, error_capacity,
                      "configured Username/Password environment variables are unavailable", 0);
            return false;
        }
        snprintf(out_client->username_env, sizeof(out_client->username_env), "%s",
                 configuration->username_env);
        snprintf(out_client->password_env, sizeof(out_client->password_env), "%s",
                 configuration->password_env);
    }
    snprintf(out_client->api_level, sizeof(out_client->api_level), "%s", configuration->api_level);
    out_client->configured_version_propagation_available =
        KMIPKIT_C_VERSION_PROPAGATION_AVAILABLE != 0;
    if (strcmp(out_client->api_level, "typed-protocol") == 0 &&
        out_client->protocol_minor == 0 &&
        !out_client->configured_version_propagation_available) {
        set_error(error, error_capacity,
                  "pinned KMIPKit C artifact does not preserve the configured KMIP 2.0 version", 0);
        return false;
    }
    int32_t limits_status = kmipkit_codec_limits_defaults(&out_client->limits);
    if (limits_status != 0) {
        set_error(error, error_capacity, "could not create codec limits", limits_status);
        kmip_live_client_close(out_client);
        return false;
    }
    if (strcmp(out_client->api_level, "typed-protocol") == 0 &&
        !create_client(configuration, out_client, error, error_capacity)) {
        kmip_live_client_close(out_client);
        return false;
    }
    return true;
}

void kmip_live_client_close(kmip_live_client_t *client) {
    if (!client) return;
    if (client->limits) kmipkit_codec_limits_release(client->limits);
    if (client->client) kmipkit_client_release(client->client);
    memset(client, 0, sizeof(*client));
}

static kmip_execution_outcome_t outcome_not_run(const char *reason) {
    kmip_execution_outcome_t outcome = {0};
    if (reason) snprintf(outcome.result_reason, sizeof(outcome.result_reason), "%s", reason);
    return outcome;
}

static kmip_execution_outcome_t outcome_failed(const char *reason) {
    kmip_execution_outcome_t outcome = {0};
    if (reason) snprintf(outcome.result_reason, sizeof(outcome.result_reason), "%s", reason);
    return outcome;
}

typedef const char *(*kmip_enumeration_name_fn)(uint32_t);
typedef uint32_t (*kmip_bitmask_split_fn)(uint32_t);

typedef struct {
    const char *name;
    uint32_t tag;
    kmip_enumeration_name_fn name_for_value;
} kmip_enum_probe_t;

static const kmip_enum_probe_t KMIP_ENUM_PROBES[] = {
    {"Adjustment Type", UINT32_C(0x420158), KMIPKIT_ENUMERATION_ADJUSTMENT_TYPE_name},
    {"Alternative Name Type", UINT32_C(0x4200C1), KMIPKIT_ENUMERATION_ALTERNATIVE_NAME_TYPE_name},
    {"Asynchronous Indicator", UINT32_C(0x420007), KMIPKIT_ENUMERATION_ASYNCHRONOUS_INDICATOR_name},
    {"Attestation Type", UINT32_C(0x4200C7), KMIPKIT_ENUMERATION_ATTESTATION_TYPE_name},
    {"Batch Error Continuation Option", UINT32_C(0x42000E), KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_name},
    {"Block Cipher Mode", UINT32_C(0x420011), KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_name},
    {"Cancellation Result", UINT32_C(0x420012), KMIPKIT_ENUMERATION_CANCELLATION_RESULT_name},
    {"Certificate Request Type", UINT32_C(0x420019), KMIPKIT_ENUMERATION_CERTIFICATE_REQUEST_TYPE_name},
    {"Certificate Type", UINT32_C(0x42001D), KMIPKIT_ENUMERATION_CERTIFICATE_TYPE_name},
    {"Client Registration Method", UINT32_C(0x4200F6), KMIPKIT_ENUMERATION_CLIENT_REGISTRATION_METHOD_name},
    {"Credential Type", UINT32_C(0x420024), KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_name},
    {"Cryptographic Algorithm", UINT32_C(0x420028), KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_name},
    {"DRBG Algorithm", UINT32_C(0x4200DB), KMIPKIT_ENUMERATION_DRBG_ALGORITHM_name},
    {"Data", UINT32_C(0x4200C2), KMIPKIT_ENUMERATION_DATA_name},
    {"Derivation Method", UINT32_C(0x420031), KMIPKIT_ENUMERATION_DERIVATION_METHOD_name},
    {"Destroy Action", UINT32_C(0x4200F3), KMIPKIT_ENUMERATION_DESTROY_ACTION_name},
    {"Digital Signature Algorithm", UINT32_C(0x4200AE), KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_name},
    {"Encoding Option", UINT32_C(0x4200A3), KMIPKIT_ENUMERATION_ENCODING_OPTION_name},
    {"Endpoint Role", UINT32_C(0x420151), KMIPKIT_ENUMERATION_ENDPOINT_ROLE_name},
    {"FIPS186 Variation", UINT32_C(0x4200DC), KMIPKIT_ENUMERATION_FIPS186_VARIATION_name},
    {"Hashing Algorithm", UINT32_C(0x420038), KMIPKIT_ENUMERATION_HASHING_ALGORITHM_name},
    {"Interop Function", UINT32_C(0x420160), KMIPKIT_ENUMERATION_INTEROP_FUNCTION_name},
    {"Item Type", 0, NULL},
    {"Key Compression Type", UINT32_C(0x420041), KMIPKIT_ENUMERATION_KEY_COMPRESSION_TYPE_name},
    {"Key Format Type", UINT32_C(0x420042), KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_name},
    {"Key Role Type", UINT32_C(0x420083), KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_name},
    {"Key Value Location Type", UINT32_C(0x4200BA), KMIPKIT_ENUMERATION_KEY_VALUE_LOCATION_TYPE_name},
    {"Key Wrap Type", UINT32_C(0x4200F8), KMIPKIT_ENUMERATION_KEY_WRAP_TYPE_name},
    {"Link Type", UINT32_C(0x42004B), KMIPKIT_ENUMERATION_LINK_TYPE_name},
    {"Mask Generator", UINT32_C(0x420101), KMIPKIT_ENUMERATION_MASK_GENERATOR_name},
    {"NIST Key Type", UINT32_C(0x42013A), KMIPKIT_ENUMERATION_NIST_KEY_TYPE_name},
    {"Name Type", UINT32_C(0x420054), KMIPKIT_ENUMERATION_NAME_TYPE_name},
    {"Object Group Member", UINT32_C(0x4200AC), KMIPKIT_ENUMERATION_OBJECT_GROUP_MEMBER_name},
    {"Object Type", UINT32_C(0x420057), KMIPKIT_ENUMERATION_OBJECT_TYPE_name},
    {"Opaque Data Type", UINT32_C(0x420059), KMIPKIT_ENUMERATION_OPAQUE_DATA_TYPE_name},
    {"Operation", UINT32_C(0x42005C), KMIPKIT_ENUMERATION_OPERATION_name},
    {"PKCS#11 Function", UINT32_C(0x42015A), KMIPKIT_ENUMERATION_PKCS_11_FUNCTION_name},
    {"PKCS#11 Return Code", UINT32_C(0x42015D), KMIPKIT_ENUMERATION_PKCS_11_RETURN_CODE_name},
    {"Padding Method", UINT32_C(0x42005F), KMIPKIT_ENUMERATION_PADDING_METHOD_name},
    {"Processing Stage", UINT32_C(0x420175), KMIPKIT_ENUMERATION_PROCESSING_STAGE_name},
    {"Profile Name", UINT32_C(0x4200EC), KMIPKIT_ENUMERATION_PROFILE_NAME_name},
    {"Protection Level", UINT32_C(0x420145), KMIPKIT_ENUMERATION_PROTECTION_LEVEL_name},
    {"Put Function", UINT32_C(0x420070), KMIPKIT_ENUMERATION_PUT_FUNCTION_name},
    {"Query Function", UINT32_C(0x420074), KMIPKIT_ENUMERATION_QUERY_FUNCTION_name},
    {"RNG Algorithm", UINT32_C(0x4200DA), KMIPKIT_ENUMERATION_RNG_ALGORITHM_name},
    {"RNG Mode", UINT32_C(0x4200F5), KMIPKIT_ENUMERATION_RNG_MODE_name},
    {"Recommended Curve", UINT32_C(0x420075), KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_name},
    {"Result Reason", UINT32_C(0x42007E), KMIPKIT_ENUMERATION_RESULT_REASON_name},
    {"Result Status", UINT32_C(0x42007F), KMIPKIT_ENUMERATION_RESULT_STATUS_name},
    {"Revocation Reason Code", UINT32_C(0x420082), KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_name},
    {"Rotate Name Type", UINT32_C(0x420171), KMIPKIT_ENUMERATION_ROTATE_NAME_TYPE_name},
    {"Secret Data Type", UINT32_C(0x420086), KMIPKIT_ENUMERATION_SECRET_DATA_TYPE_name},
    {"Shredding Algorithm", UINT32_C(0x4200F4), KMIPKIT_ENUMERATION_SHREDDING_ALGORITHM_name},
    {"Split Key Method", UINT32_C(0x42008A), KMIPKIT_ENUMERATION_SPLIT_KEY_METHOD_name},
    {"State", UINT32_C(0x42008D), KMIPKIT_ENUMERATION_STATE_name},
    {"Tag", UINT32_C(0x420138), KMIPKIT_ENUMERATION_TAG_name},
    {"Ticket Type", UINT32_C(0x42014A), KMIPKIT_ENUMERATION_TICKET_TYPE_name},
    {"Unique Identifier", UINT32_C(0x420094), KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_name},
    {"Unwrap Mode", UINT32_C(0x4200F2), KMIPKIT_ENUMERATION_UNWRAP_MODE_name},
    {"Usage Limits Unit", UINT32_C(0x420098), KMIPKIT_ENUMERATION_USAGE_LIMITS_UNIT_name},
    {"Validation Authority Type", UINT32_C(0x4200E0), KMIPKIT_ENUMERATION_VALIDATION_AUTHORITY_TYPE_name},
    {"Validation Type", UINT32_C(0x4200E5), KMIPKIT_ENUMERATION_VALIDATION_TYPE_name},
    {"Validity Indicator", UINT32_C(0x42009B), KMIPKIT_ENUMERATION_VALIDITY_INDICATOR_name},
    {"Wrapping Method", UINT32_C(0x42009E), KMIPKIT_ENUMERATION_WRAPPING_METHOD_name}
};

typedef struct {
    const char *name;
    uint32_t raw_value;
} kmip_enum_fallback_probe_t;

/* These standard value forms have no assigned name in the installed C helper. */
static const kmip_enum_fallback_probe_t KMIP_ENUM_FALLBACK_PROBES[] = {
    {"Opaque Data Type", UINT32_C(0x80000000)}, /* OASIS KMIP 2.1 §11.35 */
    {"PKCS#11 Function", UINT32_C(1)}, /* OASIS KMIP 2.1 §11.38: one-based offset */
    {"PKCS#11 Return Code", UINT32_C(0)}, /* OASIS KMIP 2.1 §11.39: CK_RV value */
    {"Tag", UINT32_C(0x420001)} /* OASIS KMIP 2.1 §11.56: assigned KMIP tag */
};

typedef struct {
    const char *name;
    uint32_t tag;
    uint32_t known_mask;
    kmip_bitmask_split_fn known_bits;
    kmip_bitmask_split_fn unknown_bits;
} kmip_bitmask_probe_t;

static const kmip_bitmask_probe_t KMIP_BITMASK_PROBES[] = {
    {"Cryptographic Usage Mask", UINT32_C(0x42002C), KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_KNOWN_MASK,
     KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_known_bits, KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_unknown_bits},
    {"Protection Storage Mask", UINT32_C(0x42015E), KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_KNOWN_MASK,
     KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_known_bits, KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_unknown_bits},
    {"Storage Status Mask", UINT32_C(0x42008E), KMIPKIT_BITMASK_STORAGE_STATUS_MASK_KNOWN_MASK,
     KMIPKIT_BITMASK_STORAGE_STATUS_MASK_known_bits, KMIPKIT_BITMASK_STORAGE_STATUS_MASK_unknown_bits}
};

static const kmip_enum_probe_t *find_enum_probe(const char *name) {
    for (size_t index = 0; index < sizeof(KMIP_ENUM_PROBES) / sizeof(KMIP_ENUM_PROBES[0]); ++index) {
        if (strcmp(name, KMIP_ENUM_PROBES[index].name) == 0) return &KMIP_ENUM_PROBES[index];
    }
    return NULL;
}

static const kmip_enum_fallback_probe_t *find_enum_fallback_probe(const char *name) {
    for (size_t index = 0; index < sizeof(KMIP_ENUM_FALLBACK_PROBES) /
                            sizeof(KMIP_ENUM_FALLBACK_PROBES[0]); ++index) {
        if (strcmp(name, KMIP_ENUM_FALLBACK_PROBES[index].name) == 0)
            return &KMIP_ENUM_FALLBACK_PROBES[index];
    }
    return NULL;
}

static const kmip_bitmask_probe_t *find_bitmask_probe(const char *name) {
    for (size_t index = 0; index < sizeof(KMIP_BITMASK_PROBES) / sizeof(KMIP_BITMASK_PROBES[0]); ++index) {
        if (strcmp(name, KMIP_BITMASK_PROBES[index].name) == 0) return &KMIP_BITMASK_PROBES[index];
    }
    return NULL;
}

static bool roundtrip_scalar_for_inspection(bool enumeration, uint32_t tag_code,
                                            int64_t raw_value) {
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_ttlv_structure_t *structure = NULL;
    kmipkit_ttlv_structure_t *updated_structure = NULL;
    kmipkit_ttlv_structure_view_t *structure_view = NULL;
    kmipkit_ttlv_item_view_t *item_view = NULL;
    kmipkit_ttlv_value_view_t *value_view = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    uint64_t item_count = 0;
    uint32_t actual_enumeration = 0;
    int32_t actual_integer = 0;
    uint8_t item_type = 0;
    bool valid = false;
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(tag_code, &raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag);
    if (status == 0) {
        status = enumeration
            ? kmipkit_ttlv_value_enumeration((uint32_t)raw_value, &value)
            : kmipkit_ttlv_value_integer((int32_t)raw_value, &value);
    }
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, &item);
    if (status == 0) {
        value = NULL;
        status = kmipkit_ttlv_structure_create(&structure);
    }
    if (status == 0)
        status = kmipkit_ttlv_structure_with_item(structure, item, limits, &updated_structure);
    if (status == 0) {
        structure = updated_structure;
        updated_structure = NULL;
        item = NULL;
        status = kmipkit_ttlv_structure_view(structure, &structure_view);
    }
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(structure_view, &item_count);
    if (status == 0 && item_count == 1)
        status = kmipkit_ttlv_structure_view_item_at(structure_view, 0, &item_view);
    if (status == 0) status = kmipkit_ttlv_item_view_type(item_view, &item_type);
    if (status == 0) status = kmipkit_ttlv_item_view_value(item_view, &value_view);
    if (status == 0 && enumeration) {
        status = kmipkit_ttlv_value_view_enumeration(value_view, &actual_enumeration);
        valid = status == 0 && item_type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION &&
                actual_enumeration == (uint32_t)raw_value;
    } else if (status == 0) {
        status = kmipkit_ttlv_value_view_integer(value_view, &actual_integer);
        valid = status == 0 && item_type == KMIPKIT_TTLV_ITEM_TYPE_INTEGER &&
                (int64_t)actual_integer == raw_value;
    }
    if (value_view) kmipkit_ttlv_value_view_release(value_view);
    if (item_view) kmipkit_ttlv_item_view_release(item_view);
    if (structure_view) kmipkit_ttlv_structure_view_release(structure_view);
    if (updated_structure) kmipkit_ttlv_structure_release(updated_structure);
    if (structure) kmipkit_ttlv_structure_release(structure);
    if (item) kmipkit_ttlv_item_release(item);
    if (value) kmipkit_ttlv_value_release(value);
    if (tag) kmipkit_tag_release(tag);
    if (raw_tag) kmipkit_raw_tag_release(raw_tag);
    if (limits) kmipkit_codec_limits_release(limits);
    return valid;
}

static bool roundtrip_ttlv_item_type(uint32_t expected_type) {
    static const uint8_t bigint_bytes[] = {UINT8_C(1)};
    static const uint8_t byte_string[] = {UINT8_C(0xA5)};
    static const uint8_t text_string[] = {'i', 't', 'e', 'm'};
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_ttlv_structure_t *nested_structure = NULL;
    kmipkit_ttlv_structure_t *structure = NULL;
    kmipkit_ttlv_structure_t *updated_structure = NULL;
    kmipkit_ttlv_structure_view_t *structure_view = NULL;
    kmipkit_ttlv_item_view_t *item_view = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    uint64_t item_count = 0;
    uint8_t actual_type = 0;
    int32_t status = kmipkit_codec_limits_defaults(&limits);
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(UINT32_C(0x420057), &raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag);

    if (status == 0) {
        switch (expected_type) {
            case KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE:
                status = kmipkit_ttlv_structure_create(&nested_structure);
                if (status == 0)
                    status = kmipkit_ttlv_value_structure(nested_structure, limits, &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_INTEGER:
                status = kmipkit_ttlv_value_integer(-7, &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_LONG_INTEGER:
                status = kmipkit_ttlv_value_long_integer(INT64_C(-7000000001), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_BIG_INTEGER:
                status = kmipkit_ttlv_value_big_integer(
                    limits, bigint_bytes, sizeof(bigint_bytes), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION:
                status = kmipkit_ttlv_value_enumeration(UINT32_C(0xF1234567), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_BOOLEAN:
                status = kmipkit_ttlv_value_boolean(1, &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING:
                status = kmipkit_ttlv_value_text_string(
                    limits, text_string, sizeof(text_string), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_BYTE_STRING:
                status = kmipkit_ttlv_value_byte_string(
                    limits, byte_string, sizeof(byte_string), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME:
                status = kmipkit_ttlv_value_date_time(INT64_C(1700000000), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_INTERVAL:
                status = kmipkit_ttlv_value_interval(UINT32_C(60), &value);
                break;
            case KMIPKIT_TTLV_ITEM_TYPE_DATE_TIME_EXTENDED:
                status = kmipkit_ttlv_value_date_time_extended(INT64_C(1700000000), &value);
                break;
            default:
                status = -1;
                break;
        }
    }
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, &item);
    if (status == 0) {
        value = NULL;
        status = kmipkit_ttlv_structure_create(&structure);
    }
    if (status == 0)
        status = kmipkit_ttlv_structure_with_item(structure, item, limits, &updated_structure);
    if (status == 0) {
        structure = updated_structure;
        updated_structure = NULL;
        item = NULL;
        status = kmipkit_ttlv_structure_view(structure, &structure_view);
    }
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(structure_view, &item_count);
    if (status == 0 && item_count == 1)
        status = kmipkit_ttlv_structure_view_item_at(structure_view, 0, &item_view);
    if (status == 0) status = kmipkit_ttlv_item_view_type(item_view, &actual_type);

    bool valid = status == 0 && item_count == 1 && actual_type == expected_type;
    if (item_view) kmipkit_ttlv_item_view_release(item_view);
    if (structure_view) kmipkit_ttlv_structure_view_release(structure_view);
    if (updated_structure) kmipkit_ttlv_structure_release(updated_structure);
    if (structure) kmipkit_ttlv_structure_release(structure);
    if (item) kmipkit_ttlv_item_release(item);
    if (nested_structure) kmipkit_ttlv_structure_release(nested_structure);
    if (value) kmipkit_ttlv_value_release(value);
    if (tag) kmipkit_tag_release(tag);
    if (raw_tag) kmipkit_raw_tag_release(raw_tag);
    if (limits) kmipkit_codec_limits_release(limits);
    return valid;
}

static bool roundtrip_all_ttlv_item_types(void) {
    static const uint32_t item_types[] = {
        KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE,
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
    for (size_t index = 0; index < sizeof(item_types) / sizeof(item_types[0]); ++index) {
        if (!KMIPKIT_ENUMERATION_ITEM_TYPE_name(item_types[index]) ||
            !roundtrip_ttlv_item_type(item_types[index])) return false;
    }
    return true;
}

static kmip_execution_outcome_t local_model_failure(const char *message,
                                                    const char *type_name) {
    kmip_execution_outcome_t outcome = { .executed = true, .library_failed = true };
    snprintf(outcome.result_reason, sizeof(outcome.result_reason), "%s: %s",
             message, type_name);
    return outcome;
}

static kmip_execution_outcome_t local_model_success(void) {
    return (kmip_execution_outcome_t){
        .executed = true, .success = true, .library_validated = true
    };
}

static kmip_execution_outcome_t execute_local_model_case(const kmip_test_case_t *test_case) {
    if (test_case->enum_type[0]) {
        if (strcmp(test_case->enum_type, "Item Type") == 0) {
            return roundtrip_all_ttlv_item_types()
                ? local_model_success()
                : local_model_failure("generic TTLV item-type roundtrip failed", test_case->enum_type);
        }
        const kmip_enum_probe_t *probe = find_enum_probe(test_case->enum_type);
        if (!probe || !probe->name_for_value || probe->tag == 0) {
            return outcome_not_run(
                "no standard field tag and named-value helper are mapped for this KMIP enumeration");
        }
        uint32_t known_value = 0;
        bool found_known_value = false;
        for (uint32_t candidate = 0; candidate <= UINT16_MAX; ++candidate) {
            if (probe->name_for_value(candidate)) {
                known_value = candidate;
                found_known_value = true;
                break;
            }
        }
        if (!found_known_value) {
            const kmip_enum_fallback_probe_t *fallback =
                find_enum_fallback_probe(test_case->enum_type);
            if (!fallback) {
                return outcome_not_run(
                    "the installed C named-value helper exposes no assigned value and no normative fallback is mapped");
            }
            known_value = fallback->raw_value;
        }
        if (!roundtrip_scalar_for_inspection(true, probe->tag, known_value) ||
            !roundtrip_scalar_for_inspection(true, probe->tag, UINT32_MAX)) {
            return local_model_failure("KMIP Enumeration roundtrip failed", test_case->enum_type);
        }
        return local_model_success();
    }
    if (test_case->bitmask_type[0]) {
        const kmip_bitmask_probe_t *probe = find_bitmask_probe(test_case->bitmask_type);
        if (!probe || !probe->known_bits || !probe->unknown_bits || probe->tag == 0) {
            return outcome_not_run("no standard tag and bit-preservation helpers are mapped for this KMIP mask");
        }
        uint32_t unknown_bit = UINT32_C(0x80000000);
        while (unknown_bit != 0 && (probe->known_mask & unknown_bit) != 0) unknown_bit >>= 1;
        uint32_t combined_mask = probe->known_mask | unknown_bit;
        int64_t unknown_as_integer = unknown_bit == UINT32_C(0x80000000)
            ? INT32_MIN : (int64_t)unknown_bit;
        if (unknown_bit == 0 ||
            probe->known_bits(combined_mask) != probe->known_mask ||
            probe->unknown_bits(combined_mask) != unknown_bit ||
            !roundtrip_scalar_for_inspection(false, probe->tag, probe->known_mask) ||
            !roundtrip_scalar_for_inspection(false, probe->tag, unknown_as_integer)) {
            return local_model_failure("KMIP bitmask construction or roundtrip failed",
                                       test_case->bitmask_type);
        }
        return local_model_success();
    }
    return outcome_not_run("local model catalog case has no enum or bitmask discriminator");
}


/* Standalone typed requests are a distinct installed construction route. The
 * fixture selector is consumed only by this independent test executable. */
static bool standalone_attribute_route_selected(void) {
    const char *route = getenv("KMIPKIT_C_ATTRIBUTE_REQUEST_ROUTE");
    return route && strcmp(route, "standalone") == 0;
}

static bool exported_attribute_payload_matches(
    kmipkit_ttlv_structure_t *payload,
    const kmipkit_unique_identifier_v1_t *identifier,
    const kmipkit_attribute_reference_v1_t *references,
    uint64_t reference_count) {
    kmipkit_ttlv_structure_view_t *view = NULL;
    uint64_t count = 0;
    bool valid = kmipkit_ttlv_structure_view(payload, &view) == 0 &&
        kmipkit_ttlv_structure_view_item_count(view, &count) == 0 &&
        count == reference_count + (identifier ? 1U : 0U);
    for (uint64_t index = 0; valid && index < reference_count; ++index) {
        kmipkit_ttlv_item_view_t *item = NULL;
        kmipkit_ttlv_value_view_t *value = NULL;
        uint32_t actual_reference = 0;
        valid = kmipkit_ttlv_structure_view_item_at(view, index + (identifier ? 1U : 0U), &item) == 0;
        if (valid && references[index].kind == KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG) {
            valid = item_view_has_tag(item, UINT32_C(0x42013B),
                                      KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION, &value) &&
                kmipkit_ttlv_value_view_enumeration(value, &actual_reference) == 0 &&
                actual_reference == references[index].tag_value;
        } else if (valid && references[index].kind == KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME) {
            valid = attribute_reference_name_matches(item, &references[index]);
        } else if (valid) {
            valid = false;
        }
        if (value) kmipkit_ttlv_value_view_release(value);
        if (item) kmipkit_ttlv_item_view_release(item);
    }
    if (view) kmipkit_ttlv_structure_view_release(view);
    return valid;
}

static int32_t build_get_attributes_item(
    const kmipkit_unique_identifier_v1_t *identifier,
    const kmipkit_attribute_reference_v1_t *references,
    uint64_t reference_count,
    kmipkit_client_batch_item_t **out_item) {
    if (!standalone_attribute_route_selected()) {
        return kmipkit_client_batch_item_get_attributes(identifier, references, reference_count, out_item);
    }
    kmipkit_get_attributes_request_t *request = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    int32_t status = kmipkit_get_attributes_request_create(identifier, references, reference_count, &request);
    if (status == 0) status = kmipkit_get_attributes_request_to_ttlv_payload(request, &payload);
    if (status == 0 && !exported_attribute_payload_matches(payload, identifier, references, reference_count)) {
        status = KMIPKIT_ERROR_INVALID_INPUT;
    }
    if (status == 0) status = kmipkit_client_batch_item_get_attributes_from_request(request, out_item);
    if (payload) kmipkit_ttlv_structure_release(payload);
    if (request) kmipkit_get_attributes_request_release(request);
    return status;
}

static int32_t read_register_fixture_bytes(
    const char *path,
    uint8_t **out_data,
    uint64_t *out_length) {
    if (!path || path[0] == '\0' || !out_data || !out_length) return -1;
    *out_data = NULL;
    *out_length = 0;
    FILE *source = fopen(path, "rb");
    if (!source) return -1;
    int32_t status = 0;
    if (fseek(source, 0, SEEK_END) != 0) status = -1;
    long file_length = status == 0 ? ftell(source) : -1;
    if (file_length <= 0 || (unsigned long)file_length > KMIP_MAX_REQUEST_BYTES ||
        file_length > INT32_MAX || fseek(source, 0, SEEK_SET) != 0) status = -1;
    uint8_t *data = NULL;
    if (status == 0) {
        data = malloc((size_t)file_length);
        if (!data) status = KMIPKIT_ERROR_RESOURCE_LIMIT;
    }
    if (status == 0 && fread(data, 1, (size_t)file_length, source) != (size_t)file_length) {
        status = -1;
    }
    (void)fclose(source);
    if (status != 0) {
        if (data) {
            memset(data, 0, (size_t)(file_length > 0 ? file_length : 0));
            free(data);
        }
        return status;
    }
    *out_data = data;
    *out_length = (uint64_t)file_length;
    return 0;
}

static int32_t build_certificate_register_item(
    kmip_live_client_t *client,
    const char *certificate_path,
    kmipkit_client_batch_item_t **out_item) {
    if (!client || !certificate_path || certificate_path[0] == '\0' || !out_item) return -1;
    uint8_t *certificate_data = NULL;
    uint64_t certificate_length = 0;
    int32_t status = read_register_fixture_bytes(
        certificate_path, &certificate_data, &certificate_length);
    kmipkit_ttlv_structure_t *certificate_attributes = NULL;
    kmipkit_ttlv_structure_t *certificate_fields = NULL;
    kmipkit_ttlv_value_t *type_value = NULL;
    kmipkit_ttlv_value_t *certificate_value = NULL;
    kmipkit_ttlv_value_t *object_value = NULL;
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_item_t *object_item = NULL;
    kmipkit_register_request_t *request = NULL;
    if (status == 0 && !create_cryptographic_key_attributes(
            client, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_RSA, 2048,
            &certificate_attributes)) status = -1;
    if (status == 0 && kmipkit_ttlv_value_integer(
            (int32_t)(KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_SIGN |
                      KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_VERIFY), &type_value) == 0 &&
        append_ttlv_value(client, &certificate_attributes, UINT32_C(0x42002C), &type_value)) {
        status = kmipkit_ttlv_value_enumeration(
            KMIPKIT_ENUMERATION_CERTIFICATE_TYPE_X_509, &type_value);
    } else {
        status = -1;
    }
    if (status == 0 && !append_ttlv_value(
            client, &certificate_attributes, UINT32_C(0x42001D), &type_value)) status = -1;
    if (status == 0 && kmipkit_ttlv_value_integer((int32_t)certificate_length, &type_value) != 0) status = -1;
    if (status == 0 && !append_ttlv_value(
            client, &certificate_attributes, UINT32_C(0x4200AD), &type_value)) status = -1;
    if (status == 0) status = kmipkit_ttlv_structure_create(&certificate_fields);
    if (status == 0) status = kmipkit_ttlv_value_enumeration(
        KMIPKIT_ENUMERATION_CERTIFICATE_TYPE_X_509, &type_value);
    if (status == 0 && !append_ttlv_value(
            client, &certificate_fields, UINT32_C(0x42001D), &type_value)) status = -1;
    if (type_value) kmipkit_ttlv_value_release(type_value);
    if (status == 0) status = kmipkit_ttlv_value_byte_string(
        client->limits, certificate_data, certificate_length, &certificate_value);
    if (status == 0 && !append_ttlv_value(
            client, &certificate_fields, UINT32_C(0x42001E), &certificate_value)) status = -1;
    if (certificate_value) kmipkit_ttlv_value_release(certificate_value);
    if (status == 0) status = kmipkit_ttlv_value_structure(
        certificate_fields, client->limits, &object_value);
    if (status == 0) certificate_fields = NULL;
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(UINT32_C(0x420013), &raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag);
    if (status == 0) status = kmipkit_ttlv_item_create(
        tag, object_value, client->limits, &object_item);
    if (status == 0) {
        kmipkit_ttlv_structure_t *owned_attributes = certificate_attributes;
        certificate_attributes = NULL;
        status = kmipkit_register_request_create(
            KMIPKIT_ENUMERATION_OBJECT_TYPE_CERTIFICATE, owned_attributes, object_item, &request);
    }
    if (status == 0) status = kmipkit_client_batch_item_register(request, out_item);
    if (request) kmipkit_register_request_release(request);
    if (object_item) kmipkit_ttlv_item_release(object_item);
    if (tag) kmipkit_tag_release(tag);
    if (raw_tag) kmipkit_raw_tag_release(raw_tag);
    if (object_value) kmipkit_ttlv_value_release(object_value);
    if (certificate_attributes) kmipkit_ttlv_structure_release(certificate_attributes);
    if (certificate_fields) kmipkit_ttlv_structure_release(certificate_fields);
    if (type_value) kmipkit_ttlv_value_release(type_value);
    if (certificate_value) kmipkit_ttlv_value_release(certificate_value);
    if (certificate_data) {
        memset(certificate_data, 0, (size_t)certificate_length);
        free(certificate_data);
    }
    return status;
}

static int32_t build_certificate_request_register_item(
    kmip_live_client_t *client,
    const char *request_path,
    kmipkit_client_batch_item_t **out_item) {
    if (!client || !request_path || request_path[0] == '\0' || !out_item) return -1;
    uint8_t *request_data = NULL;
    uint64_t request_length = 0;
    int32_t status = read_register_fixture_bytes(request_path, &request_data, &request_length);
    kmipkit_ttlv_structure_t *attributes = NULL;
    kmipkit_ttlv_structure_t *name_structure = NULL;
    kmipkit_ttlv_value_t *request_type_value = NULL;
    kmipkit_ttlv_value_t *request_bytes_value = NULL;
    kmipkit_ttlv_value_t *name_value = NULL;
    kmipkit_ttlv_value_t *name_type_value = NULL;
    kmipkit_ttlv_value_t *name_attribute_value = NULL;
    kmipkit_raw_tag_t *request_type_raw_tag = NULL;
    kmipkit_raw_tag_t *request_bytes_raw_tag = NULL;
    kmipkit_tag_t *request_type_tag = NULL;
    kmipkit_tag_t *request_bytes_tag = NULL;
    kmipkit_ttlv_item_t *request_type_item = NULL;
    kmipkit_ttlv_item_t *request_bytes_item = NULL;
    kmipkit_ttlv_item_t *object_item = NULL;
    kmipkit_register_request_t *request = NULL;
    if (status == 0) status = kmipkit_ttlv_value_enumeration(
        KMIPKIT_ENUMERATION_CERTIFICATE_REQUEST_TYPE_PKCS_10, &request_type_value);
    if (status == 0) status = kmipkit_ttlv_value_byte_string(
        client->limits, request_data, request_length, &request_bytes_value);
    if (status == 0 && !create_cryptographic_key_attributes(
        client, KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_RSA, 2048, &attributes)) status = -1;
    if (status == 0) status = kmipkit_ttlv_structure_create(&name_structure);
    static const uint8_t request_name[] = "KMIPKit C interop certificate request";
    if (status == 0) status = kmipkit_ttlv_value_text_string(
        client->limits, request_name, sizeof(request_name) - 1, &name_value);
    if (status == 0 && !append_ttlv_value(
        client, &name_structure, UINT32_C(0x42000B), &name_value)) status = -1;
    if (status == 0) status = kmipkit_ttlv_value_enumeration(
        KMIPKIT_ENUMERATION_NAME_TYPE_UNINTERPRETED_TEXT_STRING, &name_type_value);
    if (status == 0 && !append_ttlv_value(
        client, &name_structure, UINT32_C(0x420054), &name_type_value)) status = -1;
    if (status == 0 && !structure_into_value(client, &name_structure, &name_attribute_value)) status = -1;
    if (status == 0 && !append_ttlv_value(
        client, &attributes, UINT32_C(0x42000A), &name_attribute_value)) status = -1;
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(
        UINT32_C(0x420019), &request_type_raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(
        request_type_raw_tag, &request_type_tag);
    if (status == 0) status = kmipkit_ttlv_item_create(
        request_type_tag, request_type_value, client->limits, &request_type_item);
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(
        UINT32_C(0x420140), &request_bytes_raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(
        request_bytes_raw_tag, &request_bytes_tag);
    if (status == 0) status = kmipkit_ttlv_item_create(
        request_bytes_tag, request_bytes_value, client->limits, &request_bytes_item);
    if (status == 0) {
        kmipkit_ttlv_item_t *fields[] = {request_type_item, request_bytes_item};
        status = kmipkit_certificate_request_from_fields(fields, 2, &object_item);
    }
    if (status == 0) {
        kmipkit_ttlv_structure_t *owned_attributes = attributes;
        attributes = NULL;
        status = kmipkit_register_request_create(
            KMIPKIT_ENUMERATION_OBJECT_TYPE_CERTIFICATE_REQUEST,
            owned_attributes, object_item, &request);
    }
    if (status == 0) status = kmipkit_client_batch_item_register(request, out_item);
    if (request) kmipkit_register_request_release(request);
    if (object_item) kmipkit_ttlv_item_release(object_item);
    if (name_structure) kmipkit_ttlv_structure_release(name_structure);
    if (attributes) kmipkit_ttlv_structure_release(attributes);
    if (request_bytes_item) kmipkit_ttlv_item_release(request_bytes_item);
    if (request_type_item) kmipkit_ttlv_item_release(request_type_item);
    if (request_bytes_tag) kmipkit_tag_release(request_bytes_tag);
    if (request_type_tag) kmipkit_tag_release(request_type_tag);
    if (request_bytes_raw_tag) kmipkit_raw_tag_release(request_bytes_raw_tag);
    if (request_type_raw_tag) kmipkit_raw_tag_release(request_type_raw_tag);
    if (request_bytes_value) kmipkit_ttlv_value_release(request_bytes_value);
    if (request_type_value) kmipkit_ttlv_value_release(request_type_value);
    if (name_attribute_value) kmipkit_ttlv_value_release(name_attribute_value);
    if (name_type_value) kmipkit_ttlv_value_release(name_type_value);
    if (name_value) kmipkit_ttlv_value_release(name_value);
    if (request_data) {
        memset(request_data, 0, (size_t)request_length);
        free(request_data);
    }
    return status;
}

static int32_t build_get_attribute_list_item(
    const kmipkit_unique_identifier_v1_t *identifier,
    kmipkit_client_batch_item_t **out_item) {
    if (!standalone_attribute_route_selected()) {
        return kmipkit_client_batch_item_get_attribute_list(identifier, out_item);
    }
    kmipkit_get_attribute_list_request_t *request = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    int32_t status = kmipkit_get_attribute_list_request_create(identifier, &request);
    if (status == 0) status = kmipkit_get_attribute_list_request_to_ttlv_payload(request, &payload);
    if (status == 0 && !exported_attribute_payload_matches(payload, identifier, NULL, 0)) {
        status = KMIPKIT_ERROR_INVALID_INPUT;
    }
    if (status == 0) status = kmipkit_client_batch_item_get_attribute_list_from_request(request, out_item);
    if (payload) kmipkit_ttlv_structure_release(payload);
    if (request) kmipkit_get_attribute_list_request_release(request);
    return status;
}

static bool attribute_mutation_payload_matches(
    kmipkit_ttlv_structure_t *payload,
    const kmipkit_unique_identifier_v1_t *identifier) {
    static const uint8_t expected_comment[] = "KMIPKit interoperability comment";
    kmipkit_ttlv_structure_view_t *payload_view = NULL;
    kmipkit_ttlv_item_view_t *identifier_item = NULL;
    kmipkit_ttlv_item_view_t *new_attribute_item = NULL;
    kmipkit_ttlv_value_view_t *identifier_value = NULL;
    kmipkit_ttlv_value_view_t *new_attribute_value = NULL;
    kmipkit_ttlv_structure_view_t *new_attribute_view = NULL;
    kmipkit_ttlv_item_view_t *comment_item = NULL;
    kmipkit_ttlv_value_view_t *comment_value = NULL;
    uint64_t payload_count = 0;
    uint64_t new_attribute_count = 0;
    uint64_t identifier_length = 0;
    uint64_t comment_length = 0;
    int32_t status = kmipkit_ttlv_structure_view(payload, &payload_view);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(payload_view, &payload_count);
    if (status == 0 && payload_count != 2) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_structure_view_item_at(payload_view, 0, &identifier_item);
    if (status == 0 && !item_view_has_tag(
            identifier_item, UINT32_C(0x420094), KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
            &identifier_value)) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_value_view_byte_length(
        identifier_value, &identifier_length);
    bool identifier_matches = status == 0 && identifier_length == identifier->text_length;
    for (uint64_t index = 0; identifier_matches && index < identifier_length; ++index) {
        uint8_t byte = 0;
        status = kmipkit_ttlv_value_view_byte_at(identifier_value, index, &byte);
        identifier_matches = status == 0 && byte == identifier->text_data[index];
    }
    if (status == 0 && !identifier_matches) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_structure_view_item_at(
        payload_view, 1, &new_attribute_item);
    if (status == 0 && !item_view_has_tag(
            new_attribute_item, UINT32_C(0x42013D), KMIPKIT_TTLV_ITEM_TYPE_STRUCTURE,
            &new_attribute_value)) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_value_view_structure(
        new_attribute_value, &new_attribute_view);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(
        new_attribute_view, &new_attribute_count);
    if (status == 0 && new_attribute_count != 1) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_structure_view_item_at(
        new_attribute_view, 0, &comment_item);
    if (status == 0 && !item_view_has_tag(
            comment_item, UINT32_C(0x4200FD), KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
            &comment_value)) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_value_view_byte_length(comment_value, &comment_length);
    bool comment_matches = status == 0 && comment_length == sizeof(expected_comment) - 1;
    for (uint64_t index = 0; comment_matches && index < comment_length; ++index) {
        uint8_t byte = 0;
        status = kmipkit_ttlv_value_view_byte_at(comment_value, index, &byte);
        comment_matches = status == 0 && byte == expected_comment[index];
    }
    if (payload_view) kmipkit_ttlv_structure_view_release(payload_view);
    if (identifier_item) kmipkit_ttlv_item_view_release(identifier_item);
    if (new_attribute_item) kmipkit_ttlv_item_view_release(new_attribute_item);
    if (identifier_value) kmipkit_ttlv_value_view_release(identifier_value);
    if (new_attribute_value) kmipkit_ttlv_value_view_release(new_attribute_value);
    if (new_attribute_view) kmipkit_ttlv_structure_view_release(new_attribute_view);
    if (comment_item) kmipkit_ttlv_item_view_release(comment_item);
    if (comment_value) kmipkit_ttlv_value_view_release(comment_value);
    return status == 0 && identifier_matches && comment_matches;
}

static int32_t build_comment_attribute_mutation_item(
    kmip_live_client_t *client,
    const kmipkit_unique_identifier_v1_t *identifier,
    bool set_attribute,
    kmipkit_client_batch_item_t **out_item) {
    static const uint8_t comment[] = "KMIPKit interoperability comment";
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_item_t *attribute = NULL;
    kmipkit_add_attribute_request_t *add_request = NULL;
    kmipkit_set_attribute_request_t *set_request = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    int32_t status = kmipkit_ttlv_value_text_string(
        client->limits, comment, sizeof(comment) - 1, &value);
    if (status == 0) status = kmipkit_ttlv_raw_tag_create(
        UINT32_C(0x4200FD), &raw_tag);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw_tag, &tag);
    if (status == 0) status = kmipkit_ttlv_item_create(
        tag, value, client->limits, &attribute);
    if (status == 0) value = NULL;
    if (status == 0 && standalone_attribute_route_selected()) {
        if (set_attribute) {
            status = kmipkit_set_attribute_request_create(identifier, attribute, &set_request);
            if (status == 0) status = kmipkit_set_attribute_request_to_ttlv_payload(
                set_request, &payload);
            if (status == 0 && !attribute_mutation_payload_matches(payload, identifier)) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
            if (status == 0) status = kmipkit_client_batch_item_set_attribute_from_request(
                set_request, out_item);
        } else {
            status = kmipkit_add_attribute_request_create(identifier, attribute, &add_request);
            if (status == 0) status = kmipkit_add_attribute_request_to_ttlv_payload(
                add_request, &payload);
            if (status == 0 && !attribute_mutation_payload_matches(payload, identifier)) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
            if (status == 0) status = kmipkit_client_batch_item_add_attribute_from_request(
                add_request, out_item);
        }
    } else if (status == 0 && set_attribute) {
        status = kmipkit_client_batch_item_set_attribute(identifier, attribute, out_item);
    } else if (status == 0) {
        status = kmipkit_client_batch_item_add_attribute(identifier, attribute, out_item);
    }
    if (payload) kmipkit_ttlv_structure_release(payload);
    if (add_request) kmipkit_add_attribute_request_release(add_request);
    if (set_request) kmipkit_set_attribute_request_release(set_request);
    if (attribute) kmipkit_ttlv_item_release(attribute);
    if (tag) kmipkit_tag_release(tag);
    if (raw_tag) kmipkit_raw_tag_release(raw_tag);
    if (value) kmipkit_ttlv_value_release(value);
    return status;
}

static bool delete_attribute_payload_matches(
    kmipkit_ttlv_structure_t *payload,
    const kmipkit_unique_identifier_v1_t *identifier) {
    kmipkit_ttlv_structure_view_t *payload_view = NULL;
    kmipkit_ttlv_item_view_t *identifier_item = NULL;
    kmipkit_ttlv_item_view_t *reference_item = NULL;
    kmipkit_ttlv_value_view_t *identifier_value = NULL;
    kmipkit_ttlv_value_view_t *reference_value = NULL;
    uint64_t payload_count = 0;
    uint64_t identifier_length = 0;
    uint32_t reference_tag = 0;
    int32_t status = kmipkit_ttlv_structure_view(payload, &payload_view);
    if (status == 0) status = kmipkit_ttlv_structure_view_item_count(payload_view, &payload_count);
    if (status == 0 && payload_count != 2) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_structure_view_item_at(payload_view, 0, &identifier_item);
    if (status == 0 && !item_view_has_tag(
            identifier_item, UINT32_C(0x420094), KMIPKIT_TTLV_ITEM_TYPE_TEXT_STRING,
            &identifier_value)) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_value_view_byte_length(
        identifier_value, &identifier_length);
    bool identifier_matches = status == 0 && identifier_length == identifier->text_length;
    for (uint64_t index = 0; identifier_matches && index < identifier_length; ++index) {
        uint8_t byte = 0;
        status = kmipkit_ttlv_value_view_byte_at(identifier_value, index, &byte);
        identifier_matches = status == 0 && byte == identifier->text_data[index];
    }
    if (status == 0 && !identifier_matches) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_structure_view_item_at(payload_view, 1, &reference_item);
    if (status == 0 && !item_view_has_tag(
            reference_item, UINT32_C(0x42013B), KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION,
            &reference_value)) status = KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_ttlv_value_view_enumeration(reference_value, &reference_tag);
    if (payload_view) kmipkit_ttlv_structure_view_release(payload_view);
    if (identifier_item) kmipkit_ttlv_item_view_release(identifier_item);
    if (reference_item) kmipkit_ttlv_item_view_release(reference_item);
    if (identifier_value) kmipkit_ttlv_value_view_release(identifier_value);
    if (reference_value) kmipkit_ttlv_value_view_release(reference_value);
    return status == 0 && identifier_matches && reference_tag == UINT32_C(0x4200FD);
}

static int32_t build_delete_comment_attribute_item(
    const kmipkit_unique_identifier_v1_t *identifier,
    kmipkit_client_batch_item_t **out_item) {
    kmipkit_attribute_reference_v1_t reference = {
        .struct_size = (uint32_t)sizeof(kmipkit_attribute_reference_v1_t),
        .abi_version = 1,
        .kind = KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG,
        .tag_value = UINT32_C(0x4200FD)
    };
    kmipkit_delete_attribute_request_t *request = NULL;
    kmipkit_ttlv_structure_t *payload = NULL;
    int32_t status = 0;
    if (standalone_attribute_route_selected()) {
        status = kmipkit_delete_attribute_request_create(identifier, NULL, &reference, &request);
        if (status == 0) status = kmipkit_delete_attribute_request_to_ttlv_payload(request, &payload);
        if (status == 0 && !delete_attribute_payload_matches(payload, identifier)) {
            status = KMIPKIT_ERROR_INVALID_INPUT;
        }
        if (status == 0) status = kmipkit_client_batch_item_delete_attribute_from_request(
            request, out_item);
    } else {
        status = kmipkit_client_batch_item_delete_attribute(identifier, NULL, &reference, out_item);
    }
    if (payload) kmipkit_ttlv_structure_release(payload);
    if (request) kmipkit_delete_attribute_request_release(request);
    return status;
}

typedef enum {
    ATTRIBUTE_MUTATION_ADD,
    ATTRIBUTE_MUTATION_SET,
    ATTRIBUTE_MUTATION_DELETE
} attribute_mutation_kind_t;

static const char *attribute_mutation_name(attribute_mutation_kind_t mutation) {
    switch (mutation) {
        case ATTRIBUTE_MUTATION_ADD: return "Add Attribute";
        case ATTRIBUTE_MUTATION_SET: return "Set Attribute";
        case ATTRIBUTE_MUTATION_DELETE: return "Delete Attribute";
    }
    return "attribute mutation";
}

static kmip_execution_outcome_t execute_attribute_mutation_case(
    kmip_live_client_t *client,
    attribute_mutation_kind_t mutation) {
    kmip_execution_outcome_t outcome = { .executed = true };
    kmipkit_ttlv_structure_t *attributes = NULL;
    kmipkit_client_batch_item_t *create_item = NULL;
    kmipkit_client_batch_t *create_batch = NULL;
    kmipkit_client_batch_response_t *create_response = NULL;
    kmipkit_client_batch_item_response_t *create_item_response = NULL;
    kmipkit_client_batch_item_t *seed_item = NULL;
    kmipkit_client_batch_t *seed_batch = NULL;
    kmipkit_client_batch_response_t *seed_response = NULL;
    kmipkit_client_batch_item_response_t *seed_item_response = NULL;
    kmipkit_client_batch_item_t *mutation_item = NULL;
    kmipkit_client_batch_t *mutation_batch = NULL;
    kmipkit_client_batch_response_t *mutation_response = NULL;
    kmipkit_client_batch_item_response_t *mutation_item_response = NULL;
    kmipkit_unique_identifier_v1_t created_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    uint8_t identifier_present = 0;
    bool cleanup_needed = false;
    uint32_t object_type = 0;
    uint64_t response_count = 0;
    int32_t status = create_symmetric_key_attributes(client, &attributes) ? 0 :
        KMIPKIT_ERROR_INVALID_INPUT;
    if (status == 0) status = kmipkit_client_batch_item_create(
        KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY, attributes, NULL, &create_item);
    if (status == 0) status = set_batch_item_id(&create_item, "interop-create");
    if (status == 0) status = kmipkit_client_batch_create(&create_batch);
    if (status == 0 && client->username_env[0] != '\0') {
        kmipkit_authentication_t *authentication = NULL;
        if (!create_authentication(client, &authentication)) status = KMIPKIT_ERROR_INVALID_INPUT;
        else {
            status = kmipkit_client_batch_set_authentication(create_batch, authentication);
            if (status != 0) kmipkit_authentication_release(authentication);
        }
    }
    if (status == 0) status = kmipkit_client_batch_add_item(create_batch, &create_item);
    if (status == 0) status = kmipkit_client_execute(
        client->client, &create_batch, client->limits, &create_response);
    if (status == 0) status = kmipkit_client_batch_response_item_count(
        create_response, &response_count);
    if (status == 0 && response_count == 1) status = kmipkit_client_batch_response_item_at(
        create_response, 0, &create_item_response);
    else if (status == 0) status = KMIPKIT_ERROR_INVALID_INPUT;
    kmipkit_client_batch_item_metadata_v2_t create_metadata = {
        .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
        .abi_version = 2
    };
    if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
        create_item_response, &create_metadata);
    if (status == 0 && create_metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) {
        outcome.library_validated = true;
        outcome.has_result_status_code = true;
        outcome.result_status_code = create_metadata.result_status;
        outcome.has_result_reason_code = create_metadata.has_result_reason != 0;
        outcome.result_reason_code = create_metadata.result_reason;
        outcome.server_unsupported = create_metadata.has_result_reason != 0 &&
            (create_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_OPERATION_NOT_SUPPORTED ||
             create_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_FEATURE_NOT_SUPPORTED);
        outcome.policy_restricted = create_metadata.has_result_reason != 0 &&
            (create_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_NOT_AUTHORISED ||
             create_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_PERMISSION_DENIED);
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "server rejected the isolated %s setup Create",
                 attribute_mutation_name(mutation));
        goto cleanup;
    }
    if (status == 0) status = kmipkit_client_batch_item_response_create(
        create_item_response, &identifier_present, &object_type, &created_identifier);
    if (status != 0 || !identifier_present ||
        object_type != KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY ||
        created_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
        !created_identifier.text_data || created_identifier.text_length == 0) {
        outcome.library_failed = status != 0;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit did not return the created %s fixture identifier",
                 attribute_mutation_name(mutation));
        goto cleanup;
    }
    cleanup_needed = true;

    if (mutation == ATTRIBUTE_MUTATION_DELETE) {
        status = build_comment_attribute_mutation_item(
            client, &created_identifier, false, &seed_item);
        if (status == 0) status = set_batch_item_id(&seed_item, "interop-attributes");
        if (status == 0) status = kmipkit_client_batch_create(&seed_batch);
        if (status == 0 && client->username_env[0] != '\0') {
            kmipkit_authentication_t *authentication = NULL;
            if (!create_authentication(client, &authentication)) status = KMIPKIT_ERROR_INVALID_INPUT;
            else {
                status = kmipkit_client_batch_set_authentication(seed_batch, authentication);
                if (status != 0) kmipkit_authentication_release(authentication);
            }
        }
        if (status == 0) status = kmipkit_client_batch_add_item(seed_batch, &seed_item);
        if (status == 0) status = kmipkit_client_execute(
            client->client, &seed_batch, client->limits, &seed_response);
        response_count = 0;
        if (status == 0) status = kmipkit_client_batch_response_item_count(seed_response, &response_count);
        if (status == 0 && response_count == 1) status = kmipkit_client_batch_response_item_at(
            seed_response, 0, &seed_item_response);
        else if (status == 0) status = KMIPKIT_ERROR_INVALID_INPUT;
        kmipkit_client_batch_item_metadata_v2_t seed_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
            seed_item_response, &seed_metadata);
        uint8_t seed_present = 0;
        kmipkit_unique_identifier_v1_t seed_identifier = {
            .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
            .abi_version = 1
        };
        if (status == 0 && seed_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) {
            status = kmipkit_client_batch_item_response_add_attribute(
                seed_item_response, &seed_present, &seed_identifier);
        }
        if (status != 0 || !seed_present ||
            seed_metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
            !same_text_identifier(&created_identifier, &seed_identifier)) {
            outcome.library_failed = status != 0;
            outcome.library_validated = status == 0;
            outcome.has_result_status_code = status == 0;
            outcome.result_status_code = seed_metadata.result_status;
            outcome.has_result_reason_code = status == 0 && seed_metadata.has_result_reason != 0;
            outcome.result_reason_code = seed_metadata.result_reason;
            outcome.server_unsupported = status == 0 && seed_metadata.has_result_reason != 0 &&
                (seed_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_OPERATION_NOT_SUPPORTED ||
                 seed_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_FEATURE_NOT_SUPPORTED);
            outcome.policy_restricted = status == 0 && seed_metadata.has_result_reason != 0 &&
                (seed_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_NOT_AUTHORISED ||
                 seed_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_PERMISSION_DENIED);
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit could not seed the isolated Delete Attribute Comment fixture");
            goto cleanup;
        }
    }

    status = mutation == ATTRIBUTE_MUTATION_DELETE ? build_delete_comment_attribute_item(
        &created_identifier, &mutation_item) : build_comment_attribute_mutation_item(
        client, &created_identifier, mutation == ATTRIBUTE_MUTATION_SET, &mutation_item);
    if (status == 0) status = set_batch_item_id(
        &mutation_item, mutation == ATTRIBUTE_MUTATION_DELETE ?
            "interop-delete-attribute" : "interop-attributes");
    if (status == 0) status = kmipkit_client_batch_create(&mutation_batch);
    if (status == 0 && client->username_env[0] != '\0') {
        kmipkit_authentication_t *authentication = NULL;
        if (!create_authentication(client, &authentication)) status = KMIPKIT_ERROR_INVALID_INPUT;
        else {
            status = kmipkit_client_batch_set_authentication(mutation_batch, authentication);
            if (status != 0) kmipkit_authentication_release(authentication);
        }
    }
    if (status == 0) status = kmipkit_client_batch_add_item(mutation_batch, &mutation_item);
    if (status == 0) status = kmipkit_client_execute(
        client->client, &mutation_batch, client->limits, &mutation_response);
    response_count = 0;
    if (status == 0) status = kmipkit_client_batch_response_item_count(
        mutation_response, &response_count);
    if (status == 0 && response_count == 1) status = kmipkit_client_batch_response_item_at(
        mutation_response, 0, &mutation_item_response);
    else if (status == 0) status = KMIPKIT_ERROR_INVALID_INPUT;
    kmipkit_client_batch_item_metadata_v2_t mutation_metadata = {
        .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
        .abi_version = 2
    };
    if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
        mutation_item_response, &mutation_metadata);
    uint8_t mutation_present = 0;
    kmipkit_unique_identifier_v1_t mutation_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    if (status == 0 && mutation_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) {
        switch (mutation) {
            case ATTRIBUTE_MUTATION_ADD:
                status = kmipkit_client_batch_item_response_add_attribute(
                    mutation_item_response, &mutation_present, &mutation_identifier);
                break;
            case ATTRIBUTE_MUTATION_SET:
                status = kmipkit_client_batch_item_response_set_attribute(
                    mutation_item_response, &mutation_present, &mutation_identifier);
                break;
            case ATTRIBUTE_MUTATION_DELETE:
                status = kmipkit_client_batch_item_response_delete_attribute(
                    mutation_item_response, &mutation_present, &mutation_identifier);
                break;
        }
    }
    bool mutation_response_valid = status == 0 && mutation_present &&
        mutation_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS &&
        same_text_identifier(&created_identifier, &mutation_identifier);
    if (!mutation_response_valid) {
        outcome.library_failed = status != 0;
        outcome.library_validated = status == 0;
        outcome.has_result_status_code = status == 0;
        outcome.result_status_code = mutation_metadata.result_status;
        outcome.has_result_reason_code = status == 0 && mutation_metadata.has_result_reason != 0;
        outcome.result_reason_code = mutation_metadata.result_reason;
        outcome.server_unsupported = status == 0 && mutation_metadata.has_result_reason != 0 &&
            (mutation_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_OPERATION_NOT_SUPPORTED ||
             mutation_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_FEATURE_NOT_SUPPORTED);
        outcome.policy_restricted = status == 0 && mutation_metadata.has_result_reason != 0 &&
            (mutation_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_NOT_AUTHORISED ||
             mutation_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_PERMISSION_DENIED);
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit %s response did not return the created object's identifier",
                 attribute_mutation_name(mutation));
    } else {
        outcome.success = true;
        outcome.library_validated = true;
        outcome.has_result_status_code = true;
        outcome.result_status_code = mutation_metadata.result_status;
    }
    cleanup_needed = false;
    if (!cleanup_placeholder_chain_object(client, &created_identifier, false, &status)) {
        outcome.success = false;
        outcome.library_failed = status != 0;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit could not clean up the isolated %s fixture",
                 attribute_mutation_name(mutation));
    }

cleanup:
    if (cleanup_needed) {
        cleanup_needed = false;
        if (!cleanup_placeholder_chain_object(client, &created_identifier, false, &status)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit could not clean up the isolated %s fixture",
                     attribute_mutation_name(mutation));
        }
    }
    if (seed_item_response) kmipkit_client_batch_item_response_release(seed_item_response);
    if (seed_response) kmipkit_client_batch_response_release(seed_response);
    if (seed_batch) kmipkit_client_batch_release(seed_batch);
    if (seed_item) kmipkit_client_batch_item_release(seed_item);
    if (mutation_item_response) kmipkit_client_batch_item_response_release(mutation_item_response);
    if (mutation_response) kmipkit_client_batch_response_release(mutation_response);
    if (mutation_batch) kmipkit_client_batch_release(mutation_batch);
    if (mutation_item) kmipkit_client_batch_item_release(mutation_item);
    if (create_item_response) kmipkit_client_batch_item_response_release(create_item_response);
    if (create_response) kmipkit_client_batch_response_release(create_response);
    if (create_batch) kmipkit_client_batch_release(create_batch);
    if (create_item) kmipkit_client_batch_item_release(create_item);
    if (attributes) kmipkit_ttlv_structure_release(attributes);
    return outcome;
}

kmip_execution_outcome_t kmip_live_execute_case(const kmip_test_case_t *test_case,
                                                const kmip_attempt_context_t *attempt,
                                                void *user_data) {
    (void)attempt;
    kmip_live_client_t *client = user_data;
    if (!test_case || !client) {
        return outcome_not_run("live client is not initialized");
    }
    if (strcmp(test_case->mutation, "mutating") == 0 && !client->allow_mutations) {
        kmip_execution_outcome_t outcome = { .executed = true, .policy_restricted = true };
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "mutating operation requires configuration and command-line opt-in");
        return outcome;
    }
    if (strcmp(test_case->workflow_type, "server_to_client") == 0) {
        return outcome_not_run(
            "case requires server-initiated request reception, which the public C client API does not expose");
    }
    if (strcmp(client->api_level, "high-level") == 0) {
        return outcome_not_run(
            "the C binding exposes no high-level client route for this case");
    }
    bool get_constraints_boundary_workflow =
        strcmp(test_case->case_id, "test-v20-reject-get-constraints") == 0;
    bool get_usage_allocation_uid_variant =
        strcmp(test_case->case_id,
               "test-norm-payload-client-get-usage-allocation-request-20-001") == 0 ||
        strcmp(test_case->case_id,
               "test-norm-payload-client-get-usage-allocation-request-21-001") == 0;
    const char *usage_allocation_identifier_form =
        getenv("KMIPKIT_C_USAGE_ALLOCATION_IDENTIFIER_FORM");
    bool usage_allocation_identifier_explicit = usage_allocation_identifier_form &&
        strcmp(usage_allocation_identifier_form, "explicit") == 0;
    bool usage_allocation_identifier_omitted = usage_allocation_identifier_form &&
        strcmp(usage_allocation_identifier_form, "omitted") == 0;
    bool set_constraints_workflow =
        strcmp(test_case->case_id, "test-op-set-constraints") == 0;
    bool set_defaults_workflow =
        strcmp(test_case->case_id, "test-op-set-defaults") == 0;
    if (strcmp(test_case->mode, "local_model") == 0) {
        if (!get_constraints_boundary_workflow && !get_usage_allocation_uid_variant)
            return execute_local_model_case(test_case);
        if (strcmp(client->api_level, "typed-protocol") != 0) {
            return outcome_not_run(
                "the generic C TTLV API cannot invoke a typed request to verify pre-transport version rejection");
        }
    }
    if (strcmp(client->api_level, "generic-ttlv") == 0) {
        return outcome_not_run(
            "the public C API can construct and inspect generic TTLV but cannot dispatch a generic request");
    }
    if (!client->client || !client->limits) {
        return outcome_not_run("live client is not initialized");
    }
    if (strcmp(test_case->mode, "server_exchange") != 0 &&
        !get_constraints_boundary_workflow && !get_usage_allocation_uid_variant) {
        return outcome_not_run("catalog mode is not supported by the C validator");
    }
    if (get_usage_allocation_uid_variant &&
        !usage_allocation_identifier_explicit && !usage_allocation_identifier_omitted) {
        return outcome_not_run(
            "controlled peer did not select an optional Unique Identifier form");
    }
    if (strcmp(test_case->case_id, "test-op-add-attribute") == 0)
        return execute_attribute_mutation_case(client, ATTRIBUTE_MUTATION_ADD);
    if (strcmp(test_case->case_id, "test-op-set-attribute") == 0)
        return execute_attribute_mutation_case(client, ATTRIBUTE_MUTATION_SET);
    if (strcmp(test_case->case_id, "test-op-delete-attribute") == 0)
        return execute_attribute_mutation_case(client, ATTRIBUTE_MUTATION_DELETE);
    if (strcmp(test_case->operation, "Discover Versions") == 0 &&
        client->protocol_minor == 0) {
        return outcome_not_run(
            "the public C Discover Versions builder only offers KMIP 2.1 and cannot construct a KMIP 2.0 offer");
    }
    if (strcmp(test_case->operation, "Login") == 0 && client->protocol_minor == 0) {
        return outcome_not_run(
            "the public C Login builder only supports KMIP 2.1");
    }
    bool cryptographic_algorithm_attribute_workflow =
        strcmp(test_case->case_id, "test-attr-cryptographic-algorithm") == 0;
    bool cryptographic_length_attribute_workflow =
        strcmp(test_case->case_id, "test-attr-cryptographic-length") == 0;
    bool certificate_attributes_workflow =
        strcmp(test_case->case_id, "test-attr-certificate-attributes") == 0;
    bool register_certificate_object_workflow =
        strcmp(test_case->case_id, "test-op-register") == 0 ||
        strcmp(test_case->case_id, "test-obj-certificate") == 0;
    bool register_certificate_request_workflow =
        strcmp(test_case->case_id, "test-obj-certificate-request") == 0;
    bool register_object_workflow = register_certificate_object_workflow ||
        register_certificate_request_workflow;
    bool register_certificate_workflow = certificate_attributes_workflow ||
        register_certificate_object_workflow;
    const char *register_object_path = test_case->register_object_path[0] != '\0' ?
        test_case->register_object_path : test_case->certificate_object_path;
    if (register_object_workflow && register_object_path[0] == '\0') {
        return outcome_not_run("Register requires the isolated object fixture prepared by the route matrix");
    }
    bool batch_create_activate_get_attribute_list_revoke_destroy_workflow =
        strcmp(test_case->case_id, "test-batch-create-activate-destroy") == 0;
    bool batch_placeholder_chain_workflow =
        strcmp(test_case->case_id, "test-batch-placeholder-chain-resolves") == 0;
    bool add_attribute_workflow = strcmp(test_case->case_id, "test-op-add-attribute") == 0;
    bool read_back_attribute_workflow = test_case->attribute_reference_kind[0] != '\0';
    uint32_t read_back_attribute_tag = 0;
    uint32_t read_back_attribute_response_tag = 0;
    uint8_t read_back_attribute_type = 0;
    bool attribute_reference_by_tag =
        strcmp(test_case->attribute_reference_kind, "tag") == 0;
    bool attribute_reference_by_name =
        strcmp(test_case->attribute_reference_kind, "name") == 0;
    bool attribute_reference_by_all =
        strcmp(test_case->attribute_reference_kind, "all") == 0;
    if (read_back_attribute_workflow &&
        ((!attribute_reference_by_tag && !attribute_reference_by_name && !attribute_reference_by_all) ||
         (attribute_reference_by_tag &&
          !parse_attribute_reference_tag(test_case->attribute_reference_tag,
                                         &read_back_attribute_tag)) ||
         (!attribute_reference_by_all &&
          (!parse_attribute_reference_tag(test_case->attribute_response_tag,
                                          &read_back_attribute_response_tag) ||
           !parse_attribute_response_type(test_case->attribute_response_type,
                                          &read_back_attribute_type))) ||
         (attribute_reference_by_name &&
          (!test_case->attribute_reference_name[0] ||
           !test_case->attribute_vendor_identification[0])) ||
         (attribute_reference_by_all &&
          (!test_case->attribute_response_fields[0] ||
           !test_case->certificate_object_path[0])))) {
        return outcome_failed("read-back attribute fixture metadata is invalid");
    }
    bool hash_workflow = strcmp(test_case->case_id, "test-op-hash") == 0 ||
        strcmp(test_case->case_id, "test-crypto-hash-sha_256-0") == 0;
    bool rng_retrieve_workflow = strcmp(test_case->case_id, "test-op-rng-retrieve") == 0 ||
        strcmp(test_case->case_id, "test-crypto-rng-retrieve-rng-32") == 0;
    bool rng_seed_workflow = strcmp(test_case->case_id, "test-op-rng-seed") == 0 ||
        strcmp(test_case->case_id, "test-crypto-rng-seed-rng-32") == 0;
    uint32_t rng_length = strcmp(test_case->case_id, "test-crypto-rng-retrieve-rng-32") == 0 ||
        strcmp(test_case->case_id, "test-crypto-rng-seed-rng-32") == 0 ? 32U : 16U;
    bool get_attributes_workflow = strcmp(test_case->case_id, "test-op-get-attributes") == 0 ||
        cryptographic_algorithm_attribute_workflow || cryptographic_length_attribute_workflow ||
        read_back_attribute_workflow;
    bool get_attribute_list_workflow = strcmp(test_case->case_id, "test-op-get-attribute-list") == 0;
    bool get_constraints_workflow = strcmp(test_case->case_id, "test-op-get-constraints") == 0;
    bool get_usage_allocation_workflow =
        strcmp(test_case->case_id, "test-op-get-usage-allocation") == 0 ||
        get_usage_allocation_uid_variant;
    bool get_workflow = strcmp(test_case->case_id, "test-op-get") == 0;
    bool batch_two_read_items_workflow =
        strcmp(test_case->case_id, "test-batch-count-matches-items") == 0 ||
        strcmp(test_case->case_id, "test-batch-unique-item-id-response-correlation") == 0 ||
        strcmp(test_case->case_id, "test-batch-order-false-correlates-response-items") == 0 ||
        strcmp(test_case->case_id, "test-batch-continue-after-failure") == 0;
    bool continue_after_failure_workflow =
        strcmp(test_case->case_id, "test-batch-continue-after-failure") == 0;
    bool single_item_batch_options_workflow =
        strcmp(test_case->case_id, "test-batch-single-item-options-omitted") == 0;
    bool check_workflow = strcmp(test_case->case_id, "test-op-check") == 0;
    bool private_key_workflow = strcmp(test_case->case_id, "test-obj-private-key") == 0;
    bool public_key_workflow = strcmp(test_case->case_id, "test-obj-public-key") == 0;
    bool rsa_key_workflow = private_key_workflow || public_key_workflow;
    bool create_storage_masks_workflow =
        strcmp(test_case->case_id, "test-norm-payload-client-create-request-21-003") == 0;
    bool create_payload_workflow =
        strcmp(test_case->case_id, "test-norm-payload-client-create-request-20-001") == 0 ||
        strcmp(test_case->case_id, "test-norm-payload-client-create-request-20-002") == 0 ||
        strcmp(test_case->case_id, "test-norm-payload-client-create-request-21-001") == 0 ||
        strcmp(test_case->case_id, "test-norm-payload-client-create-request-21-002") == 0 ||
        create_storage_masks_workflow;
    bool create_workflow = strcmp(test_case->case_id, "test-op-create") == 0 ||
        strcmp(test_case->case_id, "test-obj-symmetric-key") == 0 || rsa_key_workflow ||
        create_payload_workflow || register_object_workflow;
    uint32_t requested_object_type = private_key_workflow ?
        KMIPKIT_ENUMERATION_OBJECT_TYPE_PRIVATE_KEY : public_key_workflow ?
        KMIPKIT_ENUMERATION_OBJECT_TYPE_PUBLIC_KEY :
        register_certificate_request_workflow ? KMIPKIT_ENUMERATION_OBJECT_TYPE_CERTIFICATE_REQUEST :
        register_certificate_workflow ? KMIPKIT_ENUMERATION_OBJECT_TYPE_CERTIFICATE :
        KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY;
    bool revoke_workflow = strcmp(test_case->case_id, "test-op-revoke") == 0;
    bool activate_workflow = strcmp(test_case->case_id, "test-op-activate") == 0;
    bool destroy_workflow = strcmp(test_case->case_id, "test-op-destroy") == 0;
    bool active_destroy_negative_workflow =
        strcmp(test_case->case_id, "test-neg-destroy-active-without-revoke") == 0;
    bool revoke_cleanup_workflow = check_workflow || activate_workflow || destroy_workflow ||
        get_constraints_workflow || get_usage_allocation_workflow || active_destroy_negative_workflow ||
        add_attribute_workflow ||
        batch_create_activate_get_attribute_list_revoke_destroy_workflow;
    bool object_lifecycle_workflow = create_workflow ||
        batch_create_activate_get_attribute_list_revoke_destroy_workflow ||
        batch_placeholder_chain_workflow || revoke_workflow ||
        revoke_cleanup_workflow || get_workflow ||
        get_attributes_workflow || get_attribute_list_workflow || get_constraints_workflow ||
        get_usage_allocation_workflow || add_attribute_workflow;
    if (strcmp(test_case->operation, "Create") == 0 && !create_workflow &&
        !batch_create_activate_get_attribute_list_revoke_destroy_workflow &&
        !batch_placeholder_chain_workflow) {
        return outcome_not_run("Create variations are not yet mapped to isolated C fixtures");
    }
    if (strcmp(test_case->operation, "Check") == 0 && !check_workflow) {
        return outcome_not_run("Check variations are not yet mapped to isolated C fixtures");
    }
    if (strcmp(test_case->operation, "Activate") == 0 && !activate_workflow) {
        return outcome_not_run("Activate variations are not yet mapped to isolated C fixtures");
    }
    if (strcmp(test_case->operation, "Revoke") == 0 && !revoke_workflow) {
        return outcome_not_run("Revoke variations are not yet mapped to isolated C fixtures");
    }
    if (strcmp(test_case->operation, "Destroy") == 0 && !destroy_workflow &&
        !active_destroy_negative_workflow) {
        return outcome_not_run("Destroy variations are not yet mapped to isolated C fixtures");
    }
    if (strcmp(test_case->operation, "Get Constraints") == 0 &&
        !get_constraints_workflow && !get_constraints_boundary_workflow) {
        return outcome_not_run("Get Constraints variations are not yet mapped to isolated C fixtures");
    }
    if (strcmp(test_case->operation, "Get") == 0 && !get_workflow &&
        !single_item_batch_options_workflow && !batch_two_read_items_workflow) {
        return outcome_not_run("object-specific Get recipes are not yet mapped to isolated C fixtures");
    }
    if (strcmp(test_case->operation, "Get Attributes") == 0 && !get_attributes_workflow) {
        return outcome_not_run(
            "attribute-specific Get Attributes recipes are not yet mapped to isolated C fixtures");
    }
    if (strcmp(test_case->operation, "Get Attribute List") == 0 && !get_attribute_list_workflow) {
        return outcome_not_run(
            "attribute-specific Get Attribute List recipes are not yet mapped to isolated C fixtures");
    }
    if (strcmp(test_case->operation, "Add Attribute") == 0 && !add_attribute_workflow) {
        return outcome_not_run("Add Attribute variations are not yet mapped to isolated C fixtures");
    }

    kmipkit_client_batch_item_t *item = NULL;
    kmipkit_client_batch_item_t *second_item = NULL;
    kmipkit_client_batch_item_t *setup_item = NULL;
    kmipkit_client_batch_item_t *activate_item = NULL;
    kmipkit_client_batch_item_t *revoke_item = NULL;
    kmipkit_client_batch_item_t *cleanup_item = NULL;
    kmipkit_ttlv_structure_t *request_attributes = NULL;
    kmipkit_ttlv_structure_t *request_storage_masks = NULL;
    kmipkit_ttlv_value_t *request_data = NULL;
    int32_t status = 0;
    const char *request_construction_step = "typed request";
    char request_validation_diagnostic[256] = {0};
    if (get_constraints_boundary_workflow) {
        status = kmipkit_client_batch_item_get_constraints(&item);
    } else if (set_constraints_workflow) {
        request_construction_step = "Set Constraints structure";
        status = kmipkit_ttlv_structure_create(&request_attributes);
        if (status == 0) status = kmipkit_client_batch_item_set_constraints(
            request_attributes, &item);
        if (status == 0) status = set_batch_item_id(&item, "interop-set-constraints");
    } else if (set_defaults_workflow) {
        request_construction_step = "Set Defaults optional payload";
        status = kmipkit_client_batch_item_set_defaults(NULL, &item);
        if (status == 0) status = set_batch_item_id(&item, "interop-set-defaults");
    } else if (strcmp(test_case->operation, "Discover Versions") == 0) {
        status = kmipkit_client_batch_item_discover_versions(&item);
    } else if (strcmp(test_case->operation, "Ping") == 0 && client->protocol_minor == 1) {
        status = kmipkit_client_batch_item_ping(&item);
    } else if (strcmp(test_case->operation, "Locate") == 0) {
        status = kmipkit_ttlv_structure_create(&request_attributes);
        if (status == 0) {
            status = kmipkit_client_batch_item_locate(1, 3, 0, 0, 0, 0, 0, 0,
                                                      request_attributes, &item);
        }
    } else if (strcmp(test_case->operation, "Query") == 0) {
        const uint32_t query_function = KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_OPERATIONS;
        if (strcmp(test_case->case_id, "test-norm-payload-client-query-request-21-002") == 0) {
            static const uint8_t query_group[] = "KMIPKit-Test-Group";
            const kmipkit_byte_span_v1_t object_group = {
                .data = query_group,
                .length = sizeof(query_group) - 1
            };
            status = kmipkit_client_batch_item_query(
                &query_function, 1, 1, &object_group, 1, &item);
        } else {
            status = kmipkit_client_batch_item_query(&query_function, 1, 0, NULL, 0, &item);
        }
    } else if (strcmp(test_case->operation, "Log") == 0) {
        static const uint8_t message[] = "KMIPKit interoperability validation probe";
        status = kmipkit_client_batch_item_log(message, sizeof(message) - 1, &item);
    } else if (strcmp(test_case->operation, "Login") == 0) {
        status = kmipkit_client_batch_item_login(0, 0, 0, 0, NULL, &item);
    } else if (hash_workflow) {
        kmipkit_ttlv_value_t *algorithm = NULL;
        status = kmipkit_ttlv_structure_create(&request_attributes);
        if (status == 0) status = kmipkit_ttlv_value_enumeration(
            KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA_256, &algorithm);
        if (status == 0 && !append_ttlv_value(client, &request_attributes, UINT32_C(0x420038), &algorithm)) {
            status = KMIPKIT_ERROR_INVALID_INPUT;
        }
        if (algorithm) kmipkit_ttlv_value_release(algorithm);
        static const uint8_t input[] = {0x61, 0x62, 0x63};
        if (status == 0) status = kmipkit_ttlv_value_byte_string(client->limits, input, sizeof(input), &request_data);
        if (status == 0) status = kmipkit_client_batch_item_hash(request_attributes, request_data, NULL, 0, 0, &item);
    } else if (rng_retrieve_workflow) {
        status = kmipkit_client_batch_item_rng_retrieve(rng_length, &item);
    } else if (rng_seed_workflow) {
        static const uint8_t seed[] = {
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31
        };
        const kmipkit_byte_span_v1_t span = {.data = seed, .length = rng_length};
        status = kmipkit_client_batch_item_rng_seed(&span, &item);
    } else if (single_item_batch_options_workflow) {
        static const uint8_t fixture_identifier[] = "fixture-key-1";
        const kmipkit_unique_identifier_v1_t identifier = {
            .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
            .abi_version = 1,
            .kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
            .text_data = fixture_identifier,
            .text_length = sizeof(fixture_identifier) - 1
        };
        status = kmipkit_client_batch_item_get(&identifier, 0, 0, 0, 0, 0, 0, NULL, &item);
        if (status == 0) status = set_batch_item_id(&item, "interop-get");
        if (status == 0 && !request_item_is_valid(
                item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
            status = KMIPKIT_ERROR_INVALID_INPUT;
        }
    } else if (batch_two_read_items_workflow) {
        static const uint8_t fixture_identifier[] = "fixture-key-1";
        static const uint8_t missing_identifier[] = "missing-key";
        const uint8_t *get_identifier_data = continue_after_failure_workflow ?
            missing_identifier : fixture_identifier;
        const uint64_t get_identifier_length = continue_after_failure_workflow ?
            sizeof(missing_identifier) - 1 : sizeof(fixture_identifier) - 1;
        const kmipkit_unique_identifier_v1_t get_identifier = {
            .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
            .abi_version = 1,
            .kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
            .text_data = get_identifier_data,
            .text_length = get_identifier_length
        };
        const kmipkit_unique_identifier_v1_t attribute_list_identifier = {
            .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
            .abi_version = 1,
            .kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
            .text_data = fixture_identifier,
            .text_length = sizeof(fixture_identifier) - 1
        };
        status = kmipkit_client_batch_item_get(
            &get_identifier, 0, 0, 0, 0, 0, 0, NULL, &item);
        if (status == 0) status = build_get_attribute_list_item(
            &attribute_list_identifier, &second_item);
        if (status == 0) status = set_batch_item_id(&item, "item-a");
        if (status == 0) status = set_batch_item_id(&second_item, "item-b");
        if (status == 0 && (!request_item_is_valid(
                item, request_validation_diagnostic, sizeof(request_validation_diagnostic)) ||
                !request_item_is_valid(
                second_item, request_validation_diagnostic, sizeof(request_validation_diagnostic)))) {
            status = KMIPKIT_ERROR_INVALID_INPUT;
        }
    } else if (object_lifecycle_workflow) {
        kmipkit_unique_identifier_v1_t placeholder = {
            .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
            .abi_version = 1,
            .kind = KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION,
            .enumeration_value = KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_ID_PLACEHOLDER
        };
        request_construction_step = register_certificate_request_workflow ? "Register Certificate Request" :
            register_certificate_workflow ? "Register Certificate" :
            rsa_key_workflow ? "RSA key Attributes" : "symmetric key Attributes";
        status = (register_certificate_workflow || register_certificate_request_workflow) ? 0 : (rsa_key_workflow ?
            create_rsa_key_attributes(client, &request_attributes) :
            create_symmetric_key_attributes(client, &request_attributes)) ? 0 : -1;
        if (status == 0 && create_storage_masks_workflow) {
            status = kmipkit_ttlv_structure_create(&request_storage_masks);
        }
        if (status == 0 && (register_certificate_workflow || register_certificate_request_workflow)) {
            status = register_certificate_request_workflow ?
                build_certificate_request_register_item(
                    client, register_object_path, create_workflow ? &item : &setup_item) :
                build_certificate_register_item(
                    client, register_object_path, create_workflow ? &item : &setup_item);
            if (status == 0 && !request_item_is_valid(
                    create_workflow ? item : setup_item,
                    request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
        } else if (status == 0) {
            request_construction_step = rsa_key_workflow ? "Create RSA Key" :
                "Create Symmetric Key";
            kmipkit_client_batch_item_t **create_item = create_workflow ? &item : &setup_item;
            status = kmipkit_client_batch_item_create(
                requested_object_type, request_attributes, request_storage_masks, create_item);
            if (status == 0 && !request_item_is_valid(
                    *create_item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
        }
        if (status == 0 && !create_workflow) {
            request_construction_step = "operation using ID Placeholder";
            if (batch_create_activate_get_attribute_list_revoke_destroy_workflow) {
                status = kmipkit_client_batch_item_activate(&placeholder, &activate_item);
                if (status == 0 && !request_item_is_valid(
                        activate_item, request_validation_diagnostic,
                        sizeof(request_validation_diagnostic))) {
                    status = KMIPKIT_ERROR_INVALID_INPUT;
                }
                if (status == 0) status = build_get_attribute_list_item(NULL, &item);
                if (status == 0 && !request_item_is_valid(
                        item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                    status = KMIPKIT_ERROR_INVALID_INPUT;
                }
            } else if (batch_placeholder_chain_workflow) {
                status = kmipkit_client_batch_item_activate(&placeholder, &activate_item);
                if (status == 0 && !request_item_is_valid(
                        activate_item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                    status = KMIPKIT_ERROR_INVALID_INPUT;
                }
                if (status == 0) status = kmipkit_client_batch_item_get(
                    &placeholder, 0, 0, 0, 0, 0, 0, NULL, &item);
                if (status == 0 && !request_item_is_valid(
                        item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                    status = KMIPKIT_ERROR_INVALID_INPUT;
                }
            } else if (active_destroy_negative_workflow) {
                status = kmipkit_client_batch_item_activate(&placeholder, &activate_item);
                if (status == 0 && !request_item_is_valid(
                        activate_item, request_validation_diagnostic,
                        sizeof(request_validation_diagnostic))) {
                    status = KMIPKIT_ERROR_INVALID_INPUT;
                }
                if (status == 0) {
                    status = kmipkit_client_batch_item_destroy(&placeholder, &item);
                }
            } else if (check_workflow) {
                status = kmipkit_client_batch_item_check(
                    &placeholder, 0, 0, 0, 0, 0, 0, &item);
            } else if (activate_workflow) {
                status = kmipkit_client_batch_item_activate(&placeholder, &item);
            } else if (revoke_workflow) {
                request_construction_step = "Revoke target operation";
                status = create_revocation_item(&placeholder, &item);
            } else if (destroy_workflow) {
                status = kmipkit_client_batch_item_destroy(&placeholder, &item);
            } else if (get_constraints_workflow) {
                status = kmipkit_client_batch_item_get_constraints(&item);
            } else if (get_usage_allocation_workflow) {
                if (get_usage_allocation_uid_variant) {
                    request_construction_step =
                        "Get Usage Allocation optional Unique Identifier variant";
                    status = create_get_usage_allocation_variant_item(
                        usage_allocation_identifier_explicit, &item);
                } else {
                    status = kmipkit_client_batch_item_get_usage_allocation(
                        &placeholder, 1, &item);
                }
            } else if (get_workflow) {
                status = kmipkit_client_batch_item_get(&placeholder, 0, 0, 0, 0, 0, 0, NULL, &item);
            } else if (get_attributes_workflow) {
                if ((read_back_attribute_workflow && !attribute_reference_by_all) ||
                    cryptographic_algorithm_attribute_workflow ||
                    cryptographic_length_attribute_workflow) {
                    kmipkit_attribute_reference_v1_t attribute_reference = {
                        .struct_size = (uint32_t)sizeof(kmipkit_attribute_reference_v1_t),
                        .abi_version = 1,
                        .kind = attribute_reference_by_name ? KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME :
                            KMIPKIT_ATTRIBUTE_REFERENCE_KIND_TAG,
                        .tag_value = read_back_attribute_workflow ? read_back_attribute_tag :
                            cryptographic_algorithm_attribute_workflow ? UINT32_C(0x420028) :
                            UINT32_C(0x42002A)
                    };
                    if (attribute_reference_by_name) {
                        attribute_reference.vendor_identification_data =
                            (const uint8_t *)test_case->attribute_vendor_identification;
                        attribute_reference.vendor_identification_length =
                            (uint64_t)strlen(test_case->attribute_vendor_identification);
                        attribute_reference.attribute_name_data =
                            (const uint8_t *)test_case->attribute_reference_name;
                        attribute_reference.attribute_name_length =
                            (uint64_t)strlen(test_case->attribute_reference_name);
                    }
                    status = build_get_attributes_item(
                        NULL, &attribute_reference, 1, &item);
                } else {
                    status = build_get_attributes_item(NULL, NULL, 0, &item);
                }
            } else if (add_attribute_workflow) {
                status = build_comment_attribute_mutation_item(
                    client, &placeholder, false, &item);
            } else {
                status = build_get_attribute_list_item(NULL, &item);
            }
            if (status == 0 && !request_item_is_valid(
                    item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
        }
        if (status == 0 && revoke_cleanup_workflow) {
            request_construction_step = "Revoke cleanup using ID Placeholder";
            status = create_revocation_item(&placeholder, &revoke_item);
            if (status == 0 && !request_item_is_valid(
                    revoke_item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
        }
        if (status == 0 && !batch_placeholder_chain_workflow) {
            request_construction_step = "Destroy using ID Placeholder";
            status = kmipkit_client_batch_item_destroy(&placeholder, &cleanup_item);
            if (status == 0 && !request_item_is_valid(
                    cleanup_item, request_validation_diagnostic, sizeof(request_validation_diagnostic))) {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
        }
        if (status == 0) {
            request_construction_step = "Unique Batch Item IDs";
            if (create_workflow) {
                status = set_batch_item_id(&item,
                    register_object_workflow ? "interop-register" : "interop-create");
            } else {
                status = set_batch_item_id(&setup_item,
                    register_certificate_workflow ? "interop-register" :
                    batch_placeholder_chain_workflow ? "placeholder-create" : "interop-create");
                if (status == 0 && (active_destroy_negative_workflow ||
                                    batch_placeholder_chain_workflow ||
                                    batch_create_activate_get_attribute_list_revoke_destroy_workflow)) {
                    status = set_batch_item_id(&activate_item,
                        active_destroy_negative_workflow ? "interop-activate-before-destroy" :
                        batch_placeholder_chain_workflow ? "placeholder-activate" : "interop-activate");
                }
                if (status == 0) status = set_batch_item_id(
                    &item, batch_placeholder_chain_workflow ? "placeholder-get" :
                           active_destroy_negative_workflow ? "interop-destroy-active-target" :
                           check_workflow ? "interop-check" :
                           activate_workflow ? "interop-activate" :
                           revoke_workflow ? "interop-revoke" :
                           destroy_workflow ? "interop-destroy-target" :
                           get_constraints_workflow ? "interop-get-constraints" :
                           get_usage_allocation_uid_variant
                               ? (usage_allocation_identifier_explicit
                                   ? "usage-allocation-explicit" : "usage-allocation-omitted") :
                           get_usage_allocation_workflow ? "interop-get-usage-allocation" :
                           get_workflow ? "interop-get" : "interop-attributes");
                if (status == 0 && revoke_cleanup_workflow) {
                    status = set_batch_item_id(&revoke_item,
                        (destroy_workflow || active_destroy_negative_workflow) ?
                            "interop-revoke-cleanup" : "interop-revoke");
                }
            }
            if (status == 0 && !batch_placeholder_chain_workflow) status = set_batch_item_id(&cleanup_item,
                (destroy_workflow || active_destroy_negative_workflow) ?
                    "interop-destroy-cleanup" : "interop-destroy");
        }
    } else {
        return outcome_not_run("no installed typed C request route is mapped for this case");
    }
    if (request_data) {
        kmipkit_ttlv_value_release(request_data);
        request_data = NULL;
    }
    if (status != 0) {
        kmip_execution_outcome_t outcome = outcome_not_run("KMIPKit rejected typed request construction");
        int32_t last_status = 0;
        uint32_t category = 0;
        uint32_t cause = 0;
        uint32_t delivery = 0;
        (void)kmipkit_last_error_get(&last_status, &category, &cause, &delivery);
        outcome.executed = true;
        outcome.library_failed = true;
        if (request_validation_diagnostic[0] != '\0') {
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit request validation failed at %.64s: %.128s",
                     request_construction_step, request_validation_diagnostic);
        } else {
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit request construction failed at %s (status %d, category=%u cause=%u)",
                     request_construction_step, status, category, cause);
        }
        if (request_attributes) kmipkit_ttlv_structure_release(request_attributes);
        return outcome;
    }

    kmipkit_client_batch_t *batch = NULL;
    kmipkit_client_batch_response_t *response = NULL;
    kmipkit_client_batch_item_response_t *response_item = NULL;
    kmipkit_client_batch_item_response_t *second_response_item = NULL;
    kmipkit_client_batch_item_response_t *setup_response_item = NULL;
    kmipkit_client_batch_item_response_t *activate_response_item = NULL;
    kmipkit_client_batch_item_response_t *revoke_response_item = NULL;
    kmipkit_client_batch_item_response_t *cleanup_response_item = NULL;
    kmipkit_ttlv_structure_t *response_attributes = NULL;
    kmipkit_unique_identifier_v1_t created_identifier = {
        .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
        .abi_version = 1
    };
    uint8_t created_identifier_present = 0;
    kmip_execution_outcome_t outcome = { .executed = true };
    status = kmipkit_client_batch_create(&batch);
    if (status == 0 && single_item_batch_options_workflow) {
        status = kmipkit_client_batch_add_item(batch, &item);
    } else if (status == 0 && batch_two_read_items_workflow) {
        if (strcmp(test_case->case_id, "test-batch-order-false-correlates-response-items") == 0) {
            status = kmipkit_client_batch_set_order_option(batch, 0);
        }
        if (status == 0 && strcmp(test_case->case_id,
                                   "test-batch-continue-after-failure") == 0) {
            status = kmipkit_client_batch_add_error_continuation_value(
                batch, KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_CONTINUE);
        }
        if (status == 0) status = kmipkit_client_batch_add_item(batch, &item);
        if (status == 0) status = kmipkit_client_batch_add_item(batch, &second_item);
    } else if (status == 0 && create_workflow) {
        status = kmipkit_client_batch_set_order_option(batch, 1);
        if (status == 0) status = kmipkit_client_batch_add_error_continuation_value(
            batch, KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_CONTINUE);
        if (status == 0) status = kmipkit_client_batch_add_item(batch, &item);
        if (status == 0) status = kmipkit_client_batch_add_item(batch, &cleanup_item);
    } else if (status == 0 && object_lifecycle_workflow) {
        status = kmipkit_client_batch_set_order_option(batch, 1);
        if (status == 0) status = kmipkit_client_batch_add_error_continuation_value(
            batch, KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_CONTINUE);
        if (status == 0) status = kmipkit_client_batch_add_item(batch, &setup_item);
        if (status == 0 && (active_destroy_negative_workflow ||
                            batch_placeholder_chain_workflow ||
                            batch_create_activate_get_attribute_list_revoke_destroy_workflow)) {
            status = kmipkit_client_batch_add_item(batch, &activate_item);
        }
        if (status == 0) status = kmipkit_client_batch_add_item(batch, &item);
        if (status == 0 && revoke_cleanup_workflow) {
            status = kmipkit_client_batch_add_item(batch, &revoke_item);
        }
        if (status == 0 && !batch_placeholder_chain_workflow) {
            status = kmipkit_client_batch_add_item(batch, &cleanup_item);
        }
    } else if (status == 0) {
        status = kmipkit_client_batch_add_item(batch, &item);
    }
    if (status == 0 && client->username_env[0] != '\0') {
        kmipkit_authentication_t *authentication = NULL;
        if (!create_authentication(client, &authentication)) {
            outcome.library_failed = true;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit could not construct configured Username/Password authentication");
            goto cleanup;
        }
        status = kmipkit_client_batch_set_authentication(batch, authentication);
        if (status != 0) {
            kmipkit_authentication_release(authentication);
            outcome.library_failed = true;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit rejected configured authentication (status %d)", status);
            goto cleanup;
        }
    }
    if (status != 0) {
        outcome.library_failed = true;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit could not assemble the typed request batch (status %d)", status);
        goto cleanup;
    }
    status = kmipkit_client_execute(client->client, &batch, client->limits, &response);
    if (status != 0) {
        int32_t last_status = 0;
        uint32_t category = 0;
        uint32_t cause = 0;
        uint32_t delivery = 0;
        (void)kmipkit_last_error_get(&last_status, &category, &cause, &delivery);
        if (get_constraints_boundary_workflow && delivery == 1) {
            outcome.success = true;
            outcome.library_validated = true;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit rejected the KMIP 2.1-only operation before sending under KMIP 2.0");
            goto cleanup;
        }
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit exchange failed with status %d (category=%u cause=%u delivery=%u)",
                 status, category, cause, delivery);
        outcome.success = false;
        goto cleanup;
    }

    uint64_t response_count = 0;
    status = kmipkit_client_batch_response_item_count(response, &response_count);
    uint64_t expected_response_count = batch_two_read_items_workflow ? 2 : create_workflow ? 2 :
        batch_placeholder_chain_workflow ? 3 :
        (active_destroy_negative_workflow ||
         batch_create_activate_get_attribute_list_revoke_destroy_workflow) ? 5 :
        revoke_cleanup_workflow ? 4 : object_lifecycle_workflow ? 3 : 1;
    if (status != 0 || response_count != expected_response_count) {
        outcome.library_failed = true;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit returned an unexpected response item count");
        goto cleanup;
    }
    if (batch_two_read_items_workflow) {
        bool get_matched = false;
        bool attribute_list_matched = false;
        for (uint64_t index = 0; status == 0 && index < 2; ++index) {
            kmipkit_client_batch_item_response_t **response_slot =
                index == 0 ? &response_item : &second_response_item;
            kmipkit_client_batch_item_metadata_v2_t metadata = {
                .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
                .abi_version = 2
            };
            uint8_t id_present = 0;
            kmipkit_byte_span_v1_t id = {0};
            status = kmipkit_client_batch_response_item_at(response, index, response_slot);
            if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
                *response_slot, &metadata);
            if (status == 0) status = kmipkit_client_batch_item_response_unique_batch_item_id(
                *response_slot, &id_present, &id);
            if (status != 0 || !id_present || !id.data) break;
            bool expected_get_status = continue_after_failure_workflow &&
                metadata.operation == KMIPKIT_ENUMERATION_OPERATION_GET &&
                metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_OPERATION_FAILED &&
                metadata.has_result_reason != 0 &&
                metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_OBJECT_NOT_FOUND;
            if (id.length == sizeof("item-a") - 1 &&
                memcmp(id.data, "item-a", sizeof("item-a") - 1) == 0 &&
                metadata.operation == KMIPKIT_ENUMERATION_OPERATION_GET &&
                (expected_get_status || (!continue_after_failure_workflow &&
                 metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS))) {
                get_matched = true;
            } else if (id.length == sizeof("item-b") - 1 &&
                       memcmp(id.data, "item-b", sizeof("item-b") - 1) == 0 &&
                       metadata.operation == KMIPKIT_ENUMERATION_OPERATION_GET_ATTRIBUTE_LIST &&
                       metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) {
                attribute_list_matched = true;
            } else {
                status = KMIPKIT_ERROR_INVALID_INPUT;
            }
        }
        if (status != 0 || !get_matched || !attribute_list_matched) {
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not correlate both read responses by Unique Batch Item ID");
            goto cleanup;
        }
        outcome.success = true;
        outcome.library_validated = true;
        goto cleanup;
    }
    if (batch_placeholder_chain_workflow) {
        kmipkit_client_batch_item_metadata_v2_t create_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        kmipkit_client_batch_item_metadata_v2_t activate_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        kmipkit_client_batch_item_metadata_v2_t get_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        status = kmipkit_client_batch_response_item_at(response, 0, &setup_response_item);
        if (status == 0) status = kmipkit_client_batch_response_item_at(
            response, 1, &activate_response_item);
        if (status == 0) status = kmipkit_client_batch_response_item_at(response, 2, &response_item);
        if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
            setup_response_item, &create_metadata);
        if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
            activate_response_item, &activate_metadata);
        if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
            response_item, &get_metadata);
        uint32_t response_object_type = 0;
        if (status == 0 && create_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS) {
            status = kmipkit_client_batch_item_response_create(
                setup_response_item, &created_identifier_present,
                &response_object_type, &created_identifier);
        }
        bool create_response_valid = status == 0 && created_identifier_present &&
            response_object_type == requested_object_type &&
            created_identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
            created_identifier.text_length != 0 && created_identifier.text_data &&
            create_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS &&
            activate_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS &&
            get_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS;
        bool activate_response_valid = create_response_valid &&
            activate_response_matches_created_key(activate_response_item, &created_identifier, &status);
        bool get_response_valid = activate_response_valid &&
            get_response_matches_created_key(response_item, &created_identifier, &status);
        bool chain_resolved = create_response_valid && activate_response_valid && get_response_valid;
        if (!chain_resolved) {
            outcome.library_failed = status != 0;
            outcome.library_validated = status == 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not resolve the Activate/Get placeholder chain to one object");
            goto cleanup;
        }
        // The normative batch ends at Get; clean up in a second request with the returned ID.
        if (!cleanup_placeholder_chain_object(client, &created_identifier, true, &status)) {
            outcome.library_failed = status != 0;
            outcome.library_validated = status == 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not complete Revoke/Destroy fixture cleanup");
            goto cleanup;
        }
        outcome.success = true;
        outcome.library_validated = true;
        goto cleanup;
    }
    if (object_lifecycle_workflow) {
        kmipkit_client_batch_item_metadata_v2_t setup_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        kmipkit_client_batch_item_metadata_v2_t cleanup_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        kmipkit_client_batch_item_metadata_v2_t revoke_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        kmipkit_client_batch_item_metadata_v2_t activate_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        kmipkit_client_batch_item_metadata_v2_t target_metadata = {
            .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
            .abi_version = 2
        };
        status = kmipkit_client_batch_response_item_at(response, 0, &setup_response_item);
        if (status == 0 && (active_destroy_negative_workflow ||
                            batch_create_activate_get_attribute_list_revoke_destroy_workflow)) {
            status = kmipkit_client_batch_response_item_at(response, 1, &activate_response_item);
            if (status == 0) status = kmipkit_client_batch_response_item_at(response, 2, &response_item);
        }
        if (status == 0 && destroy_workflow) {
            status = kmipkit_client_batch_response_item_at(response, 1, &response_item);
            if (status == 0) status = kmipkit_client_batch_response_item_at(
                response, 2, &revoke_response_item);
            if (status == 0) status = kmipkit_client_batch_response_item_at(
                response, 3, &cleanup_response_item);
        } else if (status == 0 && (active_destroy_negative_workflow ||
                                   batch_create_activate_get_attribute_list_revoke_destroy_workflow)) {
            status = kmipkit_client_batch_response_item_at(response, 3, &revoke_response_item);
            if (status == 0) status = kmipkit_client_batch_response_item_at(
                response, 4, &cleanup_response_item);
        } else if (status == 0 && revoke_cleanup_workflow) {
            status = kmipkit_client_batch_response_item_at(response, 2, &revoke_response_item);
            if (status == 0) status = kmipkit_client_batch_response_item_at(
                response, 3, &cleanup_response_item);
        }
        if (status == 0 && object_lifecycle_workflow && !create_workflow &&
            !revoke_cleanup_workflow) {
            status = kmipkit_client_batch_response_item_at(response, 2, &cleanup_response_item);
        } else if (status == 0 && create_workflow) {
            status = kmipkit_client_batch_response_item_at(response, 1, &cleanup_response_item);
        }
        if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
            setup_response_item, &setup_metadata);
        if (status == 0) status = kmipkit_client_batch_item_response_metadata_v2(
            cleanup_response_item, &cleanup_metadata);
        if (status == 0 && revoke_cleanup_workflow) {
            status = kmipkit_client_batch_item_response_metadata_v2(
                revoke_response_item, &revoke_metadata);
        }
        if (status == 0 && (active_destroy_negative_workflow ||
                            batch_create_activate_get_attribute_list_revoke_destroy_workflow)) {
            status = kmipkit_client_batch_item_response_metadata_v2(
                activate_response_item, &activate_metadata);
        }
        if (status == 0 && destroy_workflow) {
            status = kmipkit_client_batch_item_response_metadata_v2(response_item, &target_metadata);
        } else if (status == 0 && (active_destroy_negative_workflow ||
                                   batch_create_activate_get_attribute_list_revoke_destroy_workflow)) {
            status = kmipkit_client_batch_item_response_metadata_v2(response_item, &target_metadata);
        }
        if (status != 0) {
            outcome.library_failed = true;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit could not read setup or cleanup result metadata (status %d)", status);
            goto cleanup;
        }
        bool target_destroy_succeeded = (destroy_workflow || active_destroy_negative_workflow) &&
            target_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS;
        bool revoke_cleanup_acceptable = !revoke_cleanup_workflow ||
            revoke_metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
            (target_destroy_succeeded && revoke_metadata.has_result_reason != 0 &&
             revoke_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_OBJECT_NOT_FOUND);
        bool destroy_cleanup_acceptable = cleanup_metadata.result_status ==
                KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
            (target_destroy_succeeded && cleanup_metadata.has_result_reason != 0 &&
             cleanup_metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_OBJECT_NOT_FOUND);
        if (setup_metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
            !revoke_cleanup_acceptable || !destroy_cleanup_acceptable) {
            const kmipkit_client_batch_item_metadata_v2_t *failure =
                setup_metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS
                    ? &setup_metadata :
                !revoke_cleanup_acceptable
                    ? &revoke_metadata : &cleanup_metadata;
            outcome.library_validated = true;
            outcome.has_result_status_code = true;
            outcome.result_status_code = failure->result_status;
            outcome.has_result_reason_code = failure->has_result_reason != 0;
            outcome.result_reason_code = failure->result_reason;
            outcome.server_unsupported = failure->has_result_reason != 0 &&
                (failure->result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_OPERATION_NOT_SUPPORTED ||
                 failure->result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_FEATURE_NOT_SUPPORTED);
            outcome.policy_restricted = failure->has_result_reason != 0 &&
                (failure->result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_NOT_AUTHORISED ||
                 failure->result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_PERMISSION_DENIED);
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "server rejected an isolated object lifecycle batch (status=%u reason=%u)",
                     failure->result_status, failure->result_reason);
            goto cleanup;
        }
        uint32_t response_object_type = 0;
        status = (register_certificate_workflow || register_certificate_request_workflow) ?
            kmipkit_client_batch_item_response_register(
                setup_response_item, &created_identifier_present, &created_identifier) :
            kmipkit_client_batch_item_response_create(
                setup_response_item, &created_identifier_present,
                &response_object_type, &created_identifier);
        if (status != 0 || !created_identifier_present ||
            (!(register_certificate_workflow || register_certificate_request_workflow) &&
             response_object_type != requested_object_type) ||
            created_identifier.kind != KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING ||
            created_identifier.text_length == 0 || !created_identifier.text_data) {
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not expose the typed Create/Register response for the isolated fixture");
            goto cleanup;
        }
        if ((register_certificate_workflow || register_certificate_request_workflow) &&
            !response_ttlv_matches_identifier(
                setup_response_item, kmipkit_client_batch_item_response_register_ttlv,
                &created_identifier, &status)) {
            outcome.library_failed = true;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Register response TTLV disagreed with its typed Unique Identifier");
            goto cleanup;
        }
        if (batch_create_activate_get_attribute_list_revoke_destroy_workflow &&
            (activate_metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
             !activate_response_matches_created_key(
                 activate_response_item, &created_identifier, &status) ||
             target_metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
             !get_attribute_list_response_matches_key(
                 response_item, &created_identifier, &status))) {
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit batch did not confirm activation and attribute-list read-back");
            goto cleanup;
        }
        if (!destroy_workflow &&
            !destroy_response_matches_key(cleanup_response_item, &created_identifier, &status)) {
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not confirm cleanup of the isolated test object");
            goto cleanup;
        }
        if (revoke_cleanup_workflow && !destroy_workflow && !revoke_response_matches_created_key(
                revoke_response_item, &created_identifier, &status)) {
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not confirm revocation of the isolated test object");
            goto cleanup;
        }
        if (active_destroy_negative_workflow &&
            (activate_metadata.result_status != KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS ||
             !activate_response_matches_created_key(
                activate_response_item, &created_identifier, &status))) {
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not confirm activation before the negative Destroy probe");
            goto cleanup;
        }
    }
    if (!response_item) {
        status = kmipkit_client_batch_response_item_at(
            response, object_lifecycle_workflow && !create_workflow ? 1 : 0, &response_item);
    }
    if (status != 0) {
        outcome.library_failed = true;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit could not read the batch result (status %d)", status);
        goto cleanup;
    }
    kmipkit_client_batch_item_metadata_v2_t metadata = {
        .struct_size = (uint32_t)sizeof(kmipkit_client_batch_item_metadata_v2_t),
        .abi_version = 2
    };
    status = kmipkit_client_batch_item_response_metadata_v2(response_item, &metadata);
    if (status != 0) {
        outcome.library_failed = true;
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "KMIPKit could not read KMIP result metadata (status %d)", status);
        goto cleanup;
    }
    outcome.has_result_status_code = true;
    outcome.library_validated = true;
    outcome.result_status_code = metadata.result_status;
    outcome.has_result_reason_code = metadata.has_result_reason != 0;
    outcome.result_reason_code = metadata.result_reason;
    outcome.success = metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS;
    if (!outcome.success && outcome.has_result_reason_code) {
        outcome.server_unsupported =
            metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_OPERATION_NOT_SUPPORTED ||
            metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_FEATURE_NOT_SUPPORTED;
        outcome.policy_restricted =
            metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_NOT_AUTHORISED ||
            metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_PERMISSION_DENIED;
    }
    if (active_destroy_negative_workflow) {
        outcome.success = metadata.result_status == KMIPKIT_ENUMERATION_RESULT_STATUS_OPERATION_FAILED &&
            metadata.has_result_reason != 0 &&
            metadata.result_reason == KMIPKIT_ENUMERATION_RESULT_REASON_WRONG_KEY_LIFECYCLE_STATE;
        if (!outcome.success) {
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "server did not reject Destroy with Wrong Key Lifecycle State");
        } else {
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "server rejected illegal Destroy and the isolated object was cleaned up");
        }
        goto cleanup;
    }
    if (!outcome.success) {
        snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                 "server result status=%u reason=%u",
                 metadata.result_status, metadata.result_reason);
        goto cleanup;
    }
    client_response_ttlv_accessor_t server_configuration_accessor =
        set_constraints_workflow ? kmipkit_client_batch_item_response_set_constraints_ttlv :
        set_defaults_workflow ? kmipkit_client_batch_item_response_set_defaults_ttlv : NULL;
    if (server_configuration_accessor) {
        bool valid = response_ttlv_matches_field_count(
            response_item, server_configuration_accessor, 4, &status);
        if (!valid) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit server configuration response TTLV did not preserve the complete success item");
            goto cleanup;
        }
    } else if (strcmp(test_case->operation, "Log") == 0) {
        bool valid = response_ttlv_matches_field_count(
            response_item, kmipkit_client_batch_item_response_log_ttlv, 3, &status);
        if (!valid) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Log response TTLV did not preserve the complete result item");
            goto cleanup;
        }
    } else if (strcmp(test_case->operation, "Discover Versions") == 0) {
        kmipkit_protocol_version_v1_t versions[16] = {0};
        uint8_t present = 0;
        uint64_t version_count = 0;
        status = kmipkit_client_batch_item_response_discover_versions_supported_versions(
            response_item, &present, versions, 16, &version_count);
        if (status != 0) {
            outcome.success = false;
            outcome.library_failed = true;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit could not read the typed Discover Versions payload (status %d)", status);
            goto cleanup;
        }
        bool configured_version_found = false;
        for (uint64_t index = 0; index < version_count; ++index) {
            if (versions[index].major == client->protocol_major &&
                versions[index].minor == client->protocol_minor) configured_version_found = true;
        }
        if (!present || !configured_version_found) {
            outcome.success = false;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "server did not advertise the configured KMIP protocol version");
        }
    } else if (strcmp(test_case->operation, "Login") == 0) {
        uint8_t present = 0;
        uint8_t ticket_present = 0;
        uint32_t ticket_type = 0;
        kmipkit_byte_span_v1_t ticket_value = {0};
        status = kmipkit_client_batch_item_response_login(
            response_item, &present, &ticket_present, &ticket_type, &ticket_value);
        if (status != 0 || !present || !ticket_present ||
            ticket_type != KMIPKIT_ENUMERATION_TICKET_TYPE_LOGIN ||
            ticket_value.length == 0 || !ticket_value.data) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not expose a valid typed Login response");
        }
    } else if (strcmp(test_case->operation, "Ping") == 0) {
        uint8_t present = 0;
        status = kmipkit_client_batch_item_response_ping(response_item, &present);
        if (status != 0 || !present) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not expose a valid typed Ping response");
        }
    } else if (strcmp(test_case->operation, "Locate") == 0) {
        uint8_t present = 0;
        uint8_t payload_present = 0;
        uint8_t located_items_present = 0;
        int32_t located_items = 0;
        uint64_t identifier_count = 0;
        status = kmipkit_client_batch_item_response_locate(
            response_item, &present, &payload_present, &located_items_present,
            &located_items, &identifier_count);
        bool valid_response = status == 0 && present && payload_present;
        if (!valid_response) {
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit could not read the typed Locate payload (status=%d present=%u payload=%u)",
                     status, present, payload_present);
        }
        if (valid_response && located_items_present && located_items < 0) {
            valid_response = false;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "server returned a negative Located Items value");
        }
        for (uint64_t index = 0; valid_response && index < identifier_count; ++index) {
            kmipkit_unique_identifier_v1_t identifier = {
                .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
                .abi_version = 1
            };
            status = kmipkit_client_batch_item_response_locate_unique_identifier_at(
                response_item, index, &identifier);
            bool valid_identifier = identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION ||
                identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER ||
                (identifier.kind == KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING &&
                 (identifier.text_length == 0 || identifier.text_data != NULL));
            valid_response = status == 0 && valid_identifier;
            if (!valid_response) {
                snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                         "KMIPKit could not read Locate identifier %llu (status=%d kind=%u)",
                         (unsigned long long)index, status, (unsigned int)identifier.kind);
            }
        }
        if (!valid_response) {
            outcome.success = false;
            outcome.library_failed = status != 0;
        }
    } else if (strcmp(test_case->operation, "Query") == 0) {
        uint8_t present = 0;
        uint64_t field_count = 0;
        status = kmipkit_client_batch_item_response_query_field_count(
            response_item, &present, &field_count);
        if (status != 0 || !present || field_count == 0) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit did not expose Query response fields");
            goto cleanup;
        }
        bool operation_field_found = false;
        for (uint64_t index = 0; index < field_count; ++index) {
            uint8_t field_present = 0;
            uint8_t field_kind = 0;
            uint32_t field_tag = 0;
            uint32_t enumeration = 0;
            kmipkit_ttlv_item_t *field_item = NULL;
            status = kmipkit_client_batch_item_response_query_field_at(
                response_item, index, &field_present, &field_tag, &field_kind,
                &enumeration, &field_item);
            bool field_has_item = field_item != NULL;
            bool valid_field = status == 0 && field_present && field_kind >= 1 && field_kind <= 4 &&
                ((field_kind == 1 && !field_has_item) ||
                 (field_kind != 1 && field_has_item));
            if (valid_field && field_tag == UINT32_C(0x42005C) &&
                field_kind == 1 && enumeration != 0) {
                operation_field_found = true;
            }
            if (field_item) kmipkit_ttlv_item_release(field_item);
            if (!valid_field) {
                outcome.success = false;
                outcome.library_failed = status != 0 || field_present != 0;
                snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                         "KMIPKit returned a malformed typed Query response field");
                goto cleanup;
            }
        }
        int32_t query_ttlv_status = 0;
        if (!query_response_ttlv_matches_field_count(
                response_item, field_count, &query_ttlv_status)) {
            status = query_ttlv_status;
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Query response TTLV disagreed with its typed field count");
            goto cleanup;
        }
        if (!operation_field_found) {
            outcome.success = false;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Query response omitted Operation enumeration fields");
        }
    } else if (hash_workflow || rng_retrieve_workflow || rng_seed_workflow) {
        uint8_t present = 0;
        uint8_t data_present = 0;
        kmipkit_byte_span_v1_t data = {0};
        bool valid = false;
        if (hash_workflow) {
            uint8_t correlation_present = 0;
            kmipkit_byte_span_v1_t correlation = {0};
            static const uint8_t expected_sha256[] = {
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea,
                0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
                0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
                0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad
            };
            status = kmipkit_client_batch_item_response_hash(response_item, &present, &data_present,
                &data, &correlation_present, &correlation);
            valid = status == 0 && present && data_present && !correlation_present &&
                data.data && data.length == sizeof(expected_sha256) &&
                memcmp(data.data, expected_sha256, sizeof(expected_sha256)) == 0;
            if (valid) status = kmipkit_client_batch_item_response_hash_ttlv(response_item, &present, &response_attributes);
        } else if (rng_retrieve_workflow) {
            status = kmipkit_client_batch_item_response_rng_retrieve(response_item, &present, &data_present, &data);
            valid = status == 0 && present && data_present && data.data && data.length == rng_length;
            for (uint64_t index = 0; valid && index < data.length; ++index) {
                valid = data.data[index] == (uint8_t)index;
            }
            if (valid) status = kmipkit_client_batch_item_response_rng_retrieve_ttlv(response_item, &present, &response_attributes);
        } else {
            int32_t data_length = 0;
            status = kmipkit_client_batch_item_response_rng_seed(response_item, &present, &data_present, &data_length);
            valid = status == 0 && present && data_present && data_length == (int32_t)rng_length;
            if (valid) status = kmipkit_client_batch_item_response_rng_seed_ttlv(response_item, &present, &response_attributes);
        }
        if (!valid || status != 0 || !present || !response_attributes) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "typed Hash/RNG response did not match the deterministic fixture");
        }
    } else if (get_constraints_workflow) {
        if (!get_constraints_response_is_typed(response_item, &status)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Get Constraints did not return a typed Constraints structure");
            goto cleanup;
        }
    } else if (get_usage_allocation_workflow) {
        if (!get_usage_allocation_response_matches_identifier(
                response_item, &created_identifier, &status) ||
            !get_usage_allocation_response_ttlv_matches_identifier(
                response_item, &created_identifier, &status)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Get Usage Allocation typed or TTLV response lost the isolated object identifier");
            goto cleanup;
        }
    } else if (destroy_workflow) {
        if (!destroy_response_matches_key(response_item, &created_identifier, &status)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Destroy did not confirm deletion of the created object");
            goto cleanup;
        }
    } else if (revoke_workflow) {
        if (!revoke_response_matches_created_key(response_item, &created_identifier, &status)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Revoke did not return the created object's identifier");
            goto cleanup;
        }
    } else if (activate_workflow) {
        if (!activate_response_matches_created_key(response_item, &created_identifier, &status)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Activate did not return the created object's identifier");
            goto cleanup;
        }
    } else if (check_workflow) {
        if (!check_response_matches_created_key(response_item, &created_identifier, &status)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Check did not return the created object's identifier");
            goto cleanup;
        }
    } else if (get_workflow) {
        if (!get_response_matches_created_key(response_item, &created_identifier, &status)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Get did not return the created AES key object");
            goto cleanup;
        }
    } else if (get_attributes_workflow) {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t response_identifier = {
            .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
            .abi_version = 1
        };
        status = kmipkit_client_batch_item_response_get_attributes(
            response_item, &present, &response_identifier, &response_attributes);
        bool attributes_valid = response_attributes &&
            (attribute_reference_by_all ?
                attributes_contain_field_list(response_attributes,
                                              test_case->attribute_response_fields) :
             read_back_attribute_workflow ?
                attributes_contain_typed_tag(response_attributes, read_back_attribute_response_tag,
                                             read_back_attribute_type) :
             cryptographic_algorithm_attribute_workflow ?
                attributes_contain_value(response_attributes, UINT32_C(0x420028),
                    KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION,
                    KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES, 0) :
             cryptographic_length_attribute_workflow ?
                attributes_contain_value(response_attributes, UINT32_C(0x42002A),
                    KMIPKIT_TTLV_ITEM_TYPE_INTEGER, 0, 128) :
                attributes_match_created_symmetric_key(response_attributes));
        bool valid_response = status == 0 && present && attributes_valid &&
            same_text_identifier(&created_identifier, &response_identifier);
        if (!valid_response) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Get Attributes did not return the requested attribute shape");
            goto cleanup;
        }
    } else if (get_attribute_list_workflow) {
        if (!get_attribute_list_response_matches_key(response_item, &created_identifier, &status)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Get Attribute List omitted the created AES-128 attributes");
            goto cleanup;
        }
    } else if (add_attribute_workflow) {
        uint8_t present = 0;
        kmipkit_unique_identifier_v1_t response_identifier = {
            .struct_size = (uint32_t)sizeof(kmipkit_unique_identifier_v1_t),
            .abi_version = 1
        };
        status = kmipkit_client_batch_item_response_add_attribute(
            response_item, &present, &response_identifier);
        if (status != 0 || !present ||
            !same_text_identifier(&created_identifier, &response_identifier)) {
            outcome.success = false;
            outcome.library_failed = status != 0;
            snprintf(outcome.result_reason, sizeof(outcome.result_reason),
                     "KMIPKit Add Attribute response did not return the created object's identifier");
            goto cleanup;
        }
    }

cleanup:
    if (response_attributes) kmipkit_ttlv_structure_release(response_attributes);
    if (setup_response_item) kmipkit_client_batch_item_response_release(setup_response_item);
    if (activate_response_item) kmipkit_client_batch_item_response_release(activate_response_item);
    if (revoke_response_item) kmipkit_client_batch_item_response_release(revoke_response_item);
    if (cleanup_response_item) kmipkit_client_batch_item_response_release(cleanup_response_item);
    if (response_item) kmipkit_client_batch_item_response_release(response_item);
    if (second_response_item) kmipkit_client_batch_item_response_release(second_response_item);
    if (response) kmipkit_client_batch_response_release(response);
    if (batch) kmipkit_client_batch_release(batch);
    if (item) kmipkit_client_batch_item_release(item);
    if (second_item) kmipkit_client_batch_item_release(second_item);
    if (setup_item) kmipkit_client_batch_item_release(setup_item);
    if (activate_item) kmipkit_client_batch_item_release(activate_item);
    if (revoke_item) kmipkit_client_batch_item_release(revoke_item);
    if (cleanup_item) kmipkit_client_batch_item_release(cleanup_item);
    if (request_attributes) kmipkit_ttlv_structure_release(request_attributes);
    if (request_storage_masks) kmipkit_ttlv_structure_release(request_storage_masks);
    return outcome;
}
