# Design — controle de laço em Portugol-RS

## Objetivo

Permitir que programas `.pors` encerrem a iteração atual (`continue;`) ou o laço atual (`pare;`) em `enquanto` e `para`, com diagnósticos em português e geração Rust compilável.

## Comportamento

- `pare;` sai do laço mais interno; `continue;` segue para a próxima iteração do laço mais interno.
- Ambos podem aparecer em blocos aninhados dentro de um laço.
- Fora de um laço, cada construção retorna diagnóstico localizado (`pors.loop-control-outside-loop`).
- O gerador emite `break;` ou `continue;` e preserva mapeamento do comando à sua span de origem.
- Não muda a semântica nem passa a aceitar essas construções no frontend clássico.

## Arquitetura

Representar controles como uma nova variante `StmtKind::LoopControl { control }` na HIR compartilhada, com `LoopControl::{Break, Continue}`. O parser PORS mantém uma profundidade de laços durante o parsing de `enquanto` e `para`; o codegen emite o comando Rust e o walker de renomeação trata o nó como folha. O lowering clássico continua rejeitando `pare` como hoje.

## Restrições

- Sem novas dependências ou comandos Tauri.
- Não executar comandos de shell fornecidos pelo frontend.
- `continue` é a grafia proposta e aceita pelo lexer PORS existente como identificador.
- Os source maps permanecem associados ao statement original.

## Validação

- Parser: ambos aceitos dentro de laços, erro fora; testes cobrem escopo em bloco aninhado.
- Codegen: compilar e executar fixtures com `rustc`; validar stdout e que o laço termina/avança como esperado.
- Rodar suites focadas dos crates, `cargo test --workspace`, clippy, frontend e build quando viável.
