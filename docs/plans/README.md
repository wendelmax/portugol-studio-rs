# Índice de planos (`docs/plans`)

Esta pasta guarda planos históricos e de implementação do projeto. Com a
conclusão da **Fase 6 do [ADR 0016](../adr/0016-ferro-como-linguagem-autoral-e-retirada-do-por.md)**,
todos os planos de migração para Ferro (`.fer`) e Rust (`.rs`) listados
abaixo foram concluídos e permanecem arquivados como registro técnico:

| Plano | Escopo | Estado |
| --- | --- | --- |
| [`ferro-only-migration.md`](ferro-only-migration.md) | Migração completa para FERRO/Rust e retirada do legado `.por`/`.pex` (Fases 0–6) | ✅ Concluído |
| [`2026-11-21-ferro-frontend-plan.md`](2026-11-21-ferro-frontend-plan.md) | Frontend Ferro inicial e geração Rust | ✅ Concluído |
| [`2026-11-21-ferro-document-flow.md`](2026-11-21-ferro-document-flow.md) | Fluxo de documentos `.fer` na IDE Tauri | ✅ Concluído |
| [`hir-compartilhada-ferro.md`](hir-compartilhada-ferro.md) | HIR tipada compartilhada (`ferro-hir`) | ✅ Concluído |
| [`go-to-declaration.md`](go-to-declaration.md) | Ir à declaração (`F12`) com resolução por escopo | ✅ Concluído |
| [`ownership-visual-lab.md`](ownership-visual-lab.md) | Laboratório didático de ownership (`String`, `&T`, `&mut T`, escopo) | ✅ Concluído |
| [`auditoria-capturas-paridade.md`](auditoria-capturas-paridade.md) | Auditoria de capturas históricas de paridade visual | ✅ Concluído (histórico) |
| [`repriorizacao-produto-progressivo-2026-11-22.md`](repriorizacao-produto-progressivo-2026-11-22.md) | Repriorização para produto progressivo FERRO → Rust | ✅ Concluído |
| [`evolucao-progressiva-ferro-rust.md`](evolucao-progressiva-ferro-rust.md) | Proposta inicial de evolução progressiva Ferro → Rust | ✅ Concluído (superado pelo ADR 0016) |
