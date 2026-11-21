# FEAT-WORKSPACE-003 — restauração do último workspace

## Entregue

- A pasta selecionada é salva como preferência local em JSON dentro de `app_config_dir`.
- Na próxima inicialização, o Rust relê, canonicaliza, valida e enumera a pasta antes de registrar uma nova sessão.
- A IDE restaura a árvore, sem reabrir arquivos, abas ou buffers.
- Preferência ausente retorna estado vazio; JSON inválido ou pasta removida é descartado.
- Falha de persistência não impede abrir o workspace na sessão atual; erro de leitura da preferência não bloqueia o editor.
- Nenhum conteúdo-fonte é persistido por este recurso.

## Validação local

- `cd apps/desktop && npm test` — 7 suítes passaram.
- `cd apps/desktop && npm run build` — passou; aviso do chunk acima de 500 kB existente.
- `cargo fmt --all -- --check` — passou.
- `git diff --check` — passou.
- Testes Rust do desktop — este host não possui WebKitGTK/JavascriptCoreGTK 4.1; a execução local falha antes de compilar o crate. CI deve validar testes e Clippy em Linux, macOS e Windows.
