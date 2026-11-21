# ADR 0003 — Limites para execução do Portugol-RS gerado

- **Status:** aceito
- **Data:** 2026-11-21

## Contexto

Portugol-RS compila um subset validado para Rust e executa o binário gerado. A ausência de estruturas de repetição limitava os exercícios, mas loops também podem não terminar, produzir saída sem parar ou ampliar cadeias repetidamente. `Command::output` não limita o tempo nem a memória usada para capturar stdout e stderr.

## Decisão

Adicionar `enquanto` à HIR compartilhada e à geração PORS, preservando a geração clássica. Executar o binário fixo produzido pelo exportador com prazo de três segundos, captura combinada de stdout/stderr limitada a 1 MiB e orçamento cumulativo de 16 MiB para strings geradas. Verificar o tamanho antes de alocar concatenações. Encerrar e recolher o filho ao atingir um limite e retornar um resultado IPC tipado com a causa.

## Consequências

- O aluno pode usar repetição no dialeto Portugol-RS, com condição lógica e escopo léxico.
- Processos que excedem prazo, saída ou orçamento de strings deixam de bloquear indefinidamente a IDE ou acumular saída sem limite.
- O Portugol clássico mantém o comportamento e o código gerado anteriores.
- O orçamento de strings limita alocações contabilizadas pelo código gerado; não é uma quota de RSS nem um sandbox de processo.
- A compilação com `rustc` não está dentro do prazo de runtime. Não se aceitam arquivos Rust arbitrários nem comandos de shell.

## Alternativas consideradas

- Continuar sem loops não atende aos exercícios de repetição.
- Um interpretador PORS independente duplicaria semântica já fornecida pelo compilador Rust.
- `Command::output` sem supervisão não contém loops infinitos nem saída crescente.

