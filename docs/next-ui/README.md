# Ferro Studio Next — Especificações de Interface e Produto

Documentação consolidada e sem duplicatas para a evolução proprietária da interface do **Ferro Studio RS** (Rust + Tauri v2 + TypeScript/CodeMirror 6).

## Precedência (Fonte de Verdade)

1. **Código e testes atuais do repositório** — descrevem o comportamento implementado e verificado.
2. **ADRs e contratos do produto** (`docs/adr/`, `docs/UX.md`, `docs/research/ferro_MIGRATION_MATRIX.md`) — prevalecem sobre propostas históricas deste pacote quando houver divergência (ex.: ADR 0016 define `.fer` como linguagem principal).
3. **Contratos funcionais e de arquitetura desta pasta** (`product/`, `architecture/`).
4. **Design as Code e referência visual** (`design/DESIGN_AS_CODE.md`, `assets/ide-concept-reference.png`) — referência estética e de interação, subordinada à funcionalidade real e acessibilidade WCAG AA.

## Estrutura

- [`START_HERE_FOR_AGENT.md`](START_HERE_FOR_AGENT.md) — guardrails e fluxo de integração incremental.
- `assets/ide-concept-reference.png` — referência conceitual da interface proprietária.
- `design/` — [`DESIGN_AS_CODE.md`](design/DESIGN_AS_CODE.md), [`COLOR_THEMES.md`](design/COLOR_THEMES.md) e [`ACCESSIBILITY_AND_UX_CONTRACT.md`](design/ACCESSIBILITY_AND_UX_CONTRACT.md).
- `product/` — [`UI_FUNCTIONAL_CONTRACT.md`](product/UI_FUNCTIONAL_CONTRACT.md), [`LANGUAGE_SEMANTICS_CONTRACT.md`](product/LANGUAGE_SEMANTICS_CONTRACT.md), [`LEARNING_EXPERIENCE_CONTRACT.md`](product/LEARNING_EXPERIENCE_CONTRACT.md) e [`COMPATIBILITY_AND_MIGRATION.md`](product/COMPATIBILITY_AND_MIGRATION.md).
- `architecture/` — [`ENGINEERING_AS_CODE.md`](architecture/ENGINEERING_AS_CODE.md), [`TAURI_INTEGRATION_CONTRACT.md`](architecture/TAURI_INTEGRATION_CONTRACT.md), [`STATE_AND_EVENT_CONTRACT.md`](architecture/STATE_AND_EVENT_CONTRACT.md) e [`RELEASE_AND_RUNTIME_CONTRACT.md`](architecture/RELEASE_AND_RUNTIME_CONTRACT.md).
- `quality/` — [`TEST_STRATEGY_AND_QUALITY_GATES.md`](quality/TEST_STRATEGY_AND_QUALITY_GATES.md), [`TRACEABILITY_MATRIX.yaml`](quality/TRACEABILITY_MATRIX.yaml) e cenários BDD em `quality/features/`.
- `research/` — [`ADRs.md`](research/ADRs.md), [`REPOSITORY_DISCOVERY.md`](research/REPOSITORY_DISCOVERY.md) e template de descoberta.
- `agents/` — governança, plano de lacunas (`IMPLEMENTATION_GAP_REPORT.md`, `IMPLEMENTATION_PLAN.md`) e relatórios de features entregues.
- `schemas/` e `templates/` — schema JSON de rastreabilidade e templates de especificação/relatório.
