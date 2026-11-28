# Design: vetores no Ferro

## Intenção

Portar a primeira coleção composta para FERRO, permitindo que alunos agrupem e
acessem valores tipados e observem o mesmo programa no debugger e no Rust
gerado. Vetores são a próxima lacuna de linguagem indicada pela matriz FERRO.

## Sintaxe proposta

```ferro
funcao primeiro_dobro(valores: vetor<inteiro>) -> inteiro {
    retorne valores[0] * 2;
}

funcao inicio() {
    seja mutavel notas: vetor<inteiro> = [7, 8, 9];
    notas[1] = 10;
    escreval(primeiro_dobro(notas));
    escreval(tamanho(notas));
}
```

O tipo `vetor<T>` contém valores homogêneos de um tipo primitivo atual e tem
tamanho fixo após a criação. Declarações e parâmetros sempre anotam o tipo
(`vetor<inteiro>`, por exemplo); os elementos do literal não vazio `[a, b, ...]`
precisam ser homogêneos e coincidir com essa anotação. `tamanho(vetor)` retorna
`inteiro`. Índices começam em zero e precisam ser `inteiro`. Parâmetros são
imutáveis por padrão; `mutavel valores: vetor<T>` autoriza alterar o vetor local
recebido por valor sem modificar o vetor do chamador. Para manter semântica de
valor coerente entre debugger e Rust gerado, atribuições, retornos e argumentos
de função fazem cópia profunda. Mutações de elemento também exigem binding
`mutavel`.

Índice fora dos limites no debugger resulta em erro de runtime com span da
expressão. Na execução Rust gerada, o processo termina com código diferente de
zero e escreve uma mensagem FERRO de índice fora dos limites; esta é a mesma
classe observável que as demais falhas de runtime do runner atual, que não
converte panic de Rust em diagnóstico de editor.

## Abordagens

1. **Arrays tipados em HIR e `Vec<T>` no Rust, reutilizando os opcodes de array
   da VM:** escolhida. Adiciona checagem estática coerente, habilita debugger e
   deixa claro o mapeamento educacional sem depender de sintaxe do Ferro
   clássico.
2. **Baixar vetores para vários bindings escalares:** rejeitada porque perde
   índices dinâmicos, tamanho variável e expansão real no debugger.
3. **Reusar diretamente AST e semântica clássicos:** rejeitada porque recria a
   dependência Java-like que a migração está retirando e mistura contratos de
   mutabilidade.

## Objetivos e limites desta fatia

- Implementar tipos de vetor dos cinco primitivos, literais homogêneos não
  vazios, declarações, funções, indexação, escrita em índice e `tamanho`.
- Gerar Rust `Vec<T>` e bytecode para VM; preservar spans de fonte, diagnósticos
  localizados, bounds checks e limites de memória existentes.
- Comparar saída/erros da VM e de `rustc`, inclusive parâmetros/retornos e
  cópias de valor.
- Não implementar matrizes, vetores vazios, resize, slices, iteradores,
  referências, coleções aninhadas ou biblioteca de coleções nesta entrega.
- Não mudar nome, shell ou contratos Tauri do produto e não remover `.por`.

## Critérios de aceite

- Os cinco vetores primitivos são checados em literais; vetor
  heterogêneo, tipo incorreto, índice não inteiro e mutação imutável recebem
  diagnósticos FERRO com spans corretos.
- Indexação lê/escreve com índice dinâmico e rejeita limites negativos/acima do
  tamanho; debugger aponta a expressão e Rust gerado termina com mensagem clara.
- `tamanho` funciona em funções, parâmetros e laços.
- Cópias em atribuição, chamada e retorno são independentes na VM e em Rust.
- Debug snapshots mostram vetor e tipo; funções FERRO anteriores e corpus
  clássico continuam funcionando.
