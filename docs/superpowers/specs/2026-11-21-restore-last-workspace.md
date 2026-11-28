# Restaurar último workspace — desenho

**Status:** aprovado pelo escopo contínuo de implementação do Ferro Studio Next
**Rastreabilidade:** `STATE_AND_EVENT_CONTRACT.md` — estado de workspace persistente

## Objetivo

Ao iniciar, reabrir o último workspace escolhido para que o aluno retome o projeto sem repetir a navegação de pastas.

## Experiência

- Ao abrir uma pasta pelo seletor nativo, a aplicação registra o caminho canonicalizado em sua pasta de configuração local.
- Na inicialização, a IDE tenta restaurar esse workspace e repopula a árvore `.por`, `.fer` e `.pex`.
- Um caminho inexistente ou inválido é descartado silenciosamente e a IDE abre normalmente sem pasta selecionada.
- Uma falha de leitura da configuração é apresentada na barra de status, sem impedir o editor de abrir.
- A restauração só carrega o workspace na árvore; não abre arquivos nem altera abas/buffers.

## Segurança e serviços

- Persistir somente o caminho da pasta, nunca conteúdo dos arquivos ou buffers.
- O estado é gravado e lido pelo Rust em `app_config_dir`; o frontend não fornece caminhos arbitrários para restauração.
- Restaurar canonicaliza e valida a pasta novamente antes de listar entradas; cada leitura continua passando pelas verificações existentes de workspace.
- Nenhum plugin filesystem ou escopo genérico é adicionado.

## Critérios de aceite

- Depois de selecionar uma pasta, reiniciar e chamar restauração retorna árvore equivalente e novo ID de sessão.
- Sem preferência salva, retorna estado vazio sem erro.
- Pasta apagada ou movida é descartada e não volta a gerar erro nas próximas inicializações.
- Arquivos `.por`, `.fer` e `.pex` continuam listados; o estado persistido não altera conteúdo ou abas.
- Falha de escrita de preferência não impede abrir a pasta na sessão atual.
- Falha de leitura da preferência não impede iniciar a IDE.

## Fora de escopo

Persistir abas, buffers, arquivos recentes, estado de execução, posição do editor ou projetos Cargo.
