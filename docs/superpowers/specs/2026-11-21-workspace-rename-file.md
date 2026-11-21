# Renomear arquivos no Explorador — desenho

**Status:** aprovado pelo escopo contínuo e pelo contrato UI-009
**Rastreabilidade:** UI-009 `ExplorerFileActions`; AC-14; contrato de workspace

## Objetivo

Permitir renomear arquivos `.por`, `.pors` e `.pex` diretamente na árvore, preservando abas abertas e protegendo edições não salvas.

## Experiência

- Cada arquivo da árvore oferece uma ação acessível “Renomear”.
- Um diálogo permite editar o nome; a extensão precisa continuar igual à original.
- Se o arquivo estiver aberto e modificado, a IDE oferece salvar e renomear ou cancelar. A operação não descarta buffers.
- Após renomear, a árvore, o nome/caminho da aba e a lista de recentes refletem o novo caminho.
- Erros de colisão ou permissão mantêm árvore e abas no estado anterior e aparecem em português.

## Segurança e serviços

- Rust recebe o ID de workspace, o caminho relativo atual e um novo nome-base.
- O backend valida que origem e destino são arquivos diretos do mesmo diretório, dentro da raiz autorizada, sem symlinks ou traversal.
- A operação não sobrescreve destino existente. A extensão do destino deve coincidir com `.por`, `.pors` ou `.pex` da origem.
- Nenhum comando de shell ou permissão ampla de filesystem é introduzido.

## Critérios de aceite

- `.por`, `.pors` e `.pex` podem ser renomeados sem alterar bytes do conteúdo.
- Nome vazio, separador, extensão alterada, arquivo inexistente, origem por symlink e destino existente são rejeitados sem perda de dados.
- Uma origem igual ao destino é tratada como operação sem alteração.
- Aba limpa aberta mantém seu buffer e passa a salvar no novo caminho.
- Aba suja só renomeia após salvar com sucesso; cancelar ou falha de salvamento preserva documento e caminho.
- A ação é acessível por teclado e possui rótulos/estados legíveis por tecnologia assistiva.

## Fora de escopo

Excluir arquivos, mover arquivos entre pastas, renomear pastas, criar subpastas e watchers externos.
