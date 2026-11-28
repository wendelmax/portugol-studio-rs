# Design — painel de testes para exercícios

## Intenção

Tornar o painel **Testes** útil para arquivos de exercício `.pex`, reutilizando o corretor real da VM. Para documentos sem exercício associado, o painel continua em estado vazio honesto; abrir o painel nunca conta como execução nem como progresso.

## Abordagens

1. **Reutilizar `check_exercise` e o validador de progresso existentes (escolhida):** não cria runner ou semântica duplicada e mantém o progresso vinculado aos casos reais e à revisão exata.
2. Criar uma configuração de testes genérica para qualquer arquivo: exigiria definir um formato/DSL e armazenamento ainda ausentes.
3. Mostrar apenas um botão que abre o diálogo de correção atual: não entrega resultado no painel que o usuário selecionou.

## Comportamento

- Em aba associada a um exercício, mostrar quantidade real de casos e ação **Executar casos**.
- Mostrar carregamento, resultado (aprovados/total e falhas), ou erro real do serviço.
- Reutilizar a validação de progresso já existente. Só todos os casos aprovados podem concluir o exercício.
- Associar cada resultado ao objeto da aba e ao `EditorState` capturado. Descartar atualização/progresso se a aba for removida ou a revisão mudar durante a checagem.
- Sem exercício associado, manter a mensagem de nenhum teste configurado; não mostrar resultados fictícios. A execução via painel não deve abrir o diálogo modal do corretor.

## Restrições

- Sem novo comando Tauri, runner, dependência, ou mudança na VM.
- O caminho do botão existente de correção do exercício continua disponível.
- Não habilitar testes genéricos de Rust ou Ferro sem contratos reais.

## Validação

Testar estados da apresentação e integração frontend; rodar a suíte Node e build. O CI multiplataforma valida o comando Tauri/backend já existente.
