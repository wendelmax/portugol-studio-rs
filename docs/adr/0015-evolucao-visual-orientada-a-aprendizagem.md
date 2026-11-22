# ADR 0015 — Evolução visual orientada à aprendizagem

- **Estado:** Aceito
- **Data:** 2026-11-22

## Contexto

O Portugol Studio clássico já oferece referência suficiente para preservar a
familiaridade dos fluxos principais. O produto agora também conecta Portugol,
Portugol-RS e Rust, com modos de aprendizagem, source maps, exercícios e
execução/debug próprios. Reproduzir cada detalhe visual da IDE antiga limita
essa evolução sem acrescentar valor proporcional.

## Decisão

Consideramos satisfatória a paridade visual atual com o Portugol Studio
clássico. Novas telas e mudanças de layout serão projetadas para a experiência
de aprendizagem progressiva, usando Portugol Studio Next como referência de
produto e preservando os fluxos clássicos importantes. Não perseguiremos
paridade pixel a pixel com a IDE antiga.

As mudanças continuam na aplicação desktop Rust + Tauri existente. Este ADR
não autoriza trocar framework, arquitetura, runtime, formato `.por` ou
comportamento da linguagem clássica.

## Consequências

- A tela clássica permanece como modo de compatibilidade; regressões de fluxo,
  acessibilidade e leitura continuam sendo defeitos.
- Uma tela nova deve ser julgada por clareza do próximo passo de aprendizagem,
  execução real, proveniência dos exemplos, acessibilidade e integração com o
  editor desktop.
- Comparações de screenshots com a IDE Java deixam de ser critério de bloqueio
  para releases. As capturas existentes servem como referência histórica e de
  regressão onde forem equivalentes.
- Exercícios, diagnósticos e progresso continuam baseados em resultados
  verificáveis; conteúdo editorial ou simulado deve permanecer identificado.
- A simulação de ownership atual não será apresentada como trace Rust nem como
  análise do código aberto até existir um analisador real para um subconjunto
  declarado.

## Alternativas consideradas

1. Continuar a réplica visual pixel a pixel: rejeitada porque compete com as
   telas e interações necessárias para os novos níveis de aprendizagem.
2. Abandonar toda familiaridade com o Portugol Studio: rejeitada porque os
   alunos atuais precisam continuar reconhecendo os fluxos clássicos.
3. Preservar os fluxos clássicos e evoluir a experiência por necessidade
   pedagógica: escolhida por equilibrar compatibilidade e diferenciação.
