# Estados e eventos — versão proposta

## Ownership de estado

| Estado | Dono sugerido | Persistência | Observação |
|---|---|---|---|
| Buffer/revisão/dirty | document store existente | salvar/recuperação conforme produto | um único owner por arquivo |
| Workspace/arquivos | workspace service Rust + store UI | workspace | confirmar alterações não salvas |
| Tradução/mapeamento | translation state | cache por fileId/revision | nunca substituir por resposta obsoleta |
| Execução/stdout | execution store | sessão/log opcional | somente backend publica sucesso |
| Layout/tema/modo | preferences store | por workspace | não mudar fonte ao trocar modo |
| Progresso educacional | learning progress service | por aluno/local conforme produto | validado por exercício |
| Debug/memória | debugger/trace service | sessão | proveniência obrigatória |

## Máquina de estados de execução

`idle → queued → running → succeeded | failed | cancelled | timed_out`

- `stdout` ou `stderr` isolados não são status terminal.
- Só `exited(0)` validado permite sucesso. Não confundir compilação bem-sucedida com teste aprovado.
- `Stop` requisita cancelamento; não afirmar encerramento antes do backend.
- Toda execução leva `requestId`, `fileId`, `revision`, `engine`; apenas eventos da sessão ativa afetam painel em foco.

## Máquina de estados de tradução

`idle → generating → ready | partial | error`; edição após ready/partial marca `stale`. Novo request não apaga artefato anterior; apresenta indicação de desatualização até confirmação. Não sobrescrever Rust desvinculado.

## Invariantes para concorrência

- Ao digitar: incrementar revisão monotonicamente por documento.
- Debounce só otimiza, não resolve race conditions; comparar identidade + revisão ao receber resposta.
- File watcher deve distinguir mudanças externas e internas; merge/confirmar em conflito.
- `save` protegido contra gravar revisão antiga sem confirmação, preferir gravação atômica quando compatível.
- Fechamento de abas deve confirmar dirty state e limpar listeners.

## Telemetria/privacidade

Nunca registrar automaticamente o código dos alunos ou caminhos privados em telemetria remota. Logs locais devem reduzir dados sensíveis; observabilidade opt-in se existir, com política clara.
