# ADR 0015 — Evolução visual orientada à aprendizagem

- **Estado:** Aceito
- **Data:** 2026-11-22

> A decisão visual deste ADR permanece aceita. Sua política antiga de manter
> Portugol clássico como modo de compatibilidade foi supersedida pelo
> [ADR 0016](0016-pors-como-linguagem-autoral-e-retirada-do-por.md).

## Contexto

A paridade visual atual com o Portugol Studio clássico é suficiente. O produto
evolui para ensinar Portugol-RS e Rust real, com modos de aprendizagem, source
maps, exercícios e execução/debug próprios. Reproduzir cada detalhe visual da
IDE antiga limita essa evolução sem acrescentar valor proporcional.

## Decisão

Novas telas e mudanças de layout serão projetadas para os fluxos de PORS e Rust,
usando Portugol Studio Next como referência de requisitos e design. Não
perseguiremos paridade pixel a pixel com a IDE antiga.

As mudanças continuam na aplicação desktop Rust + Tauri existente. A retirada
do dialeto `.por` é definida pelo ADR 0016, não por este ADR visual.

## Consequências

- A paridade visual clássica fica como referência histórica; novos fluxos devem
  manter acessibilidade, clareza e qualidade de leitura.
- Uma tela nova deve ser julgada por clareza do próximo passo de aprendizagem,
  execução real, proveniência dos exemplos, acessibilidade e integração com o
  editor desktop.
- Comparações de screenshots com a IDE Java deixam de ser critério de bloqueio
  para releases. As capturas existentes servem como referência histórica e de
  regressão onde forem equivalentes.
- Exercícios, diagnósticos e progresso continuam baseados em resultados
  verificáveis; conteúdo editorial ou simulado deve permanecer identificado.
- Ownership não será apresentado como trace Rust nem como análise do código
  aberto até existir um analisador real para um subconjunto declarado.

## Alternativas consideradas

1. Continuar a réplica visual pixel a pixel: rejeitada porque compete com telas
   e interações necessárias aos novos níveis de aprendizagem.
2. Abandonar toda familiaridade com o Portugol Studio: rejeitada porque seus
   fluxos e metáforas ainda ajudam alunos atuais a reconhecer uma IDE.
3. Preservar a familiaridade visual útil e evoluir os fluxos para PORS/Rust por
   necessidade pedagógica: escolhida por equilibrar reconhecimento e
   diferenciação.
