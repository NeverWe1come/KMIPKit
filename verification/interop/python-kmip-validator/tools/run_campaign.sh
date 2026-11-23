#!/usr/bin/env bash
set -euo pipefail
project="$(cd -- "$(dirname -- "$0")/.." && pwd)"
export PYTHONPATH="$project/src"
exec python3 -m kmipkit_validator.campaign_runner "$@"
