#include <stddef.h>
#include <stdint.h>

#include "kmipkit.h"

/* Private prototypes for KMIPKIT-0006 declarations withheld from the public header until
 * Java and Python parity is complete. The manifest-generated internal header will replace these. */
struct kmipkit_request_message_t;
struct kmipkit_request_header_view_t;
struct kmipkit_request_batch_item_view_t;
struct kmipkit_response_message_t;
struct kmipkit_response_header_view_t;
struct kmipkit_response_batch_item_view_t;
struct kmipkit_message_extension_view_t;
struct kmipkit_message_validation_error_t;

int32_t kmipkit_request_message_from_ttlv(
    kmipkit_ttlv_structure_t *structure,
    struct kmipkit_request_message_t **out_message,
    struct kmipkit_message_validation_error_t **out_validation_error);
int32_t kmipkit_request_message_header(
    struct kmipkit_request_message_t *message,
    struct kmipkit_request_header_view_t **out_header);
int32_t kmipkit_request_header_view_protocol_version(
    struct kmipkit_request_header_view_t *header,
    kmipkit_protocol_version_v1_t *out_version);
int32_t kmipkit_request_message_batch_item_count(
    struct kmipkit_request_message_t *message,
    uint64_t *out_count);
int32_t kmipkit_request_message_batch_item_at(
    struct kmipkit_request_message_t *message,
    uint64_t index,
    struct kmipkit_request_batch_item_view_t **out_item);
int32_t kmipkit_request_batch_item_view_operation(
    struct kmipkit_request_batch_item_view_t *item,
    uint8_t *out_present,
    uint32_t *out_operation);
void kmipkit_request_message_release(struct kmipkit_request_message_t *handle);
void kmipkit_request_header_view_release(struct kmipkit_request_header_view_t *handle);
void kmipkit_request_batch_item_release(struct kmipkit_request_batch_item_view_t *handle);
void kmipkit_message_validation_error_release(
    struct kmipkit_message_validation_error_t *handle);

int32_t kmipkit_response_message_from_ttlv(
    kmipkit_ttlv_structure_t *structure,
    struct kmipkit_response_message_t **out_message,
    struct kmipkit_message_validation_error_t **out_validation_error);
int32_t kmipkit_response_message_header(
    struct kmipkit_response_message_t *message,
    struct kmipkit_response_header_view_t **out_header);
int32_t kmipkit_response_header_view_protocol_version(
    struct kmipkit_response_header_view_t *header,
    kmipkit_protocol_version_v1_t *out_version);
int32_t kmipkit_response_header_view_time_stamp(
    struct kmipkit_response_header_view_t *header,
    int64_t *out_time_stamp);
int32_t kmipkit_response_message_batch_item_count(
    struct kmipkit_response_message_t *message,
    uint64_t *out_count);
int32_t kmipkit_response_message_batch_item_at(
    struct kmipkit_response_message_t *message,
    uint64_t index,
    struct kmipkit_response_batch_item_view_t **out_item);
int32_t kmipkit_response_batch_item_view_result_status(
    struct kmipkit_response_batch_item_view_t *item,
    uint8_t *out_present,
    uint32_t *out_status);
int32_t kmipkit_response_batch_item_extension_count(
    struct kmipkit_response_batch_item_view_t *item,
    uint64_t *out_count);
int32_t kmipkit_response_batch_item_extension_at(
    struct kmipkit_response_batch_item_view_t *item,
    uint64_t index,
    struct kmipkit_message_extension_view_t **out_extension);
int32_t kmipkit_message_extension_view_ttlv(
    struct kmipkit_message_extension_view_t *extension,
    kmipkit_ttlv_structure_t **out_structure);
void kmipkit_response_message_release(struct kmipkit_response_message_t *handle);
void kmipkit_response_header_view_release(struct kmipkit_response_header_view_t *handle);
void kmipkit_response_batch_item_release(struct kmipkit_response_batch_item_view_t *handle);
void kmipkit_message_extension_view_release(struct kmipkit_message_extension_view_t *handle);

int32_t kmipkit_message_model_c_consumer_validate_request(kmipkit_ttlv_structure_t *structure) {
    struct kmipkit_request_message_t *message = NULL;
    struct kmipkit_request_header_view_t *header = NULL;
    struct kmipkit_request_batch_item_view_t *item = NULL;
    struct kmipkit_message_validation_error_t *validation_error = NULL;
    kmipkit_protocol_version_v1_t version = {0, 0};
    uint64_t item_count = 0;
    uint8_t has_operation = 0;
    uint32_t operation = 0;
    int32_t result = -1;

    if (kmipkit_request_message_from_ttlv(structure, &message, &validation_error) != 0 ||
        message == NULL || validation_error != NULL) {
        goto cleanup;
    }
    if (kmipkit_request_message_header(message, &header) != 0 || header == NULL ||
        kmipkit_request_header_view_protocol_version(header, &version) != 0 ||
        version.major != 2 || version.minor != 1 ||
        kmipkit_request_message_batch_item_count(message, &item_count) != 0 || item_count != 1 ||
        kmipkit_request_message_batch_item_at(message, 0, &item) != 0 || item == NULL ||
        kmipkit_request_batch_item_view_operation(item, &has_operation, &operation) != 0 ||
        has_operation != 1 || operation != UINT32_C(0xF1234567)) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_request_batch_item_release(item);
    kmipkit_request_header_view_release(header);
    kmipkit_request_message_release(message);
    kmipkit_message_validation_error_release(validation_error);
    return result;
}

int32_t kmipkit_message_model_c_consumer_validate_response(kmipkit_ttlv_structure_t *structure) {
    struct kmipkit_response_message_t *message = NULL;
    struct kmipkit_response_header_view_t *header = NULL;
    struct kmipkit_response_batch_item_view_t *item = NULL;
    struct kmipkit_message_extension_view_t *extension = NULL;
    struct kmipkit_message_validation_error_t *validation_error = NULL;
    kmipkit_protocol_version_v1_t version = {0, 0};
    kmipkit_ttlv_structure_t *extension_ttlv = NULL;
    int64_t time_stamp = 0;
    uint64_t item_count = 0;
    uint64_t extension_count = 0;
    uint8_t has_status = 0;
    uint32_t status = 0;
    int32_t result = -1;

    if (kmipkit_response_message_from_ttlv(structure, &message, &validation_error) != 0 ||
        message == NULL || validation_error != NULL) {
        goto cleanup;
    }
    if (kmipkit_response_message_header(message, &header) != 0 || header == NULL ||
        kmipkit_response_header_view_protocol_version(header, &version) != 0 ||
        version.major != 2 || version.minor != 1 ||
        kmipkit_response_header_view_time_stamp(header, &time_stamp) != 0 || time_stamp != 5678 ||
        kmipkit_response_message_batch_item_count(message, &item_count) != 0 || item_count != 1 ||
        kmipkit_response_message_batch_item_at(message, 0, &item) != 0 || item == NULL ||
        kmipkit_response_batch_item_view_result_status(item, &has_status, &status) != 0 ||
        has_status != 1 || status != 2 ||
        kmipkit_response_batch_item_extension_count(item, &extension_count) != 0 ||
        extension_count != 1 ||
        kmipkit_response_batch_item_extension_at(item, 0, &extension) != 0 || extension == NULL ||
        kmipkit_message_extension_view_ttlv(extension, &extension_ttlv) != 0 ||
        extension_ttlv == NULL) {
        goto cleanup;
    }
    result = 0;

cleanup:
    kmipkit_ttlv_structure_release(extension_ttlv);
    kmipkit_message_extension_view_release(extension);
    kmipkit_response_batch_item_release(item);
    kmipkit_response_header_view_release(header);
    kmipkit_response_message_release(message);
    kmipkit_message_validation_error_release(validation_error);
    return result;
}
