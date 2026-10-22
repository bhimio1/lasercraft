#!/usr/bin/env bash
# Prints the workspace version from Cargo.toml (docs/spec/release.md).
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
sed -n '/^\[workspace.package\]/,/^\[/s/^version *= *"\(.*\)"/\1/p' "$root/Cargo.toml" | head -1
