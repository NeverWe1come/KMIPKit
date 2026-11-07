#include "c_validator.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    printf("Running C validator unit tests...\n");

    // 1. Test attempt context uniqueness
    kmip_attempt_context_t ctx1, ctx2;
    kmip_init_attempt_context(&ctx1, "run-1", "c", "TTLV", "level1", "test-key");
    kmip_init_attempt_context(&ctx2, "run-1", "c", "TTLV", "level2", "test-key");

    assert(strcmp(ctx1.unique_name, ctx2.unique_name) != 0);
    assert(strstr(ctx1.unique_name, "c-TTLV-level1") != NULL);
    assert(strstr(ctx2.unique_name, "c-TTLV-level2") != NULL);

    // 2. Test capability matching
    kmip_server_capabilities_t caps;
    memset(&caps, 0, sizeof(caps));
    strncpy(caps.server_id, "test-kms", sizeof(caps.server_id) - 1);
    strncpy(caps.supported_operations[0], "Create", KMIP_MAX_NAME_LEN - 1);
    caps.supported_operations_count = 1;

    kmip_test_case_t tc_supported, tc_unsupported;
    memset(&tc_supported, 0, sizeof(tc_supported));
    strncpy(tc_supported.case_id, "test-create", KMIP_MAX_NAME_LEN - 1);
    strncpy(tc_supported.operation, "Create", KMIP_MAX_NAME_LEN - 1);

    memset(&tc_unsupported, 0, sizeof(tc_unsupported));
    strncpy(tc_unsupported.case_id, "test-archive", KMIP_MAX_NAME_LEN - 1);
    strncpy(tc_unsupported.operation, "Archive", KMIP_MAX_NAME_LEN - 1);

    char reasons[KMIP_MAX_REASONS][KMIP_MAX_NAME_LEN];
    size_t r_count = 0;

    assert(kmip_is_case_supported(&caps, &tc_supported, reasons, &r_count) == true);
    assert(r_count == 0);

    assert(kmip_is_case_supported(&caps, &tc_unsupported, reasons, &r_count) == false);
    assert(r_count == 1);
    assert(strstr(reasons[0], "Archive") != NULL);

    // 3. Test execution suite with mock driver
    kmip_test_case_t cases[2] = { tc_supported, tc_unsupported };
    kmip_driver_vtable_t driver = kmip_get_mock_driver();
    kmip_validation_summary_t summary;

    kmip_run_suite(&caps, cases, 2, &driver, false, &summary);
    assert(summary.total_cases == 2);
    assert(summary.passed == 1);
    assert(summary.unsupported == 1);
    assert(summary.results[0].status == CASE_STATUS_PASSED);
    assert(summary.results[1].status == CASE_STATUS_UNSUPPORTED_SKIPPED);

    // 4. Test equivalence verifier
    kmip_normalized_response_t resps[2];
    memset(resps, 0, sizeof(resps));
    strncpy(resps[0].status, "Success", sizeof(resps[0].status) - 1);
    strncpy(resps[0].object_type, "SymmetricKey", sizeof(resps[0].object_type) - 1);
    strncpy(resps[0].sanitized_payload_hex, "deadbeef", sizeof(resps[0].sanitized_payload_hex) - 1);

    strncpy(resps[1].status, "Success", sizeof(resps[1].status) - 1);
    strncpy(resps[1].object_type, "SymmetricKey", sizeof(resps[1].object_type) - 1);
    strncpy(resps[1].sanitized_payload_hex, "deadbeef", sizeof(resps[1].sanitized_payload_hex) - 1);

    char disc[KMIP_MAX_REASONS][KMIP_MAX_NAME_LEN];
    size_t disc_count = 0;
    assert(kmip_verify_equivalence(resps, 2, disc, &disc_count) == true);
    assert(disc_count == 0);

    // Change one to mismatch
    strncpy(resps[1].status, "OperationFailed", sizeof(resps[1].status) - 1);
    assert(kmip_verify_equivalence(resps, 2, disc, &disc_count) == false);
    assert(disc_count > 0);

    printf("All C validator tests passed successfully!\n");
    return 0;
}
