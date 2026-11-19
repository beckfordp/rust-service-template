#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# CI for {{project-name}}-rs, defined once: the sandbox runs this script directly
# and GitHub Actions calls it (.github/workflows/ci.yml). Stops at the first
# failing step; each step announces itself with a "== " line. The steps are
# gluon-rs's scripts/lib/ci.sh (gluon TD-43.2), Linux and macOS.
# shellcheck source-path=SCRIPTDIR
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

# The crates depend on gluon-rs by path (../../lib/gluon-rs in gluon's layout);
# elsewhere, set GLUON_RS.
GLUON_RS="${GLUON_RS:-$PWD/../../lib/gluon-rs}"
if [ ! -f "$GLUON_RS/scripts/lib/ci.sh" ]; then
  echo "FAIL no scripts/lib/ci.sh under $GLUON_RS (set GLUON_RS)"
  exit 1
fi
# shellcheck source=../../../lib/gluon-rs/scripts/lib/ci.sh
. "$GLUON_RS/scripts/lib/ci.sh"

ci_fmt
ci_clippy
ci_coverage
# .cargo/audit.toml's ignores hold only while these are lockfile-only
ci_audit rsa libsqlite3-sys
