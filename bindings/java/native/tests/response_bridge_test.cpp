#include "../response_bridge.hpp"

#include <cassert>
#include <cstdint>
#include <limits>

namespace {

enum class Failure {
    none,
    response_getter,
    invalid_presence,
    excessive_fields,
    ttlv_getter,
    ttlv_presence_mismatch,
    invalid_payload,
    fields_array,
    identifier_record,
    ttlv_handle_array,
    result_array,
};

struct State {
    Failure failure = Failure::none;
    int32_t response_status = 0;
    int32_t ttlv_status = 0;
    int32_t last_status = 0;
    uint8_t present = 1;
    uint8_t ttlv_present = 1;
    bool return_null_ttlv = false;
    int64_t field_values[2] = {17, 29};
    size_t field_count = 2;
    int32_t response_get_calls = 0;
    int32_t ttlv_get_calls = 0;
    int32_t fields_calls = 0;
    int32_t identifier_calls = 0;
    int32_t handle_array_calls = 0;
    int32_t result_calls = 0;
    int32_t local_deletes = 0;
    int32_t ttlv_releases = 0;
    kmipkit_unique_identifier_v1_t identifier{};
    int fields_ref = 0;
    int identifier_ref = 0;
    int ttlv_handle_ref = 0;
    int result_ref = 0;
    int ttlv_native = 0;
};

int32_t get_response(
        void* context, uint8_t* present,
        kmipkit_unique_identifier_v1_t* identifier,
        int64_t* fields, size_t field_capacity, size_t* field_count) {
    auto& state = *static_cast<State*>(context);
    ++state.response_get_calls;
    if (state.failure == Failure::response_getter) return state.response_status;
    *present = state.failure == Failure::invalid_presence ? 2 : state.present;
    assert(identifier->struct_size == sizeof(*identifier));
    assert(identifier->abi_version == 1);
    identifier->kind = state.identifier.kind;
    identifier->enumeration_value = state.identifier.enumeration_value;
    identifier->integer_value = state.identifier.integer_value;
    identifier->text_data = state.identifier.text_data;
    identifier->text_length = state.identifier.text_length;
    const size_t reported_count = state.failure == Failure::excessive_fields
            ? std::numeric_limits<size_t>::max() : state.field_count;
    for (size_t index = 0; index < state.field_count && index < 2 && index < field_capacity; ++index) {
        fields[index] = state.field_values[index];
    }
    *field_count = reported_count;
    return state.response_status;
}

int32_t get_response_ttlv(void* context, uint8_t* present, void** ttlv) {
    auto& state = *static_cast<State*>(context);
    ++state.ttlv_get_calls;
    *present = state.failure == Failure::ttlv_presence_mismatch
            ? static_cast<uint8_t>(state.present == 1 ? 0 : 1)
            : state.ttlv_present;
    *ttlv = state.return_null_ttlv ? nullptr : static_cast<void*>(&state.ttlv_native);
    return state.failure == Failure::ttlv_getter ? state.ttlv_status : 0;
}

bool validate_response(
        void* context, uint8_t, const kmipkit_unique_identifier_v1_t*,
        const int64_t*, size_t) {
    return static_cast<State*>(context)->failure != Failure::invalid_payload;
}

void throw_status(void* context, int32_t status) {
    static_cast<State*>(context)->last_status = status;
}

void release_ttlv(void* context, void* ttlv) {
    if (ttlv != nullptr) ++static_cast<State*>(context)->ttlv_releases;
}

void* make_fields(void* context, uint8_t, const int64_t*, size_t) {
    auto& state = *static_cast<State*>(context);
    ++state.fields_calls;
    return state.failure == Failure::fields_array ? nullptr : &state.fields_ref;
}

void* make_identifier(void* context, const kmipkit_unique_identifier_v1_t*) {
    auto& state = *static_cast<State*>(context);
    ++state.identifier_calls;
    return state.failure == Failure::identifier_record ? nullptr : &state.identifier_ref;
}

void* make_ttlv_handle(void* context, void*) {
    auto& state = *static_cast<State*>(context);
    ++state.handle_array_calls;
    return state.failure == Failure::ttlv_handle_array ? nullptr : &state.ttlv_handle_ref;
}

void* make_result(void* context, void*, void*, void*) {
    auto& state = *static_cast<State*>(context);
    ++state.result_calls;
    return state.failure == Failure::result_array ? nullptr : &state.result_ref;
}

void delete_local(void* context, void*) {
    ++static_cast<State*>(context)->local_deletes;
}

kmipkit::jni::ResponseBridgeOperations operations(State& state) {
    return {
            &state,
            &get_response,
            &get_response_ttlv,
            &validate_response,
            &throw_status,
            &release_ttlv,
            &make_fields,
            &make_identifier,
            &make_ttlv_handle,
            &make_result,
            &delete_local};
}

void response_getter_error_is_reported_without_fetching_ttlv() {
    State state{};
    state.failure = Failure::response_getter;
    state.response_status = KMIPKIT_ERROR_INVALID_INPUT;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.last_status == KMIPKIT_ERROR_INVALID_INPUT);
    assert(state.ttlv_get_calls == 0);
    assert(state.ttlv_releases == 0);
}

void invalid_presence_is_rejected_before_fetching_ttlv() {
    State state{};
    state.failure = Failure::invalid_presence;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.last_status == KMIPKIT_ERROR_INVALID_INPUT);
    assert(state.ttlv_get_calls == 0);
}

void excessive_field_count_is_rejected_before_fetching_ttlv() {
    State state{};
    state.failure = Failure::excessive_fields;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.last_status == KMIPKIT_ERROR_INVALID_INPUT);
    assert(state.ttlv_get_calls == 0);
}

void ttlv_getter_error_is_reported_without_allocating_java_values() {
    State state{};
    state.failure = Failure::ttlv_getter;
    state.ttlv_status = KMIPKIT_ERROR_INVALID_INPUT;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.last_status == KMIPKIT_ERROR_INVALID_INPUT);
    assert(state.fields_calls == 0);
    assert(state.ttlv_releases == 0);
}

void mismatched_ttlv_presence_releases_native_ttlv() {
    State state{};
    state.failure = Failure::ttlv_presence_mismatch;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.last_status == KMIPKIT_ERROR_INVALID_INPUT);
    assert(state.ttlv_releases == 1);
    assert(state.fields_calls == 0);
}

void missing_ttlv_for_present_response_is_rejected() {
    State state{};
    state.return_null_ttlv = true;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.last_status == KMIPKIT_ERROR_INVALID_INPUT);
    assert(state.ttlv_releases == 0);
}

void invalid_payload_releases_native_ttlv() {
    State state{};
    state.failure = Failure::invalid_payload;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.last_status == KMIPKIT_ERROR_INVALID_INPUT);
    assert(state.ttlv_releases == 1);
}

void fields_array_failure_releases_native_ttlv() {
    State state{};
    state.failure = Failure::fields_array;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.ttlv_releases == 1);
    assert(state.local_deletes == 0);
}

void identifier_failure_deletes_fields_and_releases_native_ttlv() {
    State state{};
    state.failure = Failure::identifier_record;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.local_deletes == 1);
    assert(state.ttlv_releases == 1);
}

void handle_array_failure_deletes_prior_refs_and_releases_native_ttlv() {
    State state{};
    state.failure = Failure::ttlv_handle_array;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.local_deletes == 2);
    assert(state.ttlv_releases == 1);
}

void result_array_failure_deletes_local_refs_and_releases_native_ttlv() {
    State state{};
    state.failure = Failure::result_array;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == nullptr);
    assert(state.local_deletes == 3);
    assert(state.ttlv_releases == 1);
}

void successful_marshalling_transfers_ttlv_and_deletes_local_refs() {
    State state{};
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == &state.result_ref);
    assert(state.ttlv_releases == 0);
    assert(state.local_deletes == 3);
    assert(state.identifier_calls == 1);
    assert(state.result_calls == 1);
}

void absent_response_skips_identifier_creation_and_transfers_null_ttlv() {
    State state{};
    state.present = 0;
    state.ttlv_present = 0;
    state.return_null_ttlv = true;
    const auto ops = operations(state);

    assert(kmipkit::jni::marshal_client_batch_item_response(ops) == &state.result_ref);
    assert(state.identifier_calls == 0);
    assert(state.ttlv_releases == 0);
    assert(state.local_deletes == 2);
}

} // namespace

int main() {
    response_getter_error_is_reported_without_fetching_ttlv();
    invalid_presence_is_rejected_before_fetching_ttlv();
    excessive_field_count_is_rejected_before_fetching_ttlv();
    ttlv_getter_error_is_reported_without_allocating_java_values();
    mismatched_ttlv_presence_releases_native_ttlv();
    missing_ttlv_for_present_response_is_rejected();
    invalid_payload_releases_native_ttlv();
    fields_array_failure_releases_native_ttlv();
    identifier_failure_deletes_fields_and_releases_native_ttlv();
    handle_array_failure_deletes_prior_refs_and_releases_native_ttlv();
    result_array_failure_deletes_local_refs_and_releases_native_ttlv();
    successful_marshalling_transfers_ttlv_and_deletes_local_refs();
    absent_response_skips_identifier_creation_and_transfers_null_ttlv();
    return 0;
}
