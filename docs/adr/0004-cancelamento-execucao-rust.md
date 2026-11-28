# ADR 0004 — Cancelamento de execução Rust standalone

- **Status:** aceito
- **Data:** 2026-11-22

## Contexto

O PR #66 habilitou execução de arquivos `.rs` como processo nativo supervisonado, com limites de compilação/tempo/saída e confirmação de segurança. A execução é aguardada por uma única chamada IPC e não recebe ID, enquanto o botão Interromper/Shift+F7 depende de um ID conhecido pelo backend. Assim o aluno não consegue interromper um `.rs` antes do limite automático.

## Decisão

Atribuir o ID no backend antes de iniciar a execução, associá-lo a um token `AtomicBool`, expor esse token ao comando `stop` e passá-lo ao supervisor do serviço. O supervisor verifica o token durante compilação e runtime; um pedido aceito termina em resultado tipado `cancelled` somente depois de encerrar e recolher o processo supervisionado. A UI apresenta o resultado final retornado e nunca presume que o pedido de Stop encerrou o processo.

A implementação fica limitada a arquivos `.rs`; `.fer` e o runtime Ferro mantêm seus fluxos atuais nesta decisão. O usuário já recebe aviso explícito de que execução Rust nativa não tem sandbox; o cancelamento não altera essa fronteira de segurança.

## Alternativas consideradas

- **Eventos de execução unificados para todos os motores:** recusada nesta fatia por reescrever fluxos estáveis de Ferro e `.fer` sem necessidade para o objetivo imediato.
- **Esperar somente o limite de 3 segundos:** recusada porque deixa o controle Interromper e Shift+F7 sem efeito durante execução Rust.

## Consequências

- O contrato frontend passa a registrar uma execução Rust antes de aguardar seu resultado e pode reutilizar `stop(id)`.
- O ID/tokens precisam ser removidos após sucesso, erro ou cancelamento para evitar runs bloqueadas.
- Um resultado de término natural prevalece se ocorrer ao mesmo tempo que Stop; a UI só anuncia cancelamento com confirmação final do serviço.
- A execução permanece nativa, limitada e sem sandbox.
