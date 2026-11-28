# Execução cancelável de arquivos Rust (`.rs`)

## Resultado

Arquivos Rust standalone podem ser interrompidos pelo botão **Parar** e pelo atalho **Shift+F7** durante compilação ou execução. A interface mostra “Interrupção solicitada…” somente quando Tauri aceitou o pedido; o estado cancelado só é apresentado quando o serviço devolve o resultado tipado com a fase confirmada. A saída capturada até a interrupção continua disponível no console.

O backend associa um identificador e um token atômico à execução. O supervisor existente mata e recolhe a árvore de processos no cancelamento, mantendo os limites de tempo e saída. O registro é removido após o término, inclusive quando a tarefa `spawn_blocking` falha. O consentimento existente para executar Rust sem sandbox continua sendo solicitado antes do registro.

## Escopo

- Incluído: compilação/executação de arquivos `.rs` standalone, cancelamento via toolbar e Shift+F7, resultado tipado com fase `compilation` ou `runtime`.
- Não incluído: cancelar `.fer`, cancelar execução da VM clássica, depuração de Rust, sandbox ou mudança nas permissões concedidas ao processo.
- O cancelamento é cooperativo entre ciclos curtos do supervisor (até aproximadamente 5 ms, além do tempo de encerramento/recolha do sistema operacional).

## Validação executada

Na worktree `feat/rust-execution-cancel`:

- `cargo fmt --all`
- `cargo test -p ferro-service --offline --locked` — 30 unitários, 9 testes FERRO, 8 testes Rust standalone e 10 de integração passaram.
- `cargo test -p ferro-desktop --lib --offline --locked` — 35 testes passaram.
- `cd apps/desktop && npm test` — 135 testes passaram.
- `cd apps/desktop && npm run build` — build concluído; Vite emitiu o aviso existente de bundle principal acima de 500 kB.
- `git diff --check`

O teste Rust standalone cobre cancelamento de um processo infinito após início comprovado por arquivo marcador e cancelamento já solicitado durante compilação. A cobertura roda no Linux local; CI deve validar Linux, Windows e macOS antes do merge.

## Pendências após esta entrega

Executar manualmente no app desktop nos três sistemas suportados; confirmar botão, Shift+F7, compilação interrompida, saída parcial e encerramento de subprocessos. Cancelamento de `.fer` permanece uma lacuna separada da paridade.
