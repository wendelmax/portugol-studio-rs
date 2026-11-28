#!/bin/sh
set -eu

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
source_bin="$script_dir/binaries/ferro"
target_dir="$HOME/.local/bin"
target_bin="$target_dir/ferro"
profile="$HOME/.zprofile"

if [ ! -x "$source_bin" ]; then
  printf 'Binário do Ferro CLI não encontrado em: %s\n' "$source_bin" >&2
  exit 1
fi

mkdir -p "$target_dir"
install -m 755 "$source_bin" "$target_bin"
if ! grep -Fqx 'export PATH="$HOME/.local/bin:$PATH"' "$profile" 2>/dev/null; then
  printf '\nexport PATH="$HOME/.local/bin:$PATH"\n' >> "$profile"
fi
printf 'Ferro CLI instalado em %s\nAbra um novo Terminal para usar `ferro`.\n' "$target_bin"
