#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <artifacts-dir>" >&2
  exit 1
fi

DIR="$1"
(
  cd "$DIR"
  if [[ -f SHA256SUMS ]]; then
    sha256sum -c SHA256SUMS
  else
    find . -maxdepth 2 -type f ! -name "SHA256SUMS" -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
    cat SHA256SUMS
  fi
)
