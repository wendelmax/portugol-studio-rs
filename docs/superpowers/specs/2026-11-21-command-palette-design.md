# Design: paleta de comandos acessível

## Objetivo

Adicionar a paleta de comandos prevista em FT-09 sem duplicar o registro de atalhos nem expor ações que não podem ser executadas no estado atual.

## Experiência

- `Ctrl+Shift+P` abre um diálogo modal; o campo de busca recebe foco imediatamente.
- Busca ignora caixa, acentos e ordem de palavras, procurando em nome e atalho.
- Setas percorrem resultados, Enter executa o comando selecionado e Escape fecha.
- Resultados usam ações já registradas em `shortcuts`; itens indisponíveis no contexto atual não aparecem.
- O diálogo informa estado vazio e restaura foco ao fechar.
- A lista de atalhos passa a documentar o novo atalho.

## Limites

A paleta não cria comandos novos de execução ou de sistema, não executa ações indisponíveis e não substitui menus/atalhos existentes. A busca fica local.

## Validação

Testes unitários cobrem busca sem diacríticos e seleção cíclica; testes de contrato verificam diálogo, campo, estado vazio e atalho. Suite frontend, build e CI validam integração.
