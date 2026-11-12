#!/bin/sh
# Regenerate THIRD_PARTY_NOTICES.md.
#
# Requirements:
#   cargo install cargo-about --locked --features cli
#   (aarch64-apple-darwin target is the only shipped architecture)
set -eu

cd "$(dirname "$0")/.."

cargo about generate --offline --target aarch64-apple-darwin \
  scripts/about.hbs -o target/rust_third_party.md

{
  cat <<'EOF'
# Third-Party Notices

MacUtl is a macOS fork of NeoUtl. This file lists every third-party
component that is compiled into MacUtl, bundled into `MacUtl.app`, or shipped
in this repository, together with its license and the applicable notices.

MacUtl itself is licensed under the GNU Affero General Public License,
version 3 or later. See `LICENSE` and `MACOS_CHANGES.md`.

## 1. Native libraries bundled in MacUtl.app

EOF
  cat third_party/NON_RUST_COMPONENTS.md
  printf '\n\n---\n\n'
  cat target/rust_third_party.md
} > THIRD_PARTY_NOTICES.md

echo "wrote THIRD_PARTY_NOTICES.md ($(wc -l < THIRD_PARTY_NOTICES.md) lines)"
