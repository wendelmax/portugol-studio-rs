# Plano: evidência visual de paridade

## Objetivo

Manter `docs/PARIDADE.md` fiel às capturas verificáveis e fechar a comparação
da tela inicial sem inferir paridade a partir de telas diferentes.

## Constatação

`home-dark.webp` e `home-light.webp` têm 1600 × 533 pixels e mostram o editor
Java à esquerda e a tela inicial do port à direita. A composição não compara
duas telas iniciais. As imagens do editor permanecem comparações válidas.

## Entrega desta etapa

- Remover os links de tela inicial da tabela de comparação validada.
- Marcar os estados Dark e Ferro como pendentes.
- Preservar os arquivos atuais e explicar por que não comprovam a paridade.

## Próxima etapa

Capturar a tela inicial do Ferro Studio Java 2.7.5 e do port em Linux/Xvfb,
na mesma resolução e tema, registrar as condições de captura e então atualizar
a tabela. A comparação só será marcada como validada após inspeção das duas
capturas lado a lado.

## Validação

Esta etapa altera somente documentação. Validar o diff e a formatação Markdown;
os testes de código não são afetados.
