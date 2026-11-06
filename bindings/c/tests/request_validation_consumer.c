#include <stdint.h>
#include <stddef.h>
#include <string.h>

#include "kmipkit.h"

int32_t kmipkit_request_validation_c_consumer(void) {
    static const uint8_t create_schema[] = "RequestPayload/Create";
    static const char expected_code[] = "KMIPKIT-0051-R019";
    static const char expected_path[] = "Request Payload.Object Type";
    static const char expected_document[] = "KMIP v2.1";
    static const uint8_t expected_reference[] = {
        0xc2, 0xa7, '6', '.', '1', '.', '8', ',', ' ', 'T', 'a', 'b', 'l', 'e', ' ', '1', '8', '6'
    };
    kmipkit_ttlv_structure_t *structure = 0;
    kmipkit_validation_report_t *report = 0;
    kmipkit_validation_issue_view_v1_t issue = {0};
    uint8_t is_valid = 1;
    uint64_t issue_count = 0;
    int32_t status = kmipkit_ttlv_structure_create(&structure);

    if (status == 0) {
        status = kmipkit_ttlv_structure_validation_report_v1(
                structure, create_schema, sizeof(create_schema) - 1, &report);
    }
    if (status == 0) status = kmipkit_validation_report_is_valid(report, &is_valid);
    if (status == 0 && is_valid != 0) status = 1;
    if (status == 0) status = kmipkit_validation_report_issue_count(report, &issue_count);
    if (status == 0 && issue_count != 2) status = 1;
    if (status == 0) status = kmipkit_validation_report_issue_at_v1(report, 0, &issue);
    if (status == 0 && (issue.severity != 1 || issue.source_kind != 1
            || issue.code.length != sizeof(expected_code) - 1
            || memcmp(issue.code.data, expected_code, sizeof(expected_code) - 1) != 0
            || issue.path.length != sizeof(expected_path) - 1
            || memcmp(issue.path.data, expected_path, sizeof(expected_path) - 1) != 0
            || issue.source_identifier.length != sizeof(expected_document) - 1
            || memcmp(issue.source_identifier.data, expected_document,
                    sizeof(expected_document) - 1) != 0
            || issue.source_reference.length != sizeof(expected_reference)
            || memcmp(issue.source_reference.data, expected_reference,
                    sizeof(expected_reference)) != 0)) status = 1;

    kmipkit_validation_report_release(report);
    kmipkit_ttlv_structure_release(structure);
    return status;
}
