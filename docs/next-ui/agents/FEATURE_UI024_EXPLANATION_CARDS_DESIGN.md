# Design: explicações de conceitos por modo

## Intenção

Dar ao aluno um painel curto de consulta dentro do editor: no modo Beginner, conceitos básicos de Ferro; no Bridge, os mesmos fundamentos junto de explicações explícitas sobre mutabilidade e ownership em Rust. O painel é aberto por escolha do aluno e não afirma inferir conceitos da seleção do código.

## Objetivos e limites

- Implementar UI-023/024 e UC-13 como um catálogo local e pequeno, vinculado ao modo de aprendizagem já persistido.
- Mostrar um conceito escolhido manualmente, com definição, exemplo de Ferro e, quando pertinente, exemplo Rust editorial.
- Identificar a origem como seleção manual e os exemplos Bridge como didáticos, nunca como resultado de tradução ou compilação.
- Não criar progresso, resolver AST, alterar fonte, introduzir serviço/IPC ou runtime Rust.

## Abordagens consideradas

1. Inferir o conceito pela seleção/AST atual. Rejeitada nesta fatia: a AST fornece símbolos e diagnósticos, mas não um mapa didático de conceitos; inferir por regex contrariaria o contrato.
2. Criar um serviço de lições completo, versão de conteúdo, progresso e persistência. Rejeitada como escopo inicial: não há backend de progresso e abertura não pode ser contada como conclusão.
3. Adicionar a aba `Aprender` ao painel inferior existente, com seleção explícita e catálogo editorial versionado no frontend. Escolhida como menor fatia vertical útil: segue navegação visual existente, não muda arquitetura e pode ser automatizada com Node test runner.

## Comportamento e conteúdo

- Beginner oferece variáveis, condicionais e repetições.
- Bridge oferece esses conceitos e mutabilidade/ownership.
- Trocar modo atualiza a lista; se a escolha anterior não existir no novo modo, seleciona o primeiro conceito disponível. Voltar ao modo anterior pode escolher novamente o conceito desejado.
- Ownership diferencia claramente o Ferro atual do modelo Rust. O exemplo Rust explica move; não será apresentado como código gerado pela aba ativa.
- Nenhum conceito atribuído automaticamente; painel declara `Conceito selecionado manualmente`.
- A abertura/troca de cartão não registra progresso.

## Compatibilidade e risco

HTML/CSS/TypeScript existentes; sem biblioteca ou mudança Tauri. Risco baixo. A nova aba não mexe no console, mensagens, debugger, buffer, exportação ou resultado de execução. Rollback: remover aba/painel, catálogo, eventos, testes e linha de rastreabilidade.

## Validação

Testes Node para IDs, requisitos por modo, campos obrigatórios, explicação correta de ownership e rótulo não gerado. `npm test`, `npm run build`, gates Rust e CI. Sem runner DOM/E2E instalado, interação visual fica registrada como pendente.
