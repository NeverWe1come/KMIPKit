#!/usr/bin/env bash
set -euo pipefail
project="$(cd -- "$(dirname -- "$0")/.." && pwd)"
exec "$project/target/debug/rust-kmip-validator" query-campaign "$@"
