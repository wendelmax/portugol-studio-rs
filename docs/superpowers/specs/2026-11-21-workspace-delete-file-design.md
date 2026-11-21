# Excluir arquivos do workspace — desenho

**Status:** aprovado pelo escopo contínuo do Portugol Studio Next e pelo contrato UI-009/FL-05
**Rastreabilidade:** UI-009 `ExplorerFileActions`; UC-03; AC-14; contrato Tauri de workspace

## Intenção

Permitir excluir arquivos `.por`, `.pors` e `.pex` pela árvore do workspace, com confirmação explícita, escopo validado no Rust e proteção dos buffers abertos.

## Objetivos

- Uma ação acessível de exclusão em cada arquivo suportado, visível também em touch.
- Uma confirmação em português mostra o nome e alerta que a exclusão não pode ser desfeita.
- Arquivo limpo pode ser excluído diretamente após confirmação.
- Para abas com alterações, oferecer “Salvar alterações e manter arquivo”, “Excluir e descartar alterações” e “Cancelar”. Salvar mantém o arquivo e exige uma confirmação separada para excluí-lo; isso evita uma ação “salvar e apagar” que apagaria a própria cópia salva.
- Depois da exclusão confirmada e bem-sucedida, remover o item da árvore, recentes e abas associadas. Se o comando falhar, preservar abas e estado frontend.

## Abordagens consideradas

1. **Comando Rust do workspace + confirmação própria acessível (escolhida):** segue o serviço e os limites de filesystem já existentes; suporta as três opções do contrato de documento alterado e testes de estado.
2. **Diálogo nativo binário:** simples, mas não consegue comunicar a escolha de salvar e manter o arquivo, nem cobrir bem o fluxo de três opções.
3. **Plugin genérico de filesystem:** introduziria uma superfície de permissão mais ampla que os comandos scoped existentes.

## Decisões

- Backend valida workspace, caminho relativo, extensão, arquivo regular e symlinks; só remove arquivos, nunca diretórios.
- O resultado contém árvore atualizada e o caminho absoluto/relativo removido para sincronizar abas e recentes.
- “Salvar alterações” salva as abas e encerra a solicitação de exclusão sem remover o arquivo; o usuário confirma a exclusão em uma ação seguinte, já sem alterações pendentes.
- “Excluir e descartar” só fecha as abas associadas após o backend confirmar sucesso.
- Resposta de exercício em `.pex` permanece buffer em memória; salvar usa o fluxo Save As existente, e descartar só fecha essa resposta se a exclusão do arquivo de exercício for bem-sucedida.

## Não objetivos

- Lixeira/restauração, excluir pastas, exclusão em lote, mover arquivos, watchers e exclusão fora do workspace.

## Critérios de aceite

- `.por`, `.pors` e `.pex` podem ser excluídos com o conteúdo e a árvore atualizados; arquivos irmãos permanecem intactos.
- Traversal, symlink, arquivo ausente, diretório e extensão não suportada são rejeitados.
- Colisão/race ou erro de permissão não descarta abas nem remove estado local.
- Cancelar preserva arquivo e buffers; falha ao salvar mantém arquivo; escolha explícita “excluir e descartar” só remove abas depois do comando bem-sucedido.
- A ação tem rótulo acessível, suporte de teclado e visibilidade em dispositivos sem hover.
