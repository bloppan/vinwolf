#!/usr/bin/env bash
set -euo pipefail

BIN="vinwolf-target"

SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
MANIFEST_PATH="$ROOT_DIR/$BIN/Cargo.toml"
SUBMODULE_DIR="$ROOT_DIR/external/conformance_testing/"

if [[ ! -e "$ROOT_DIR/external/conformance_testing/.git" ]]; then
  echo "Submódulo external/conformance_testing no inicializado"; exit 1
fi

cargo build --manifest-path "$MANIFEST_PATH" --release
cp "$ROOT_DIR/target/release/$BIN" "$SUBMODULE_DIR/linux/tiny/x86_64/"
cargo build --manifest-path "$MANIFEST_PATH" --release --features=full --no-default-features
cp "$ROOT_DIR/target/release/$BIN" "$SUBMODULE_DIR/linux/full/x86_64/"

