#!/usr/bin/env bash
# Compila `portugol-wasm` para `wasm32-unknown-unknown` e monta o bundle
# estático pronto para publicar em `apps/web-playground/dist/`.

set -euo pipefail

cd "$(dirname "$0")/.."

cargo build --release --target wasm32-unknown-unknown -p portugol-wasm

dist_dir="apps/web-playground/dist"
mkdir -p "$dist_dir"

cp target/wasm32-unknown-unknown/release/portugol_wasm.wasm "$dist_dir/portugol_wasm.wasm"
cp apps/web-playground/index.html \
   apps/web-playground/style.css \
   apps/web-playground/app.mjs \
   apps/web-playground/playground-core.mjs \
   "$dist_dir/"

echo "Web Playground pronto em $dist_dir"
ls -lh "$dist_dir"
