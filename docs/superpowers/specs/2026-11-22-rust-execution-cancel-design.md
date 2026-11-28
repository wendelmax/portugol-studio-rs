# Interrupção de execução Rust standalone

## Intenção

Permitir que o aluno interrompa um programa `.rs` em execução pelo botão Interromper e pelo atalho Shift+F7, usando o mesmo ID de execução do restante da IDE. A interface só apresentará o resultado cancelado quando o serviço Rust confirmar a interrupção.

## Estado atual

`run_rust_source` é aguardado diretamente por `spawn_blocking` e não possui ID ou token de cancelamento. `Runs` e o comando `stop` tratam somente o interpretador clássico. Por isso `btn-stop` continua desativado durante a execução de `.rs`, apesar de o supervisor já conseguir encerrar a árvore de processos.

## Abordagens consideradas

1. **Escolhida — registro de execução + cancel token atômico:** Tauri aloca um ID, registra um `Arc<AtomicBool>` e passa o token ao serviço. A UI pode chamar Stop enquanto aguarda `run_rust_source(id, source)`. O supervisor verifica o token durante compilação e execução, mata/recolhe a árvore e devolve `cancelled` com fase e saída capturada. Reutiliza o padrão de ID/Stop e processo supervisionado atual.
2. **Substituir todo o runtime por eventos streaming comuns:** unificaria Ferro, `.fer` e `.rs`, mas mudaria o contrato de eventos e várias telas para resolver apenas o cancelamento `.rs`; risco e regressão maiores.
3. **Manter somente os limites automáticos de 20s/3s:** menor mudança, mas não cumpre o controle Interromper e Shift+F7 já esperado na IDE.

## Decisões

- O backend gera IDs com o contador atual compartilhado de `Runs`; IDs não são escolhidos pelo frontend.
- `register_rust_source_run` registra o token antes da execução e retorna o ID. `run_rust_source(id, source)` executa somente com registro existente e remove o token em todos os retornos. `stop(id)` reconhece tanto `RunHandle` clássico quanto token Rust e retorna se aceitou o pedido.
- O serviço preserva `run_rust_source(source)` para consumidores sem cancelamento e adiciona `run_rust_source_with_cancel(source, token)` para Tauri.
- O supervisor observa cancelamento durante compilação e runtime; mata e recolhe o processo supervisionado, limita os pipes, e retorna `RustSourceRunResult::Cancelled { phase, stdout, stderr }`.
- A UI define `runId` quando recebe o registro, deixa Stop habilitado, mantém a proteção contra aba/documento obsoleto e só mostra cancelamento a partir do resultado final.
- Cancelar a confirmação de segurança não registra ID nem inicia compilador. O limite existente continua sendo fallback.

## Fora de escopo

Execução `.fer`, runtime clássico, depuração, entrada interativa, Cargo e garantias de sandbox. A execução nativa segue usando permissões da conta local.

## Risco e rollback

Risco principal: corrida entre registro/Stop e início do processo; o token permanece ativo antes de `rustc`, então Stop anterior à compilação também é observado. Se o processo terminar ao mesmo tempo, o resultado real do supervisor prevalece e a UI não inventa cancelamento. Rollback: retirar token e comandos de registro/stop, restaurar a execução direta atual; o limite temporal segue contendo programas longos.

## Critérios de aceite

- Um `.rs` infinito pode ser interrompido antes do limite de 3 segundos e retorna `cancelled`.
- Cancelamento durante compilação também encerra/recolhe o compilador.
- Stop retorna aceitação somente para ID Rust registrado ou run clássico ativo.
- A UI habilita Stop durante `.rs`, mostra “interrompendo” apenas após aceitação e apresenta cancelado só após resultado do backend.
- Aba/documento alterado continua descartando resultado antigo; não cancela silenciosamente outra sessão.
- Testes de serviço cobrem runtime e fase de compilação; testes frontend verificam contrato/API/fluxo e capacidade do botão.
