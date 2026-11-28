# Contrato de semântica: Portugol ⇄ Ferro ⇄ Rust

**Não existe garantia de tradução automática universal.** Tratar o Ferro como dialeto **proposto**; não afirmar que compila em rustc diretamente.

## Descoberta obrigatória
Inventariar gramática real, bibliotecas, precedência, coercões, índices de vetores, escopo, comportamento das strings e entrada/saída do Portugol implementado. Não partir de suposições ou de um único exemplo de código.

## Camadas propostas (adaptar ao código real)
`source + language_id → lexer/parser → AST → symbol table/type checker → IR semântico → { interpreter, Rust emitter, diagnostics, source maps }`

Se o projeto já possui AST, usar e ampliar a representação em vez de criar árvore paralela sem justificativa. A saída da tradução inclui `(source revision, diagnostics, Rust source, mappings, status)`.

## Semânticas que precisam de decisões explícitas

| Conceito | Decisão exigida |
|---|---|
| `inteiro` | tamanho, overflow e coerções; `i32` é sugestão, não equivalência garantida |
| `real` | f32/f64, precisão, formatação |
| `cadeia` / `caracter` | string/char, unicode, clonagem, empréstimo e I/O |
| `logico` | bool e comparação |
| vetores/matrizes | índices, tamanho, limites, cópia e alocação |
| parâmetro de função | passagem por valor/referência e mutabilidade |
| escrita/leitura | formatos, stdin, newline e locale |
| bibliotecas | shims/adapters por assinatura + contratos de efeito |
| errors | falhas em tempo de execução versus compilação, mensagens pt-BR |
| ownership/borrowing | ensino só quando relevante; não inferir uma semântica de runtime incompatível |

## Tradução e mapeamento
- Nunca simplesmente substituir keywords: converter AST/IR e garantir comportamento.
- Mapeamento é por span semântico; marcar `approximate`, `generated-only` ou `unmapped` quando necessário.
- Emitir status `partial` para funções/expressões sem equivalência segura, não gerar Rust enganoso.
- Código gerado sinalizado como derivado e read-only; ao desvincular, estudante possui o arquivo e geração não sobrescreve.
- Testar equivalência observável para subset suportado: stdout, stderr, exit status, entrada, estruturas e operações.
- Para erros de compilação, mapear diagnósticos de Rust ao trecho original quando fonte map permitir, preservando mensagem original para especialistas.

## Exemplo didático (não contrato sintático definitivo)

```text
Ferro:
funcao inicio() {
  seja mutavel idade = 16;
  idade += 1;
  escreval(idade);
}

Rust:
fn main() {
  let mut idade: i32 = 16;
  idade += 1;
  println!("{}", idade);
}
```

## Golden fixtures
Cada regra suportada deve ter fixture `{entrada Portugol, Rust esperado, stdin, stdout/exit esperado, diagnósticos, mapping}`. Diferenciar testes de snapshot textual dos testes de equivalência de execução.
