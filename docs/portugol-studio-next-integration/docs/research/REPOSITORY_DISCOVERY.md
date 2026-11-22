# Inventário atual do repositório

- **Snapshot:** `b98d890` (`main`, 2026-11-22), depois do merge do PR #74. Este documento substitui o inventário anterior baseado em `28b686e`/`264d11c`.
- **Tauri:** v2 (`tauri = "2"`; lockfile resolve 2.12.5), com plugin de diálogo. Não há plugin de shell/processo; os comandos permitidos são enumerados em `apps/desktop/src-tauri/src/lib.rs`, sob capabilities explícitas.
- **Rust workspace:** edition 2024; crates para sintaxe, HIR, frontend Portugol-RS, geração Rust, semântica, bytecode, VM, serviço e CLI, mais o app Tauri (`Cargo.toml`). O app desktop é intencionalmente excluído dos `default-members` para evitar requisitos WebKit no `cargo test` padrão. MSRV não declarada.
- **Frontend:** HTML/CSS/TypeScript sem framework SPA, Vite 8.3.5 e TypeScript 7.0.2. CodeMirror 6 mantém os modos Portugol, Portugol-RS e Rust (`apps/desktop/package.json`, `src/main.ts`).
- **Formatos:** `.por`, `.pors`, `.rs` e `.pex`. `.pors` é um dialeto didático com subset próprio; `.rs` standalone não implica projeto Cargo.
- **Execução Portugol:** `api.run` chama a VM real. Eventos `run:*` são associados a ID de execução; há entrada interativa, cancelamento, saída, diagnósticos e debugger (breakpoints, pause/continue/step, pilha/variáveis) para a VM.
- **Portugol-RS:** frontend lexer/parser e validação semântica próprios, HIR compartilhada, geração de Rust e source maps. O subset inclui declarações mutáveis, condições, `enquanto`, `para`, controle de laços e `leia` tipado. `rustc` compila o código gerado dentro do serviço supervisionado; compilação/execução têm limites, saída capada e cancelamento. Ainda não há debugger Portugol-RS.
- **Rust standalone:** `.rs` editável em CodeMirror, lint `rustc`, `rustfmt`, execução/cancelamento e argumentos `argv`. O lint opera em um único buffer como crate `lib` Rust 2021; fonte até 1 MiB, execução do compilador até 5 s e saída até 1 MiB. Execução usa consentimento explícito e não tem sandbox; Cargo, dependências, módulos e debugger estão fora do suporte.
- **Tradução:** `export_rust`/`export_portugol_rs` preservam semântica apenas para construções suportadas; source maps têm qualidade `exact`, `approximate` ou `generated_only`. Preview gerado é distinguido de `.rs` do aluno e mantém estados stale/detached. Não afirmar equivalência universal.
- **Workspace:** `src-tauri/src/workspace.rs` e IPC implementam abrir/restaurar última pasta, listagem recursiva, leitura/escrita, criação, renomeação, exclusão e exportação seletiva. A lista suporta `.por`, `.pors`, `.rs` e exercícios. Há validação de paths, identidade da pasta registrada e manejo de conflitos; o explorer UI apresenta estados de loading/error/empty, ações e guarda de arquivo sujo.
- **Aprendizagem:** modos Beginner/Bridge/Rust; conteúdo local com seleção explícita e explicação contextual baseada em mapeamento exato; progresso local gravado somente após avaliação real do exercício; laboratório de ownership com fixtures e proveniência `simulação didática`. Modo ainda é preferência global/localStorage, não por workspace.
- **Layout e temas:** shell Portugol Studio Next, activity rail, explorer, editor, painéis inferior/lateral, temas presets/customizados e persistência local de preferências/layout. Ainda não há regressão visual automatizada do app por viewport.
- **Segurança:** frontend não pode escolher comando de shell. Execução Rust local é não sandboxed e exige consentimento. Workspace é limitado à pasta explicitamente registrada, com validações contra escape/path mutation.
- **Testes:** `npm test --prefix apps/desktop` usa Node `node:test`; `npm run build` usa `tsc && vite build`. O serviço tem testes de integração com `rustc`; `cargo test -p portugol-service --offline --locked` passou com 77 testes no snapshot. CI também valida Tauri e smoke gráfico em Linux, Windows e macOS. Gherkin em `docs/portugol-studio-next/features/` é especificação sem runner BDD configurado; não há E2E visual/DOM dedicado aos fluxos de edição/learning.

## Gap atualizado

| ID | Situação | Evidência | Próxima ação |
|---|---|---|---|
| UC-04/06 / UI-003/011/018..022/033 | Implementado para Portugol/Portugol-RS e Rust standalone, com os limites descritos acima. | `apps/desktop/src/api.ts`, `src-tauri/src/lib.rs`, `crates/portugol-service/src/` | Preservar contratos; fechar cobertura visual/E2E e corrigir bugs de fluxos observados. |
| UI-001/002/008/009 / UC-01/03 | Workspace simples já implementado; não oferece Cargo project discovery/build. | `apps/desktop/src-tauri/src/workspace.rs`, testes `workspace-*` | Não reimplementar explorer; priorizar E2E de dirty/save/delete e decidir como preferências se associam a workspace. |
| UI-015/016 / UC-07/08 | Preview semântico para subset; `.rs` standalone separado com lint/execução/formatação. | `portugol-service::export_rust`, `check_rust_source`, `run_rust_source`, `format_rust_source` | Cargo projects exigem projeto próprio de design/segurança: manifestação, dependências/build scripts, cwd, cancelamento e execução. |
| UI-023/024/025 / UC-13 | Catálogo contextual e progresso validado locais; sem currículo versionado, autoria docente ou sync. | `learning-content.ts`, `learning-context.ts`, `exercise-progress.ts` | Expandir conteúdo versionado por conceitos; não inferir proficiência pela abertura da tela. |
| UI-026/027/032 / UC-09/15 | Debugger/variáveis só para VM Portugol. Ownership lab é simulação didática verificada; trace Rust ausente. | `api.ts`, `ownership-lab.ts`, fixtures `tests/fixtures/ownership/` | Não apresentar simulação como observação real; debugger Rust é projeto separado. |
| FT-02/03/05/07/08 / critérios visuais | Unit/integration/smoke nativo existem; BDD, interação DOM e regressão visual não estão configurados. | `package.json`, workflows CI e `docs/portugol-studio-next/features/` | Definir runner e cenários antes de declarar paridade visual/funcional completa. |

## Decisões a preservar

- Manter o projeto atual Rust + Tauri v2, CodeMirror 6, serviços e formatos; não reconstruir a arquitetura.
- Portugol clássico permanece compatível e a VM é origem do debugger clássico.
- Portugol-RS é subset/dialeto educacional, não uma gramática oficial do Rust.
- Preview gerado, buffer Rust do aluno e simulação de ownership têm proveniências distintas.
- Não adicionar shell arbitrário, execução sem consentimento ou afirmações de sandbox.
