#!/usr/bin/env bash
# Thin wrapper around `cargo xtask package` for users who prefer a script path.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$WORKSPACE_ROOT"
exec cargo xtask package "$@"
