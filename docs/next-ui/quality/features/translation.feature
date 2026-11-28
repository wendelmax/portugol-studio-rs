# language: pt
@FT-04 @translation
Funcionalidade: Tradução de Ferro para Rust com proveniência
  Como estudante
  Quero comparar meu algoritmo com Rust
  Para entender equivalências sem perder meu código

  Cenário: código gerado é somente leitura
    Dado que o documento ativo tem uma tradução válida
    Quando abro o editor Rust gerado
    Então o editor identifica a origem como "Rust gerado"
    E o conteúdo gerado não pode ser editado diretamente

  Cenário: descartar resposta antiga
    Dado que pedi tradução da revisão 3
    E depois editei o arquivo para revisão 4
    Quando a tradução da revisão 3 chega depois da revisão 4
    Então o resultado da revisão 3 não substitui a tradução atual
    E o documento permanece identificado como desatualizado até receber resultado válido

  Cenário: seleção destaca correspondência sem relação um para um
    Dado que existe mapeamento semântico de uma declaração para múltiplos trechos Rust
    Quando seleciono a declaração de origem
    Então os trechos Rust mapeados são destacados
    E a explicação usa o conceito correspondente

  Cenário: tradução parcial é apresentada honestamente
    Dado que a engine oferece tradução apenas parcial
    Quando a tradução termina
    Então a interface informa "Tradução parcial"
    E os problemas indicam trechos não traduzidos
    E não é exibido o status de tradução completa

  Cenário: desvincular Rust preserva trabalho
    Dado que o editor Rust gerado contém uma tradução válida
    Quando escolho "Desvincular e editar"
    E confirmo a ação
    Então um novo documento Rust editável é criado
    E traduções seguintes não sobrescrevem o novo documento
