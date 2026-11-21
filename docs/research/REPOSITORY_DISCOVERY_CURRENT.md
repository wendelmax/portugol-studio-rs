# Diagnóstico atual do repositório

- Commit/base: `36ce717` (`main`, origin/main no início da worktree).
- Tauri: v2 (`tauri = "2"` em `apps/desktop/src-tauri/Cargo.toml`; Cargo.lock resolve 2.12.5); plugin direto `tauri-plugin-dialog` 2.8.4.
- Frontend: HTML, CSS e TypeScript sem framework SPA; Vite 8.3.5, TypeScript 7.0.2 e CodeMirror 6 (`apps/desktop/package.json`, `apps/desktop/src/main.ts`).
- Núcleo: Rust edition 2024; crates syntax/AST, semantics, bytecode, VM, service, CLI e exportador Rust (`Cargo.toml`, `crates/*`).
- Execução: a UI chama `run(source, ...)` via `src/api.ts`; `src-tauri/src/lib.rs` inicia a VM real e emite `run:*` com ID numérico. O debugger retorna posição, pilha e variáveis da VM Portugol. Não há trace Rust nem evento de ownership.
- Ponte Rust existente: `export_rust(source)` usa `portugol-service` e cobre um subset; a prévia é identificada como Rust gerado/editorial. Não há toolchain/editor Rust integrado.
- Educação existente: `learning-content.ts` contém cinco conceitos em Bridge, incluindo ownership com exemplo editorial de move. `renderLearningPanel()` mostra definição e exemplos por seleção manual. Não há navegação por passos nem visualização de stack/heap.
- Persistência de aprendizado: progresso local registra apenas correção de exercício real ligada ao hash da revisão; abrir conceito não grava progresso.
- Segurança: capability `default.json` não concede shell/processo; comandos permitidos são enumerados em `lib.rs`. A feature proposta não altera IPC nem capabilities.
- Testes: `npm test` usa Node test runner; testes DOM/E2E/BDD não estão configurados. `npm run build` é `tsc && vite build`. `cargo test --offline --locked` executa o núcleo (membros padrão) e inclui corpus. CI testa Linux/Windows/macOS; o smoke gráfico Linux exercita a janela gráfica da VM, não o painel Aprender.

## Lacunas mapeadas

| ID | Situação observada | Evidência | Próxima ação desta fatia |
|---|---|---|---|
| UI-023/024, UC-13 | Parcial: explicação manual e exemplo Rust editorial | `apps/desktop/src/learning-content.ts`; `apps/desktop/src/main.ts:721-749`; `apps/desktop/index.html:194-221` | Preservar cartão e acrescentar aula guiada apenas para ownership |
| UI-026, UC-09 | Existe para a VM Portugol | `apps/desktop/src/api.ts` (`DebugInfo`, `DebugVar`); `src-tauri/src/lib.rs` (`PausedEvent`) | Não reutilizar como se fosse trace Rust |
| UI-027, UC-15, AC-13 | Ausente | busca por painel de memória não encontra componente; contrato pede proveniência | Criar modelo didático explícito e visualização navegável com proveniência persistente |
| FT-07 | NOT_RUN no app | Não há runner DOM/E2E/BDD no package | Testar modelo e fixtures de compilação; reportar cobertura visual manual como pendente |

## Baseline executado

| Comando | Resultado | Evidência/limite |
|---|---|---|
| `cd apps/desktop && npm test` | PASS | Baseline: 4 arquivos; final: 31 testes, sem falhas, incluindo fixtures de ownership validadas com `rustc` |
| `cd apps/desktop && npm run build` | PASS | TypeScript e Vite 8.3.5; aviso de bundle >500 kB. Usou `node_modules` temporário com lockfile idêntico ao principal |
| `cd apps/desktop && npm ci --offline` | BLOCKED | Cache npm não contém `w3c-keyname@2.2.8`; dependências preinstaladas foram usadas somente para validar o build |
| `cargo fmt --all --check` | PASS | Após a feature |
| `CARGO_TARGET_DIR=/tmp/portugol-target cargo test --offline --locked` | PASS | Após permitir loopback; inclui regressão do corpus e doctests |
| `CARGO_TARGET_DIR=/tmp/portugol-target cargo clippy --all-targets --offline -- -D warnings` | PASS | Núcleo completo, sem warnings |
| `cargo test -p portugol-desktop` | NOT_RUN | Máquina sem dependências de desenvolvimento WebKitGTK 4.1; CI possui jobs Tauri Linux/Windows/macOS |
| UI E2E, leitor de tela e capturas da feature | NOT_RUN | Não há runner DOM/Playwright/Chromium configurado; smoke gráfico CI é da janela da VM, não do painel Aprender |

## Decisões e compatibilidade

- A VM Portugol continua sendo a única origem de execução/debugger.
- O laboratório é conteúdo didático versionado em código, não interpreta buffer Rust e não afirma refletir estado real do processo.
- Nenhum comando Tauri, capability, permissão, dependência npm ou semântica de Portugol será alterado.
- Os quatro cenários de Rust serão verificados com `rustc` nas fixtures de teste. Se o runner local não tiver toolchain, o build do frontend não depende dessas verificações; CI mantém Rust instalado.
