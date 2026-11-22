# Projeto: cancelamento de execução Portugol-RS

## Objetivo

Habilitar Parar e Shift+F7 em `.pors` durante compilação e execução, exibindo o resultado como cancelado apenas após confirmação do supervisor.

## Fluxo

1. Depois das verificações existentes de toolchain/documento, a interface registra um ID com `register_rust_source_run`.
2. Chama `run_portugol_rs(id, source)`; o comando recupera o mesmo `AtomicBool` registrado.
3. `stop(id)` marca o token. O supervisor mata e recolhe o grupo/process job.
4. O serviço retorna `Cancelled { phase, stdout, stderr }`; Tauri remove o registro após o retorno.
5. UI preserva guardas de aba/documento, mostra saída parcial e limpa ID/busy state.

## Limites e compatibilidade

- Compilação: 20 s e limite de saída já definido pelo supervisor.
- Runtime: limite atual de 3 s, saída combinada de 1 MiB e orçamento de strings de 16 MiB.
- `run_portugol_rs(source)` permanece disponível e delega ao novo serviço com cancelamento desabilitado.
- O parser, sintaxe `.pors`, VM clássica, segurança de arquivos `.rs` e consentimento Rust não mudam.

## Validação esperada

Teste de serviço confirma cancelamento tipado em compilação e em processo real gerado; testes Tauri cobrem ID/token compartilhados; contrato frontend verifica registro antes de execução, Stop aceito e estado final confirmado. Rodar suítes service/Tauri/frontend, build, Clippy e CI Linux/macOS/Windows.
