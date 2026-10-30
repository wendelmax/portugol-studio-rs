# Portugol Studio RS

Port do [Portugol Studio](https://github.com/UNIVALI-LITE/Portugol-Studio) (Java/Swing)
para **Rust**, com a IDE desktop em **Tauri v2** e o núcleo da linguagem em crates
Rust independentes.

> Status: **fases 1, 2 e 3 concluídas** — lexer, parser, AST, análise
> semântica, compilador para bytecode e máquina virtual com as bibliotecas de
> console (Util, Matematica, Texto, Tipos). Todos os programas de exemplo completos do repositório
> original passam; os erros restantes estão em trechos da ajuda (fragmentos que
> usam variáveis não declaradas) e em 2 exemplos com erros de sintaxe no próprio
> arquivo original.

## Por que Tauri (e não iced)?

O núcleo do produto é uma **IDE**: editor de código, abas, árvore de arquivos,
diagnósticos, console e depurador. Nesse cenário, o custo de construir um editor
de texto completo (seleção, multi-cursor, undo/redo, IME, busca, dobramento,
autocomplete) do zero em iced é muito maior do que o custo de usar a webview.

| Critério | Tauri v2 | iced |
|---|---|---|
| Editor de código | CodeMirror/Monaco prontos | implementar do zero |
| UI complexa (abas, painéis, menus) | HTML/CSS | widgets limitados |
| Núcleo em Rust | sim | sim |
| Consistência visual | depende da WebView | igual em todos os SOs |
| Tamanho/pegada | WebView do sistema | binário nativo |

Sniffnet **não é um framework**: é um app escrito em Rust com iced. Ele serve de
referência de arquitetura (separação núcleo/GUI), não como dependência.

## Estrutura

```
portugol-studio-rs/
├── crates/
│   ├── portugol-syntax/    # lexer (logos), parser, AST, diagnósticos ✅
│   ├── portugol-semantics/ # símbolos, escopos, tipos, checagens ✅
│   ├── portugol-bytecode/  # instruções e compilador AST -> bytecode ✅
│   ├── portugol-vm/        # máquina virtual, E/S, bibliotecas de console ✅
│   └── portugol-cli/       # `portugol`: valida e executa .por ✅
├── apps/
│   └── desktop/           # Tauri v2 + frontend (próxima fase)
└── docs/
    └── ARQUITETURA.md     # plano de port e decisões técnicas
```

## Como validar

Requisitos: Rust 1.85+ (edição 2024).

```bash
cargo test
cargo run -p portugol-cli -- caminho/para/arquivo.por
cargo run -p portugol-cli -- --quiet caminho/para/diretorio
cargo run -p portugol-cli -- --syntax-only arquivo.por   # só sintaxe

# executar um programa
cargo run -p portugol-cli -- executar arquivo.por
cargo run -p portugol-cli -- executar arquivo.por -- param1 param2  # inicio(cadeia params[])
cargo run -p portugol-cli -- executar --bytecode arquivo.por        # lista o bytecode
```

A análise semântica só roda quando o arquivo não tem erro de sintaxe, para
evitar diagnósticos em cascata.

O CLI aceita UTF-8 e cai automaticamente para ISO-8859-1 (os exemplos históricos
do Portugol Studio usam Latin-1).

Para inspecionar tokens:

```bash
cargo run -p portugol-syntax --example tokens -- arquivo.por
```

## Cobertura atual

Fonte: `ide/src/main/assets` do repositório original (196 arquivos `.por`).

```
196 arquivo(s), 7 com erro(s), 31 erro(s), 117 aviso(s)
```

Sintaxe — dois defeitos reais no código de exemplo original:

- `bibliotecas/tipos/logico.por`: `caso` sem expressão antes de `}`.
- `jogos/arkanoid.por`: parâmetro `real` sem nome.

Semântica — 29 erros, todos em `ajuda/recursos/exemplos/` (fragmentos de
documentação, não programas completos):

- 27 × `sem.symbol-not-declared`: trechos que usam variáveis declaradas fora do
  fragmento;
- 1 × `sem.redeclared`: `declaracoes/variavel/exemplo1.por` redeclara `var4`;
- 1 × `sem.incompatible-types`: `operacoes_logicas/nao/exemplo1.por` usa
  `nao 5 < 4`. No original `nao` liga mais forte que os operadores binários
  (`expressao(31)` no parser ANTLR), então a expressão é `(nao 5) < 4` e também
  é inválida lá.

Os avisos (contados, mas não listados pelo CLI) seguem o Java: conversão
implícita `inteiro ↔ real`, símbolo global ocultado e vetor/matriz que poderia
ter menos dimensões.

A suíte `crates/portugol-semantics/tests/analisador.rs` espelha os 34 casos de
`AnalisadorSemanticoTest.java` e acrescenta casos para constantes, retorno
obrigatório, aridade, conversões, referência, bibliotecas e vetores/matrizes.

### Execução

`cargo run --release -p portugol-vm --example corpus -- <dir>` executa todos os
exemplos sem entrada e com limite de 20 milhões de instruções:

```
  91  executou até o fim
  54  parou no leia (sem entrada)
  40  biblioteca ainda não portada (Graficos, Teclado, Mouse, Sons, Arquivos...)
   4  limite de instruções (laço de jogo/espera)
   7  não analisado (erros de sintaxe/semântica acima)
```

Nenhum exemplo termina em erro de execução inesperado. A semântica de execução
segue o Java gerado pelo original (inteiro de 32 bits, `Double.toString`,
`"x" + verdadeiro == "xtrue"` etc.); a lista completa está em
[`docs/ARQUITETURA.md`](docs/ARQUITETURA.md#como-está-implementado-fase-3).
A suíte `crates/portugol-vm/tests/execucao.rs` cobre aritmética, conversões,
fluxo, referências, vetores/matrizes, `leia`, bibliotecas e interrupção.

## Roadmap

1. **Sintaxe** ✅ — lexer, parser, AST, diagnósticos com spans, recuperação de erro.
2. **Semântica** ✅ — tabela de símbolos, escopos, tipos, constantes, checagens.
3. **Runtime** ✅ — compilação para bytecode + VM, bibliotecas padrão de console.
4. **IDE** — Tauri v2, CodeMirror 6, abas, árvore de arquivos, console, execução.
5. **Depurador** — breakpoints, step, inspeção de variáveis (via VM).
6. **Paridade** — gráficos/jogos, ajuda, plugins, instalador e atualização.

Detalhes, estimativas e riscos em [`docs/ARQUITETURA.md`](docs/ARQUITETURA.md).

## Licença

O projeto original é **GPL-3.0**. Este port é uma obra derivada e permanece
GPL-3.0 (`GPL-3.0-only`).
