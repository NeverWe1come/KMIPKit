#ifndef KMIP_C_VALIDATOR_H
#define KMIP_C_VALIDATOR_H

#include <stdbool.h>
#include <stddef.h>

#define KMIP_MAX_NAME_LEN 256
#define KMIP_MAX_CASES 512
#define KMIP_MAX_REASONS 8

typedef enum {
    CASE_STATUS_PASSED = 0,
    CASE_STATUS_FAILED,
    CASE_STATUS_UNSUPPORTED_SKIPPED,
    CASE_STATUS_UNSUPPORTED_VERIFIED_REJECTED,
    CASE_STATUS_UNEXPECTED_SUCCESS
} kmip_case_status_t;

typedef struct {
    char case_id[KMIP_MAX_NAME_LEN];
    char operation[KMIP_MAX_NAME_LEN];
    char object_type[KMIP_MAX_NAME_LEN];
    char attribute[KMIP_MAX_NAME_LEN];
    char category[KMIP_MAX_NAME_LEN];
} kmip_test_case_t;

typedef struct {
    char server_id[KMIP_MAX_NAME_LEN];
    char supported_operations[KMIP_MAX_CASES][KMIP_MAX_NAME_LEN];
    size_t supported_operations_count;
    char supported_objects[KMIP_MAX_CASES][KMIP_MAX_NAME_LEN];
    size_t supported_objects_count;
    char supported_attributes[KMIP_MAX_CASES][KMIP_MAX_NAME_LEN];
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
    bool success;
    char result_reason[KMIP_MAX_NAME_LEN];
} kmip_execution_outcome_t;

typedef struct {
    kmip_execution_outcome_t (*execute_case)(
        const kmip_test_case_t *test_case,
        const kmip_attempt_context_t *attempt
    );
} kmip_driver_vtable_t;

typedef struct {
    char case_id[KMIP_MAX_NAME_LEN];
    char operation[KMIP_MAX_NAME_LEN];
    kmip_case_status_t status;
    char detail[KMIP_MAX_NAME_LEN];
} kmip_case_result_t;

typedef struct {
    char server_id[KMIP_MAX_NAME_LEN];
    char language[32];
    size_t total_cases;
    size_t passed;
    size_t failed;
    size_t unsupported;
    kmip_case_result_t results[KMIP_MAX_CASES];
    size_t results_count;
} kmip_validation_summary_t;

/* Public API */
bool kmip_load_capabilities(const char *json_path, kmip_server_capabilities_t *out_caps);
bool kmip_load_catalog(const char *json_path, kmip_test_case_t *out_cases, size_t *out_count);
bool kmip_is_case_supported(const kmip_server_capabilities_t *caps, const kmip_test_case_t *test_case, char out_reasons[KMIP_MAX_REASONS][KMIP_MAX_NAME_LEN], size_t *out_reasons_count);
void kmip_init_attempt_context(kmip_attempt_context_t *ctx, const char *run_id, const char *lang, const char *enc, const char *level, const char *prefix);

kmip_driver_vtable_t kmip_get_mock_driver(void);
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
