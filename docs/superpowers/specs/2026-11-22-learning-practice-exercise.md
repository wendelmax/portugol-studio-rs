# Abrir exercício relacionado ao conceito

## Objetivo

Permitir iniciar uma prática real a partir do inspetor Aprender quando o catálogo possui um exercício Ferro explicitamente relacionado ao conceito.

## Comportamento

- Mapear somente conceitos com correspondência pedagógica clara para exercícios built-in existentes: variáveis → exercício 2; decisões → 12; repetições → 24.
- Exibir “Abrir exercício relacionado” somente quando houver mapeamento. Conceitos Rust sem exercício Rust correspondente continuam com a proposta editorial e não fingem compatibilidade.
- Abrir o exercício existente em uma aba de exercício pela API atual, sem marcar progresso nem executar/corrigir automaticamente.
- Falhas de carregamento são apresentadas em português, sem sucesso falso.

## Não objetivos

Criar exercício, mudar catálogo do backend, adicionar avaliação, associar exercício a ownership/borrowing ou alterar o progresso.

## Aceite

A função de navegação abre o ID mapeado do catálogo real e reporta ID inexistente/erro; testes cobrem mapeamento, abertura e falhas. O cartão mantém a proveniência editorial e a abertura não conta como conclusão.
