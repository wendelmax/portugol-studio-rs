# Design: frontend inicial para Portugol-RS

## Intenção

Introduzir um dialeto educacional opcional que expressa mutabilidade explícita e tipos, e que possa gerar Rust pelo mesmo backend HIR já usado pelo Portugol clássico. O dialeto terá extensão proposta `.pors`; arquivos `.por` e seu comportamento permanecem intactos.

## Gramática inicial proposta

```ebnf
programa    = funcao_inicio ;
funcao_inicio = "funcao" "inicio" "(" ")" bloco ;
bloco       = "{" { declaracao | atribuicao | escrita | condicional } "}" ;
declaracao  = "seja" [ "mutavel" ] identificador ":" tipo "=" expressao ";" ;
tipo        = "inteiro" | "real" | "logico" | "cadeia" | "caracter" ;
atribuicao  = identificador ("=" | "+=" | "-=" | "*=" | "/=" | "%=") expressao ";" ;
escrita     = "escreva" "(" [ expressao { "," expressao } ] ")" ";" ;
condicional = "se" "(" expressao ")" bloco [ "senao" bloco ] ;
```

Expressões incluem literais, nomes, parênteses, unários `-`, `+`, `!` e operadores binários aritméticos, comparação e booleanos com precedência convencional. A primeira fatia suporta apenas a função de entrada `inicio`, sem parâmetros ou retorno; funções adicionais, loops, vetores, referências e bibliotecas devem produzir diagnóstico localizado, não resultado parcial silencioso.

`seja` é imutável por padrão; `seja mutavel` permite atribuição. Tipos suportados seguem o conjunto de valores da HIR. Variável precisa ser declarada antes do uso e toda declaração inicial exige valor inicial. Essa restrição evita ensinar valores padrão implícitos e mantém o primeiro subset simples; atribuição posterior a uma variável mutável segue permitida. Sombreamento em blocos segue bindings HIR distintos.

## Abordagens consideradas

1. **Reusar o parser clássico via reescrita textual:** rejeitada porque substituir tokens não preserva regras novas nem diagnósticos e viola o contrato semântico.
2. **Parser separado baixando diretamente para HIR:** escolhida porque deixa `.por` intacto, cria identidade/imutabilidade no frontend e reutiliza validação estrutural e backend comuns.
3. **Parser e runtime próprios do zero:** rejeitada nesta fase porque duplicaria execução e geração sem necessidade; a HIR e o gerador atual já cobrem o subconjunto pretendido.

## Limites

- Nenhuma associação de `.pors` no diálogo, Tauri, CLI ou CodeMirror nesta fatia.
- Nenhuma alegação de compilador Rust completo: apenas o subset listado.
- Nenhuma mudança no IPC, permissões, toolchain ou execução de shell.
- Diagnósticos são erros reais do frontend, com spans UTF-8 e códigos estáveis.

## Aceite

- Parser aceita o exemplo de variável mutável fornecido pelo produto e baixa-o a HIR tipada.
- Parser rejeita atribuição a binding imutável, nomes desconhecidos, tipos incompatíveis e sintaxe ausente com spans corretos.
- Geração através de `export_portugol_rs` compila com `rustc` e preserva output para fixtures suportadas.
- Testes clássicos `.por` seguem verdes sem alterar API/saída.
- A extensão `.pors` fica documentada como proposta; não é anunciada na IDE até a próxima fatia.
