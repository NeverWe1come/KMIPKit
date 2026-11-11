#ifndef KMIP_C_VALIDATOR_H
#define KMIP_C_VALIDATOR_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define KMIP_MAX_NAME_LEN 256
#define KMIP_MAX_CASES 4096
#define KMIP_MAX_CAPABILITIES 512
#define KMIP_MAX_REASONS 8
#define KMIP_VERSION_MASK_2_0 1U
#define KMIP_VERSION_MASK_2_1 2U
#define KMIP_MAX_VERSIONS 2

typedef enum {
    CASE_STATUS_PASSED = 0,
    CASE_STATUS_FAILED,
    CASE_STATUS_SERVER_UNSUPPORTED,
    CASE_STATUS_POLICY_RESTRICTED,
    CASE_STATUS_NOT_RUN
} kmip_case_status_t;

typedef struct {
    char case_id[KMIP_MAX_NAME_LEN];
    char operation[KMIP_MAX_NAME_LEN];
    char object_type[KMIP_MAX_NAME_LEN];
    char attribute[KMIP_MAX_NAME_LEN];
    char category[KMIP_MAX_NAME_LEN];
    char mode[32];
    char mutation[32];
    char workflow_type[64];
    char enum_type[KMIP_MAX_NAME_LEN];
    char bitmask_type[KMIP_MAX_NAME_LEN];
    char normative_sections[KMIP_MAX_VERSIONS][512];
    unsigned int protocol_version_mask;
} kmip_test_case_t;

typedef struct {
    char server_id[KMIP_MAX_NAME_LEN];
    char server_vendor[KMIP_MAX_NAME_LEN];
    char server_product[KMIP_MAX_NAME_LEN];
    char server_version[KMIP_MAX_NAME_LEN];
    char server_configuration_fingerprint[65];
    char protocol_version[8];
    char transport[16];
    char encoding[16];
    char api_level[32];
    char host[KMIP_MAX_NAME_LEN];
    uint16_t port;
    char https_url[KMIP_MAX_NAME_LEN];
    char ca_cert_path[KMIP_MAX_NAME_LEN];
    char client_cert_path[KMIP_MAX_NAME_LEN];
    char client_key_path[KMIP_MAX_NAME_LEN];
    char server_name_override[KMIP_MAX_NAME_LEN];
    char username_env[KMIP_MAX_NAME_LEN];
    char password_env[KMIP_MAX_NAME_LEN];
    bool dedicated_test_environment_confirmed;
    bool probe_unsupported;
    bool allow_mutations;
} kmip_connection_config_t;

typedef struct {
    char server_id[KMIP_MAX_NAME_LEN];
    char supported_operations[KMIP_MAX_CAPABILITIES][KMIP_MAX_NAME_LEN];
    size_t supported_operations_count;
    char supported_objects[KMIP_MAX_CAPABILITIES][KMIP_MAX_NAME_LEN];
    size_t supported_objects_count;
    char supported_attributes[KMIP_MAX_CAPABILITIES][KMIP_MAX_NAME_LEN];
    size_t supported_attributes_count;
} kmip_server_capabilities_t;

typedef struct {
    char run_id[KMIP_MAX_NAME_LEN];
    char language[32];
    char encoding[32];
    char api_level[32];
    char unique_name[KMIP_MAX_NAME_LEN];
} kmip_attempt_context_t;

typedef struct {
    bool executed;
    bool success;
    bool library_validated;
    bool library_failed;
    bool server_unsupported;
    bool policy_restricted;
    bool has_result_status_code;
    bool has_result_reason_code;
    uint32_t result_status_code;
    uint32_t result_reason_code;
    char result_reason[KMIP_MAX_NAME_LEN];
} kmip_execution_outcome_t;

typedef struct {
    kmip_execution_outcome_t (*execute_case)(
        const kmip_test_case_t *test_case,
        const kmip_attempt_context_t *attempt,
        void *user_data
    );
    void *user_data;
} kmip_driver_vtable_t;

typedef struct {
    char case_id[KMIP_MAX_NAME_LEN];
    char operation[KMIP_MAX_NAME_LEN];
    kmip_case_status_t status;
    char library_correctness[16];
    bool has_result_status_code;
    bool has_result_reason_code;
    uint32_t result_status_code;
    uint32_t result_reason_code;
    char detail[KMIP_MAX_NAME_LEN];
} kmip_case_result_t;

typedef struct {
    char server_id[KMIP_MAX_NAME_LEN];
    char server_vendor[KMIP_MAX_NAME_LEN];
    char server_product[KMIP_MAX_NAME_LEN];
    char server_version[KMIP_MAX_NAME_LEN];
    char server_configuration_fingerprint[65];
    char protocol_version[8];
    char transport[16];
    char encoding[16];
    char api_level[32];
    char library_commit[41];
    char artifact_sha256[65];
    char catalog_sha256[65];
    char run_id[KMIP_MAX_NAME_LEN];
    char tested_at[32];
    char language[32];
    size_t total_cases;
    size_t passed;
    size_t failed;
    size_t unsupported;
    size_t policy_restricted;
    size_t not_run;
    kmip_case_result_t results[KMIP_MAX_CASES];
    size_t results_count;
} kmip_validation_summary_t;

/* Public API */
bool kmip_load_capabilities(const char *json_path, kmip_server_capabilities_t *out_caps);
bool kmip_load_connection(const char *json_path, kmip_connection_config_t *out_connection);
bool kmip_load_catalog(const char *json_path, kmip_test_case_t *out_cases, size_t *out_count);
bool kmip_is_case_supported(const kmip_server_capabilities_t *caps, const kmip_test_case_t *test_case, char out_reasons[KMIP_MAX_REASONS][KMIP_MAX_NAME_LEN], size_t *out_reasons_count);
void kmip_init_attempt_context(kmip_attempt_context_t *ctx, const char *run_id, const char *lang, const char *enc, const char *level, const char *prefix);

void kmip_run_suite(
    const kmip_server_capabilities_t *caps,
    const kmip_test_case_t *cases,
    size_t case_count,
    const kmip_driver_vtable_t *driver,
    bool probe_unsupported,
    kmip_validation_summary_t *out_summary
);

void kmip_render_ansi(const kmip_validation_summary_t *summary);
bool kmip_render_markdown(const kmip_validation_summary_t *summary, const char *file_path);
bool kmip_render_html(const kmip_validation_summary_t *summary, const char *file_path);
bool kmip_render_json(const kmip_validation_summary_t *summary, const char *file_path);

/* Equivalence Verifier & Recipe Runner */
typedef struct {
    char status[64];
    char result_reason[KMIP_MAX_NAME_LEN];
    char object_type[64];
    char sanitized_payload_hex[KMIP_MAX_NAME_LEN];
} kmip_normalized_response_t;

bool kmip_verify_equivalence(
    const kmip_normalized_response_t *responses,
    size_t count,
    char out_discrepancies[KMIP_MAX_REASONS][KMIP_MAX_NAME_LEN],
    size_t *out_discrepancies_count
);

bool kmip_run_recipe_with_teardown(
    bool (*setup_fn)(void *user_data),
    bool (*target_fn)(void *user_data),
    bool (*teardown_fn)(void *user_data),
    void *user_data
);

#endif /* KMIP_C_VALIDATOR_H */
