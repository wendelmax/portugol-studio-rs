# Design: estado vazio do painel Testes

## Intenção

Adicionar a aba inferior **Testes** prevista em UI-018/UI-020 e no FT-06. O painel deve informar com clareza que nenhum teste está configurado e que esta IDE ainda não possui um runner integrado, sem inventar contagens nem resultados.

## Abordagens

- Criar um runner próprio agora: rejeitado para esta fatia, pois ainda não há formato de descoberta de testes nem contrato de execução.
- Adicionar a aba com estado vazio honesto usando o tablist existente: escolhido por ser verificável e não afetar o runtime.
- Mostrar controles desabilitados de execução: rejeitado porque não há ação real para ligar aos controles.

## Limites e compatibilidade

- Manter Tauri, CodeMirror e a arquitetura de painéis atual.
- Não chamar comandos Rust nem iniciar processos.
- Não modificar documentos, exercícios ou progresso.
- Exibir o painel em português, com estado anunciado por `role="status"`.
- A indisponibilidade do runner deve explicar que executar testes ainda não foi integrado; a edição de `.rs` continua sem prometer compilação/execução Rust.

## Aceite

- A nova aba Testes integra o tablist inferior e abre seu painel associado.
- O painel apresenta “Nenhum teste configurado” e explica a ausência de runner.
- Não exibe contagens de aprovação/falha nem afirma que o código foi testado.
- O painel permanece acessível por teclado e identificado semanticamente.
- Testes verificam estrutura/contrato e a suíte frontend e build continuam passando.

## Aprovação e escopo

O usuário autorizou continuar implementando as lacunas do projeto. A escolha do estado vazio como primeiro passo é uma decisão de implementação para manter o escopo seguro e reversível; um runner real exige design próprio.
