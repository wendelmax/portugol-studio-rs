# Prévia e exportação para Rust

O botão **Rust** no editor mostra uma tradução local do programa ativo. O painel acompanha alterações e troca de abas. **Copiar** copia o arquivo gerado e **Salvar .rs** o exporta para continuar fora da IDE. O código não é enviado a nenhum serviço. Arquivos `.rs` podem ser compilados e executados diretamente com `rustc` instalado; a IDE também executa o código da prévia `.pors`.

## Subconjunto disponível

A versão inicial gera Rust para `funcao inicio()` sem parâmetros, declarações locais escalares, expressões tipadas, atribuições, `escreva`, `se` e `senao`. Tipos `inteiro`, `real`, `logico`, `cadeia` e `caracter` são aceitos quando a operação preserva a semântica.

`inteiro` continua sendo de 32 bits, como no Portugol; o Rust gerado usa `i32` e operações wrapping. A cadeia vira `String`, literais são escapados e a concatenação gera uma expressão que não move o valor original. `escreva` não adiciona quebra de linha.

O exportador prefixa nomes de variáveis com `pt_` para evitar palavras reservadas e colisões com auxiliares Rust. Variáveis sem inicializador recebem os valores padrão do interpretador. A saída de `logico` e `real` conserva a grafia e a formatação do Portugol.

Quando o código usa recursos fora desse subconjunto — por exemplo `leia`, laços, bibliotecas, vetores, matrizes, funções auxiliares ou conversões implícitas — a prévia explica o primeiro recurso sem suporte e sua posição. Nenhum código parcial é apresentado como tradução válida. Editar e executar o programa original continuam usando o compilador e a VM de Portugol.

Esta prévia não implementa o compilador Rust completo, o dialeto Portugol-RS, ownership ou borrowing. Essas etapas continuam planejadas separadamente.

## Executar arquivos Rust (`.rs`)

A IDE compila arquivos `.rs` como Rust 2021 usando `rustc` encontrado no `PATH`. A toolchain Rust precisa estar instalada no sistema; os instaladores do Portugol Studio não incluem nem instalam `rustc`. Sem a toolchain, o botão Executar fica indisponível. A execução de cada arquivo exige ação explícita e uma confirmação por sessão.

A compilação tem limite de 20 segundos e a execução de 3 segundos. A fonte e a saída combinada têm limite de 1 MiB. A entrada padrão é fechada e projetos Cargo, dependências externas, módulos externos, entrada interativa, cancelamento manual e depuração Rust não estão disponíveis neste fluxo. O supervisor aplica o limite ao processo associado; código nativo sem sandbox ainda pode criar processos destacados ou escapar do grupo/job antes de ser associado.

**Aviso:** programas Rust nativos executam com as permissões da conta local e não possuem sandbox. Execute somente código em que você confia.
