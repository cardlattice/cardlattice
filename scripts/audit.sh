#!/usr/bin/env bash
set -euo pipefail
if [ -n "$(cargo tree --workspace --all-features --target all --invert libsqlite3-sys 2>/dev/null)" ]; then
  printf '%s\n' 'The SQLite advisory exception is invalid: libsqlite3-sys is now in the build graph.' >&2
  exit 1
fi
cargo audit
cargo deny check
pnpm audit
