#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/build-docker-image.sh [--tag IMAGE] [--skip-target-build] [--dry-run]

Builds the vinwolf-target Docker image from the tiny/full binaries produced by
scripts/build.sh.

Options:
  -t, --tag IMAGE        Docker image tag (default: vinwolf-target:latest)
      --skip-target-build
                         Reuse existing tests/conformance_testing binaries.
      --dry-run          Prepare and validate the Docker build context only.
  -h, --help             Show this help.
EOF
}

IMAGE_TAG="${IMAGE_TAG:-vinwolf-target:latest}"
RUN_TARGET_BUILD=1
DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    -t|--tag)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for $1" >&2
        exit 64
      fi
      IMAGE_TAG="$2"
      shift 2
      ;;
    --skip-target-build)
      RUN_TARGET_BUILD=0
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 64
      ;;
  esac
done

SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFORMANCE_DIR="$ROOT_DIR/tests/conformance_testing"
TINY_BIN="$CONFORMANCE_DIR/linux/tiny/x86_64/vinwolf-target"
FULL_BIN="$CONFORMANCE_DIR/linux/full/x86_64/vinwolf-target"
DOCKERFILE="$ROOT_DIR/fuzz/Dockerfile"
ENTRYPOINT="$ROOT_DIR/fuzz/docker-entrypoint.sh"

if (( RUN_TARGET_BUILD )); then
  VINWOLF_BUILD_HOST_ONLY=1 "$SCRIPT_DIR/build.sh"
fi

for path in "$TINY_BIN" "$FULL_BIN" "$DOCKERFILE" "$ENTRYPOINT"; do
  if [[ ! -f "$path" ]]; then
    echo "Required file not found: $path" >&2
    exit 66
  fi
done

BUILD_CONTEXT="$(mktemp -d "${TMPDIR:-/tmp}/vinwolf-target-docker.XXXXXX")"
cleanup() {
  rm -rf "$BUILD_CONTEXT"
}
trap cleanup EXIT

install -m 0644 "$DOCKERFILE" "$BUILD_CONTEXT/Dockerfile"
install -m 0755 "$ENTRYPOINT" "$BUILD_CONTEXT/docker-entrypoint.sh"
install -m 0755 "$TINY_BIN" "$BUILD_CONTEXT/vinwolf-target-tiny"
install -m 0755 "$FULL_BIN" "$BUILD_CONTEXT/vinwolf-target-full"

if (( DRY_RUN )); then
  echo "Prepared and validated temporary Docker build context."
  echo "Would run: docker build --tag $IMAGE_TAG $BUILD_CONTEXT"
  exit 0
fi

docker build --tag "$IMAGE_TAG" "$BUILD_CONTEXT"
