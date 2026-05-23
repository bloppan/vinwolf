#!/usr/bin/env bash
set -euo pipefail

BIN="vinwolf-target"
CONFIGS=(tiny full)
ARCHS=(aarch64-unknown-linux-musl x86_64-apple-darwin aarch64-apple-darwin)

SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
MANIFEST_PATH="$ROOT_DIR/Cargo.toml"
SUBMODULE_DIR="${VINWOLF_CONFORMANCE_DIR:-$ROOT_DIR/tests/conformance_testing}"

if [[ ! -e "$SUBMODULE_DIR/.git" ]]; then
  echo "Submódulo tests/conformance_testing no inicializado"; exit 1
fi

cargo build --manifest-path "$MANIFEST_PATH" --release -p "$BIN"
mkdir -p "$SUBMODULE_DIR/linux/tiny/x86_64"
cp "$ROOT_DIR/target/release/$BIN" "$SUBMODULE_DIR/linux/tiny/x86_64/"
cargo build --manifest-path "$MANIFEST_PATH" --release -p "$BIN" --features=full --no-default-features
mkdir -p "$SUBMODULE_DIR/linux/full/x86_64"
cp "$ROOT_DIR/target/release/$BIN" "$SUBMODULE_DIR/linux/full/x86_64/"

if [[ "${VINWOLF_BUILD_HOST_ONLY:-0}" == "1" ]]; then
  exit 0
fi

for arch in "${ARCHS[@]}"; do
  rustup target add "$arch" >/dev/null 2>&1 || true
done

for cfg in "${CONFIGS[@]}"; do
  for arch in "${ARCHS[@]}"; do
    echo ">> Compilando $BIN [$cfg / $arch]..."

    if [[ "$arch" == *musl* || "$arch" == *apple-darwin* ]] && command -v cargo-zigbuild >/dev/null 2>&1; then
      CMD=(cargo zigbuild --manifest-path "$MANIFEST_PATH" --release --target "$arch")
    else
      CMD=(cargo build    --manifest-path "$MANIFEST_PATH" --release --target "$arch")
    fi

    if [[ "$cfg" == "tiny" ]]; then
      "${CMD[@]}" -p "$BIN"
    else
      "${CMD[@]}" -p "$BIN" --no-default-features --features full
    fi

    case "$arch" in
      x86_64-*) out_arch="x86_64" ;;
      aarch64-*) out_arch="aarch64" ;;
      *) out_arch="$arch" ;;
    esac

    if [[ "$arch" == *apple-darwin* ]]; then
      out_os="macos"
    else
      out_os="linux"
    fi

    dist_dir="$ROOT_DIR/target/dist/$out_os/$cfg/$out_arch"
    subm_dir="$SUBMODULE_DIR/$out_os/$cfg/$out_arch/"
    mkdir -p "$dist_dir" "$subm_dir"

    src_bin="$ROOT_DIR/target/$arch/release/$BIN"
    cp "$src_bin" "$dist_dir/"
    cp "$src_bin" "$subm_dir/"

    echo "OK -> $dist_dir/$BIN"
    echo "OK -> $SUBMODULE_DIR/$BIN"
  done
done

