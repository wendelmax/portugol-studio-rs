# Especificação: prévia e exportação de Rust

## Objetivo

Permitir que uma pessoa compare seu programa Ferro com uma versão Rust real, compilável e exportável. A conversão deve deixar claro quando o programa contém construções ainda não suportadas.

## Experiência

- Um comando **Ver Rust** no editor abre um painel lado a lado com o Ferro atual e o resultado Rust.
- O painel atualiza quando o código muda, sem executar automaticamente o programa.
- Construções fora do subconjunto mostram uma explicação em português e não exibem código parcial como tradução válida.
- A pessoa pode copiar o Rust e salvar um arquivo `.rs`.

## Subconjunto inicial

- Um único programa com `funcao inicio()` sem parâmetros e sem valor de retorno.
- Variáveis escalares locais e constantes dos tipos inteiro, real, lógico, cadeia e caracter.
- Literais, variáveis, parênteses, operações unárias e binárias compatíveis com as operações Rust.
- Atribuições simples e compostas para variável local.
- `escreva` sem quebra de linha, preservando a saída concatenada do Ferro.
- `se` e `senao` sem chamadas ou estruturas não documentadas.
- O tipo `inteiro` mantém 32 bits e wrap-around; o Rust usa `i32` e operações wrapping equivalentes.

`leia`, bibliotecas, vetores, matrizes, funções auxiliares, laços, `limpa`, operações de cadeia com conversões implícitas e referências ficam fora do primeiro subconjunto. Um programa que use qualquer uma delas recebe diagnóstico de recurso não suportado.

Declarações com múltiplos nomes, `vago`, conversões numéricas implícitas e constantes de cadeia também ficam fora do primeiro subconjunto.

## Regras semânticas

- A validação sintática e semântica clássica roda primeiro.
- O lowering mantém tipo e span de origem em cada declaração e expressão suportada.
- Variáveis Ferro podem ser reatribuídas; o gerador usa `let mut` mesmo quando o fluxo observado não reassocia.
- `cadeia` vira `String`; literais são convertidos para `String` e a concatenação usa `format!` para não gerar erros de ownership.
- `escreva` gera `print!`, sem newline; o Rust gerado inclui `use std::io::{self, Write};` e descarte explícito do resultado de `flush`.
- O gerador falha fechado: sem suporte ou tipos incertos, não retorna uma tradução Rust.
- A versão inicial exporta código; não executa `rustc` pela interface.

## Acessibilidade e privacidade

- A prévia permanece local e não transmite código.
- Botões têm rótulos acessíveis, foco visível e atalhos continuam iguais.
- Fechar a prévia restaura o foco ao editor.

## Critérios de aceite

1. Exemplos suportados geram um arquivo Rust completo e compilável por `rustc`.
2. Os resultados compilados preservam valores e saída nos casos cobertos.
3. Cada recurso fora do subconjunto produz diagnóstico estável, em português, com span.
4. O analisador e a VM Ferro não mudam seu comportamento.
5. O painel sincroniza com a aba ativa e não mostra código desatualizado.

