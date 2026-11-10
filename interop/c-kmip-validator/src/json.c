#include "json.h"

#include <ctype.h>
#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define JSON_MAX_BYTES (16U * 1024U * 1024U)
#define JSON_MAX_TOKENS 100000U

static void skip_space(const char *source, size_t length, size_t *position) {
    while (*position < length && isspace((unsigned char)source[*position])) {
        (*position)++;
    }
}

static int add_token(json_document_t *document, json_token_type_t type, size_t start, int parent) {
    if (document->token_count >= document->token_capacity) {
        return -1;
    }
    size_t index = document->token_count++;
    document->tokens[index] = (json_token_t){
        .type = type, .start = start, .end = start, .next = index + 1, .parent = parent
    };
    return (int)index;
}

static bool parse_value(json_document_t *document, size_t length, size_t *position, int parent);

static bool parse_string(json_document_t *document, size_t length, size_t *position, int parent) {
    size_t start = ++(*position);
    while (*position < length) {
        unsigned char value = (unsigned char)document->source[*position];
        if (value == '"') {
            int token = add_token(document, JSON_TOKEN_STRING, start, parent);
            if (token < 0) return false;
            document->tokens[token].end = *position;
            (*position)++;
            document->tokens[token].next = document->token_count;
            return true;
        }
        if (value < 0x20) return false;
        if (value == '\\') {
            (*position)++;
            if (*position >= length) return false;
            char escape = document->source[*position];
            if (escape == 'u') {
                for (size_t i = 1; i <= 4; ++i) {
                    if (*position + i >= length || !isxdigit((unsigned char)document->source[*position + i])) return false;
                }
                *position += 4;
            } else if (!strchr("\"\\/bfnrt", escape)) {
                return false;
            }
        }
        (*position)++;
    }
    return false;
}

static bool parse_container(json_document_t *document, size_t length, size_t *position,
                            int parent, json_token_type_t type, char close) {
    int token = add_token(document, type, *position, parent);
    if (token < 0) return false;
    (*position)++;
    skip_space(document->source, length, position);
    if (*position < length && document->source[*position] == close) {
        (*position)++;
        document->tokens[token].end = *position;
        document->tokens[token].next = document->token_count;
        return true;
    }
    for (;;) {
        if (type == JSON_TOKEN_OBJECT) {
            if (*position >= length || document->source[*position] != '"' ||
                !parse_string(document, length, position, token)) return false;
            skip_space(document->source, length, position);
            if (*position >= length || document->source[(*position)++] != ':') return false;
            skip_space(document->source, length, position);
        }
        if (!parse_value(document, length, position, token)) return false;
        skip_space(document->source, length, position);
        if (*position >= length) return false;
        char separator = document->source[(*position)++];
        if (separator == close) break;
        if (separator != ',') return false;
        skip_space(document->source, length, position);
    }
    document->tokens[token].end = *position;
    document->tokens[token].next = document->token_count;
    return true;
}

static bool parse_value(json_document_t *document, size_t length, size_t *position, int parent) {
    skip_space(document->source, length, position);
    if (*position >= length) return false;
    char value = document->source[*position];
    if (value == '"') return parse_string(document, length, position, parent);
    if (value == '{') return parse_container(document, length, position, parent, JSON_TOKEN_OBJECT, '}');
    if (value == '[') return parse_container(document, length, position, parent, JSON_TOKEN_ARRAY, ']');

    size_t start = *position;
    while (*position < length && !isspace((unsigned char)document->source[*position]) &&
           document->source[*position] != ',' && document->source[*position] != ']' &&
           document->source[*position] != '}') {
        (*position)++;
    }
    if (start == *position) return false;
    int token = add_token(document, JSON_TOKEN_PRIMITIVE, start, parent);
    if (token < 0) return false;
    document->tokens[token].end = *position;
    document->tokens[token].next = document->token_count;
    return true;
}

bool json_document_load(const char *path, json_document_t *out_document) {
    if (!path || !out_document) return false;
    memset(out_document, 0, sizeof(*out_document));
    FILE *file = fopen(path, "rb");
    if (!file) return false;
    if (fseek(file, 0, SEEK_END) != 0) { fclose(file); return false; }
    long length = ftell(file);
    if (length <= 0 || (unsigned long)length > JSON_MAX_BYTES || fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        return false;
    }
    out_document->source = malloc((size_t)length + 1);
    out_document->token_capacity = JSON_MAX_TOKENS;
    out_document->tokens = calloc(out_document->token_capacity, sizeof(*out_document->tokens));
    if (!out_document->source || !out_document->tokens) {
        fclose(file);
        json_document_release(out_document);
        return false;
    }
    size_t read = fread(out_document->source, 1, (size_t)length, file);
    fclose(file);
    if (read != (size_t)length) { json_document_release(out_document); return false; }
    out_document->source[read] = '\0';
    size_t position = 0;
    bool valid = parse_value(out_document, read, &position, -1);
    skip_space(out_document->source, read, &position);
    if (!valid || position != read || out_document->token_count == 0) {
        json_document_release(out_document);
        return false;
    }
    return true;
}

void json_document_release(json_document_t *document) {
    if (!document) return;
    free(document->source);
    free(document->tokens);
    memset(document, 0, sizeof(*document));
}

static bool raw_string_equals(const json_document_t *document, int token, const char *value) {
    if (!document || !value || token < 0 || (size_t)token >= document->token_count ||
        document->tokens[token].type != JSON_TOKEN_STRING) return false;
    size_t start = document->tokens[token].start;
    size_t length = document->tokens[token].end - start;
    return strlen(value) == length && memcmp(document->source + start, value, length) == 0;
}

int json_object_get(const json_document_t *document, int object, const char *name) {
    if (!document || object < 0 || (size_t)object >= document->token_count ||
        document->tokens[object].type != JSON_TOKEN_OBJECT || !name) return -1;
    size_t cursor = (size_t)object + 1;
    size_t end = document->tokens[object].next;
    while (cursor < end) {
        size_t key = cursor;
        if (key + 1 >= end) return -1;
        size_t value = key + 1;
        if (raw_string_equals(document, (int)key, name)) return (int)value;
        cursor = document->tokens[value].next;
    }
    return -1;
}

int json_array_get(const json_document_t *document, int array, size_t index) {
    if (!document || array < 0 || (size_t)array >= document->token_count ||
        document->tokens[array].type != JSON_TOKEN_ARRAY) return -1;
    size_t cursor = (size_t)array + 1;
    size_t end = document->tokens[array].next;
    while (cursor < end) {
        if (index-- == 0) return (int)cursor;
        cursor = document->tokens[cursor].next;
    }
    return -1;
}

size_t json_array_length(const json_document_t *document, int array) {
    if (!document || array < 0 || (size_t)array >= document->token_count ||
        document->tokens[array].type != JSON_TOKEN_ARRAY) return 0;
    size_t count = 0;
    for (size_t cursor = (size_t)array + 1; cursor < document->tokens[array].next;
         cursor = document->tokens[cursor].next) count++;
    return count;
}

bool json_object_member_at(const json_document_t *document, int object, size_t index,
                           int *out_key, int *out_value) {
    if (!document || object < 0 || (size_t)object >= document->token_count ||
        document->tokens[object].type != JSON_TOKEN_OBJECT || !out_key || !out_value) return false;
    size_t cursor = (size_t)object + 1;
    size_t end = document->tokens[object].next;
    while (cursor + 1 < end) {
        size_t key = cursor;
        size_t value = key + 1;
        if (index-- == 0) { *out_key = (int)key; *out_value = (int)value; return true; }
        cursor = document->tokens[value].next;
    }
    return false;
}

bool json_token_equals(const json_document_t *document, int token, const char *value) {
    if (!document || !value || token < 0 || (size_t)token >= document->token_count) return false;
    const json_token_t *entry = &document->tokens[token];
    size_t length = entry->end - entry->start;
    return strlen(value) == length && memcmp(document->source + entry->start, value, length) == 0;
}

bool json_string_copy(const json_document_t *document, int token, char *out, size_t capacity) {
    if (!document || token < 0 || (size_t)token >= document->token_count || !out || capacity == 0 ||
        document->tokens[token].type != JSON_TOKEN_STRING) return false;
    const json_token_t *entry = &document->tokens[token];
    size_t input = entry->start;
    size_t output = 0;
    while (input < entry->end) {
        unsigned char value = (unsigned char)document->source[input++];
        if (value == '\\') {
            if (input >= entry->end) return false;
            char escape = document->source[input++];
            switch (escape) {
                case '"': value = '"'; break;
                case '\\': value = '\\'; break;
                case '/': value = '/'; break;
                case 'b': value = '\b'; break;
                case 'f': value = '\f'; break;
                case 'n': value = '\n'; break;
                case 'r': value = '\r'; break;
                case 't': value = '\t'; break;
                case 'u':
                    if (input + 4 > entry->end) return false;
                    input += 4;
                    value = '?';
                    break;
                default: return false;
            }
        }
        if (output + 1 >= capacity) return false;
        out[output++] = (char)value;
    }
    out[output] = '\0';
    return true;
}

bool json_boolean(const json_document_t *document, int token, bool *out_value) {
    if (!document || !out_value || token < 0 || (size_t)token >= document->token_count ||
        document->tokens[token].type != JSON_TOKEN_PRIMITIVE) return false;
    if (json_token_equals(document, token, "true")) { *out_value = true; return true; }
    if (json_token_equals(document, token, "false")) { *out_value = false; return true; }
    return false;
}

bool json_integer(const json_document_t *document, int token, long *out_value) {
    if (!document || !out_value || token < 0 || (size_t)token >= document->token_count ||
        document->tokens[token].type != JSON_TOKEN_PRIMITIVE) return false;
    const json_token_t *entry = &document->tokens[token];
    size_t length = entry->end - entry->start;
    if (length == 0 || length >= 32) return false;
    char buffer[32];
    memcpy(buffer, document->source + entry->start, length);
    buffer[length] = '\0';
    errno = 0;
    char *end = NULL;
    long value = strtol(buffer, &end, 10);
    if (errno || !end || *end) return false;
    *out_value = value;
    return true;
}
