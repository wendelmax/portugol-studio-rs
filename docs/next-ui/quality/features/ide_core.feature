# language: pt
@FT-01 @FT-02 @FT-08 @core
Funcionalidade: Fluxo básico da IDE e proteção dos documentos
  Como estudante
  Quero abrir, editar e salvar programas
  Para aprender sem perder minhas alterações

  Cenário: abrir um exemplo e navegar no explorador
    Dado que o catálogo contém o exemplo "Meu primeiro algoritmo"
    Quando abro o exemplo "Meu primeiro algoritmo"
    Então o explorador mostra o arquivo de origem
    E a aba do arquivo está ativa
    E o editor contém o conteúdo real do exemplo

  Cenário: mudar para modo ponte não reescreve código
    Dado que abri um arquivo Portugol com conteúdo conhecido
    Quando seleciono o modo "Ponte para Rust"
    Então o conteúdo original do buffer permanece idêntico
    E a interface mostra os recursos disponíveis de comparação

  Cenário: salvar alterações
    Dado que editei um documento salvo
    Quando aciono "Salvar"
    Então a revisão é persistida
    E o indicador de alterações não salvas desaparece somente após confirmação

  Esquema do Cenário: proteger arquivo com alterações ao fechar
    Dado que tenho alterações não salvas
    Quando tento fechar a aba
    E escolho "<decisao>" no diálogo
    Então o resultado é "<resultado>"
    Exemplos:
      | decisao   | resultado                                           |
      | Cancelar  | aba e alterações permanecem abertas                |
      | Descartar | a aba fecha e o conteúdo salvo permanece           |
      | Salvar    | a aba fecha somente após salvar com sucesso        |
