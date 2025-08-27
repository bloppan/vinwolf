#!/usr/bin/env bash
set -euo pipefail

SUBMODULE_PATH="external/conformance_testing"
ROOT="$(git rev-parse --show-toplevel)"
LAST_COMMIT=$(git -C "$ROOT" rev-parse --short=8 HEAD)

# Default msg
MSG="${2:-Update release $LAST_COMMIT}"

cd "$ROOT/$SUBMODULE_PATH"
git add bin
if ! git diff --cached --quiet; then
  git commit -m "$MSG"
  git push
fi

SUBMODULE_LAST_COMMIT=$(git rev-parse --short=8 HEAD)

cd "$ROOT"
git add "$SUBMODULE_PATH"
if ! git diff --cached --quiet; then
  git commit -m "Update submodule pointer: $SUBMODULE_LAST_COMMIT"
  git push
fi

echo "Done."
