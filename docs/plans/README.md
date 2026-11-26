# Índice de planos (`docs/plans`)

Esta pasta guarda planos históricos e de implementação do projeto. Com a
conclusão da **Fase 6 do [ADR 0016](../adr/0016-pors-como-linguagem-autoral-e-retirada-do-por.md)**,
todos os planos de migração para Portugol-RS (`.pors`) e Rust (`.rs`) listados
abaixo foram concluídos e permanecem arquivados como registro técnico:

| Plano | Escopo | Estado |
| --- | --- | --- |
| [`pors-only-migration.md`](pors-only-migration.md) | Migração completa para PORS/Rust e retirada do legado `.por`/`.pex` (Fases 0–6) | ✅ Concluído |
| [`2026-11-21-portugol-rs-frontend-plan.md`](2026-11-21-portugol-rs-frontend-plan.md) | Frontend Portugol-RS inicial e geração Rust | ✅ Concluído |
| [`2026-11-21-pors-document-flow.md`](2026-11-21-pors-document-flow.md) | Fluxo de documentos `.pors` na IDE Tauri | ✅ Concluído |
| [`hir-compartilhada-portugol-rs.md`](hir-compartilhada-portugol-rs.md) | HIR tipada compartilhada (`portugol-hir`) | ✅ Concluído |
| [`go-to-declaration.md`](go-to-declaration.md) | Ir à declaração (`F12`) com resolução por escopo | ✅ Concluído |
| [`ownership-visual-lab.md`](ownership-visual-lab.md) | Laboratório didático de ownership (`String`, `&T`, `&mut T`, escopo) | ✅ Concluído |
| [`auditoria-capturas-paridade.md`](auditoria-capturas-paridade.md) | Auditoria de capturas históricas de paridade visual | ✅ Concluído (histórico) |
| [`repriorizacao-produto-progressivo-2026-11-22.md`](repriorizacao-produto-progressivo-2026-11-22.md) | Repriorização para produto progressivo PORS → Rust | ✅ Concluído |
| [`evolucao-progressiva-portugol-rust.md`](evolucao-progressiva-portugol-rust.md) | Proposta inicial de evolução progressiva Portugol → Rust | ✅ Concluído (superado pelo ADR 0016) |
