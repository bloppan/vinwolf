#!/usr/bin/env bash
set -euo pipefail

BIN="vinwolf-target"
CONFIGS=(tiny full)
ARCHS=(x86_64-unknown-linux-musl aarch64-unknown-linux-musl)

SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
MANIFEST_PATH="$ROOT_DIR/Cargo.toml"

for arch in "${ARCHS[@]}"; do
  rustup target add "$arch" >/dev/null 2>&1 || true
done

for cfg in "${CONFIGS[@]}"; do
  for arch in "${ARCHS[@]}"; do
    echo ">> Compilando $BIN [$cfg / $arch]..."

    if [[ "$arch" == *musl* ]] && command -v cargo-zigbuild >/dev/null 2>&1; then
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

    out_dir="$ROOT_DIR/target/dist/$cfg/$out_arch"
    mkdir -p "$out_dir"
    cp "$ROOT_DIR/target/$arch/release/$BIN" "$out_dir/"
    echo "OK -> $out_dir/$BIN"
  done
done

