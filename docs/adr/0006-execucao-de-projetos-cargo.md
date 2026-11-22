# ADR 0006 — Executar projetos Cargo pelo workspace

- **Status:** aceito
- **Data:** 2026-11-22

## Contexto

O suporte Rust standalone aceita apenas um buffer `.rs`; módulos e dependências Cargo não são resolvidos. A IDE já registra workspaces e restringe a abertura de fontes ao diretório escolhido.

## Decisão

Ao abrir um `.rs` pelo explorador, procurar o `Cargo.toml` ancestral mais próximo, sem seguir manifestos ou componentes de caminho simbólicos. Os entrypoints do serviço Rust revalidam o manifesto com `symlink_metadata` antes de cada processo Cargo; não basta o explorador ter validado a raiz. Quando encontrado, Executar usa Cargo por comandos fixos, em vez do runner standalone: `cargo build --message-format=json --manifest-path …`, seguido de `cargo run --quiet --manifest-path …`. A raiz e o caminho relativo da fonte vêm do workspace registrado no backend, nunca de um caminho absoluto arbitrário enviado pelo frontend.

Build e runtime são supervisionados separadamente: 20 s para compilar, 3 s para executar, 1 MiB de saída e o cancelamento já integrado ao botão Parar. O compilador e o programa recebem stdin fechado. Erros de `rustc` são extraídos dos eventos JSON de Cargo em texto renderizado. `cargo run` preserva `default-run`, o ambiente do pacote e os diagnósticos padrão de alvos ausentes ou ambíguos.

Antes da primeira execução, a confirmação informa que Cargo pode acessar a rede e executar build scripts, macros procedurais e dependências com as permissões da conta local. Não há sandbox. Arquivos Rust modificados no mesmo workspace precisam ser salvos antes do build. O fluxo standalone segue disponível para `.rs` fora de projetos Cargo.

## Alternativas consideradas

- **Expor um terminal ou argumentos Cargo ao frontend:** recusada; amplia a superfície de comando e permite shell arbitrário.
- **Usar `cargo run` como um único processo:** recusada; não permite aplicar limites distintos de compilação e runtime.
- **Tratar cada `.rs` de um Cargo workspace como buffer standalone:** recusada; gera diagnósticos falsos para imports e módulos do projeto.

## Consequências e limites

- Dependências podem ser baixadas pelo Cargo; executar em rede é comportamento padrão do toolchain.
- Projetos com múltiplos binários sem `default-run` ainda não permitem selecionar alvo pela interface; Cargo informa a ambiguidade.
- O linter standalone não é usado em fontes identificadas como Cargo, evitando falsos erros de dependência; `cargo check` e diagnóstico editorial associado a linha permanecem trabalho futuro.
- Cargo e código de build/dependência não são isolados do usuário. O aviso de consentimento é uma divulgação, não uma sandbox.
- Nenhuma capability Tauri de shell ampla é adicionada.
