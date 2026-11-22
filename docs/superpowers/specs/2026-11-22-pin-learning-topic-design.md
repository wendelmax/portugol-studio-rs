# Fixar tópico no inspetor de aprendizagem — design

## Intenção

No modo Bridge, mover o cursor pode substituir a explicação contextual por outro conceito. A pessoa precisa poder manter um tópico visível enquanto explora outras partes do programa.

## Objetivos

- Adicionar um controle acessível “Fixar tópico” / “Desafixar tópico” ao cartão de aprendizagem.
- Fixar o conceito atual somente na aba e no modo ativos; seleção de código continua atualizando o editor e os source maps, mas não substitui o tópico fixado.
- Permitir selecionar outro conceito enquanto fixado; o novo conceito passa a ser o tópico fixado.
- Ao desafixar, voltar à seleção manual existente ou ao contexto mapeado atual.
- Exibir proveniência “Tópico fixado” para deixar explícito que o conteúdo não foi detectado na seleção atual.

## Fora de escopo

- Persistir pins depois de fechar aba ou reiniciar a IDE.
- Adicionar conteúdo pedagógico, alegar domínio ou alterar progresso.
- Alterar seleção, fonte, tradução, modo, ferramenta ou editor.

## Abordagens

1. Salvar um pin global em `localStorage`. Persiste, mas mistura workspaces/documentos e cria política de ciclo de vida/migração.
2. Salvar um tópico por aba e modo na memória da sessão. **Escolhida:** acompanha o documento em foco, evita vazamento de contexto entre abas e não introduz persistência nova.
3. Bloquear o contexto automático até o usuário fechar o inspetor. Não dá controle explícito sobre qual tópico manter e não equivale a fixar um conceito.

## Comportamento

- Cada aba mantém no máximo um `conceptId` fixado por modo (`beginner`, `bridge`, `rust`).
- O render usa o pin apenas se o conceito existir no catálogo do modo atual; caso contrário, usa o fluxo atual sem alegar pin ativo.
- A seleção manual em um conceito atualiza o pin daquele modo, se já estiver fixado.
- Desafixar limpa apenas o modo atual e restaura a explicação contextual por source map exato quando disponível.
- A seleção do editor permanece funcional e pode navegar para o Rust mapeado normalmente; o pin afeta apenas o conteúdo do inspetor.

## Validação

Testar pin/unpin, isolamento entre modos e troca de conceito fixado, além do contrato de UI e acessibilidade do botão. Rodar suíte frontend, build, format gates e CI desktop. Persistência e interação visual Tauri permanecem fora desta fatia.
