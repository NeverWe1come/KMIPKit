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
    printf 'The merged LLVM profile is missing: %s\n' "${profile_data}" >&2
    exit 1
fi

# Ask Cargo for its exact instrumented artifact paths. Cargo 1.100 changed the
# intermediate layout, and a separate build directory is also configurable.
readonly artifact_manifest="${report_dir}/cargo-artifacts.jsonl"
(
    export CARGO_TARGET_DIR="${llvm_cov_target}"
    coverage_environment="$(cargo llvm-cov show-env --sh)"
    eval "${coverage_environment}"
    cargo test -p kmipkit-ffi --all-targets --all-features --locked --no-run \
        --message-format=json
) > "${artifact_manifest}"
readonly object_manifest="${report_dir}/coverage-objects.bin"
python3 "${repository_root}/scripts/ffi_coverage_objects.py" "${artifact_manifest}" \
    > "${object_manifest}"
mapfile -d '' -t object_paths < "${object_manifest}"
coverage_objects=()
for object_path in "${object_paths[@]}"; do
    coverage_objects+=(-object "${object_path}")
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
