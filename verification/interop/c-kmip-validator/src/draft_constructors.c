#include "generic_wire.h"

#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef int32_t (*constructor_t)(kmipkit_ttlv_structure_t *, kmipkit_client_batch_item_t **);

struct draft_route {
    const char *symbol;
    constructor_t construct;
    uint32_t operation;
};

#define ROUTE(name, operation) { #name, name, operation }

static const struct draft_route ROUTES[] = {
    ROUTE(kmipkit_client_batch_item_create_group, 0x3c),
    ROUTE(kmipkit_client_batch_item_obliterate, 0x3d),
    ROUTE(kmipkit_client_batch_item_create_user, 0x3e),
    ROUTE(kmipkit_client_batch_item_create_credential, 0x3f),
    ROUTE(kmipkit_client_batch_item_deactivate, 0x40),
    ROUTE(kmipkit_client_batch_item_encapsulate, 0x41),
    ROUTE(kmipkit_client_batch_item_decapsulate, 0x42),
};

static int check_route(const struct draft_route *route) {
    kmipkit_ttlv_structure_t *payload = NULL;
    kmipkit_client_batch_item_t *item = NULL;
    uint32_t actual = 0;
    int32_t status = kmipkit_ttlv_structure_create(&payload);
    if (status == 0) {
        status = route->construct(payload, &item);
    }
    if (status == 0) {
        status = kmipkit_client_batch_item_operation(item, &actual);
    }
    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(payload);
    return status == 0 && actual == route->operation;
}

static int check_tags(void) {
    uint32_t code;
    unsigned assigned;
    int input;
    while ((input = scanf("%" SCNx32 " %u", &code, &assigned)) == 2) {
        kmipkit_raw_tag_t *raw = NULL;
        kmipkit_tag_t *checked = NULL;
        uint32_t actual = 0;
        int raw_ok = kmipkit_ttlv_raw_tag_create(code, &raw) == 0 &&
                     kmipkit_ttlv_raw_tag_value(raw, &actual) == 0 && actual == code;
        int32_t status = raw_ok ? kmipkit_ttlv_raw_tag_try_checked(raw, &checked) : KMIPKIT_ERROR_INVALID_SCHEMA;
        int checked_ok = raw_ok && (assigned
            ? status == 0 && kmipkit_ttlv_tag_value(checked, &actual) == 0 && actual == code
            : status == KMIPKIT_ERROR_INVALID_SCHEMA && checked == NULL);
        kmipkit_tag_release(checked);
        kmipkit_raw_tag_release(raw);
        printf("%s %s\n", raw_ok ? "passed" : "failed", checked_ok ? "passed" : "failed");
    }
    return input == EOF && !ferror(stdin) ? EXIT_SUCCESS : EXIT_FAILURE;
}

static bool print_bytes(kmipkit_ttlv_value_view_t *value) {
    uint64_t length = 0;
    if (kmipkit_ttlv_value_view_byte_length(value, &length) != 0)
        return false;
    if (putchar('"') == EOF) return false;
    for (uint64_t index = 0; index < length; ++index) {
        uint8_t byte = 0;
        if (kmipkit_ttlv_value_view_byte_at(value, index, &byte) != 0 ||
            printf("%02x", (unsigned)byte) < 0)
            return false;
    }
    return putchar('"') != EOF;
}

static bool print_scalar(uint8_t type, kmipkit_ttlv_value_view_t *value) {
    int32_t integer = 0;
    int64_t signed_value = 0;
    uint32_t unsigned_value = 0;
    uint8_t bit = 0;
    int32_t status;

    switch (type) {
        case 2:
            return kmipkit_ttlv_value_view_integer(value, &integer) == 0 &&
                   printf("%" PRId32, integer) >= 0;
        case 3: case 9: case 11:
            if (type == 3) status = kmipkit_ttlv_value_view_long_integer(value, &signed_value);
            else if (type == 9) status = kmipkit_ttlv_value_view_date_time(value, &signed_value);
            else status = kmipkit_ttlv_value_view_date_time_extended(value, &signed_value);
            return status == 0 && printf("%" PRId64, signed_value) >= 0;
        case 5: case 10:
            status = type == 5 ? kmipkit_ttlv_value_view_enumeration(value, &unsigned_value)
                               : kmipkit_ttlv_value_view_interval(value, &unsigned_value);
            return status == 0 && printf("%" PRIu32, unsigned_value) >= 0;
        case 6:
            return kmipkit_ttlv_value_view_boolean(value, &bit) == 0 &&
                   fputs(bit ? "true" : "false", stdout) >= 0;
        case 4: case 7: case 8: case 12: case 13: case 14:
            return print_bytes(value);
        default:
            return false;
    }
}

static bool print_item(kmipkit_ttlv_item_view_t *view, bool first) {
    uint8_t type = 0;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_view_t *value = NULL;
    kmipkit_ttlv_structure_view_t *structure = NULL;
    uint32_t code = 0;
    uint64_t count = 0;
    bool ok = kmipkit_ttlv_item_view_tag(view, &tag) == 0 &&
              kmipkit_ttlv_tag_value(tag, &code) == 0 &&
              kmipkit_ttlv_item_view_type(view, &type) == 0 &&
              kmipkit_ttlv_item_view_value(view, &value) == 0;
    if (ok && type == 1) {
        ok = kmipkit_ttlv_value_view_structure(value, &structure) == 0 &&
             kmipkit_ttlv_structure_view_item_count(structure, &count) == 0;
    }
    if (ok) {
        printf("%s[%" PRIu32 ",%u,%" PRIu64, first ? "" : ",", code, (unsigned)type, count);
        if (type != 1) {
            ok = putchar(',') != EOF && print_scalar(type, value);
        }
        putchar(']');
        for (uint64_t index = 0; ok && index < count; ++index) {
            kmipkit_ttlv_item_view_t *child = NULL;
            ok = kmipkit_ttlv_structure_view_item_at(structure, index, &child) == 0 && print_item(child, false);
            kmipkit_ttlv_item_view_release(child);
        }
    }
    kmipkit_ttlv_structure_view_release(structure);
    kmipkit_ttlv_value_view_release(value);
    kmipkit_tag_release(tag);
    return ok;
}

static int hex_digit(char character) {
    if (character >= '0' && character <= '9') return character - '0';
    if (character >= 'a' && character <= 'f') return character - 'a' + 10;
    return -1;
}

static uint64_t read_unsigned(const uint8_t *input, size_t count) {
    uint64_t value = 0;
    for (size_t index = 0; index < count; ++index) value = (value << 8) | input[index];
    return value;
}

static int64_t read_signed(const uint8_t *input, size_t count) {
    uint64_t value = read_unsigned(input, count);
    if ((input[0] & 0x80U) == 0) return (int64_t)value;
    uint64_t mask = count == 4 ? UINT32_MAX : UINT64_MAX;
    return -1 - (int64_t)(mask - value);
}

static int32_t construct_scalar(uint8_t type, const uint8_t *data, size_t length,
                                kmipkit_codec_limits_t *limits, kmipkit_ttlv_value_t **out) {
    switch (type) {
        case 2: if (length == 4) return kmipkit_ttlv_value_integer((int32_t)read_signed(data, length), out); break;
        case 3: if (length == 8) return kmipkit_ttlv_value_long_integer(read_signed(data, length), out); break;
        case 4: if (length && length % 8 == 0) return kmipkit_ttlv_value_big_integer(limits, data, length, out); break;
        case 5: if (length == 4) return kmipkit_ttlv_value_enumeration((uint32_t)read_unsigned(data, length), out); break;
        case 6:
            if (length == 8 && read_unsigned(data, length) < 2)
                return kmipkit_ttlv_value_boolean(data[7], out);
            break;
        case 7: return kmipkit_ttlv_value_text_string(limits, data, length, out);
        case 8: return kmipkit_ttlv_value_byte_string(limits, data, length, out);
        case 9: if (length == 8) return kmipkit_ttlv_value_date_time(read_signed(data, length), out); break;
        case 10: if (length == 4) return kmipkit_ttlv_value_interval((uint32_t)read_unsigned(data, length), out); break;
        case 11: if (length == 8) return kmipkit_ttlv_value_date_time_extended(read_signed(data, length), out); break;
        default: break;
    }
    return KMIPKIT_ERROR_INVALID_INPUT;
}

static int32_t construct_item(const uint8_t *wire, size_t available, unsigned depth,
                              kmipkit_codec_limits_t *limits, size_t *consumed,
                              kmipkit_ttlv_item_t **item) {
    if (available < 8 || depth >= 32) return KMIPKIT_ERROR_INVALID_INPUT;
    size_t length = (size_t)read_unsigned(wire + 4, 4);
    size_t padding = (8 - length % 8) % 8;
    if (length > available - 8 || padding > available - 8 - length)
        return KMIPKIT_ERROR_INVALID_INPUT;
    const uint8_t *data = wire + 8;
    for (size_t index = 0; index < padding; ++index)
        if (data[length + index] != 0) return KMIPKIT_ERROR_INVALID_INPUT;

    kmipkit_raw_tag_t *raw = NULL;
    kmipkit_tag_t *tag = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    int32_t status = kmipkit_ttlv_raw_tag_create((uint32_t)read_unsigned(wire, 3), &raw);
    if (status == 0) status = kmipkit_ttlv_raw_tag_try_checked(raw, &tag);
    if (status == 0 && wire[3] == 1) {
        kmipkit_ttlv_structure_t *fields = NULL;
        status = kmipkit_ttlv_structure_create(&fields);
        for (size_t offset = 0; status == 0 && offset < length;) {
            kmipkit_ttlv_item_t *field = NULL;
            size_t count = 0;
            status = construct_item(data + offset, length - offset, depth + 1, limits, &count, &field);
            if (status == 0) {
                kmipkit_ttlv_structure_t *updated = NULL;
                status = kmipkit_ttlv_structure_with_item(fields, field, limits, &updated);
                fields = updated;
                offset += count;
            }
        }
        if (status == 0) {
            status = kmipkit_ttlv_value_structure(fields, limits, &value);
        } else {
            kmipkit_ttlv_structure_release(fields);
        }
    } else if (status == 0) {
        status = construct_scalar(wire[3], data, length, limits, &value);
    }
    if (status == 0) status = kmipkit_ttlv_item_create(tag, value, limits, item);
    kmipkit_tag_release(tag);
    kmipkit_raw_tag_release(raw);
    if (status == 0) *consumed = 8 + length + padding;
    return status;
}

static bool decode_hex(const char *text, size_t length, uint8_t *wire) {
    if (length % 2 != 0) return false;
    for (size_t index = 0; index < length; index += 2) {
        int high = hex_digit(text[index]);
        int low = hex_digit(text[index + 1]);
        if (high < 0 || low < 0) return false;
        wire[index / 2] = (uint8_t)(high * 16 + low);
    }
    return true;
}

static bool inspect_line(char *line, uint8_t *wire, kmipkit_codec_limits_t *limits, bool construct) {
    size_t length = strcspn(line, "\n");
    if (line[length] != '\n' || !decode_hex(line, length, wire)) return false;
    kmipkit_ttlv_item_t *item = NULL;
    kmipkit_ttlv_decode_error_v1_t error = {
        .struct_size = (uint32_t)sizeof(error), .abi_version = 1,
    };
    size_t consumed = 0;
    int32_t status = construct
        ? construct_item(wire, length / 2, 0, limits, &consumed, &item)
        : kmipkit_ttlv_decode(wire, length / 2, &error, &item);
    if (status != 0 || (construct && consumed != length / 2)) {
        kmipkit_ttlv_item_release(item);
        puts("null");
        return true;
    }
    kmipkit_ttlv_item_view_t *view = NULL;
    bool ok = kmipkit_ttlv_item_view_from_item(item, &view) == 0;
    bool roundtrip = true;
    if (ok && construct) {
        size_t expected = length / 2;
        uint8_t *encoded = malloc(expected);
        size_t offset = 0;
        ok = encoded != NULL;
        if (ok) {
            roundtrip = encode_generic_item(view, encoded, expected, &offset) &&
                        offset == expected && memcmp(wire, encoded, expected) == 0;
        }
        free(encoded);
    }
    if (ok && roundtrip) {
        fputc('[', stdout);
        ok = print_item(view, true);
        puts("]");
    } else {
        puts("null");
    }
    kmipkit_ttlv_item_view_release(view);
    kmipkit_ttlv_item_release(item);
    return ok;
}

static int inspect_ttlv(bool construct) {
    enum { MAX_HEX_LINE = 2 * 1024 * 1024 + 2 };
    char *line = malloc(MAX_HEX_LINE);
    uint8_t *wire = malloc(MAX_HEX_LINE / 2);
    kmipkit_codec_limits_t *limits = NULL;
    bool ok = line != NULL && wire != NULL && kmipkit_codec_limits_defaults(&limits) == 0;
    if (ok) {
        printf("{\"library_commit\":\"%s\",\"artifact_sha256\":\"%s\"}\n",
               KMIPKIT_LIBRARY_COMMIT, KMIPKIT_C_ARTIFACT_SHA256);
    }
    while (ok && fgets(line, MAX_HEX_LINE, stdin)) {
        ok = inspect_line(line, wire, limits, construct);
    }
    kmipkit_codec_limits_release(limits);
    free(wire);
    free(line);
    return ok && !ferror(stdin) ? EXIT_SUCCESS : EXIT_FAILURE;
}

enum { KEY_FIXTURE_LIMIT = 4096, KEY_FIXTURE_FIELDS = 16 };

static int32_t decode_fixture_item(const char *hex, kmipkit_ttlv_item_t **item) {
    uint8_t wire[KEY_FIXTURE_LIMIT];
    size_t length = strlen(hex);
    if (length > sizeof(wire) * 2 || !decode_hex(hex, length, wire))
        return KMIPKIT_ERROR_INVALID_INPUT;
    kmipkit_ttlv_decode_error_v1_t error = {
        .struct_size = (uint32_t)sizeof(error), .abi_version = 1,
    };
    return kmipkit_ttlv_decode(wire, length / 2, &error, item);
}

static const char *fixture_verdict(int32_t status, kmipkit_ttlv_item_t *item,
                                        const uint8_t *expected, size_t length) {
    if (status == KMIPKIT_ERROR_INVALID_INPUT || status == KMIPKIT_ERROR_INVALID_SCHEMA ||
        status == KMIPKIT_ERROR_TYPED_ATTRIBUTE_TAG_MISMATCH ||
        status == KMIPKIT_ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH)
        return "rejected";
    if (status != 0) return "failed";

    uint8_t actual[KEY_FIXTURE_LIMIT];
    size_t written = 0;
    kmipkit_ttlv_item_view_t *view = NULL;
    bool equal = kmipkit_ttlv_item_view_from_item(item, &view) == 0 &&
                 encode_generic_item(view, actual, length, &written) &&
                 written == length && memcmp(actual, expected, length) == 0;
    kmipkit_ttlv_item_view_release(view);
    return equal ? "passed" : "failed";
}

static bool inspect_model_line(char *line, bool managed) {
    const char *format_text = strtok(line, " \n");
    const char *wire_hex = strtok(NULL, " \n");
    uint32_t format;
    char suffix;
    if (format_text == NULL || wire_hex == NULL ||
        sscanf(format_text, "%" SCNu32 "%c", &format, &suffix) != 1)
        return false;
    uint8_t expected[KEY_FIXTURE_LIMIT];
    size_t length = strlen(wire_hex);
    if (length > sizeof(expected) * 2 || !decode_hex(wire_hex, length, expected)) return false;

    kmipkit_ttlv_item_t *fields[KEY_FIXTURE_FIELDS] = {NULL};
    kmipkit_ttlv_item_t *input = NULL, *constructed = NULL, *read = NULL;
    bool ok = decode_fixture_item(wire_hex, &input) == 0;
    size_t count = 0;
    for (const char *field = strtok(NULL, " \n"); ok && field != NULL; field = strtok(NULL, " \n")) {
        if (count == KEY_FIXTURE_FIELDS) {
            ok = false;
            break;
        }
        ok = decode_fixture_item(field, &fields[count++]) == 0;
    }
    if (ok) {
        int32_t construction = managed
            ? kmipkit_managed_object_from_fields(format, fields, count, &constructed)
            : kmipkit_key_material_from_fields(format, fields, count, &constructed);
        int32_t decoding = managed
            ? kmipkit_managed_object_from_item(format, input, &read)
            : kmipkit_key_material_from_item(format, input, &read);
        printf("{\"construct\":\"%s\",\"read\":\"%s\"}\n",
               fixture_verdict(construction, constructed, expected, length / 2),
               fixture_verdict(decoding, read, expected, length / 2));
    }
    kmipkit_ttlv_item_release(read);
    kmipkit_ttlv_item_release(constructed);
    kmipkit_ttlv_item_release(input);
    for (size_t index = 0; index < count; ++index) kmipkit_ttlv_item_release(fields[index]);
    return ok;
}

static int inspect_model(bool managed) {
    char line[4 * KEY_FIXTURE_LIMIT + 128];
    printf("{\"library_commit\":\"%s\",\"artifact_sha256\":\"%s\"}\n",
           KMIPKIT_LIBRARY_COMMIT, KMIPKIT_C_ARTIFACT_SHA256);
    while (fgets(line, sizeof line, stdin)) {
        if (strchr(line, '\n') == NULL || !inspect_model_line(line, managed)) return EXIT_FAILURE;
    }
    return !ferror(stdin) && !ferror(stdout) ? EXIT_SUCCESS : EXIT_FAILURE;
}

static bool inspect_typed_attribute_line(char *line) {
    const char *tag_text = strtok(line, " \n");
    const char *input_hex = strtok(NULL, " \n");
    const char *expected_hex = strtok(NULL, " \n");
    uint32_t tag;
    char suffix;
    if (tag_text == NULL || input_hex == NULL || expected_hex == NULL ||
        sscanf(tag_text, "%" SCNu32 "%c", &tag, &suffix) != 1)
        return false;

    uint8_t expected[KEY_FIXTURE_LIMIT];
    size_t length = strlen(expected_hex);
    if (length > sizeof(expected) * 2 || !decode_hex(expected_hex, length, expected))
        return false;

    kmipkit_ttlv_item_t *input = NULL, *constructed = NULL, *read = NULL;
    kmipkit_ttlv_value_t *value = NULL;
    bool ok = decode_fixture_item(input_hex, &input) == 0 &&
              kmipkit_ttlv_item_value_copy(input, &value) == 0;
    if (ok) {
        int32_t construction = kmipkit_typed_attribute_from_value(tag, value, &constructed);
        int32_t decoding = kmipkit_typed_attribute_from_item(tag, input, &read);
        printf("{\"construct\":\"%s\",\"read\":\"%s\"}\n",
               fixture_verdict(construction, constructed, expected, length / 2),
               fixture_verdict(decoding, read, expected, length / 2));
    }
    kmipkit_ttlv_value_release(value);
    kmipkit_ttlv_item_release(read);
    kmipkit_ttlv_item_release(constructed);
    kmipkit_ttlv_item_release(input);
    return ok;
}

static int inspect_typed_attributes(void) {
    char line[4 * KEY_FIXTURE_LIMIT + 128];
    printf("{\"library_commit\":\"%s\",\"artifact_sha256\":\"%s\"}\n",
           KMIPKIT_LIBRARY_COMMIT, KMIPKIT_C_ARTIFACT_SHA256);
    while (fgets(line, sizeof line, stdin)) {
        if (strchr(line, '\n') == NULL || !inspect_typed_attribute_line(line)) return EXIT_FAILURE;
    }
    return !ferror(stdin) && !ferror(stdout) ? EXIT_SUCCESS : EXIT_FAILURE;
}

static int inspect_version(void) {
    char line[32];
    printf("{\"library_commit\":\"%s\",\"artifact_sha256\":\"%s\"}\n",
           KMIPKIT_LIBRARY_COMMIT, KMIPKIT_C_ARTIFACT_SHA256);
    while (fgets(line, sizeof line, stdin)) {
        uint32_t code;
        char kind[12], suffix;
        if (sscanf(line, "%11s %" SCNu32 "%c", kind, &code, &suffix) != 3 || suffix != '\n')
            return EXIT_FAILURE;
        for (int32_t minor = 0; minor <= 1; ++minor) {
            bool assigned;
            if (strcmp(kind, "tag") == 0)
                assigned = kmipkit_version_gate_validate_tag(2, minor, code) == 0;
            else if (strcmp(kind, "key-format") == 0)
                assigned = KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_name(code) != NULL;
            else if (strcmp(kind, "object-type") == 0)
                assigned = KMIPKIT_ENUMERATION_OBJECT_TYPE_name(code) != NULL;
            else
                return EXIT_FAILURE;
            puts(assigned ? "true" : "false");
        }
    }
    return !ferror(stdin) && !ferror(stdout) ? EXIT_SUCCESS : EXIT_FAILURE;
}

int main(int argc, char **argv) {
    int passed = 1;
    if (argc == 2 && strcmp(argv[1], "--inspect-oasis-ttlv") == 0) {
        return inspect_ttlv(false);
    }
    if (argc == 2 && strcmp(argv[1], "--construct-oasis-ttlv") == 0) {
        return inspect_ttlv(true);
    }
    if (argc == 2 && strcmp(argv[1], "--inspect-table-key-material") == 0) {
        return inspect_model(false);
    }
    if (argc == 2 && strcmp(argv[1], "--inspect-table-managed-objects") == 0) {
        return inspect_model(true);
    }
    if (argc == 2 && strcmp(argv[1], "--inspect-table-typed-attributes") == 0) {
        return inspect_typed_attributes();
    }
    if (argc == 2 && strcmp(argv[1], "--inspect-table-version") == 0) {
        return inspect_version();
    }
    puts(KMIPKIT_LIBRARY_COMMIT);
    puts(KMIPKIT_C_ARTIFACT_SHA256);
    if (argc == 2 && strcmp(argv[1], "--tags") == 0) {
        return check_tags();
    }
    if (argc != 1) {
        return EXIT_FAILURE;
    }
    for (size_t index = 0; index < sizeof ROUTES / sizeof ROUTES[0]; ++index) {
        int result = check_route(&ROUTES[index]);
        printf("%s %s\n", ROUTES[index].symbol, result ? "passed" : "failed");
        passed &= result;
    }
    return passed ? EXIT_SUCCESS : EXIT_FAILURE;
}
