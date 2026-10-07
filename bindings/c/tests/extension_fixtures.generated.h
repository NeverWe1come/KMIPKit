/* Generated from specification/api/public-api.json and tests/fixtures/extensions/cases.json. API contract sha256: f54ae735a2ca4d7173b0c493c367c9fde70d17e02b06cd7134572f068c8d8c1d. Do not edit. */
#ifndef KMIPKIT_EXTENSION_FIXTURES_GENERATED_H
#define KMIPKIT_EXTENSION_FIXTURES_GENERATED_H
#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
typedef struct kmipkit_fixture_item { uint32_t tag; uint8_t type; const char *text; int64_t signed_value; uint32_t enum_value; const struct kmipkit_fixture_item *children; size_t child_count; } kmipkit_fixture_item_t;
typedef struct { uint32_t tag; uint8_t type; const char *text; int64_t minimum; int64_t maximum; bool has_range; const struct kmipkit_fixture_schema *nested; } kmipkit_fixture_child_rule_t;
typedef struct kmipkit_fixture_schema { uint8_t type; const char *text; int64_t minimum; int64_t maximum; bool has_range; const kmipkit_fixture_child_rule_t *children; size_t child_count; } kmipkit_fixture_schema_t;
typedef struct { const char *id; const char *vendor; const char *name; const char *version; const uint32_t *path; size_t path_count; uint8_t discriminator_type; const char *discriminator_text; int64_t discriminator_number; const kmipkit_fixture_schema_t *schema; } kmipkit_fixture_definition_t;
typedef struct { const char *id; const char *vendor; bool critical; const kmipkit_fixture_item_t *payload; size_t payload_count; const char *outcome; const char *const *matched_ids; size_t matched_count; bool typed; } kmipkit_fixture_case_t;
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_0_schema_0_schema_0 = { 7U, "route-alpha", INT64_C(0), INT64_C(0), false, NULL, 0U };
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_0_schema_0_schema_1 = { 7U, "route-beta", INT64_C(0), INT64_C(0), false, NULL, 0U };
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_0_schema_0_schema_2 = { 7U, "shared", INT64_C(0), INT64_C(0), false, NULL, 0U };
static const kmipkit_fixture_child_rule_t kmipkit_fixture_schema_0_schema_0_rules[] = {
    { UINT32_C(0x540011), 7U, "route-alpha", INT64_C(0), INT64_C(0), false, &kmipkit_fixture_schema_0_schema_0_schema_0 },
    { UINT32_C(0x540012), 7U, "route-beta", INT64_C(0), INT64_C(0), false, &kmipkit_fixture_schema_0_schema_0_schema_1 },
    { UINT32_C(0x540013), 7U, "shared", INT64_C(0), INT64_C(0), false, &kmipkit_fixture_schema_0_schema_0_schema_2 }
};
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_0_schema_0 = { 1U, NULL, 0, 0, false, kmipkit_fixture_schema_0_schema_0_rules, 3U };
static const kmipkit_fixture_child_rule_t kmipkit_fixture_schema_0_rules[] = {
    { UINT32_C(0x540010), 1U, NULL, INT64_C(0), INT64_C(0), false, &kmipkit_fixture_schema_0_schema_0 }
};
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_0 = { 1U, NULL, 0, 0, false, kmipkit_fixture_schema_0_rules, 1U };
static const uint32_t kmipkit_fixture_path_0[] = { UINT32_C(0x540010), UINT32_C(0x540012) };
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_1_schema_0 = { 7U, "alpha-v1", INT64_C(0), INT64_C(0), false, NULL, 0U };
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_1_schema_1 = { 3U, NULL, INT64_C(0), INT64_C(99), true, NULL, 0U };
static const kmipkit_fixture_child_rule_t kmipkit_fixture_schema_1_rules[] = {
    { UINT32_C(0x420001), 7U, "alpha-v1", INT64_C(0), INT64_C(0), false, &kmipkit_fixture_schema_1_schema_0 },
    { UINT32_C(0x420004), 3U, NULL, INT64_C(0), INT64_C(99), true, &kmipkit_fixture_schema_1_schema_1 }
};
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_1 = { 1U, NULL, 0, 0, false, kmipkit_fixture_schema_1_rules, 2U };
static const uint32_t kmipkit_fixture_path_1[] = { UINT32_C(0x420001) };
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_2_schema_0_schema_0 = { 7U, "route-alpha", INT64_C(0), INT64_C(0), false, NULL, 0U };
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_2_schema_0_schema_1 = { 7U, "route-beta", INT64_C(0), INT64_C(0), false, NULL, 0U };
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_2_schema_0_schema_2 = { 7U, "shared", INT64_C(0), INT64_C(0), false, NULL, 0U };
static const kmipkit_fixture_child_rule_t kmipkit_fixture_schema_2_schema_0_rules[] = {
    { UINT32_C(0x540011), 7U, "route-alpha", INT64_C(0), INT64_C(0), false, &kmipkit_fixture_schema_2_schema_0_schema_0 },
    { UINT32_C(0x540012), 7U, "route-beta", INT64_C(0), INT64_C(0), false, &kmipkit_fixture_schema_2_schema_0_schema_1 },
    { UINT32_C(0x540013), 7U, "shared", INT64_C(0), INT64_C(0), false, &kmipkit_fixture_schema_2_schema_0_schema_2 }
};
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_2_schema_0 = { 1U, NULL, 0, 0, false, kmipkit_fixture_schema_2_schema_0_rules, 3U };
static const kmipkit_fixture_child_rule_t kmipkit_fixture_schema_2_rules[] = {
    { UINT32_C(0x540010), 1U, NULL, INT64_C(0), INT64_C(0), false, &kmipkit_fixture_schema_2_schema_0 }
};
static const kmipkit_fixture_schema_t kmipkit_fixture_schema_2 = { 1U, NULL, 0, 0, false, kmipkit_fixture_schema_2_rules, 1U };
static const uint32_t kmipkit_fixture_path_2[] = { UINT32_C(0x540010), UINT32_C(0x540011) };
static const kmipkit_fixture_definition_t kmipkit_fixture_definitions[] = {
    { "ambiguous.beta", "example.vendor", "ambiguous-beta", "1", kmipkit_fixture_path_0, 2U, 7U, "route-beta", INT64_C(0), &kmipkit_fixture_schema_0 },
    { "known.alpha", "example.vendor", "alpha", "1", kmipkit_fixture_path_1, 1U, 7U, "alpha-v1", INT64_C(0), &kmipkit_fixture_schema_1 },
    { "ambiguous.alpha", "example.vendor", "ambiguous-alpha", "1", kmipkit_fixture_path_2, 2U, 7U, "route-alpha", INT64_C(0), &kmipkit_fixture_schema_2 },
};
static const kmipkit_fixture_item_t kmipkit_fixture_payload_0[] = {
    { UINT32_C(0x420001), 7U, "alpha-v1", INT64_C(0), UINT32_C(0), NULL, 0U },
    { UINT32_C(0x420002), 5U, NULL, INT64_C(0), UINT32_C(4294967295), NULL, 0U },
    { UINT32_C(0x420004), 3U, NULL, INT64_C(42), UINT32_C(0), NULL, 0U },
    { UINT32_C(0x420006), 7U, "preserve-this-child-order", INT64_C(0), UINT32_C(0), NULL, 0U },
    { UINT32_C(0x540001), 7U, "preserve-vendor-range-tag", INT64_C(0), UINT32_C(0), NULL, 0U }
};
static const char *const kmipkit_fixture_matched_0[] = { "known.alpha" };
static const kmipkit_fixture_item_t kmipkit_fixture_payload_1[] = {
    { UINT32_C(0x420001), 7U, "alpha-v1", INT64_C(0), UINT32_C(0), NULL, 0U },
    { UINT32_C(0x420004), 3U, NULL, INT64_C(100), UINT32_C(0), NULL, 0U }
};
static const char *const kmipkit_fixture_matched_1[] = { "known.alpha" };
static const kmipkit_fixture_item_t kmipkit_fixture_payload_2[] = {
    { UINT32_C(0x540010), 1U, NULL, INT64_C(0), UINT32_C(0), kmipkit_fixture_payload_2 + 3U, 3U },
    { UINT32_C(0x540014), 5U, NULL, INT64_C(0), UINT32_C(314159), NULL, 0U },
    { UINT32_C(0x540015), 7U, "preserve-this-child-order", INT64_C(0), UINT32_C(0), NULL, 0U },
    { UINT32_C(0x540011), 7U, "route-alpha", INT64_C(0), UINT32_C(0), NULL, 0U },
    { UINT32_C(0x540012), 7U, "route-beta", INT64_C(0), UINT32_C(0), NULL, 0U },
    { UINT32_C(0x540013), 7U, "shared", INT64_C(0), UINT32_C(0), NULL, 0U }
};
static const char *const kmipkit_fixture_matched_2[] = { "ambiguous.alpha", "ambiguous.beta" };
static const kmipkit_fixture_item_t kmipkit_fixture_payload_3[] = {
    { UINT32_C(0x540021), 5U, NULL, INT64_C(0), UINT32_C(8675309), NULL, 0U },
    { UINT32_C(0x540022), 7U, "unknown-extension-content", INT64_C(0), UINT32_C(0), NULL, 0U }
};
static const kmipkit_fixture_item_t kmipkit_fixture_payload_4[] = {
    { UINT32_C(0x420001), 7U, "alpha-v1", INT64_C(0), UINT32_C(0), NULL, 0U },
    { UINT32_C(0x420004), 3U, NULL, INT64_C(42), UINT32_C(0), NULL, 0U },
    { UINT32_C(0x420005), 7U, "SYNTHETIC-ONLY-EXTENSION-SECRET-7C4A", INT64_C(0), UINT32_C(0), NULL, 0U }
};
static const char *const kmipkit_fixture_matched_4[] = { "known.alpha" };
static const kmipkit_fixture_case_t kmipkit_fixture_cases[] = {
    { "valid-recognized", "example.vendor", false, kmipkit_fixture_payload_0, 5U, "recognized", kmipkit_fixture_matched_0, 1U, true },
    { "invalid-schema", "example.vendor", false, kmipkit_fixture_payload_1, 2U, "unrecognized.schema_invalid", kmipkit_fixture_matched_1, 1U, false },
    { "multiply-matching", "example.vendor", false, kmipkit_fixture_payload_2, 3U, "unrecognized.ambiguous", kmipkit_fixture_matched_2, 2U, false },
    { "unknown-preserved", "example.vendor", false, kmipkit_fixture_payload_3, 2U, "unrecognized.no_match", NULL, 0U, false },
    { "synthetic-secret-bearing", "example.vendor", false, kmipkit_fixture_payload_4, 3U, "recognized", kmipkit_fixture_matched_4, 1U, true },
};
#define KMIPKIT_FIXTURE_CASE_COUNT 5U
#define KMIPKIT_FIXTURE_DEFINITION_COUNT 3U
#endif
