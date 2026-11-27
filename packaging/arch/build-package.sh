#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo_root"

raw_version="${1:-}"
if [[ -z "$raw_version" ]]; then
  raw_version="$(sed -n '/^\[workspace.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p}' Cargo.toml)"
fi

pkgver="${raw_version#v}"
pkgver="${pkgver//-/.}"
if [[ ! "$pkgver" =~ ^[0-9A-Za-z._]+$ ]]; then
  echo "Versão inválida para pkgver do Arch Linux: '$raw_version' -> '$pkgver'" >&2
  exit 1
fi

output_dir="${2:-$repo_root/target/arch}"
mkdir -p "$output_dir"
output_dir="$(cd "$output_dir" && pwd)"

build_dir="$(mktemp -d "/tmp/portugol-arch-build-$$.XXXXXX")"
cleanup() {
  rm -rf "$build_dir"
}
trap cleanup EXIT

tar \
  --exclude="./.git" \
  --exclude="./target" \
  --exclude="./apps/desktop/node_modules" \
  --exclude="./apps/desktop/dist" \
  --exclude="./packaging/arch/*.tar.gz" \
  --exclude="./packaging/arch/*.pkg.tar.*" \
  --transform="s,^\.,portugol-studio-rs-${pkgver}," \
  -czf "${build_dir}/portugol-studio-rs-${pkgver}.tar.gz" \
  .

cp packaging/arch/PKGBUILD "${build_dir}/PKGBUILD"
sed -i "s/^pkgver=.*/pkgver=${pkgver}/" "${build_dir}/PKGBUILD"

if [[ "$(id -u)" -eq 0 ]]; then
  if ! id -u builder >/dev/null 2>&1; then
    useradd -m builder
  fi
  chown -R builder:builder "$build_dir"
  su - builder -c "cd '$build_dir' && makepkg --noconfirm"
else
  (cd "$build_dir" && makepkg --noconfirm)
fi

shopt -s nullglob
packages=("${build_dir}"/portugol-studio-rs-*.pkg.tar.zst)
shopt -u nullglob

if [[ ${#packages[@]} -ne 1 ]]; then
  echo "Esperado exatamente 1 pacote .pkg.tar.zst, encontrados: ${#packages[@]}" >&2
  exit 1
fi

pkg_path="${packages[0]}"
pkg_list="$(tar --zstd -tf "$pkg_path")"
for required in \
  "usr/bin/portugol-desktop" \
  "usr/bin/portugol-studio-rs" \
  "usr/share/applications/portugol-studio-rs.desktop" \
  "usr/share/mime/packages/portugol-studio-rs.xml" \
  "usr/share/icons/hicolor/scalable/apps/portugol-studio-rs.svg" \
  "usr/share/licenses/portugol-studio-rs/LICENSE"; do
  if ! grep -Fxq "$required" <<<"$pkg_list"; then
    echo "Arquivo obrigatório ausente no pacote Arch: $required" >&2
    exit 1
  fi
done

pkg_name="$(basename "$pkg_path")"
cp "$pkg_path" "${output_dir}/${pkg_name}"
echo "Pacote Arch Linux gerado em: ${output_dir}/${pkg_name}"
