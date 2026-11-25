# language: pt
@FT-06 @FT-09 @FT-10
Funcionalidade: Testes, layout e exportação
  Como estudante
  Quero uma IDE confiável, navegável e com resultados verificáveis
  Para trabalhar de maneira independente

  Cenário: painel de testes sem casos disponíveis
    Dado que nenhum teste está configurado
    Quando abro a aba "Testes"
    Então vejo "Nenhum teste configurado"
    E nenhuma contagem fictícia de testes aprovados é exibida

  Cenário: ausência de toolchain Rust
    Dado que a toolchain Rust não está instalada
    Quando escolho o motor Rust
    Então a execução Rust fica indisponível
    E a interface mostra como configurar a ferramenta

  Cenário: redimensionar e restaurar painel
    Dado que abri uma área de trabalho
    Quando altero o tamanho do explorador
    E fecho e reabro a área de trabalho
    Então o tamanho do explorador é restaurado dentro dos limites permitidos

  Cenário: paleta de comandos via teclado
    Dado que a IDE tem foco
    Quando aciono o atalho da paleta de comandos
    Então a paleta é exibida com foco no campo de busca
    E posso executar um comando habilitado pelo teclado

  Cenário: exportar projeto localmente
    Dado que o projeto contém arquivos persistidos
    Quando escolho exportar projeto
    Então o pacote inclui os arquivos selecionados
    E a interface não afirma que o projeto foi publicado na nuvem
