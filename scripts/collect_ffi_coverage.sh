#!/usr/bin/env bash
set -euo pipefail

readonly llvm_version="20.1.8"
readonly llvm_major="20"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd -- "${script_dir}/.." && pwd)"
report_dir="${1:-${repository_root}/coverage-ffi}"
report_dir="$(mkdir -p -- "${report_dir}" && cd -- "${report_dir}" && pwd)"
target_root="${CARGO_TARGET_DIR:-${repository_root}/target}"
llvm_cov_target="${target_root}/llvm-cov-target"
source_file="${repository_root}/crates/kmipkit-ffi/src/extension_registry.rs"

if [[ "$(uname -s)" != Linux ]]; then
    printf 'The C consumer FFI coverage collector currently supports Linux only.\n' >&2
    exit 2
fi

for tool in "llvm-cov-${llvm_major}"; do
    if ! command -v "${tool}" >/dev/null 2>&1; then
        printf 'Required pinned FFI coverage tool is missing: %s (LLVM %s).\n' "${tool}" "${llvm_version}" >&2
        exit 2
    fi
    if ! "${tool}" --version 2>&1 | grep -Fq "${llvm_version}"; then
        printf 'FFI coverage tool %s must be LLVM %s.\n' "${tool}" "${llvm_version}" >&2
        exit 2
    fi
done

if [[ ! -f "${source_file}" ]]; then
    printf 'Expected Rust C ABI implementation source is missing: %s\n' "${source_file}" >&2
    exit 1
fi

# Keep cargo-llvm-cov's target and merged profile from the workspace collection.
cargo llvm-cov --no-clean -p kmipkit-ffi --test c_api_coverage --all-features --locked

readonly profile_data="${llvm_cov_target}/KMIPKit.profdata"
readonly ffi_dso="${llvm_cov_target}/debug/deps/libkmipkit_ffi.so"
readonly ffi_staticlib="${llvm_cov_target}/debug/deps/libkmipkit_ffi.a"
if [[ ! -f "${profile_data}" || ! -f "${ffi_dso}" || ! -f "${ffi_staticlib}" ]]; then
    printf 'The instrumented Rust C ABI objects or merged LLVM profile are missing under %s.\n' "${llvm_cov_target}" >&2
    exit 1
fi

test_binaries=("${llvm_cov_target}"/debug/deps/c_api_coverage-*)
test_binary=""
for candidate in "${test_binaries[@]}"; do
    if [[ -f "${candidate}" && -x "${candidate}" && "${candidate}" != *.d ]]; then
        if [[ -n "${test_binary}" ]]; then
            printf 'Multiple C ABI coverage test binaries found; expected one under %s.\n' "${llvm_cov_target}/debug/deps" >&2
            exit 1
        fi
        test_binary="${candidate}"
    fi
done
if [[ -z "${test_binary}" ]]; then
    printf 'The C ABI coverage test executable is missing under %s.\n' "${llvm_cov_target}/debug/deps" >&2
    exit 1
fi

readonly raw_report="${report_dir}/coverage-raw.json"
"llvm-cov-${llvm_major}" export -format=text \
    -instr-profile "${profile_data}" \
    -object "${ffi_dso}" \
    -object "${ffi_staticlib}" \
    -object "${test_binary}" \
    --sources "${source_file}" > "${raw_report}"

python3 "${repository_root}/scripts/coverage_gate.py" normalize \
    --workspace "${repository_root}" \
    --input "${raw_report}" \
    --output "${report_dir}/coverage.json"
printf 'Rust C ABI coverage report: %s\n' "${report_dir}/coverage.json"
