#include "kmipkit.h"

int32_t kmipkit_0053_typed_attribute_from_item(
    uint32_t expected_tag,
    kmipkit_ttlv_item_t *item,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_typed_attribute_from_item(expected_tag, item, out_item);
}

int32_t kmipkit_0053_typed_attribute_from_value(
    uint32_t expected_tag,
    kmipkit_ttlv_value_t *value,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_typed_attribute_from_value(expected_tag, value, out_item);
}

int32_t kmipkit_0053_typed_attribute_value(
    uint32_t expected_tag,
    kmipkit_ttlv_item_t *item,
    kmipkit_ttlv_value_t **out_value) {
    return kmipkit_typed_attribute_value(expected_tag, item, out_value);
}

int32_t kmipkit_0053_typed_attribute_from_named_activation_date(
    kmipkit_ttlv_value_t *value,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_typed_attribute_from_value(
        KMIPKIT_STANDARD_ATTRIBUTE_ACTIVATION_DATE,
        value,
        out_item);
}

uint32_t kmipkit_0053_standard_attribute_tag(uint32_t attribute_tag) {
    return KMIPKIT_STANDARD_ATTRIBUTE_TAG(attribute_tag);
}

int32_t kmipkit_0053_attribute_set_try_push_typed(
    kmipkit_attribute_set_t *attributes,
    uint32_t expected_tag,
    kmipkit_ttlv_item_t *item) {
    return kmipkit_attribute_set_try_push_typed(attributes, expected_tag, item);
}

int32_t kmipkit_0053_attribute_set_typed_count(
    kmipkit_attribute_set_t *attributes,
    uint32_t expected_tag,
    uint64_t *out_count) {
    return kmipkit_attribute_set_typed_count(attributes, expected_tag, out_count);
}

int32_t kmipkit_0053_attribute_set_typed_at(
    kmipkit_attribute_set_t *attributes,
    uint32_t expected_tag,
    uint64_t occurrence,
    kmipkit_ttlv_item_t **out_item) {
    return kmipkit_attribute_set_typed_at(attributes, expected_tag, occurrence, out_item);
}

static const uint32_t standard_attribute_tags[] = {
    KMIPKIT_STANDARD_ATTRIBUTE_ACTIVATION_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_APPLICATION_SPECIFIC_INFORMATION,
    KMIPKIT_STANDARD_ATTRIBUTE_ARCHIVE_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_VENDOR_ATTRIBUTE,
    KMIPKIT_STANDARD_ATTRIBUTE_CERTIFICATE_TYPE,
    KMIPKIT_STANDARD_ATTRIBUTE_COMPROMISE_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_COMPROMISE_OCCURRENCE_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_CONTACT_INFORMATION,
    KMIPKIT_STANDARD_ATTRIBUTE_CRYPTOGRAPHIC_ALGORITHM,
    KMIPKIT_STANDARD_ATTRIBUTE_CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
    KMIPKIT_STANDARD_ATTRIBUTE_CRYPTOGRAPHIC_LENGTH,
    KMIPKIT_STANDARD_ATTRIBUTE_CRYPTOGRAPHIC_PARAMETERS,
    KMIPKIT_STANDARD_ATTRIBUTE_CRYPTOGRAPHIC_USAGE_MASK,
    KMIPKIT_STANDARD_ATTRIBUTE_DEACTIVATION_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_DESTROY_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_DIGEST,
    KMIPKIT_STANDARD_ATTRIBUTE_INITIAL_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_KEY_FORMAT_TYPE,
    KMIPKIT_STANDARD_ATTRIBUTE_LAST_CHANGE_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_LEASE_TIME,
    KMIPKIT_STANDARD_ATTRIBUTE_LINK,
    KMIPKIT_STANDARD_ATTRIBUTE_NAME,
    KMIPKIT_STANDARD_ATTRIBUTE_OBJECT_GROUP,
    KMIPKIT_STANDARD_ATTRIBUTE_OBJECT_TYPE,
    KMIPKIT_STANDARD_ATTRIBUTE_OPAQUE_DATA_TYPE,
    KMIPKIT_STANDARD_ATTRIBUTE_PROCESS_START_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_PROTECT_STOP_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_REVOCATION_REASON,
    KMIPKIT_STANDARD_ATTRIBUTE_STATE,
    KMIPKIT_STANDARD_ATTRIBUTE_UNIQUE_IDENTIFIER,
    KMIPKIT_STANDARD_ATTRIBUTE_USAGE_LIMITS,
    KMIPKIT_STANDARD_ATTRIBUTE_FRESH,
    KMIPKIT_STANDARD_ATTRIBUTE_CERTIFICATE_LENGTH,
    KMIPKIT_STANDARD_ATTRIBUTE_DIGITAL_SIGNATURE_ALGORITHM,
    KMIPKIT_STANDARD_ATTRIBUTE_X509_CERTIFICATE_IDENTIFIER,
    KMIPKIT_STANDARD_ATTRIBUTE_X509_CERTIFICATE_ISSUER,
    KMIPKIT_STANDARD_ATTRIBUTE_X509_CERTIFICATE_SUBJECT,
    KMIPKIT_STANDARD_ATTRIBUTE_KEY_VALUE_LOCATION,
    KMIPKIT_STANDARD_ATTRIBUTE_KEY_VALUE_PRESENT,
    KMIPKIT_STANDARD_ATTRIBUTE_ORIGINAL_CREATION_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_ALTERNATIVE_NAME,
    KMIPKIT_STANDARD_ATTRIBUTE_RANDOM_NUMBER_GENERATOR,
    KMIPKIT_STANDARD_ATTRIBUTE_PKCS12_FRIENDLY_NAME,
    KMIPKIT_STANDARD_ATTRIBUTE_DESCRIPTION,
    KMIPKIT_STANDARD_ATTRIBUTE_COMMENT,
    KMIPKIT_STANDARD_ATTRIBUTE_SENSITIVE,
    KMIPKIT_STANDARD_ATTRIBUTE_ALWAYS_SENSITIVE,
    KMIPKIT_STANDARD_ATTRIBUTE_EXTRACTABLE,
    KMIPKIT_STANDARD_ATTRIBUTE_NEVER_EXTRACTABLE,
    KMIPKIT_STANDARD_ATTRIBUTE_SHORT_UNIQUE_IDENTIFIER,
    KMIPKIT_STANDARD_ATTRIBUTE_NIST_KEY_TYPE,
    KMIPKIT_STANDARD_ATTRIBUTE_PROTECTION_LEVEL,
    KMIPKIT_STANDARD_ATTRIBUTE_PROTECTION_PERIOD,
    KMIPKIT_STANDARD_ATTRIBUTE_QUANTUM_SAFE,
    KMIPKIT_STANDARD_ATTRIBUTE_PROTECTION_STORAGE_MASK,
    KMIPKIT_STANDARD_ATTRIBUTE_ROTATE_INTERVAL,
    KMIPKIT_STANDARD_ATTRIBUTE_ROTATE_AUTOMATIC,
    KMIPKIT_STANDARD_ATTRIBUTE_ROTATE_OFFSET,
    KMIPKIT_STANDARD_ATTRIBUTE_ROTATE_DATE,
    KMIPKIT_STANDARD_ATTRIBUTE_ROTATE_GENERATION,
    KMIPKIT_STANDARD_ATTRIBUTE_ROTATE_NAME,
    KMIPKIT_STANDARD_ATTRIBUTE_ROTATE_LATEST,
};

uint32_t kmipkit_0053_standard_attribute_tag_count(void) {
    return (uint32_t)(sizeof(standard_attribute_tags) / sizeof(standard_attribute_tags[0]));
}

uint32_t kmipkit_0053_standard_attribute_tag_at(uint32_t index) {
    if (index >= kmipkit_0053_standard_attribute_tag_count()) {
        return 0;
    }
    return standard_attribute_tags[index];
}
