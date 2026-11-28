# Como contribuir

Este guia vale para qualquer pessoa ou sessão trabalhando no projeto ao mesmo
tempo que outras. O objetivo é que duas pessoas nunca peguem a mesma issue e
que mudanças paralelas não entrem em conflito.

Para preparar o ambiente, compilar e testar, veja a seção
[Para desenvolvedores](README.md#para-desenvolvedores) do README.

## 1. Escolher uma issue

As issues abertas têm duas labels:

| Label | Significado |
|---|---|
| `status: pronta` | pode ser reservada |
| `status: em andamento` | já tem responsável e uma branch `issue/<número>` |
| `status: bloqueada` | depende de outra issue ainda aberta |
| `área: ...` | parte do código que a issue altera (shell da IDE, CSS, linguagem PORS, docs, CI) |

Use o script para ver o que está livre:

```bash
scripts/issue.sh listar
```

Ele separa as issues prontas para pegar, as prontas mas com a área ocupada,
as em andamento (com o tempo desde a última atividade) e as bloqueadas.

### Regras

- **Só pegue issue `status: pronta`**, sem dependência aberta.
- **Uma issue em andamento por área.** Duas issues da mesma área mexem nos
  mesmos arquivos e geram conflito. Se tiver certeza de que não há arquivos em
  comum, `pegar` aceita `--forcar`.
- **Reserve antes de começar** com `scripts/issue.sh pegar <número>`.
- **Desistiu ou travou?** Rode `scripts/issue.sh liberar <número>` para
  devolver a issue. A branch é apagada se não tiver trabalho; se tiver, fica
  para quem continuar.
- **Reserva expira** depois de 48 horas sem commit na branch nem atualização no
  PR. O `listar` marca a issue como `EXPIRADA`; nesse caso, comente na issue e
  libere antes de reservar de novo.

### Como a reserva funciona

`scripts/issue.sh pegar <número>`:

1. confere se a issue está pronta, sem dependência aberta e com a área livre;
2. cria a branch `issue/<número>` pela API do GitHub com um commit vazio de
   reserva. Se a branch já existe, a criação falha e o script para: só uma
   pessoa consegue reservar cada issue, mesmo que duas tentem ao mesmo tempo;
3. troca a label para `status: em andamento` e comenta quem reservou
   (`ISSUE_RESPONSAVEL`, ou o `user.name` do git);
4. abre um PR em rascunho com `Closes #<número>`.

## 2. Trabalhar na issue

Use uma pasta separada (worktree) por issue, para não misturar com o que
estiver aberto na pasta principal:

```bash
git fetch origin issue/<número>
git worktree add ../portugol-studio-rs-issue-<número> issue/<número>
cd ../portugol-studio-rs-issue-<número>
```

- Mantenha o escopo da issue. Achou outro problema? Abra uma issue nova.
- PRs pequenos entram mais rápido e conflitam menos. Uma issue grande pode
  virar vários PRs; nesse caso use `Parte de #<número>` em vez de `Closes`.
- Antes de cada commit, atualize a branch com a `main`
  (`git fetch origin && git rebase origin/main`).
- Não altere arquivos fora do escopo nem apague arquivos de outras pessoas,
  inclusive os não versionados.

### Commits

As mensagens seguem [Conventional Commits](https://www.conventionalcommits.org/pt-br/):
`feat`, `fix`, `docs`, `refactor`, `test`, `build`, `ci`, `chore`, com escopo
opcional (`feat(pors): ...`, `fix(ide): ...`). O tipo define o número da
próxima versão e o grupo em que o commit aparece nas notas de release
(veja [`docs/DISTRIBUICAO.md`](docs/DISTRIBUICAO.md)).

Não adicione trailers de coautoria nem assinaturas de ferramentas às mensagens
de commit e às descrições de PR.

### Validação

Rode o que corresponde ao que você alterou:

| Alterou | Comandos |
|---|---|
| crates Rust | `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |
| frontend da IDE | em `apps/desktop`: `npx tsc --noEmit`, `npm test`, `npm run build` |
| app Tauri | `cargo clippy -p ferro-desktop -- -D warnings` |

O CI roda só os jobs ligados aos arquivos alterados: mudanças apenas em
documentação não disparam build nem testes.

## 3. Entregar

1. Descreva no PR o que mudou, por que, e o resultado dos comandos de
   validação. Diga o que não foi testado.
2. Marque o PR como pronto: `gh pr ready issue/<número>`.
3. O merge é feito com **squash** e só com o CI verde. A issue fecha sozinha
   pelo `Closes #<número>`.
4. Depois do merge, rode `scripts/issue.sh atualizar`: issues que dependiam da
   sua passam de `bloqueada` para `pronta`.
5. Remova a worktree: `git worktree remove ../portugol-studio-rs-issue-<número>`.

## 4. Criar issues

Uma issue boa pode ser pega por alguém sem contexto da conversa em que nasceu:

- **Problema:** o que está errado ou falta, com caminho de arquivo quando
  ajudar.
- **Proposta:** o que fazer, sem detalhar a implementação além do necessário.
- **Critérios de aceite:** como saber que terminou, incluindo testes.
- **Dependências:** escreva exatamente `Depende de #<número>`; o script lê
  essa frase para bloquear e desbloquear a issue.
- **Labels:** uma de status (`status: pronta` ou `status: bloqueada`) e uma de
  área.
