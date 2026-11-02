#!/usr/bin/env bash
set -euo pipefail
if [ -n "$(cargo tree --workspace --all-features --target all --invert rsa 2>/dev/null)" ]; then
  printf '%s\n' 'The RSA advisory exception is no longer valid: rsa is in the build graph.' >&2
  exit 1
fi
cargo audit
cargo deny check
pnpm audit
