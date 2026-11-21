# Plano incremental: Portugol-RS com HIR compartilhada

## Fatia 1 — fundação tipada

- [x] Adicionar crate `portugol-hir` e tipos neutros (`Span`, `Type`, `BindingId`, `Mutability`, nós do subset).
- [x] Adicionar testes do modelo e lowering clássico antes de refatorar o exportador.
- [x] Adaptar o gerador de `portugol-rust` para consumir a HIR; manter API e erros, com testes de saída comparados ao gerador legado.
- [x] Rodar testes da crate/exportador, Clippy e gates disponíveis; build Tauri bloqueado pela dependência WebKitGTK 4.1 ausente no ambiente.
- [x] Criar relatório de evidências em `docs/reports/FEAT-LANG-001-hir-compartilhada.md`.

## Fatia 2 — frontend Portugol-RS

- [ ] Definir gramática versionada para `seja`, `mutavel`, tipos, expressões, funções, referências e diagnósticos pt-BR.
- [ ] Implementar lexer/parser separado ou reutilizar infraestrutura comprovadamente apropriada sem mudar `.por`.
- [ ] Baixar AST para HIR comum; recusar sintaxe não implementada com erro localizado.
- [ ] Golden tests de parse, semântica, spans, bindings e erros; compatibilidade clássica regressiva.

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

A fatia inicial criou o modelo e o adaptador AST clássico, e o exportador agora consome a HIR. Ela não habilita o frontend Portugol-RS ou `.pors`. O relatório de evidências registra limites e resultados dos gates.

## Regras de encerramento

Não remover `docs/portugol-studio-next-integration/` até todos os critérios e rastreabilidades dessa integração estarem concluídos, os relatórios/evidências terem sido preservados e os testes disponíveis terem sido executados. Itens sem capacidade ou infraestrutura permanecem como bloqueios documentados, não como suporte implementado.
