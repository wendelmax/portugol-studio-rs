# Plano: prévia e exportação de Rust

**Spec:** `docs/superpowers/specs/2026-11-21-exportacao-rust.md`
**ADR:** `docs/adr/0001-webstudio-benchmark-e-ponte-progressiva-rust.md`

## Tarefa 1 — lowering tipado e geração Rust no serviço

- Criar um crate dedicado dependente de syntax e semantics, com HIR mínima contendo tipos/span e falhas estáveis de suporte.
- Validar semântica Portugol antes do lowering.
- Implementar geração do subconjunto da spec sem conversão textual de tokens.
- Testes unitários RED→GREEN para literais, escopo, atribuição, expressões, escrita, condicionais, falhas, spans e escaping.
- Teste de integração compila e executa programas gerados com `rustc`, comparando saída.

**Esperado:** `portugol-service` expõe `export_rust(source)` com código ou diagnósticos; fonte clássica continua sem mudanças.

## Tarefa 2 — comando desktop e prévia sincronizada

- Adicionar comando Tauri `export_rust` e tipo de retorno serializável.
- Adicionar painel acessível ao lado do editor, ligado à aba atual.
- Recalcular após alterações com debounce e identificar estado sem tradução.
- Adicionar copiar e salvar `.rs`; o painel não executa código.
- Cobrir API/estado e construir o frontend.

**Esperado:** fluxo visível de Portugol para Rust com erro explícito para código não suportado.

## Tarefa 3 — documentação e validação de integração

- Atualizar arquitetura/paridade e ajuda para explicar limites do primeiro subset.
- Rodar formatter, Clippy, testes workspace, testes Node e build de produção do frontend.
- Revisar o diff completo na worktree isolada e preparar PR.

**Esperado:** PR pequeno, reproduzível e sem dependência de rede durante exportação.

## Restrições

- Não alterar resultados, parser, bytecode ou VM do Portugol clássico.
- Não dizer que o subconjunto implementa Rust completo ou faz análise de ownership.
- Não misturar conteúdo GPL do benchmark.

## Estado de execução

- [x] HIR tipada mínima, lowering estrito e exportação Rust no serviço.
- [x] Prévia lateral Tauri com sucesso/erro, cópia e salvamento `.rs`.
- [ ] CI completo, revisão final e PR.
