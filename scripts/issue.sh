#!/usr/bin/env bash
# Distribui as issues entre colaboradores sem que duas pessoas peguem a mesma.
#
# Uso:
#   scripts/issue.sh listar             issues prontas, em andamento e bloqueadas
#   scripts/issue.sh pegar <n>          reserva a issue: branch issue/<n>, label e PR em rascunho
#   scripts/issue.sh liberar <n>        devolve a issue; apaga a branch se não houver trabalho nela
#   scripts/issue.sh atualizar          bloqueia ou desbloqueia issues conforme as dependências
#
# A reserva é a criação da branch issue/<n> pela API do GitHub: se ela já
# existe, a criação falha, então só uma pessoa consegue pegar cada issue.
#
# Variáveis:
#   ISSUE_RESPONSAVEL   nome que aparece no comentário de reserva (padrão: git config user.name)
#   ISSUE_EXPIRA_HORAS  horas sem atividade até a reserva ser considerada expirada (padrão: 48)
#
# Regras completas em CONTRIBUTING.md.

set -euo pipefail

PRONTA="status: pronta"
ANDAMENTO="status: em andamento"
BLOQUEADA="status: bloqueada"

REPO="${ISSUE_REPO:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}"
RESPONSAVEL="${ISSUE_RESPONSAVEL:-$(git config user.name 2>/dev/null || true)}"
RESPONSAVEL="${RESPONSAVEL:-$(hostname)}"
EXPIRA_HORAS="${ISSUE_EXPIRA_HORAS:-48}"

falha() {
    echo "erro: $*" >&2
    exit 1
}

# Números citados como "Depende de #N" no corpo da issue.
dependencias() {
    gh issue view "$1" -R "$REPO" --json body -q .body |
        grep -oiE 'depende de #[0-9]+' | grep -oE '[0-9]+' | sort -u || true
}

dependencias_abertas() {
    local dep
    for dep in $(dependencias "$1"); do
        [[ "$(gh issue view "$dep" -R "$REPO" --json state -q .state)" == "OPEN" ]] && echo "$dep"
    done
    return 0
}

labels() {
    gh issue view "$1" -R "$REPO" --json labels -q '.labels[].name'
}

area() {
    labels "$1" | grep '^área: ' || true
}

# Issues em andamento na mesma área, exceto a própria.
area_ocupada() {
    local n="$1" a="$2"
    [[ -z "$a" ]] && return 0
    gh issue list -R "$REPO" --state open --label "$ANDAMENTO" --label "$a" \
        --json number -q '.[].number' | grep -vx "$n" || true
}

trocar_status() {
    local n="$1" novo="$2" s
    local args=()
    for s in "$PRONTA" "$ANDAMENTO" "$BLOQUEADA"; do
        [[ "$s" != "$novo" ]] && args+=(--remove-label "$s")
    done
    gh issue edit "$n" -R "$REPO" --add-label "$novo" "${args[@]}" >/dev/null
}

horas_desde() {
    jq -nr --arg d "$1" '((now - ($d | fromdateiso8601)) / 3600) | floor'
}

# Horas desde a última atividade na branch issue/<n> ou no PR dela; vazio se não houver branch.
horas_sem_atividade() {
    local n="$1" data pr
    # Sem a branch, o gh api falha (404) e escreve o erro na saída: descarta.
    data="$(gh api "repos/$REPO/branches/issue/$n" -q .commit.commit.committer.date 2>/dev/null)" || return 0
    [[ -z "$data" ]] && return 0
    pr="$(gh pr list -R "$REPO" --head "issue/$n" --state open --json updatedAt -q '.[0].updatedAt' 2>/dev/null)" || pr=""
    [[ -n "$pr" && "$pr" > "$data" ]] && data="$pr"
    horas_desde "$data"
}

cmd_listar() {
    local issues
    issues="$(gh issue list -R "$REPO" --state open --limit 200 --json number,title,labels \
        -q '.[] | [.number, .title, ([.labels[].name] | join(";"))] | @tsv')"

    local livres=() ocupadas=() andamento=() bloqueadas=() sem_status=()
    local n titulo lbls a abertas ocup horas
    while IFS=$'\t' read -r n titulo lbls; do
        [[ -z "$n" ]] && continue
        a="$(tr ';' '\n' <<<"$lbls" | grep '^área: ' | head -1 || true)"
        case ";$lbls;" in
            *";$ANDAMENTO;"*)
                horas="$(horas_sem_atividade "$n")"
                if [[ -z "$horas" ]]; then
                    andamento+=("#$n  $titulo  [${a:-sem área}]  sem branch issue/$n")
                elif (( horas >= EXPIRA_HORAS )); then
                    andamento+=("#$n  $titulo  [${a:-sem área}]  EXPIRADA: ${horas}h sem atividade")
                else
                    andamento+=("#$n  $titulo  [${a:-sem área}]  última atividade há ${horas}h")
                fi
                ;;
            *";$BLOQUEADA;"*)
                abertas="$(dependencias_abertas "$n" | sed 's/^/#/' | paste -sd' ' -)"
                if [[ -z "$abertas" ]]; then
                    bloqueadas+=("#$n  $titulo  dependências fechadas; rode: scripts/issue.sh atualizar")
                else
                    bloqueadas+=("#$n  $titulo  aguarda $abertas")
                fi
                ;;
            *";$PRONTA;"*)
                abertas="$(dependencias_abertas "$n" | sed 's/^/#/' | paste -sd' ' -)"
                ocup="$(area_ocupada "$n" "$a" | sed 's/^/#/' | paste -sd' ' -)"
                if [[ -n "$abertas" ]]; then
                    bloqueadas+=("#$n  $titulo  marcada como pronta, mas aguarda $abertas")
                elif [[ -n "$ocup" ]]; then
                    ocupadas+=("#$n  $titulo  [$a] ocupada por $ocup")
                else
                    livres+=("#$n  $titulo  [${a:-sem área}]")
                fi
                ;;
            *)
                sem_status+=("#$n  $titulo")
                ;;
        esac
    done <<<"$issues"

    secao() {
        local titulo="$1"; shift
        echo "$titulo"
        if (( $# == 0 )); then echo "  (nenhuma)"; else printf '  %s\n' "$@"; fi
        echo
    }
    secao "Prontas para pegar:" "${livres[@]}"
    secao "Prontas, mas com a área ocupada:" "${ocupadas[@]}"
    secao "Em andamento:" "${andamento[@]}"
    secao "Bloqueadas:" "${bloqueadas[@]}"
    (( ${#sem_status[@]} > 0 )) && secao "Sem label de status:" "${sem_status[@]}"
    return 0
}

cmd_pegar() {
    local n="${1:-}" forcar="${2:-}"
    [[ "$n" =~ ^[0-9]+$ ]] || falha "uso: scripts/issue.sh pegar <número> [--forcar]"

    local estado titulo
    estado="$(gh issue view "$n" -R "$REPO" --json state -q .state)"
    [[ "$estado" == "OPEN" ]] || falha "a issue #$n não está aberta"
    labels "$n" | grep -qx "$PRONTA" || falha "a issue #$n não está marcada como \"$PRONTA\""

    local abertas a ocup
    abertas="$(dependencias_abertas "$n" | sed 's/^/#/' | paste -sd' ' -)"
    [[ -z "$abertas" ]] || falha "a issue #$n depende de $abertas, ainda aberta(s)"
    a="$(area "$n")"
    ocup="$(area_ocupada "$n" "$a" | sed 's/^/#/' | paste -sd' ' -)"
    if [[ -n "$ocup" && "$forcar" != "--forcar" ]]; then
        falha "a área \"$a\" já tem $ocup em andamento; escolha outra issue ou use --forcar se não houver conflito de arquivos"
    fi

    titulo="$(gh issue view "$n" -R "$REPO" --json title -q .title)"
    local main_sha tree commit
    main_sha="$(gh api "repos/$REPO/git/ref/heads/main" -q .object.sha)"
    tree="$(gh api "repos/$REPO/git/commits/$main_sha" -q .tree.sha)"
    commit="$(gh api "repos/$REPO/git/commits" -f message="chore: reserva a issue #$n" \
        -f tree="$tree" -f "parents[]=$main_sha" -q .sha)"

    if ! gh api "repos/$REPO/git/refs" -f ref="refs/heads/issue/$n" -f sha="$commit" >/dev/null 2>&1; then
        falha "a issue #$n já foi reservada (a branch issue/$n existe); escolha outra"
    fi

    trocar_status "$n" "$ANDAMENTO"
    gh issue comment "$n" -R "$REPO" --body "Reservada por $RESPONSAVEL na branch \`issue/$n\`." >/dev/null
    local pr
    pr="$(gh pr create -R "$REPO" --draft --base main --head "issue/$n" --title "$titulo" \
        --body "Closes #$n" 2>/dev/null | tail -1 || true)"

    echo "Issue #$n reservada por $RESPONSAVEL."
    [[ -n "$pr" ]] && echo "PR em rascunho: $pr"
    cat <<EOF

Para trabalhar numa pasta separada:
  git fetch origin issue/$n
  git worktree add ../$(basename "$(git rev-parse --show-toplevel)")-issue-$n issue/$n

Ao terminar, marque o PR como pronto para revisão: gh pr ready issue/$n
EOF
}

cmd_liberar() {
    local n="${1:-}"
    [[ "$n" =~ ^[0-9]+$ ]] || falha "uso: scripts/issue.sh liberar <número>"

    local novo="$PRONTA"
    [[ -n "$(dependencias_abertas "$n")" ]] && novo="$BLOQUEADA"
    trocar_status "$n" "$novo"

    local adiante
    adiante="$(gh api "repos/$REPO/compare/main...issue/$n" -q .ahead_by 2>/dev/null)" || adiante=""
    if [[ -z "$adiante" ]]; then
        echo "Sem branch issue/$n."
    elif (( adiante <= 1 )); then
        gh pr list -R "$REPO" --head "issue/$n" --state open --json number -q '.[].number' |
            while read -r pr; do gh pr close "$pr" -R "$REPO" >/dev/null; done
        gh api -X DELETE "repos/$REPO/git/refs/heads/issue/$n" >/dev/null
        echo "Branch issue/$n apagada (não tinha trabalho além da reserva)."
    else
        echo "A branch issue/$n tem $((adiante - 1)) commit(s) de trabalho e foi mantida."
    fi
    gh issue comment "$n" -R "$REPO" --body "Liberada por $RESPONSAVEL." >/dev/null
    echo "Issue #$n liberada como \"$novo\"."
}

cmd_atualizar() {
    local n abertas mudou=0
    for n in $(gh issue list -R "$REPO" --state open --label "$BLOQUEADA" --json number -q '.[].number'); do
        if [[ -z "$(dependencias_abertas "$n")" ]]; then
            trocar_status "$n" "$PRONTA"
            gh issue comment "$n" -R "$REPO" --body "Dependências fechadas; issue pronta para ser reservada." >/dev/null
            echo "#$n: bloqueada -> pronta"
            mudou=1
        fi
    done
    for n in $(gh issue list -R "$REPO" --state open --label "$PRONTA" --json number -q '.[].number'); do
        abertas="$(dependencias_abertas "$n" | sed 's/^/#/' | paste -sd' ' -)"
        if [[ -n "$abertas" ]]; then
            trocar_status "$n" "$BLOQUEADA"
            echo "#$n: pronta -> bloqueada (aguarda $abertas)"
            mudou=1
        fi
    done
    (( mudou )) || echo "Nada a atualizar."
}

case "${1:-}" in
    listar) cmd_listar ;;
    pegar) shift; cmd_pegar "$@" ;;
    liberar) shift; cmd_liberar "$@" ;;
    atualizar) cmd_atualizar ;;
    *) sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'; exit 1 ;;
esac
