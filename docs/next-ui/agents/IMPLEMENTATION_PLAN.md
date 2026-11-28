# Plano incremental — adaptar ao diagnóstico do repositório

## Fase 0: sem escrever código
- Inventário Rust + Tauri, frontend e testes.
- Baseline de build/test, registro de resultados reais.
- Component mapping: existente → design proposto; definir gaps.
- ADRs somente onde decisões novas são necessárias.

## Fase 1: fluxo existente com contrato explícito
- Priorizar corrigir o que falta em abrir → editar → salvar → executar → console.
- `requestId`, `revision`, errors e cancelamento validados de ponta a ponta.
- Testar contra o interpreter efetivo, não simulação.

## Fase 2: design system não disruptivo
- Tokens/estrutura visual em componentes já existentes; preservar atalhos.
- Acessibilidade e regressão visual; layouts responsivos de desktop.
- Feature flag para painéis ainda não suportados.

## Fase 3: ponte Ferro–Rust
- Contrato semântico e fixtures primeiro; tradutor por AST/IR.
- UI lado a lado, source maps, estado stale e detach.
- Testes de equivalência Rust onde há toolchain.

## Fase 4: debug/educação
- Integrar trace/debug existente antes de criar simulação.
- Inspector contextual com origem verificada, progressão e exercícios.
- Ownership visual rotulado por provenance.

## Fase 5: release
- Compatibilidade, performance em máquina escolar, acessibilidade e pacote desktop.
- Gates em CI e rollback para migração.

Cada fase pode ser subdividida em PRs pequenos. **Não executar fases cegamente**; se a funcionalidade já existe, adicionar apenas lacunas verificadas.
