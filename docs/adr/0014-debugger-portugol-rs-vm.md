# ADR 0014: Depuração Portugol-RS pela VM compartilhada

- **Estado:** aceito; paridade diferencial ampliada e validação visual permanecem pendentes.
- **Data:** 2026-11-22

## Contexto

O depurador existente depende do bytecode e da VM. Portugol-RS já possui frontend HIR e gera Rust para execução normal, mas arquivos `.pors` não têm checkpoints ou snapshots de variáveis. GDB/LLDB varia por plataforma; instrumentar o Rust gerado criaria um protocolo próprio de execução e depuração.

## Decisão

Criar um backend HIR→bytecode e executar as sessões de depuração com `portugol-vm`. A execução comum `.pors` continua usando `rustc`. O serviço compartilha `RunHandle`, `DebugControl`, `RunObserver`, entrada tipada e eventos `run:*` com Portugol clássico. Bindings usam seus IDs HIR como slots locais, e cada opcode mantém o span de origem.

## Consequências

- A depuração de `.pors` pode funcionar sem `rustc` e reaproveita os controles existentes.
- VM e Rust gerado são dois backends; testes diferenciais de saída, falhas e spans são necessários antes de ampliar a gramática coberta.
- A inspeção mostra valores do modelo Portugol-RS na VM, não ownership, stack ou heap do Rust nativo.
- O backend VM aplica 3 s de runtime ativo, 1 MiB de saída e 16 MiB de alocações de cadeias; pausa de debugger e espera por `leia` não consomem runtime.
- A implementação cobre declarações, atribuições, expressões primitivas, `escreva`, `leia`, condicionais e laços. O bytecode marca a visibilidade dos slots para que snapshots respeitem escopos lexicais e sombreamento. Smoke visual nativo e paridade diferencial de falhas continuam pendentes.
