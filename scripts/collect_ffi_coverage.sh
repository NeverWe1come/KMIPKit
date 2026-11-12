#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd -- "${script_dir}/.." && pwd)"
report_dir="${1:-${repository_root}/coverage-ffi}"
report_dir="$(mkdir -p -- "${report_dir}" && cd -- "${report_dir}" && pwd)"
target_root="${CARGO_TARGET_DIR:-${repository_root}/target}"
llvm_cov_target="${target_root}/llvm-cov-target"
# Keep the coverage report aligned with the full FFI source tree as modules are split out.
mapfile -d '' -t source_files < <(
    find "${repository_root}/crates/kmipkit-ffi/src" -type f -name "*.rs" -print0 | sort -z
)

if [[ "$(uname -s)" != Linux ]]; then
    printf 'The C consumer FFI coverage collector currently supports Linux only.\n' >&2
    exit 2
fi

for source_file in "${source_files[@]}"; do
    if [[ ! -f "${source_file}" ]]; then
        printf 'Expected Rust C ABI implementation source is missing: %s\n' "${source_file}" >&2
        exit 1
    fi
done

# Keep cargo-llvm-cov's target and merged profile from the workspace collection.
readonly cargo_output="${report_dir}/cargo-llvm-cov.log"
cargo_status=0
CARGO_TERM_COLOR=never cargo llvm-cov --no-clean -p kmipkit-ffi --all-targets --all-features --locked \
    > "${cargo_output}" 2>&1 || cargo_status=$?
cat "${cargo_output}"
if [[ "${cargo_status}" -ne 0 ]]; then
    exit "${cargo_status}"
fi

# Export profiles with the LLVM tools bundled for the active rustc toolchain.
rust_host="$(rustc -vV | sed -n 's/^host: //p')"
if [[ -z "${rust_host}" ]]; then
    printf 'Could not determine the active rustc host triple.\n' >&2
    exit 2
fi
rust_sysroot="$(rustc --print sysroot)"
llvm_cov="${rust_sysroot}/lib/rustlib/${rust_host}/bin/llvm-cov"
if [[ ! -x "${llvm_cov}" ]]; then
    printf 'LLVM coverage tool for active rustc is missing: %s\n' "${llvm_cov}" >&2
    exit 2
fi

readonly profile_data="${llvm_cov_target}/KMIPKit.profdata"
if [[ ! -f "${profile_data}" ]]; then
    printf 'The merged LLVM profile is missing under %s.\n' "${llvm_cov_target}" >&2
    exit 1
fi

mapfile -t test_binaries < <(
    sed -nE \
        -e '/Running unittests src\/lib\.rs /s@.*\((.*kmipkit_ffi-[[:xdigit:]]+)\).*@\1@p' \
        -e '/Running tests\/c_api_coverage\.rs /s@.*\((.*c_api_coverage-[[:xdigit:]]+)\).*@\1@p' \
        "${cargo_output}"
)
if [[ "${#test_binaries[@]}" -ne 2 ]]; then
    printf 'Cargo did not report both the FFI unit and C consumer test executables in %s.\n' \
        "${cargo_output}" >&2
    exit 1
fi
coverage_objects=()
for test_binary in "${test_binaries[@]}"; do
    if [[ "${test_binary}" != /* ]]; then
        test_binary="${repository_root}/${test_binary}"
    fi
    if [[ ! -f "${test_binary}" || ! -x "${test_binary}" ]]; then
        printf 'The FFI coverage test executable is missing under %s.\n' "${test_binary}" >&2
        exit 1
    fi
    coverage_objects+=(-object "${test_binary}")
done

readonly raw_report="${report_dir}/coverage-raw.json"
"${llvm_cov}" export -format=text \
    -instr-profile "${profile_data}" \
    "${coverage_objects[@]}" \
    --sources "${source_files[@]}" > "${raw_report}"

python3 "${repository_root}/scripts/coverage_gate.py" normalize \
    --workspace "${repository_root}" \
    --input "${raw_report}" \
    --output "${report_dir}/coverage.json"
printf 'Rust C ABI coverage report: %s\n' "${report_dir}/coverage.json"
