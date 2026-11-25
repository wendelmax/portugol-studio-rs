# Runtime, empacotamento e segurança de execução

## Descobrir antes de mudar
- Platforms efetivas (Windows/Linux/macOS) e targets Tauri.
- Versão de Tauri, plugin FS, dialog, shell/process ou backend Rust customizado.
- Instalação de Rust toolchain requerida no host ou embarcada, se houver.
- Atualização automática, assinatura e distribuição atual.

## Requisitos
1. Ao executar Portugol, usar interpreter já existente quando possível; não ligar conversão Rust a execução clássica sem decisão explícita.
2. Para Rust real: invocar toolchain validada; delimitar arquivos no workspace/sandbox, inputs controlados e timeout. Não assumir que um subprocesso Rust é sandbox de segurança.
3. Interromper árvore de subprocessos no cancelamento/timeout onde a plataforma permitir; remover arquivos temporários apenas depois de confirmar término.
4. Tamanho máximo de logs; backpressure para stdout/stderr; UI não deve congelar em loop de impressão.
5. Recuperação em crash deve respeitar consentimento, backups locais e política de dados do projeto.
6. Falha em ferramenta ausente: mensagem instrutiva e controle desabilitado/alternativa; nunca simular resultado.
7. Compatibilidade em CI: smoke tests de cada target suportado, quando houver runners disponíveis.
8. Não quebrar formato existente e não migrar dados destrutivamente sem backup/plano de rollback.

## Critérios
- Loop infinito encerra por limite e libera recursos.
- Projeto com espaços/acentos no caminho funciona.
- Fechamento abrupto não corrompe arquivo salvo.
- Build, distribuição e permissões permanecem verificadas nas plataformas atuais.
