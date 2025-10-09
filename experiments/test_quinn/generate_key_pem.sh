#!/bin/bash

# Verificar si se pasó la seed como argumento
if [ -z "$1" ]; then
  echo "Uso: $0 <seed_hex>"
  exit 1
fi

SEED="$1"
PREFIX="302e020100300506032b657004220420"
FULL_HEX="${PREFIX}${SEED}"

# Convertir hex a binario usando un loop (sin xxd ni dependencias externas)
FULL_BIN=""
i=0
while [ $i -lt ${#FULL_HEX} ]; do
  BYTE="${FULL_HEX:$i:2}"
  FULL_BIN="$FULL_BIN\\x$BYTE"
  i=$((i+2))
done

# Codificar en base64 (usando base64 estándar, sin -w 0 si no está disponible; ajusta si es macOS con base64 -b 0)
B64=$(printf "$FULL_BIN" | base64)

# Generar PEM y guardar en key.pem
echo "-----BEGIN PRIVATE KEY-----" > key.pem
echo "$B64" >> key.pem
echo "-----END PRIVATE KEY-----" >> key.pem

echo "key.pem generado correctamente."
