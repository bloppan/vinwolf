#!/usr/bin/env bash
set -euo pipefail

BIN="vinwolf-target"
CONFIGS=(tiny full)
ARCHS=(x86_64-unknown-linux-musl aarch64-unknown-linux-musl)

# Añadir targets si no existen
for arch in "${ARCHS[@]}"; do
  rustup target add "$arch" >/dev/null 2>&1 || true
done

for cfg in "${CONFIGS[@]}"; do
  for arch in "${ARCHS[@]}"; do
    echo ">> Compilando $BIN [$cfg / $arch]..."

    # Usar zigbuild si el target es musl y está instalado
    if [[ "$arch" == *musl* ]] && command -v cargo-zigbuild >/dev/null 2>&1; then
      CMD=(cargo zigbuild --release --target "$arch")
    else
      CMD=(cargo build --release --target "$arch")
    fi

    # Selección de features según config
    if [[ "$cfg" == "tiny" ]]; then
      # tiny es la default → no pasamos flags
      "${CMD[@]}" -p "$BIN"
    else
      # full → desactivamos default y activamos full
      "${CMD[@]}" -p "$BIN" --no-default-features --features full
    fi

    # Normalizamos nombre de carpeta
    case "$arch" in
      x86_64-*) out_arch="x86_64" ;;
      aarch64-*) out_arch="aarch64" ;;
      *) out_arch="$arch" ;;
    esac

    out_dir="dist/$cfg/$out_arch"
    mkdir -p "$out_dir"
    cp "target/$arch/release/$BIN" "$out_dir/"
    echo "OK -> $out_dir/$BIN"
  done
done
