# Design — loop `para` no Portugol-RS

## Intenção

Ampliar o subset executável de `.pors` com o loop contado comum nos exemplos de Portugol Studio, sem mudar a linguagem clássica nem duplicar o parser ou o backend. O estudante declara o controle mutável, vê a condição e a atualização explicitamente e obtém Rust gerado com mapeamento para a origem.

## Estado atual

- O frontend `portugol-rs-frontend` tipa declarações, atribuições, `escreva`, `se` e `enquanto`; `para` produz diagnóstico localizado de construção não suportada.
- A HIR comum representa `While`, e `portugol-rust` já gera Rust com source maps e executa o resultado em processo com limites de tempo, saída e memória de strings.
- O AST clássico já reconhece `Stmt::For`, mas o exportador clássico não o baixa para esta HIR; a compatibilidade clássica continua fora do escopo.

## Sintaxe desta fatia

```text
para (seja mutavel i: inteiro = 0; i < 10; i += 1) {
    escreva(i);
}
```

- Inicializador obrigatório: uma declaração `seja mutavel nome: inteiro|real = expr`.
- Condição obrigatória e do tipo `logico`.
- Atualização obrigatória: atribuição composta `+=`, `-=`, `*=`, `/=` ou `%=` ao binding mutável do inicializador, com operandos compatíveis com seu tipo numérico.
- O binding do inicializador existe na condição, atualização e corpo, e sai do escopo após o loop.
- O corpo tem seu próprio escopo. `pare`, `continue`, `faca-enquanto` e cabeçalhos `para` com campos opcionais (como `para (;;)` ) continuam não suportados; o bloco do corpo pode ser vazio.
- O corpo pode não terminar; a execução continua protegida pelo limite existente do runner.

## Abordagens

1. **Desdobrar o loop para bloco + `while` no codegen (escolhida):** preserva a semântica de avaliação da condição antes do corpo e da atualização após o corpo; reaproveita a HIR, o gerador, os source maps e as quotas já existentes.
2. **Gerar `for` idiomático Rust por range:** não representa condições e atualizações arbitrárias de Portugol sem analisar ou reescrever expressões, o que poderia alterar semântica.
3. **Criar uma execução de `para` no frontend/VM:** duplicaria o runtime Rust que já é o backend da ponte.

## Restrições

- Não alterar comandos Tauri/IPC, permissões, execução clássica, editor, dependências ou associação `.pors`.
- O lowering semântico deve produzir HIR tipada e binding IDs; nenhuma troca textual de palavras-chave.
- A geração deve manter source maps dos campos do loop e do statement completo.
- O Rust gerado deve respeitar o overflow Portugol-RS já estabelecido para inteiros.
- Diagnósticos de condição, mutabilidade e tipo apontam para o span responsável.

## Arquivos previstos

- `crates/portugol-hir/src/lib.rs`: variante HIR `For` com initializer, condition, step e body tipados.
- `crates/portugol-rs-frontend/src/parser.rs` e `tests/{parser,semantics}.rs`: gramática, escopo, validação e spans.
- `crates/portugol-service/src/rename.rs`: percorrer declaração, condição, atualização e corpo para preservar referências e renomeação semânticas.
- `crates/portugol-rust/src/codegen.rs` e `src/lib.rs`: lowerer/generator, source maps e teste de execução com `rustc`.
- `docs/PARIDADE.md` e relatório FT-16: subset e validações.

## Fora de escopo

Funções adicionais, vetores, `pare`/`continue`, execução interativa `.pors`, tradução universal, suporte de `para` no exportador clássico ou alterações de semântica no Portugol clássico.

## Critérios de aceite

- O parser reconhece a sintaxe definida e associa condição/atualização ao binding declarado no loop.
- Escopo do controle não vaza e atualizações inválidas/condições não lógicas produzem diagnóstico localizado.
- Um loop finito compila e executa com a saída esperada em `rustc`, e source maps cobrem inicialização, condição, atualização e corpo.
- Loops que não terminam continuam sujeitos ao limite tipado de execução vigente.
- Suítes Rust, Clippy, formatação e CI multiplataforma passam.
