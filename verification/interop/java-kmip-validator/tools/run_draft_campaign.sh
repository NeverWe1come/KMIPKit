#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/.."
exec ./build/install/java-kmip-validator/bin/java-kmip-validator "$@"
