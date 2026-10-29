# Portugol Studio RS

Port do [Portugol Studio](https://github.com/UNIVALI-LITE/Portugol-Studio) (Java/Swing)
para **Rust**, com a IDE desktop em **Tauri v2** e o núcleo da linguagem em crates
Rust independentes.

> Status: **fase 1 concluída** — lexer, parser e AST do Portugol 2.0, com
> 194 dos 196 arquivos `.por` do repositório original analisados sem erros.
> Os 2 restantes são exemplos com erros de sintaxe no próprio arquivo original.

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
│   ├── portugol-syntax/   # lexer (logos), parser, AST, diagnósticos ✅
│   └── portugol-cli/      # validador sintático de .por ✅
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
```

O CLI aceita UTF-8 e cai automaticamente para ISO-8859-1 (os exemplos históricos
do Portugol Studio usam Latin-1).

Para inspecionar tokens:

```bash
cargo run -p portugol-syntax --example tokens -- arquivo.por
```

## Cobertura atual

Fonte: `ide/src/main/assets` do repositório original (196 arquivos `.por`).

```
196 arquivo(s), 2 com erro(s), 2 erro(s), 0 aviso(s)
```

Os dois erros são defeitos reais no código de exemplo original:

- `bibliotecas/tipos/logico.por`: `caso` sem expressão antes de `}`.
- `jogos/arkanoid.por`: parâmetro `real` sem nome.

## Roadmap

1. **Sintaxe** ✅ — lexer, parser, AST, diagnósticos com spans, recuperação de erro.
2. **Semântica** — tabela de símbolos, escopos, tipos, constantes, checagens.
3. **Runtime** — compilação para bytecode + VM, bibliotecas padrão de console.
4. **IDE** — Tauri v2, CodeMirror 6, abas, árvore de arquivos, console, execução.
5. **Depurador** — breakpoints, step, inspeção de variáveis (via VM).
6. **Paridade** — gráficos/jogos, ajuda, plugins, instalador e atualização.

Detalhes, estimativas e riscos em [`docs/ARQUITETURA.md`](docs/ARQUITETURA.md).

## Licença

O projeto original é **GPL-3.0**. Este port é uma obra derivada e permanece
GPL-3.0 (`GPL-3.0-only`).
