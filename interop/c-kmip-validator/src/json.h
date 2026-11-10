#ifndef KMIP_INTEROP_JSON_H
#define KMIP_INTEROP_JSON_H

#include <stdbool.h>
#include <stddef.h>

typedef enum {
    JSON_TOKEN_OBJECT,
    JSON_TOKEN_ARRAY,
    JSON_TOKEN_STRING,
    JSON_TOKEN_PRIMITIVE
} json_token_type_t;

typedef struct {
    json_token_type_t type;
    size_t start;
    size_t end;
    size_t next;
    int parent;
} json_token_t;

typedef struct {
    char *source;
    json_token_t *tokens;
    size_t token_count;
    size_t token_capacity;
} json_document_t;

bool json_document_load(const char *path, json_document_t *out_document);
void json_document_release(json_document_t *document);
int json_object_get(const json_document_t *document, int object, const char *name);
int json_array_get(const json_document_t *document, int array, size_t index);
size_t json_array_length(const json_document_t *document, int array);
bool json_object_member_at(const json_document_t *document, int object, size_t index,
                           int *out_key, int *out_value);
bool json_string_copy(const json_document_t *document, int token, char *out, size_t capacity);
bool json_token_equals(const json_document_t *document, int token, const char *value);
bool json_boolean(const json_document_t *document, int token, bool *out_value);
bool json_integer(const json_document_t *document, int token, long *out_value);

#endif
