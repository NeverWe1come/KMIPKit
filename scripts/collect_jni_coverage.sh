#!/usr/bin/env bash
set -euo pipefail

readonly llvm_version="20.1.8"
readonly llvm_major="20"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd -- "${script_dir}/.." && pwd)"
output_dir="${1:-${repository_root}/target/coverage-jni}"
output_dir="$(mkdir -p -- "${output_dir}" && cd -- "${output_dir}" && pwd)"

for tool in "clang++-${llvm_major}" "llvm-profdata-${llvm_major}" "llvm-cov-${llvm_major}"; do
    if ! command -v "${tool}" >/dev/null 2>&1; then
        printf 'Required pinned JNI coverage tool is missing: %s (LLVM %s).\n' "${tool}" "${llvm_version}" >&2
        exit 2
    fi
    if ! "${tool}" --version 2>&1 | grep -Fq "${llvm_version}"; then
        printf 'JNI coverage tool %s must be LLVM %s.\n' "${tool}" "${llvm_version}" >&2
        exit 2
    fi
done

if [[ "$(uname -s)" != Linux ]]; then
    printf 'JNI native coverage collection currently supports Linux only.\n' >&2
    exit 2
fi

export CXX="clang++-${llvm_major}"
export KMIPKIT_JNI_COVERAGE=llvm
export LLVM_PROFILE_FILE="${output_dir}/profraw/%m-%p.profraw"
mkdir -p -- "${output_dir}/profraw"

mvn -B -f "${repository_root}/bindings/java/pom.xml" clean verify

profile_files=("${output_dir}"/profraw/*.profraw)
if [[ ! -f "${profile_files[0]}" ]]; then
    printf 'The Java suite produced no instrumented JNI profile data.\n' >&2
    exit 1
fi

readonly profile_data="${output_dir}/jni.profdata"
readonly raw_report="${output_dir}/coverage-raw.json"
readonly native_library="${repository_root}/bindings/java/target/native/libkmipkit_jni.so"
readonly ignored_source_regex='^/usr/lib/jvm/[^/]+/include/jni\.h$|^/opt/hostedtoolcache/Java_[^/]+/[^/]+/[^/]+/include/jni\.h$|/bindings/c/include/kmipkit\.h$'
"llvm-profdata-${llvm_major}" merge -sparse "${profile_files[@]}" -o "${profile_data}"
"llvm-cov-${llvm_major}" export "${native_library}" --instr-profile="${profile_data}" \
    --format=text --ignore-filename-regex="${ignored_source_regex}" > "${raw_report}"
python3 "${repository_root}/scripts/coverage_gate.py" normalize \
    --workspace "${repository_root}" \
    --input "${raw_report}" \
    --output "${output_dir}/coverage.json"
printf 'JNI bridge LLVM report: %s\n' "${output_dir}/coverage.json"
