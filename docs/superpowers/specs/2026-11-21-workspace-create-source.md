# Criar programa no Explorador — desenho

**Status:** aprovado pelo escopo contínuo de implementação do Portugol Studio Next
**Rastreabilidade:** workspace Explorer; arquivos `.por` e `.pors`

## Objetivo

Permitir criar um arquivo-fonte vazio dentro da raiz do workspace aberto e abrir o arquivo imediatamente no editor. Portugol clássico usa `.por`; Portugol-RS usa `.pors`.

## Experiência

- O cabeçalho do Explorador oferece “Novo arquivo”.
- Um diálogo acessível pede o nome base e o formato: Portugol (`.por`) ou Portugol-RS (`.pors`).
- A extensão escolhida é acrescentada quando o nome não tem extensão; uma extensão conflitante ou caminho com separadores é rejeitado com mensagem clara.
- O novo arquivo aparece no Explorador e abre em uma aba editável. O buffer vazio começa limpo e o salvamento existente grava no workspace.
- Cancelar ou falhar mantém árvore, abas e buffers existentes.

## Segurança e serviços

- O comando Rust recebe `workspace_id` e um único nome de arquivo relativo, sem diretórios.
- Só cria `.por` ou `.pors` diretamente na raiz canonicalizada do workspace.
- Usa criação exclusiva para não sobrescrever arquivos existentes; traversal, separadores, links simbólicos, colisões e outras extensões são rejeitados.
- Nenhuma permissão genérica de filesystem nem comando de shell é introduzido.

## Critérios de aceite

- Criar `ola` como Portugol produz `ola.por`; como Portugol-RS produz `ola.pors`.
- Extensão coincidente é preservada; conflito, nome vazio, separador e colisão retornam erro sem alterar conteúdo existente.
- O arquivo criado consta na árvore e abre em aba imediatamente.
- O salvamento do buffer criado usa o comando já escopado ao workspace.
- Cancelar o diálogo não chama o comando nem altera seleção.
- Testes de frontend validam normalização de nomes e contrato do diálogo; testes Rust validam isolamento, extensões e não sobrescrita.

## Fora de escopo

Criar arquivos em subpastas, renomear, apagar, criar pastas, persistir workspace entre reinícios ou gerenciar projetos Cargo.
