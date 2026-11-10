#include "c_validator.h"
#include "live_driver.h"
#include "kmipkit.h"
#include "json.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

static size_t test_execute_count;

static kmip_execution_outcome_t test_execute_case(
    const kmip_test_case_t *test_case,
    const kmip_attempt_context_t *attempt,
    void *user_data) {
    (void)attempt;
    (void)user_data;
    test_execute_count++;
    kmip_execution_outcome_t outcome = {
        .executed = strcmp(test_case->case_id, "test-archive") != 0,
        .success = strcmp(test_case->case_id, "test-server-unsupported") != 0 &&
                   strcmp(test_case->case_id, "test-policy-restricted") != 0,
        .library_validated = strcmp(test_case->case_id, "test-server-unsupported") != 0 &&
                             strcmp(test_case->case_id, "test-policy-restricted") != 0,
        .server_unsupported = strcmp(test_case->case_id, "test-server-unsupported") == 0,
        .policy_restricted = strcmp(test_case->case_id, "test-policy-restricted") == 0,
        .result_reason = {0}
    };
    return outcome;
}

int main(void) {
    printf("Running C validator unit tests...\n");

    // 0. Confirm the shared release catalog is consumed in full.
    kmip_test_case_t catalog[KMIP_MAX_CASES];
    size_t catalog_count = 0;
    assert(kmip_load_catalog("../shared/catalog/kmipkit-1.0-test-catalog.json",
                             catalog, &catalog_count));
    assert(catalog_count == 218);
    assert(strcmp(catalog[catalog_count - 1].case_id,
                  "test-v20-reject-set-defaults") == 0);
    assert(strcmp(catalog[0].mode, "server_exchange") == 0);
    assert(catalog[0].protocol_version_mask ==
           (KMIP_VERSION_MASK_2_0 | KMIP_VERSION_MASK_2_1));
    assert(strcmp(catalog[7].normative_sections[0], "6.1.37") == 0);
    assert(strcmp(catalog[7].normative_sections[1], "6.1.40") == 0);
    assert(strcmp(catalog[47].workflow_type, "server_to_client") == 0);
    assert(strcmp(catalog[135].mode, "local_model") == 0);
    assert(strcmp(catalog[135].enum_type, "Adjustment Type") == 0);
    assert(catalog[213].protocol_version_mask == KMIP_VERSION_MASK_2_0);

    kmip_connection_config_t connection;
    assert(kmip_load_connection("../shared/profiles/connection.example.json", &connection));
    assert(strcmp(connection.server_id, "example-kms") == 0);
    assert(strcmp(connection.server_product, "Example KMIP test server") == 0);
    assert(strcmp(connection.server_version, "not deployed") == 0);
    assert(strcmp(connection.protocol_version, "2.1") == 0);
    assert(strcmp(connection.transport, "raw_tls") == 0);
    assert(connection.port == 5696);
    assert(connection.dedicated_test_environment_confirmed);
    assert(connection.probe_unsupported);
    assert(!connection.allow_mutations);

    // 0a. Resolve a typed request builder through the installed public C ABI.
    kmipkit_client_batch_item_t *public_item = NULL;
    uint32_t operation = 0;
    assert(kmipkit_client_batch_item_discover_versions(&public_item) == 0);
    assert(kmipkit_client_batch_item_operation(public_item, &operation) == 0);
    assert(operation == KMIPKIT_ENUMERATION_OPERATION_DISCOVER_VERSIONS);
    kmipkit_client_batch_item_release(public_item);

    // 0b. Generic TTLV construction and navigation use the installed library.
    kmipkit_raw_tag_t *raw_tag = NULL;
    kmipkit_tag_t *checked_tag = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    kmipkit_codec_limits_t *limits = NULL;
    kmipkit_ttlv_item_t *generic_item = NULL;
    kmipkit_ttlv_structure_t *generic_structure = NULL;
    kmipkit_ttlv_structure_view_t *structure_view = NULL;
    kmipkit_ttlv_item_view_t *item_view = NULL;
    kmipkit_ttlv_value_view_t *value_view = NULL;
    uint64_t item_count = 0;
    uint32_t raw_value = 0;
    uint8_t item_type = 0;
    assert(kmipkit_ttlv_raw_tag_create(UINT32_C(0x42007f), &raw_tag) == 0);
    assert(kmipkit_ttlv_raw_tag_try_checked(raw_tag, &checked_tag) == 0);
    assert(kmipkit_ttlv_value_enumeration(UINT32_C(1), &value) == 0);
    assert(kmipkit_codec_limits_defaults(&limits) == 0);
    assert(kmipkit_ttlv_item_create(checked_tag, value, limits, &generic_item) == 0);
    value = NULL; /* The public item builder consumed it. */
    assert(kmipkit_ttlv_structure_create(&generic_structure) == 0);
    kmipkit_ttlv_structure_t *updated_structure = NULL;
    assert(kmipkit_ttlv_structure_with_item(generic_structure, generic_item, limits,
                                             &updated_structure) == 0);
    generic_structure = updated_structure;
    generic_item = NULL; /* The public structure builder consumed it. */
    assert(kmipkit_ttlv_structure_view(generic_structure, &structure_view) == 0);
    assert(kmipkit_ttlv_structure_view_item_count(structure_view, &item_count) == 0);
    assert(item_count == 1);
    assert(kmipkit_ttlv_structure_view_item_at(structure_view, 0, &item_view) == 0);
    assert(kmipkit_ttlv_item_view_type(item_view, &item_type) == 0);
    assert(item_type == KMIPKIT_TTLV_ITEM_TYPE_ENUMERATION);
    assert(kmipkit_ttlv_item_view_value(item_view, &value_view) == 0);
    assert(kmipkit_ttlv_value_view_enumeration(value_view, &raw_value) == 0);
    assert(raw_value == 1);
    kmipkit_ttlv_value_view_release(value_view);
    kmipkit_ttlv_item_view_release(item_view);
    kmipkit_ttlv_structure_view_release(structure_view);
    kmipkit_ttlv_structure_release(generic_structure);
    kmipkit_codec_limits_release(limits);
    kmipkit_tag_release(checked_tag);
    kmipkit_raw_tag_release(raw_tag);

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

    // 3. Capability profiles are hypotheses and must not suppress execution.
    kmip_test_case_t tc_server_unsupported = tc_unsupported;
    strncpy(tc_server_unsupported.case_id, "test-server-unsupported",
            sizeof(tc_server_unsupported.case_id) - 1);
    kmip_test_case_t tc_policy_restricted = tc_unsupported;
    strncpy(tc_policy_restricted.case_id, "test-policy-restricted",
            sizeof(tc_policy_restricted.case_id) - 1);
    kmip_test_case_t cases[4] = {
        tc_supported, tc_unsupported, tc_server_unsupported, tc_policy_restricted
    };
    kmip_driver_vtable_t driver = { .execute_case = test_execute_case };
    kmip_validation_summary_t summary;

    test_execute_count = 0;
    kmip_run_suite(&caps, cases, 4, &driver, false, &summary);
    assert(summary.total_cases == 4);
    assert(test_execute_count == 4);
    assert(summary.passed == 1);
    assert(summary.not_run == 1);
    assert(summary.unsupported == 1);
    assert(summary.policy_restricted == 1);
    assert(summary.results[0].status == CASE_STATUS_PASSED);
    assert(summary.results[1].status == CASE_STATUS_NOT_RUN);
    assert(summary.results[2].status == CASE_STATUS_SERVER_UNSUPPORTED);
    assert(summary.results[3].status == CASE_STATUS_POLICY_RESTRICTED);

    kmip_live_client_t mutation_client = {0};
    kmip_test_case_t mutating_case = {0};
    snprintf(mutating_case.case_id, sizeof(mutating_case.case_id), "test-create");
    snprintf(mutating_case.operation, sizeof(mutating_case.operation), "Create");
    snprintf(mutating_case.mutation, sizeof(mutating_case.mutation), "mutating");
    kmip_execution_outcome_t mutation_outcome =
        kmip_live_execute_case(&mutating_case, NULL, &mutation_client);
    assert(mutation_outcome.executed && mutation_outcome.policy_restricted);
    mutation_client.allow_mutations = true;
    mutation_outcome = kmip_live_execute_case(&mutating_case, NULL, &mutation_client);
    assert(!mutation_outcome.executed && !mutation_outcome.policy_restricted);
    snprintf(mutating_case.mutation, sizeof(mutating_case.mutation), "read_only");
    snprintf(mutating_case.workflow_type, sizeof(mutating_case.workflow_type), "server_to_client");
    mutation_outcome = kmip_live_execute_case(&mutating_case, NULL, &mutation_client);
    assert(!mutation_outcome.executed &&
           strstr(mutation_outcome.result_reason, "server-initiated") != NULL);

    // Every applicable catalog row must receive an explicit execution status
    // and every row not executed must explain the missing route or precondition.
    kmip_server_capabilities_t catalog_caps = {0};
    snprintf(catalog_caps.server_id, sizeof(catalog_caps.server_id), "offline-plan");
    kmip_live_client_t offline_client = {0};
    kmip_driver_vtable_t live_driver = {
        .execute_case = kmip_live_execute_case,
        .user_data = &offline_client
    };
    kmip_validation_summary_t catalog_summary;
    kmip_run_suite(&catalog_caps, catalog, catalog_count, &live_driver, false, &catalog_summary);
    assert(catalog_summary.results_count == catalog_count);
    assert(catalog_summary.passed == 67); /* 64 enum and 3 bitmask model cases. */
    for (size_t i = 0; i < catalog_summary.results_count; ++i) {
        const kmip_case_result_t *result = &catalog_summary.results[i];
        assert(result->case_id[0] != '\0');
        if (catalog[i].enum_type[0] || catalog[i].bitmask_type[0]) {
            assert(result->status == CASE_STATUS_PASSED);
            assert(strcmp(result->library_correctness, "passed") == 0);
        }
        if (result->status == CASE_STATUS_NOT_RUN ||
            result->status == CASE_STATUS_POLICY_RESTRICTED) {
            assert(result->detail[0] != '\0');
        }
    }

    // 3a. C emits the versioned shared evidence schema, including separate
    // library and server outcomes and all required execution provenance.
    snprintf(summary.run_id, sizeof(summary.run_id), "c-unit-report");
    snprintf(summary.tested_at, sizeof(summary.tested_at), "2026-11-10T00:00:00Z");
    snprintf(summary.server_id, sizeof(summary.server_id), "test-kms");
    snprintf(summary.server_product, sizeof(summary.server_product), "fixture");
    snprintf(summary.server_version, sizeof(summary.server_version), "1");
    memset(summary.server_configuration_fingerprint, 'a', 64);
    summary.server_configuration_fingerprint[64] = '\0';
    snprintf(summary.protocol_version, sizeof(summary.protocol_version), "2.1");
    snprintf(summary.transport, sizeof(summary.transport), "raw_tls");
    snprintf(summary.encoding, sizeof(summary.encoding), "TTLV");
    snprintf(summary.api_level, sizeof(summary.api_level), "typed-protocol");
    snprintf(summary.library_commit, sizeof(summary.library_commit), "abcdef1234567890");
    memset(summary.artifact_sha256, 'b', 64);
    summary.artifact_sha256[64] = '\0';
    memset(summary.catalog_sha256, 'c', 64);
    summary.catalog_sha256[64] = '\0';
    assert(kmip_render_json(&summary, ".test-c-report.json"));
    json_document_t report;
    assert(json_document_load(".test-c-report.json", &report));
    assert(json_token_equals(&report, json_object_get(&report, 0, "schema_version"), "2.0"));
    assert(json_token_equals(&report, json_object_get(&report, 0, "protocol_version"), "2.1"));
    int report_server = json_object_get(&report, 0, "server");
    assert(json_token_equals(&report, json_object_get(&report, report_server, "id"), "test-kms"));
    int report_cases = json_object_get(&report, 0, "cases");
    assert(json_array_length(&report, report_cases) == 4);
    assert(json_token_equals(&report, json_object_get(&report, json_array_get(&report, report_cases, 2),
                                                       "server_compatibility"), "unsupported"));
    assert(json_token_equals(&report, json_object_get(&report, json_array_get(&report, report_cases, 3),
                                                       "server_compatibility"), "policy_restricted"));
    json_document_release(&report);
    remove(".test-c-report.json");

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
