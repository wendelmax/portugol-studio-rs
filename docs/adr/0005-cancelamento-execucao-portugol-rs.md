# ADR 0005: Cancelamento de execução Portugol-RS

## Estado

Aceita.

## Contexto

O botão Parar/Shift+F7 já envia IDs para cancelamento da VM clássica e agora cancela arquivos Rust standalone. Execuções `.pors` ainda bloqueiam sem ID Tauri e compilam com `Command::output`, sem timeout/cancelamento.

## Decisão

Reutilizar o registro de execução Rust no estado `Runs` para `.pors`. O serviço terá `run_portugol_rs_with_cancel`, usará o supervisor de árvore tanto em `rustc` quanto no binário gerado e devolverá resultado tipado com fase de cancelamento. Manter `run_portugol_rs` como API compatível não cancelável, usando token falso. A UI só anuncia cancelamento confirmado pelo resultado do serviço.

## Consequências

- `.pors` ganha Stop/Shift+F7 e limites explícitos de compilação.
- `.rs` e `.pors` compartilham cancelamento por ID/token, sem duplicar mecanismo Tauri.
- A gramática e as capabilities não mudam; `.pors` continua restrito ao subset validado.
- Cancelamento da VM clássica segue seu `RunHandle` existente.
