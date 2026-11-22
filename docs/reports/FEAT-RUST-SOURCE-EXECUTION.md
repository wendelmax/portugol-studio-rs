# Execução de arquivos Rust standalone

## Entregue nesta implementação

- Comando Tauri tipado para compilar e executar um único buffer `.rs` como Rust 2021.
- Limites de 1 MiB para fonte e saída combinada, 20 segundos para compilação e 3 segundos para execução.
- Diagnósticos do rustc com posições convertidas de bytes UTF-8 para offsets UTF-16 do CodeMirror.
- Supervisor encerra grupos/árvores de processos que herdaram os pipes, para evitar que filhos em segundo plano prendam a captura de saída.
- Confirmação de segurança por sessão antes da primeira execução; cancelamento não chama o compilador.
- Resultados obsoletos são descartados quando a aba ativa ou o estado exato do documento muda.
- Execução Rust sem depurador, sem Cargo/dependências e sem sandbox. O processo executa com as permissões da conta local.

## Validação

Validação executada nesta worktree (Linux x86_64, `rustc` disponível):

- `cargo fmt --all -- --check` — passou.
- `cargo test -p portugol-service --offline --locked` — passou: 55 testes, incluindo 6 cenários Linux de execução Rust standalone; o caso Windows de filho com pipe herdado também está coberto e roda no CI Windows.
- `cargo test -p portugol-desktop --lib --offline --locked` — passou: 35 testes.
- `cd apps/desktop && npm test` — passou: 134 testes.
- `cd apps/desktop && npm run build` — passou (TypeScript + Vite); Vite emitiu o aviso existente de chunk acima de 500 kB.

A execução requer `rustc` acessível no `PATH`; os instaladores não o incluem.

## Fora do escopo

Projetos Cargo, crates externos, módulos, stdin interativo, depuração e isolamento de processos. A implementação não promete sandbox contra código nativo arbitrário.
