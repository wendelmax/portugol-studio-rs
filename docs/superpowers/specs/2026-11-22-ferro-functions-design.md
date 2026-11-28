# Design: funções definidas pelo aluno em FERRO

## Intenção

Ampliar Ferro com funções nomeadas, parâmetros, chamadas e retornos. É a
primeira lacuna de linguagem estrutural da matriz FERRO, necessária para
exercícios reutilizáveis, abstração e aproximação semântica a Rust.

## Sintaxe proposta

```text
funcao dobro(valor: inteiro) -> inteiro {
    retorne valor * 2;
}

funcao inicio() {
    escreva(dobro(21));
}
```

Parâmetros usam `nome: tipo` e são imutáveis por padrão; `mutavel nome: tipo`
permite reatribuição dentro da função. O tipo de retorno usa `-> tipo`; função
sem retorno omite a seta e usa `retorne;` opcional. Tipos aceitos são os cinco
primitivos FERRO atuais. Argumentos são passados por valor; o Rust gerado clona
argumentos `cadeia` para preservar o comportamento de valor do dialeto. A
prévia identifica esse clone gerado. `inicio()` continua único, sem parâmetros
nem retorno.

## Objetivos e limites

- Resolver todas as assinaturas antes de validar corpos, permitindo chamadas
  para frente e recursão.
- Validar aridade, tipos, mutabilidade, escopo e retorno total em cada caminho
  de uma função não vazia.
- Executar o mesmo programa pelo compilador Rust e pelo bytecode/VM usado no
  debugger, com spans, frames e testes diferenciais.
- Rejeitar nomes duplicados, retorno incompatível/ausente, retorno com valor em
  função vazia e assinaturas inválidas com diagnósticos FERRO localizados.
- Nesta fatia não há funções como valores, closures, métodos, generics,
  sobrecarga nem parâmetros por referência explícita. Ownership de referências
  será tratado em uma feature separada.
- Não alterar parsing ou execução de `.por` e não remover o frontend clássico.

## Abordagens

1. Reescrita de texto ou geração inline: rejeitada; não representa chamadas,
   frames ou spans corretamente.
2. Adicionar statements/calls em HIR compartilhada, compilar para `Op::Call` e
   emitir funções Rust tipadas: escolhida. A VM já possui frames e chamadas.
3. Reescrever o runtime FERRO inteiro: rejeitada; duplicaria VM, debugger e
   limites.

## Critérios de aceite

- Testes cobrem forward calls, parâmetros, retorno, chamadas aninhadas,
  recursão limitada, mutabilidade de parâmetro, strings por valor e funções sem
  retorno.
- Aridade, tipo, binding desconhecido, função duplicada e caminho sem retorno
  são diagnosticados na assinatura/span correto.
- Casos válidos produzem o mesmo stdout pela VM e Rust gerado; debugger expõe
  frames/variáveis reais na VM.
- `rustc` valida o código gerado; formatter, spans e source maps permanecem
  válidos.
- Testes clássicos continuam sem mudança.
