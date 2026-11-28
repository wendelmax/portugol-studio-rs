# Modo Rust — plano de implementação

> **For agentic workers:** use execução nativa tarefa por tarefa e revisão ao final. Cada tarefa tem teste e verificação próprios.

**Goal:** tornar o nível Rust selecionável quando `rustc` estiver disponível e conectá-lo ao conteúdo/editor/execução Rust já existentes.

**Architecture:** estender o estado pedagógico e o catálogo frontend; conectar disponibilidade à consulta Tauri existente. Não há novos comandos nem serviços.

**Tech Stack:** TypeScript, CodeMirror 6, Tauri 2, Node `node:test`.

**Spec:** `docs/ferro-studio-next-integration/docs/agents/FEATURE_UI028_RUST_MODE_DESIGN.md`.

## Restrições globais

- Trocar modo não altera conteúdo, extensão ou motor implicitamente.
- Execução Rust usa somente o executor existente, consentimento existente e limites existentes.
- Sem novo framework, dependência, comando Tauri, permissão ou runtime.
- Se toolchain faltar, declarar a capacidade ausente sem simular resultados.

## Foco de revisão

- Valor persistido `rust` com toolchain ausente: selector deve recuar para modo utilizável, sem alegar execução.
- Alternar modo com arquivo Ferro ativo: buffer e linguagem não mudam.
- Conceito ownership: laboratório continua rotulado como simulação didática.
- Arquivo Rust ativo: caminho de consentimento e comandos de execução continuam intactos.
- Storage inacessível/corrompido: app mantém fallback Beginner sem quebrar boot.

## Arquivos

- Modificar `apps/desktop/src/learning-mode.ts` e `tests/learning-mode.test.mjs` para reconhecer/persistir o terceiro valor.
- Modificar `apps/desktop/src/learning-content.ts` e `tests/learning-content.test.mjs` para definir conceitos por nível.
- Modificar `apps/desktop/index.html`, `src/main.ts` e `tests/learning-mode.test.mjs` para estado de disponibilidade, descrição, visibilidade do conteúdo e preservação do documento.
- Atualizar `docs/PARIDADE.md`, `docs/ferro-studio-next-integration/docs/agents/IMPLEMENTATION_GAP_REPORT.md`, `docs/ferro-studio-next-integration/docs/quality/TRACEABILITY_MATRIX.yaml` e relatório de feature.

## Tarefas

### 1. Persistência e catálogo

- [x] Testar que `loadLearningMode` e `saveLearningMode` aceitam `rust`; valores desconhecidos continuam `beginner`.
- [x] Testar que o modo Rust apresenta conteúdos de mutabilidade, ownership e borrowing.
- [x] Executar os testes focados e observar as falhas esperadas antes de implementar.
- [x] Implementar somente a extensão dos tipos e seleção do catálogo.
- [x] Executar testes focados e `npm test`.

### 2. Seletor e painel

- [x] Testar markup e integração: opção Rust começa desabilitada, estado real do toolchain a habilita, modo Rust não reconfigura documento, e texto de indisponibilidade nomeia `rustc`.
- [x] Executar teste focado e observar falha esperada antes da integração.
- [x] Conectar o seletor à capacidade Tauri existente; renderizar descrição/conceitos com proveniência correta e permitir retorno Beginner/Bridge.
- [x] Executar `npm test` e `npm run build`.

### 3. Evidência e revisão

- [x] Atualizar matriz de rastreabilidade, gap report e `docs/PARIDADE.md`; corrigir a alegação anterior de editor Rust ausente.
- [x] Completar relatório segundo `templates/FEATURE_REPORT.template.md`.
- [x] Executar `cargo fmt --all --check`, `cargo test --workspace --offline --locked` com a repetição do teste de rede autorizada, `cargo clippy --workspace --all-targets --offline --locked -- -D warnings`, build/test frontend e `git diff --check`.
- [x] Inspecionar diff por mudança de buffer/engine acidental, permissões novas e regressão de consentimento.
