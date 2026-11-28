# Design: HIR compartilhada como fundação de Ferro

## Objetivo

Preparar a arquitetura existente para um frontend Ferro sem alterar o parser/runtime clássico. Uma HIR semanticamente tipada será a fronteira comum entre frontends e consumidores como exportação Rust e análise didática.

## Estado atual verificado

- Workspace Rust edition 2024; desktop Tauri v2, frontend HTML/CSS/TypeScript com Vite e CodeMirror 6.
- `ferro-syntax` fornece AST/spans; `ferro-semantics` analisa o dialeto clássico; bytecode/VM executam `.por`.
- `ferro-rust` valida Ferro e reduz um subconjunto em tipos privados `HProgram/HStmt/HExpr`; `HExpr.code` guarda código Rust.
- Não existe parser Ferro nem fluxo `.fer` no CLI/IDE.
- Eventos de execução atuais não incluem revisão de documento. Nenhum comando de shell é exposto ao frontend.

## Contratos da fatia

1. Adicionar `ferro-hir`, independente de sintaxe/gerador.
2. Usar `Span { start, end }` em offsets UTF-8 semiabertos, sem dependência da crate parser.
3. Representar `Type`, `BindingId`, `Mutability`, `Program`, `Stmt` e `Expr` por enums/structs tipados; guardar span em cada nó relevante.
4. O lowering do pseudocódigo clássico resolve nomes em `BindingId` e cria HIR sem texto Rust.
5. `export` consome a HIR compartilhada; o lowering/gerador anterior fica disponível apenas sob testes como oráculo de equivalência.
6. Construções não suportadas mantêm o erro e a localização originais.
7. `.por` mantém parser, semântica, execução e associação atuais. `.fer` não será registrado antes do frontend end-to-end.

## Fora de escopo desta fatia

- Gramática e parser Ferro, `seja`/`mutavel`, referências e análise de ownership.
- Extensão `.fer` na IDE/CLI e conversão automática.
- Compilar/executar código Rust ou disponibilizar `rustc` à interface.
- HIR completa para toda linguagem clássica, mapeamentos UI ou alteração dos comandos IPC.

## Aceite e verificação

- Testes unitários do modelo HIR para spans, IDs, tipos e mutabilidade.
- Testes de lowering clássico, incluindo resolução de binding e escopos aninhados.
- Testes de não regressão do exportador, diagnósticos fora do subset e compilação/saída observável com `rustc` quando disponível.
- `cargo fmt --check`, `cargo test -p ferro-hir -p ferro-rust` e, antes do PR, gates de workspace/CI disponíveis.

## Riscos

- IDs incorretos sob shadowing podem alterar referências; cobrir escopos e identidade com testes.
- Refactor alterar saída/semântica Java-compatible; manter fixtures/asserções do exportador e corpus clássico.
- HIR pequena demais incentivar adaptações com perda de span; spans são obrigatórios nos nós e bindings.
