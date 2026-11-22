# Repriorização do produto progressivo — 2026-11-22

## Direção

O objetivo passa a ser um ambiente desktop que leva o aluno de Portugol a
Portugol-RS e Rust real. A paridade visual com o Portugol Studio clássico já é
suficiente; a evolução de UI deve atender ao fluxo de aprendizagem e às
features novas, conforme o [ADR 0015](../adr/0015-evolucao-visual-orientada-a-aprendizagem.md).

## Estado-base

- `main` em `0adeecb`, após merge da PR #92; não havia issues ou PRs abertas na
  consulta de 2026-11-22.
- Tag atual `v0.1.0`.
- Portugol clássico, shell Tauri, modos Beginner/Bridge/Rust, `.pors`, arquivos
  `.rs`, source maps, exemplos, exercícios, execução e primeira fatia de
  debugger já existem. Consultar [`PARIDADE.md`](../PARIDADE.md) e
  [`REPOSITORY_DISCOVERY_CURRENT.md`](../research/REPOSITORY_DISCOVERY_CURRENT.md)
  antes de abrir novas tarefas.
- Ownership é uma simulação editorial validada por fixtures; não interpreta o
  buffer do aluno nem produz trace do Rust.
- A matriz visual do Java permanece como evidência histórica; screenshots
  equivalentes não são gate para novas releases.

## Ordem de trabalho

### P0 — Recuperar e consolidar o estado de trabalho

Antes de abrir mais frentes, comparar cada worktree com `main`, registrar PR ou
commit correspondente e recuperar qualquer delta único. Não remover worktrees
com alterações ou arquivos não rastreados. Corrigir as cópias locais de
relatórios antigos somente depois de confirmar quais documentos são fonte
vigente. A pasta `docs/portugol-studio-next-integration` permanece até seus
requisitos terem sido conciliados e implementados; ela não deve ser copiada
para PRs como se fosse documentação rastreada.

**Saída:** inventário de worktrees com decisão preservar/descartar, matriz de
paridade coerente com `main` e kit de integração pronto para remoção somente no
fechamento do objetivo original.

**Auditoria concluída em 2026-11-22:** 21 worktrees anteriores foram
comparadas com a `main` e os PRs. Quinze worktrees limpas de mudanças já
mescladas foram removidas. Seis foram preservadas por conter modificações,
documentação não rastreada ou links compartilhados para dependências. A
worktree `53fd` contém uma implementação antiga que se sobrepõe às PRs #89 e
#90; ela não foi reaplicada nem removida. A matriz de paridade já estava
atualizada pela PR #92. As seis worktrees preservadas ainda precisam ser
revisitadas antes da remoção do kit de integração.

### P1 — Tornar o fluxo Portugol-RS confiável de ponta a ponta

Fechar as regressões do ciclo editar → diagnosticar → executar/depurar → ver
resultado. Priorizar correlação de eventos e troca de documento durante uma
sessão, cobertura diferencial do subset VM/Rust gerado, erros com spans e
limites de runtime/saída/memória. Cada construção suportada deve ter casos de
sucesso e falha; construção não suportada deve ser diagnosticada sem alegar
compatibilidade.

**Saída:** subset `.pors` documentado e reproduzível em runtime normal e
debugger, com regressões que não contaminam a aba ativa com eventos obsoletos.

### P2 — Ligar conceitos a prática verificável

Ampliar o caminho de conceitos e prática para além dos três mapeamentos atuais
de exercícios Portugol. Definir o formato de exercícios progressivos para
`.pors` e/ou Rust antes de reutilizar o formato `.pex`; executar casos reais e
registrar progresso somente após aprovação. Conteúdo editorial deve ser
claramente distinguido de código gerado, compilado ou executado.

**Saída:** conceitos mapeados a exercícios existentes ou a um formato novo com
testes determinísticos, sem CTA para prática inexistente e sem progresso por
abertura de cartão.

### P3 — Evoluir a interface conforme os fluxos de aprendizagem

Redesenhar painéis e navegação quando uma feature precisar de melhor
hierarquia, comparação de linguagens, explicação de origem, prática ou
depuração. Preservar familiaridade dos comandos clássicos, temas, teclado e
acessibilidade. Não criar mudanças visuais sem comportamento correspondente.

**Saída:** incrementos de UI medidos por tarefa concluída e compreensão do
estado, executados no app Tauri; screenshot de comparação com a IDE antiga não
é requisito de aceite.

### P4 — Ownership real e ferramentas Rust avançadas

Separar em projetos próprios: análise de ownership para um subconjunto Rust
declarado, debugger de Rust e lint de projetos Cargo. A análise deve usar spans
e resultados comparados com `rustc`; o debugger clássico/VM não é fonte de
eventos de memória Rust. Não ampliar o laboratório atual por cenários
roteirizados sem atualizar sua proveniência.

**Saída:** cada feature tem limites explícitos, diagnóstico ou trace verificável
e critérios independentes de entrega. Reutilizar estruturas existentes apenas
quando a semântica for compatível; não iniciar refactor arquitetural preventivo.

### P5 — Lançamento e limpeza documental

Depois dos gates P1/P2 e do smoke desktop Linux, revisar instalação,
compatibilidade, README conciso, créditos/licenças e notas de release. Manter a
linha `v0.x` até uma decisão explícita de versão 1.0. Só então remover o kit
`portugol-studio-next-integration`, após confirmar que nenhuma decisão ou
requisito ainda depende dele.

## Critérios transversais

- Manter Portugol clássico e `.por` compatíveis.
- Preservar Rust + Tauri desktop e os contratos existentes; não tratar Vite
  dev server como produto web.
- Não executar comandos de shell arbitrários pelo frontend nem declarar
  sandbox quando a execução não for isolada.
- Associar explicações contextuais somente a source maps válidos da revisão
  atual; rejeitar respostas obsoletas.
- Validar testes locais pertinentes e CI multiplataforma; relatar limite de
  ambiente como `NOT_RUN`, nunca como PASS.
- Trabalhar em worktree isolada e mesclar PR após checks verdes; remover a
  branch integrada.

## Próxima fatia recomendada

Continuar em P1: fechar o caso automatizado de eventos obsoletos do debugger
PORS iniciado após a auditoria. Em seguida, ampliar testes diferenciais do
subset e cobrir spans e limites antes de ampliar conteúdo ou redesenhar
painéis.
