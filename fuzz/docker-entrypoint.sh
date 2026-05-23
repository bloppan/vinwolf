#!/usr/bin/env sh
set -eu

usage() {
  cat <<'EOF'
Usage:
  docker run -e JAM_FUZZ=1 \
    -e JAM_FUZZ_SPEC=tiny|full \
    -e JAM_FUZZ_DATA_PATH=/path/to/data \
    -e JAM_FUZZ_SOCK_PATH=/path/to/socket \
    [-e JAM_FUZZ_LOG_LEVEL=level] \
    <image>

Legacy explicit fuzz mode:
  docker run <image> [sock=/path/to/socket]

Only JAM_FUZZ=1 enables standard fuzz mode. Any other value, including an unset
JAM_FUZZ, runs vinwolf-target-tiny --fuzz. In that mode, JAM_FUZZ_* variables
are ignored. JAM_FUZZ_LOG_LEVEL is optional and is never required to start fuzz
mode.
EOF
}

target_for_spec() {
  case "$1" in
    tiny|TINY|Tiny)
      echo "/usr/local/bin/vinwolf-target-tiny"
      ;;
    full|FULL|Full)
      echo "/usr/local/bin/vinwolf-target-full"
      ;;
    *)
      echo "Invalid spec '$1'. Expected 'tiny' or 'full'." >&2
      exit 64
      ;;
  esac
}

run_fuzz() {
  spec="$1"
  sock="$2"
  data_path="${3:-}"

  if [ -z "$sock" ]; then
    echo "Socket path cannot be empty." >&2
    exit 64
  fi

  target="$(target_for_spec "$spec")"
  mkdir -p "$(dirname "$sock")"

  if [ -n "$data_path" ]; then
    mkdir -p "$data_path"
    cd "$data_path"
  fi

  exec "$target" --fuzz "$sock"
}

run_tiny_fuzz() {
  sock="$1"

  if [ -n "$sock" ]; then
    mkdir -p "$(dirname "$sock")"
    exec /usr/local/bin/vinwolf-target-tiny --fuzz "$sock"
  fi

  exec /usr/local/bin/vinwolf-target-tiny --fuzz
}

if [ "${JAM_FUZZ:-}" = "1" ]; then
  missing=0

  if [ -z "${JAM_FUZZ_SPEC:-}" ]; then
    echo "JAM_FUZZ_SPEC must be set and non-empty when JAM_FUZZ=1." >&2
    missing=1
  fi

  if [ -z "${JAM_FUZZ_DATA_PATH:-}" ]; then
    echo "JAM_FUZZ_DATA_PATH must be set and non-empty when JAM_FUZZ=1." >&2
    missing=1
  fi

  if [ -z "${JAM_FUZZ_SOCK_PATH:-}" ]; then
    echo "JAM_FUZZ_SOCK_PATH must be set and non-empty when JAM_FUZZ=1." >&2
    missing=1
  fi

  if [ "$missing" -ne 0 ]; then
    exit 64
  fi

  run_fuzz "$JAM_FUZZ_SPEC" "$JAM_FUZZ_SOCK_PATH" "$JAM_FUZZ_DATA_PATH"
fi

legacy_sock=""

for arg in "$@"; do
  case "$arg" in
    sock=*)
      legacy_sock="${arg#sock=}"
      ;;
    socket=*)
      legacy_sock="${arg#socket=}"
      ;;
    target-sock=*)
      legacy_sock="${arg#target-sock=}"
      ;;
    spec=*)
      ;;
    "")
      ;;
    -h|--help|help)
      usage
      exit 0
      ;;
    *)
      echo "Unsupported argument when JAM_FUZZ is not 1: $arg" >&2
      usage >&2
      exit 64
      ;;
  esac
done

run_tiny_fuzz "$legacy_sock"
