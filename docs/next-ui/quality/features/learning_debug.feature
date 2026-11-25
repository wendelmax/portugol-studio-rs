# language: pt
@FT-05 @FT-07 @learning
Funcionalidade: Ensino contextual e depuração honesta
  Como estudante
  Quero compreender o significado do código e visualizar conceitos de memória
  Para progredir da lógica para Rust

  Cenário: explicar mutabilidade ao selecionar um trecho
    Dado que o editor tem um trecho mapeado ao conceito "mutable-binding"
    Quando seleciono o trecho
    Então o painel de ensino mostra o conceito de mutabilidade
    E a explicação diferencia o comportamento em Portugol e em Rust

  Cenário: abrir explicação não conclui exercício
    Dado que um exercício está em andamento
    Quando abro a explicação do conceito
    Então o progresso do exercício não é marcado como concluído

  Cenário: depuração nativa indisponível
    Dado que o adaptador de depuração não oferece stepping
    Quando abro a perspectiva de depuração
    Então os controles de stepping ficam indisponíveis
    E a interface explica a limitação

  Cenário: simulação de ownership é identificada
    Dado que abri uma lição simulada de ownership
    Quando avanço um passo
    Então o visualizador mostra "Simulação didática"
    E não apresenta o estado como observação real de memória
