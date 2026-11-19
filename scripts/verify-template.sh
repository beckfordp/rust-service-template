#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Verifies the template: generates both variants (kafka=false, kafka=true) into a
# scratch gluon layout (services/, lib/gluon-rs linked to gluon's), and for each
# runs the generated README's steps 1, 2 and 5 — rename to -rs, `cargo update -w
# --offline`, scripts/ci.sh — and gluon bin/check-locks on its lock. Each check
# printed PASS or FAIL, its log kept.
# Usage: scripts/verify-template.sh   (from anywhere). Exits 1 on any FAIL.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GLUON="$(cd "$ROOT/../.." && pwd)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/verify-template.XXXXXX")"
mkdir -p "$WORK/services" "$WORK/lib"
ln -s "$GLUON/lib/gluon-rs" "$WORK/lib/gluon-rs"
echo "template — work and logs in $WORK"
FAILED=0

# check LABEL LOG COMMAND...: PASS when COMMAND exits 0.
check() {
  local label="$1" log="$WORK/$2"
  shift 2
  if "$@" > "$log" 2>&1; then
    echo "PASS $label"
  else
    echo "FAIL $label (log: $log)"
    FAILED=$((FAILED + 1))
  fi
}

generate() {
  local name="$1" kafka="$2"
  cd "$WORK/services" &&
    cargo generate --path "$ROOT" template --name "$name" \
      --define short_name=demo --define "kafka=$kafka" --vcs none --silent &&
    mv "$name" "$name-rs" &&
    cd "$name-rs" &&
    cargo update -w --offline
}

for variant in "demo-service false" "demo-events-service true"; do
  read -r name kafka <<< "$variant"
  check "$name (kafka=$kafka) generates" "$name-generate.log" generate "$name" "$kafka"
  check "$name: scripts/ci.sh" "$name-ci.log" bash "$WORK/services/$name-rs/scripts/ci.sh"
  check "$name: no Liquid left unrendered" "$name-liquid.log" \
    bash -c "! grep -rn '{{\\|{%' '$WORK/services/$name-rs' --exclude-dir=target"
  check "$name: locked crates old enough for the mirror" "$name-locks.log" \
    "$GLUON/bin/check-locks" "$WORK/services/$name-rs/Cargo.lock"
done

if [ "$FAILED" = 0 ]; then
  echo "PASS: the template's two variants generate and pass CI"
else
  echo "FAIL: $FAILED check(s) failed"
  exit 1
fi
