# Diagnóstico atual do repositório

- **Snapshot base:** `6fa03ec` (`main`, 2026-11-22), depois do merge do PR #77; esta branch implementa a próxima lacuna do Cargo.
- **Tauri:** v2 (`tauri = "2"`; Cargo.lock resolve 2.12.5), com `tauri-plugin-dialog`; os comandos publicados estão enumerados em `apps/desktop/src-tauri/src/lib.rs` e as capabilities ficam em `apps/desktop/src-tauri/capabilities/`.
- **Frontend:** HTML, CSS e TypeScript sem framework SPA; Vite 8.3.5, TypeScript 7.0.2 e CodeMirror 6, incluindo linguagens Portugol, Portugol-RS e Rust (`apps/desktop/package.json`, `apps/desktop/src/main.ts`).
- **Núcleo:** Rust edition 2024, crates para sintaxe/AST, HIR, semântica, bytecode, VM, serviços, CLI e geração Rust (`Cargo.toml`, `crates/*`). Os membros da IDE Tauri não fazem parte dos default-members do `cargo test` padrão.
- **Execução clássica:** editor captura código/argumentos e chama a API Tauri; `portugol-service` executa a VM real, emite eventos `run:*` correlacionados por ID e expõe debugger com pausas, stepping, pilha e variáveis. Entradas `leia` chegam pelo console.
- **Portugol-RS (`.pors`):** parser/semântica e HIR próprias; subset suportado é convertido pelo backend Rust comum, com source maps e diagnósticos. Há `enquanto`, `para`, `pare`, `continue` e entrada tipada `leia`, com limites/cancelamento na compilação e execução. Não é a gramática completa do Rust.
- **Rust (`.rs`):** CodeMirror Rust, execução/cancelamento, `rustfmt` e lint `rustc` para arquivos standalone. No workspace, um `Cargo.toml` ancestral seleciona build/runtime Cargo supervisionados; build scripts e dependências exigem consentimento porque não há sandbox. `cargo check` sob demanda associa diagnósticos ao arquivo aberto. Esta branch implementa descoberta offline e seleção explícita de binários em workspaces multi-bin; lint Cargo contínuo e debugger Rust continuam pendentes.
- **Tradução:** `export_rust` e `export_portugol_rs` geram somente subconjuntos semanticamente suportados; source maps distinguem mapeamento exato, aproximado e generated-only. Preview identifica fonte gerada e mantém estado stale por documento. Não é tradução universal.
- **Workspace:** `apps/desktop/src-tauri/src/workspace.rs` e comandos em `lib.rs` abrem e restauram a última pasta, listam `.por`, `.pors`, `.rs` e exercícios, leem/escrevem fontes, criam/renomeiam/excluem arquivos e exportam seleção. Há validação de caminhos, operações relativas ao workspace e cobertura Rust; a UI contém explorer e fluxos de confirmação para arquivo sujo.
- **Aprendizagem:** modos Beginner/Bridge/Rust, catálogo local, explicação contextual por source map exato, progresso local por exercícios verificados e laboratório visual de ownership baseado em fixtures verificadas. O laboratório está rotulado como simulação didática; não é trace de processo Rust. A preferência de modo ainda é global/local, não por workspace.
- **UI e temas:** shell desktop Portugol Studio Next, activity rail, explorer, editor, painel inferior e inspector; temas e layout são persistidos localmente. Há testes de modelo/configuração, mas não regressão visual de páginas Tauri em viewports fixos.
- **Segurança:** frontend não recebe shell arbitrário; processos Rust usam invocações fixas, limites e supervisão. A execução standalone não é sandboxed e a UI pede consentimento. Operações de workspace são confinadas ao diretório registrado e têm validações de caminho/identidade.
- **Testes:** `npm test --prefix apps/desktop` (Node `node:test`, 30 testes top-level), `npm run build --prefix apps/desktop`, `cargo test -p portugol-service --offline --locked` (83 testes), Clippy/format e CI para núcleo e Tauri em Linux, Windows e macOS. Gherkin em docs é especificação: não há runner BDD, DOM ou E2E de interação do painel.

## Lacunas verificadas para priorização

| ID | Situação no snapshot | Evidência | Próxima ação segura |
|---|---|---|---|
| UI-001/002/008/009 → UC-01/03 | Implementado local: abrir/restaurar workspace, explorer, criar/renomear/excluir arquivo e exportar seleção. Não equivale a abrir projetos Cargo. | `apps/desktop/src-tauri/src/workspace.rs`; `apps/desktop/src/main.ts`; `apps/desktop/tests/workspace-explorer.test.mjs`, `workspace-file-actions.test.mjs`, `workspace-export.test.mjs` | Corrigir relatórios antigos; priorizar cobertura E2E de arquivos sujos/fluxo Tauri antes de ampliar árvore. |
| UI-015/016 → UC-07/08 | Parcial: Rust standalone, projetos Cargo e preview `.pors` têm contratos distintos. Cargo compila/executa binário padrão ou alvo binário explicitamente escolhido; `cargo check` é explícito pela paleta e associa diagnósticos ao editor; não há debugger nem lint contínuo Cargo. | `apps/desktop/src/api.ts`; `crates/portugol-service/src/cargo_project.rs`; `docs/adr/0006-execucao-de-projetos-cargo.md`; `docs/adr/0007-analisar-projetos-cargo-sob-demanda.md`; `docs/adr/0008-selecao-de-alvo-cargo.md` | Avaliar lint contínuo respeitando consentimento e debugger Rust; expandir seleção a examples/tests somente com UX clara. |
| UI-023/024 → UC-13 | Parcial: catálogo local e explicação contextual por source map exato; sem currículo versionado/autoria docente. | `apps/desktop/src/learning-content.ts`, `learning-context.ts`, testes correspondentes | Acrescentar conteúdo somente com revisão pedagógica e associação verificável; seleção de cartão não deve gerar progresso. |
| UI-026/027/032 → UC-09/15 | Debugger/variáveis existem para VM Portugol; breakpoints/stepping e ownership trace não existem para Rust. Ownership atual é simulação verificada. | `apps/desktop/src/api.ts` (`DebugInfo`), `apps/desktop/src/ownership-lab.ts`, fixtures `apps/desktop/tests/fixtures/ownership/` | Não reutilizar dados da VM como trace Rust. Manter proveniência de simulação e avaliar debugger Rust como projeto separado. |
| UI-028 → UC-05 | Implementado funcionalmente; persistência do modo é global/local, não vinculada ao workspace. | `apps/desktop/src/learning-mode.ts`, testes `learning-mode.test.mjs` | Considerar preferência por workspace depois de definir o ciclo de vida e migração do store. |
| FT-02/03/05/07/08 e critérios visuais | Cobertura unitária/integrada existe, mas features Gherkin não têm steps executáveis nem E2E visual do app. | `apps/desktop/package.json`; `docs/portugol-studio-next/features/*.feature`; workflow CI | Priorizar um runner de interação nativa/DOM e evidência visual por modo; não reportar Gherkin como teste executado. |

## Validação observada no ciclo mais recente

| Comando/execução | Resultado | Evidência e limite |
|---|---|---|
| `cargo fmt --all --check` | PASS | Executado antes do PR #74 e validado novamente em CI multiplataforma. |
| `cargo test -p portugol-service --offline --locked` | PASS | 86 testes no Linux local, incluindo 9 de projetos Cargo e seleção do binário correto. |
| `cargo clippy -p portugol-service --all-targets --offline --locked -- -D warnings` | PASS | Local para o serviço. |
| `cargo test -p portugol-desktop --offline --locked --lib` | NOT_RUN | A compilação local GTK/Wry excedeu a quota temporária de disco; CI desta branch ainda executará a camada Tauri. |
| `npm test --prefix apps/desktop` | PASS | 31 testes top-level (154 subtestes Node); não é E2E de interação. |
| `npm run build --prefix apps/desktop` | PASS | Build local; Vite ainda informa chunk principal acima de 500 kB. |
| Gherkin/BDD, DOM/E2E e regressão visual | NOT_RUN | Infraestrutura de runner não configurada no snapshot. |

## Limites de compatibilidade

- Portugol clássico e formato `.por` permanecem a linguagem/runtime de compatibilidade.
- `.pors` é dialeto didático proposto; não deve ser descrito como Rust compatível com `rustc` fora do subset emitido.
- `.rs` standalone não implica workspace Cargo, módulos ou dependências.
- Progresso exige resultados reais de avaliação; abrir lição ou painel não prova domínio.
- Debugger da VM não representa execução Rust; o ownership lab continua identificado como simulação.
