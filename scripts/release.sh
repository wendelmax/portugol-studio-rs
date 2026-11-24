#!/usr/bin/env bash
# Prepara uma nova versão: calcula o número, atualiza os manifestos, gera as
# notas a partir dos commits desde a última tag e acrescenta ao CHANGELOG.md.
#
# Uso: scripts/release.sh [automatica|patch|minor|major|X.Y.Z] [--dry-run]
#
# Não faz commit nem cria tag; o workflow "Preparar release" cuida disso.
# Com --dry-run só mostra a versão e as notas, sem alterar arquivos.
#
# Saídas (quando GITHUB_OUTPUT existe): versao, tag e o arquivo de notas em
# target/release-notes.md.

set -euo pipefail

cd "$(dirname "$0")/.."

tipo="${1:-automatica}"
dry_run=false
[[ "${2:-}" == "--dry-run" || "${1:-}" == "--dry-run" ]] && dry_run=true
[[ "$tipo" == "--dry-run" ]] && tipo=automatica

ultima_tag="$(git describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null || true)"
atual="$(sed -n '/^\[workspace.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p}' Cargo.toml)"
intervalo="${ultima_tag:+$ultima_tag..}HEAD"

# Assuntos dos commits, sem merges nem commits de release anteriores.
mapfile -t commits < <(git log --no-merges --format='%s' "$intervalo" |
    grep -vE '^chore\(release\)' || true)

if [[ ${#commits[@]} -eq 0 ]]; then
    echo "Nenhum commit desde ${ultima_tag:-o início}; nada para lançar." >&2
    exit 1
fi

IFS=. read -r maior menor correcao <<<"${atual%%-*}"

if [[ "$tipo" == "automatica" ]]; then
    tipo=patch
    for s in "${commits[@]}"; do
        if [[ "$s" =~ ^[a-z]+(\([^\)]*\))?!: ]]; then tipo=major; break; fi
        [[ "$s" =~ ^feat(\([^\)]*\))?: ]] && tipo=minor
    done
    if git log --format='%b' "$intervalo" | grep -q '^BREAKING CHANGE'; then
        tipo=major
    fi
    # Antes da 1.0, mudanças incompatíveis sobem apenas o número do meio.
    [[ "$tipo" == "major" && "$maior" == "0" ]] && tipo=minor
fi

case "$tipo" in
    major) nova="$((maior + 1)).0.0" ;;
    minor) nova="$maior.$((menor + 1)).0" ;;
    patch) nova="$maior.$menor.$((correcao + 1))" ;;
    [0-9]*.[0-9]*.[0-9]*) nova="$tipo" ;;
    *) echo "Tipo de versão inválido: $tipo" >&2; exit 1 ;;
esac

tag="v$nova"
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
    echo "A tag $tag já existe." >&2
    exit 1
fi

# Notas agrupadas pelo tipo do commit (Conventional Commits).
declare -A titulos=(
    [feat]="Novidades"
    [fix]="Correções"
    [perf]="Desempenho"
    [docs]="Documentação"
    [outros]="Manutenção"
)
declare -A grupos=()
for s in "${commits[@]}"; do
    if [[ "$s" =~ ^([a-z]+)(\(([^\)]*)\))?!?:[[:space:]]*(.*)$ ]]; then
        t="${BASH_REMATCH[1]}"
        escopo="${BASH_REMATCH[3]}"
        texto="${BASH_REMATCH[4]}"
    else
        t=outros escopo="" texto="$s"
    fi
    [[ -n "${titulos[$t]:-}" ]] || t=outros
    linha="- ${escopo:+**$escopo:** }$texto"
    grupos[$t]+="$linha"$'\n'
done

notas=""
for t in feat fix perf docs outros; do
    [[ -n "${grupos[$t]:-}" ]] || continue
    notas+="### ${titulos[$t]}"$'\n\n'"${grupos[$t]}"$'\n'
done

repo_url="https://github.com/wendelmax/portugol-studio-rs"
if [[ -n "$ultima_tag" ]]; then
    notas+="Comparação completa: $repo_url/compare/$ultima_tag...$tag"$'\n'
fi

if $dry_run; then
    echo "Versão: $atual -> $nova ($tag)"
    echo
    printf '%s' "$notas"
    exit 0
fi

# Manifestos com a versão do aplicativo.
sed -i "/^\[workspace.package\]/,/^\[/{s/^version = \".*\"/version = \"$nova\"/}" Cargo.toml
for json in apps/desktop/package.json apps/desktop/src-tauri/tauri.conf.json; do
    sed -i "0,/\"version\": \".*\"/s//\"version\": \"$nova\"/" "$json"
done
# package-lock.json repete a versão do pacote raiz em dois lugares.
node -e '
    const fs = require("fs");
    const f = "apps/desktop/package-lock.json";
    const lock = JSON.parse(fs.readFileSync(f, "utf8"));
    lock.version = process.argv[1];
    if (lock.packages && lock.packages[""]) lock.packages[""].version = process.argv[1];
    fs.writeFileSync(f, JSON.stringify(lock, null, 2) + "\n");
' "$nova"
# Cargo.lock: só os pacotes do próprio workspace.
cargo update --workspace --quiet

data="$(date -u +%Y-%m-%d)"
mkdir -p target
printf '%s' "$notas" > target/release-notes.md
{
    echo "# Histórico de versões"
    echo
    echo "## $nova ($data)"
    echo
    printf '%s' "$notas"
    if [[ -f CHANGELOG.md ]]; then
        echo
        tail -n +3 CHANGELOG.md
    fi
} > target/CHANGELOG.novo.md
mv target/CHANGELOG.novo.md CHANGELOG.md

echo "Versão $atual -> $nova"
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    {
        echo "versao=$nova"
        echo "tag=$tag"
    } >> "$GITHUB_OUTPUT"
fi
