# Prévia e exportação para Rust

O botão **Rust** no editor mostra uma tradução local do programa ativo. O painel acompanha alterações e troca de abas. **Copiar** copia o arquivo gerado e **Salvar .rs** o exporta para continuar fora da IDE. O código não é enviado a nenhum serviço e a IDE não executa o arquivo exportado.

## Subconjunto disponível

A versão inicial gera Rust para `funcao inicio()` sem parâmetros, declarações locais escalares, expressões tipadas, atribuições, `escreva`, `se` e `senao`. Tipos `inteiro`, `real`, `logico`, `cadeia` e `caracter` são aceitos quando a operação preserva a semântica.

`inteiro` continua sendo de 32 bits, como no Portugol; o Rust gerado usa `i32` e operações wrapping. A cadeia vira `String`, literais são escapados e a concatenação gera uma expressão que não move o valor original. `escreva` não adiciona quebra de linha.

O exportador prefixa nomes de variáveis com `pt_` para evitar palavras reservadas e colisões com auxiliares Rust. Variáveis sem inicializador recebem os valores padrão do interpretador. A saída de `logico` e `real` conserva a grafia e a formatação do Portugol.

Quando o código usa recursos fora desse subconjunto — por exemplo `leia`, laços, bibliotecas, vetores, matrizes, funções auxiliares ou conversões implícitas — a prévia explica o primeiro recurso sem suporte e sua posição. Nenhum código parcial é apresentado como tradução válida. Editar e executar o programa original continuam usando o compilador e a VM de Portugol.

Esta prévia não implementa o compilador Rust completo, o dialeto Portugol-RS, ownership ou borrowing. Essas etapas continuam planejadas separadamente.
