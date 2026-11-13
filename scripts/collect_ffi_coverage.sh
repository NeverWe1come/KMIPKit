#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd -- "${script_dir}/.." && pwd)"
report_dir="${1:-${repository_root}/coverage-ffi}"
report_dir="$(mkdir -p -- "${report_dir}" && cd -- "${report_dir}" && pwd)"
target_root="${CARGO_TARGET_DIR:-${repository_root}/target}"
ffi_target="${target_root}/ffi-coverage"

if [[ "$(uname -s)" != Linux ]]; then
    printf 'The C consumer FFI coverage collector currently supports Linux only.\n' >&2
    exit 2
fi

# Keep this package report isolated from the workspace report and its profraw files.
readonly raw_report="${report_dir}/coverage-raw.json"
readonly cargo_output="${report_dir}/cargo-llvm-cov.log"
cargo_status=0
CARGO_TERM_COLOR=never CARGO_TARGET_DIR="${ffi_target}" cargo llvm-cov \
    -p kmipkit-ffi --all-targets --all-features --locked \
    --json --output-path "${raw_report}" > "${cargo_output}" 2>&1 || cargo_status=$?
cat "${cargo_output}"
if [[ "${cargo_status}" -ne 0 ]]; then
    exit "${cargo_status}"
fi

python3 "${repository_root}/scripts/coverage_gate.py" normalize \
    --workspace "${repository_root}" \
    --input "${raw_report}" \
    --output "${report_dir}/coverage.json"
printf 'Rust C ABI coverage report: %s\n' "${report_dir}/coverage.json"
