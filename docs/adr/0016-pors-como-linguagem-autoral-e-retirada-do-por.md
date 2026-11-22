# ADR 0016: PORS como linguagem autoral e retirada do Portugol clássico

- **Status:** Aceita
- **Data:** 2026-11-22
- **Responsável:** mantenedor do produto

## Contexto

O produto já oferece Portugol clássico (`.por`) e uma ponte inicial para Rust.
O dialeto `.pors` possui parser e HIR próprios, geração de Rust para execução
normal e compilação HIR → bytecode para depuração. Ainda é um subset: não tem
funções definidas pelo aluno, coleções, bibliotecas ou vários comandos que
existem no Portugol clássico. Uma parte substancial da IDE, dos exemplos, dos
exercícios e do CLI ainda depende da linguagem clássica.

O objetivo de produto foi atualizado: o aluno aprenderá a linguagem autoral
Portugol-RS e poderá avançar para Rust real (`.rs`). O Portugol Studio original
continua responsável por arquivos e materiais `.por`. `portugol-studio-next`
e seus documentos são referência de requisitos e design; isso não altera o
nome do produto.

## Decisão

1. Tornar `.pors` a única linguagem Portugol criada, analisada, executada e
   depurada por este produto. Manter suporte a Rust real e projetos Cargo.
2. Portar para PORS as capacidades úteis do Portugol Studio e acrescentar
   conceitos de Rust quando houver semântica executável e explicável. A matriz
   de port define equivalência, adaptação, ordem e critérios de aceite.
3. Preservar HIR, bytecode e VM compartilhados onde sustentem PORS; retirar
   apenas código exclusivo do front-end/runtime clássico depois que não houver
   consumidor.
4. Manter `.por` e `.pex` funcionando enquanto lacunas PORS bloquearem os
   fluxos equivalentes. Não alterar nem converter silenciosamente arquivos do
   usuário. A retirada de suporte no aplicativo acontece somente depois que a
   matriz atingir seu gate final.
5. No gate final, remover reconhecimento e abertura de `.por`/`.pex`, recursos
   clássicos exclusivos, parser/analisador/compiler clássicos sem consumidores,
   comandos e associação de arquivo legados. Usuários que precisem desses
   formatos usam o Portugol Studio original.
6. Não renomear produto, pacotes, executáveis ou repositório por causa de
   `portugol-studio-next`.
7. Não prometer compatibilidade com toda a linguagem Rust: PORS continua sendo
   um dialeto educacional, com regras documentadas e exportação correta somente
   para o subset validado.

## Alternativas consideradas

### Manter os dois dialetos indefinidamente

Rejeitada como direção de produto. Mantém dois compiladores, dois formatos e
dois currículos e enfraquece o foco de PORS/Rust.

### Apagar `.por` e o runtime clássico imediatamente

Rejeitada. PORS ainda não substitui funções, coleções, bibliotecas, exemplos,
ajuda e exercícios; a remoção quebraria fluxos existentes antes de portar suas
capacidades.

### Portar primeiro, retirar `.por` depois de um gate explícito

Escolhida. Permite construir e verificar cada capacidade em PORS, substituindo
conteúdo e fluxos antes de apagar os componentes clássicos.

## Consequências

- PORS passa a ser prioridade de gramática, HIR, backend, VM, diagnóstico,
  documentação e editor.
- Cada recurso deve definir comportamento comum ou diferença declarada entre
  execução normal (Rust gerado) e depuração (VM), com testes diferenciais onde
  a semântica for equivalente.
- `.pex` não será estendido como formato principal PORS; exercícios PORS/Rust
  terão formato e avaliador próprios.
- Progresso, testes e recursos de aprendizagem devem usar resultados
  verificáveis do código PORS/Rust.
- A matriz clássica em `docs/PARIDADE.md` vira referência histórica para o
  inventário; `docs/plans/pors-only-migration.md` passa a ser a fonte de estado
  e aceite da migração.
- A linha de versão permanece `v0.x` conforme decisão anterior. A retirada de
  `.por` exigirá notas de release claras sobre a incompatibilidade.

## Gate para remover `.por`

A remoção só pode ser aberta quando todos os itens forem verdadeiros:

- Cada capacidade mantida do inventário clássico tem equivalente PORS/Rust
  validado ou justificativa explícita de não aplicação, sem pendência escondida.
- Exemplos, ajuda, exercícios, modelos e onboarding levam a `.pors`/`.rs`; o
  avaliador progressivo testa programas reais e o progresso é registrado só
  após aprovação.
- Editor, arquivos, workspace, CLI, Tauri, associação do sistema operacional,
  exportação e distribuição aceitam `.pors`/`.rs` e rejeitam `.por`/`.pex` com
  mensagem direta.
- O CI exercita a gramática, VM/debugger, Rust gerado, bibliotecas, limites e
  fluxos Tauri PORS/Rust em Linux, Windows e macOS.
- A busca no código não encontra caminho de produção nem referência de usuário
  que anuncie suporte a `.por`/`.pex`; apenas fixtures/migração histórica
  explicitamente isoladas podem permanecer.
- O corte não remove a VM compartilhada nem o debugger PORS.
