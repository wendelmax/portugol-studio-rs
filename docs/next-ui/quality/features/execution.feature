# language: pt
@FT-03 @execution
Funcionalidade: Execução observável e cancelável
  Como estudante
  Quero executar o meu algoritmo
  Para ver resultados verdadeiros, erros e progresso

  Cenário: execução bem-sucedida
    Dado que o motor Ferro está disponível
    E o documento ativo válido está na revisão 7
    Quando pressiono F5
    Então a execução captura a revisão 7
    E a saída identifica o motor Ferro
    E o botão de parar fica disponível durante a execução
    Quando chega o evento de término com código 0
    Então a execução é marcada como concluída com sucesso

  Cenário: execução falha
    Dado que estou executando o documento ativo
    Quando chega o evento de término com código 1
    Então a execução é marcada como falha
    E nenhuma mensagem de sucesso é exibida

  Cenário: usuário cancela execução longa
    Dado que uma execução demorada está em andamento
    Quando aciono "Parar"
    Então é enviado o cancelamento para a execução ativa
    E a interface não indica sucesso para a sessão cancelada

  Cenário: eventos atrasados não contaminam nova execução
    Dado que uma primeira execução terminou
    E uma segunda execução está ativa
    Quando chega um evento atrasado da primeira execução
    Então o evento não modifica o estado da segunda execução
