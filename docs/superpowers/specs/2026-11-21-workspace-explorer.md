# Explorador de workspace — desenho

**Status:** aprovado para implementação
**Rastreabilidade:** UI-001, UI-002, UI-008; UC-01, UC-03; FT-01
**Escopo:** primeira fatia vertical, somente leitura

## Objetivo

Permitir escolher uma pasta de projeto, navegar por arquivos suportados e abrir um arquivo na aba já existente, mantendo o fluxo atual de abrir/salvar arquivos individuais.

## Experiência

- Adicionar uma perspectiva “Explorador” na área lateral existente, sem substituir a tela/editor atual.
- A ação “Abrir pasta” usa o diálogo nativo de diretório já disponível no Tauri.
- Exibir nome da pasta e árvore recolhível de arquivos `.por`, `.fer` e `.pex`.
- Seleção por mouse e teclado; Enter abre o arquivo. Arquivo já aberto ativa sua aba, sem duplicá-lo.
- Estados vazios, carregando e erro têm texto acessível em português. Pastas sem arquivos suportados exibem orientação.
- A árvore é somente leitura nesta etapa; não há criar, renomear ou excluir.

## Fronteira de segurança e serviços

- O diálogo nativo seleciona a pasta; o frontend não recebe autorização genérica de filesystem.
- O backend canonicaliza a raiz e enumera recursivamente arquivos com extensões aceitas, retornando caminhos relativos normalizados.
- Ignorar symlinks e entradas que escapem da raiz; não seguir links de diretório nesta versão.
- Ao abrir, o frontend solicita conteúdo por identificador/caminho relativo ao workspace. O Rust valida novamente a raiz, rejeita traversal e lê apenas arquivo regular suportado.
- Nenhum comando de shell ou plugin filesystem amplo será adicionado. Reutilizar convenções de Tauri/serviços existentes.
- Fechar/trocar workspace não descarta abas existentes nem buffers sujos. O workspace selecionado é contexto de navegação; abrir um arquivo continua usando a gestão atual de abas e dirty state.

## Componentes e fluxo

1. `WorkspaceSelector` dispara seleção de diretório via plugin de diálogo.
2. `WorkspaceService` chama comando Tauri para validar/abrir e listar arquivos.
3. `ExplorerTree` renderiza a árvore, com `data-testid="project-tree"` e foco/teclado acessíveis.
4. Ao selecionar arquivo, `WorkspaceService.readWorkspaceFile(relativePath)` retorna conteúdo; o fluxo atual de documento/aba abre ou ativa o arquivo.
5. Erros de permissão, raiz inválida, arquivo removido e path traversal são mostrados sem limpar workspace anterior nem alterar buffers.

## Critérios de aceite

- Escolher uma pasta válida mostra apenas `.por`, `.fer` e `.pex` encontrados dentro da raiz.
- Estrutura de pastas é representada; diretórios vazios podem ser omitidos.
- Abrir um arquivo produz aba correta e conteúdo real; reabrir ativa a mesma aba.
- Abrir arquivos individuais pelo fluxo atual permanece funcional.
- Path traversal e symlink apontando para fora nunca expõem conteúdo.
- Cancelar o diálogo mantém o workspace anterior.
- Erro de listagem/leitura conserva estado anterior e apresenta mensagem legível.
- Árvore funciona com teclado e expõe nomes/estados acessíveis.
- Nenhuma operação de mutação aparece habilitada nesta entrega.

## Testes

- Rust unit tests: normalização/extensões, traversal, symlink externo, raiz inválida e arquivo não regular.
- Testes frontend no runner existente: agrupamento da árvore, estados, cancelamento, ativação de aba existente e abertura de novo arquivo.
- Testes de contrato dos comandos Tauri e build existentes.
- Executar qualidade aplicável e registrar bloqueios de dependências desktop local; CI deve cobrir build Tauri em Linux.

## Riscos e rollback

- **Risco:** diferenças de canonicalização/symlink entre Linux, macOS e Windows. Mitigar com `Path` do Rust e testes direcionados em CI.
- **Risco:** arquivos removidos entre listagem e abertura. A leitura refaz validação e retorna erro sem alterar o documento ativo.
- **Compatibilidade:** fluxo individual `.por`/`.fer`/`.pex`, abas e salvamento ficam preservados.
- **Rollback:** remover perspectiva e comandos de workspace; comandos existentes de abrir/salvar arquivos não dependem deles.

## Fora de escopo

CRUD da árvore, persistência/restauração de workspace, watchers externos, gerenciamento de projetos Cargo, descoberta de exercícios/testes, compilação Rust e alterações ao parser/runtime.
