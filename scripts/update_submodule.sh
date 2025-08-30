#!/usr/bin/env bash
set -xeuo pipefail

SUBMODULE_PATH="external/conformance_testing"
ROOT="$(git rev-parse --show-toplevel)"
LAST_COMMIT=$(git -C "$ROOT" rev-parse --short=8 HEAD)

# Default msg
MSG="${1:-Update release $LAST_COMMIT}"
echo $MSG

cd "$ROOT/$SUBMODULE_PATH"
git add .
if ! git diff --cached --quiet; then
  git commit -m "$MSG"
  git push
fi

SUBMODULE_LAST_COMMIT=$(git rev-parse --short=8 HEAD)

cd "$ROOT"
git add "$SUBMODULE_PATH"
if ! git diff --cached --quiet; then
  git commit -m "Update conformance_testing submodule pointer: $SUBMODULE_LAST_COMMIT"
  git push
fi

echo "Done."
