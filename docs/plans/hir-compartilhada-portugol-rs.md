# Plano incremental: Portugol-RS com HIR compartilhada

## Fatia 1 — fundação tipada

- [x] Adicionar crate `portugol-hir` e tipos neutros (`Span`, `Type`, `BindingId`, `Mutability`, nós do subset).
- [x] Adicionar testes do modelo e lowering clássico antes de refatorar o exportador.
- [x] Adaptar o gerador de `portugol-rust` para consumir a HIR; manter API e erros, com testes de saída comparados ao gerador legado.
- [x] Rodar testes da crate/exportador, Clippy e gates disponíveis; build Tauri bloqueado pela dependência WebKitGTK 4.1 ausente no ambiente.
- [x] Criar relatório de evidências em `docs/reports/FEAT-LANG-001-hir-compartilhada.md`.

## Fatia 2 — frontend Portugol-RS

- [x] Definir gramática versionada do subset inicial em `docs/superpowers/specs/2026-11-21-portugol-rs-frontend-design.md`; referências, funções adicionais e estruturas de controle avançadas permanecem fora deste subset.
- [x] Implementar lexer/parser separado em `portugol-rs-frontend`, sem alterar `.por`.
- [x] Produzir HIR comum diretamente durante parse/análise; construções não implementadas têm diagnósticos localizados.
- [x] Cobrir parse, tipos, mutabilidade, spans UTF-8, bindings/sombreamento, erros e corpus clássico.

## Fatia 3 — arquivos `.pors`

- [ ] Registrar `.pors` no abrir/salvar/associação do Tauri e CLI somente após análise real estar integrada.
- [ ] Preservar conteúdo e buffers não salvos ao alternar modo.
- [ ] Distinguir linguagem detectada da preferência visual; nunca inferir sucesso pelo sufixo.
- [ ] Testar abrir, editar, salvar, reabrir, analisar e mensagens de erro em fluxo real.

## Fatia 4 — tradução e execução

- [ ] Implementar geração Rust semântica a partir da HIR, com mapas de origem e subset explícito.
- [ ] Tratar resultados obsoletos por revisão de documento quando a aplicação suportar revisão no contrato.
- [ ] Adicionar execução Rust apenas por serviço limitado, com toolchain/configuração validada e sem shell arbitrário no frontend.

## Fatia 5 — aprendizagem de ownership

- [ ] Definir modelo didático para move/borrow/escopos e diagnósticos alinhados às regras Rust.
- [ ] Rotular simulação como simulação; separar de observações reais do compilador/runtime.
- [ ] Testar casos válidos/inválidos e não representar moved-from como owner ativo.

## Estado desta entrega

A HIR clássica e o backend compartilhado foram integrados no PR #39. Esta worktree implementa o subset inicial de Portugol-RS e API de geração Rust. A extensão `.pors` ainda não está integrada à UI/CLI/abertura e salvamento; ownership e execução do dialeto também permanecem pendentes. Consulte `docs/reports/FEAT-LANG-002-portugol-rs-frontend.md`.

## Regras de encerramento

Não remover `docs/portugol-studio-next-integration/` até todos os critérios e rastreabilidades dessa integração estarem concluídos, os relatórios/evidências terem sido preservados e os testes disponíveis terem sido executados. Itens sem capacidade ou infraestrutura permanecem como bloqueios documentados, não como suporte implementado.
