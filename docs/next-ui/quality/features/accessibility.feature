# language: pt
Funcionalidade: Acessibilidade da IDE
  Como aluno que navega por teclado
  Quero acessar todas as funcionalidades básicas sem usar mouse

  Cenário: Executar e acompanhar um programa só pelo teclado
    Dado que o editor está focado
    Quando eu salvar e executar usando os atalhos disponíveis
    Então posso navegar até o console e ler o resultado
    E o foco permanece visível durante todo o fluxo

  Cenário: Explicar recurso indisponível
    Dado que o debugger não é suportado no ambiente atual
    Quando eu focar o controle de depuração
    Então encontro uma explicação acessível do motivo da indisponibilidade
