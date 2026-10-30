# Arquitetura do port para Rust

## 1. Decisão de framework: Tauri v2

### Contexto

O Portugol Studio é uma IDE com:

- editor de código com syntax highlighting, autocomplete e erros inline;
- abas e árvore de arquivos;
- console de entrada/saída;
- depurador com breakpoints e inspeção;
- ajuda integrada e exemplos;
- bibliotecas de gráficos/jogos.

### Alternativas

| | Tauri v2 | iced | egui |
|---|---|---|---|
| Editor de código | CodeMirror 6 | construir do zero | imaturável para IDE |
| Layout de IDE | HTML/CSS flexível | limitado | limitado |
| Núcleo Rust | total | total | total |
| Ecossistema | enorme (web) | pequeno | médio |
| Depurador/console | xterm.js, painéis | manual | manual |
| Consistência | WebView varia por SO | nativa | nativa |

**Decisão: Tauri v2.** A UI da IDE é o maior custo do port e é exatamente onde a
webview economiza meses de trabalho. iced seria adequado para um app nativo de
dados densos (como o Sniffnet), não para um editor de código completo.

### Consequências

- Duas linguagens no repositório: Rust (núcleo) + TypeScript (frontend).
- Necessário lidar com diferenças de WebView (WebView2/WebKitGTK/WKWebView).
- Dados de alta frequência (diagnósticos, output do console) devem ser
  agrupados (_batching_) antes de cruzar o IPC.

## 2. Workspace

```
crates/
  portugol-syntax/     # lexer, parser, AST, spans, diagnósticos         (feito)
  portugol-semantics/  # símbolos, escopos, tipos, análise semântica    (feito)
  portugol-bytecode/   # compilador AST -> bytecode + source map
  portugol-vm/         # máquina virtual com step/breakpoints
  portugol-stdlib/     # Matematica, Texto, Util, Tipos, Calendario, Arquivos
  portugol-service/    # fachada usada pelo Tauri (parse/analyze/run/debug)
apps/
  desktop/
    src-tauri/         # comandos e eventos Tauri
    src/               # frontend (Vite + TypeScript)
tests/
  corpus/              # .por + saída esperada
  differential/        # comparação com o Java original
```

## 3. Núcleo da linguagem

### Lexer/parser

- `logos` para o lexer (rápido, derivado, spans em bytes).
- Parser recursivo descendente escrito à mão.
- AST tipada, com `Span` em todos os nós.
- Recuperação de erro para a IDE continuar mostrando diagnósticos.
- Extensão sobre a gramática original: aceita trechos sem `programa { }`
  (os exemplos de ajuda são fragmentos).

Não portar o ANTLR mecanicamente foi deliberado: a IDE precisa de árvore de
sintaxe tolerante, spans e mensagens melhores. A gramática `.g4` é usada como
especificação (285 linhas).

### Análise semântica

Portar de `core/.../analise/semantica`:

- tabela de símbolos e escopos (global, função, bloco);
- resolução de nomes, bibliotecas e aliases (`inclua biblioteca X --> y`);
- tipos, conversões implícitas e checagem de chamadas;
- constantes, vetores/matrizes e parâmetros por referência (`&`);
- avisos (variável não inicializada, símbolo global ocultado etc.).

Saída atual: `Vec<Diagnostic>` com códigos `sem.*`, reutilizando o tipo da
sintaxe. A HIR anotada com tipos fica para a fase 3, quando o compilador de
bytecode precisar dela.

Implementação (`crates/portugol-semantics`):

- `types.rs`: `Ty`, tabelas de compatibilidade (atribuição, binária, unária,
  composta) portadas de `TabelaCompatibilidadeTiposPortugol.java`;
- `symbols.rs`: `Symbol`, `FunctionInfo`, `ScopeTable` com escopos léxicos;
- `analyzer.rs`: passagem única na ordem do Java — includes → pré-registro de
  funções → globais → corpos de funções → comandos soltos no topo.

Decisões:

- `Ty::Unknown` para retornos de bibliotecas ainda não modeladas: não gera
  diagnóstico e se propaga de forma permissiva. Só a inclusão e o alias da
  biblioteca são validados.
- `inicio` só é exigida quando o arquivo tem bloco `programa` explícito.
- Retorno obrigatório usa fluxo conservador (`retorne`; `se/senao` com ambos
  retornando; `escolha` com `caso contrario` e todos os casos retornando).
- Atribuição replica a ordem do Java para o diagnóstico de "não inicializada"
  (`a = a + 1` com `a` sem valor é erro).
- Funções de biblioteca aceitam vetores/matrizes inteiros como argumento.

### Runtime: por que bytecode

O depurador precisa pausar, inspecionar e continuar a execução. Um interpretador
recursivo (`eval`) dificulta isso em Rust (sem geradores estáveis). A escolha é
compilar a AST para **bytecode** com:

- pilha de operandos e _frames_ de chamada explícitos;
- `source map` instrução → span;
- instruções de linha para breakpoints;
- possibilidade de `step`, `pause`, `continue` e `inspect`.

Isso também melhora desempenho e isola o runtime do parser.

### Bibliotecas padrão

| Biblioteca original | Estratégia no port |
|---|---|
| Matematica, Texto, Util, Tipos, Calendario | Rust puro |
| Arquivos | `std::fs` com sandbox por workspace |
| Internet, ServicosWeb | `reqwest` + runtime async |
| Sons | `rodio` |
| Teclado, Mouse, Graficos | host functions ligadas à janela/canvas |

## 4. IDE (Tauri v2)

### Frontend

- **Vite + TypeScript** (sem framework pesado; Svelte é opcional).
- **CodeMirror 6**:
  - linguagem Lezer para Portugol (highlighting e indentação);
  - `@codemirror/lint` para diagnósticos do backend;
  - autocomplete via comandos Tauri;
  - busca, dobramento e multi-cursor nativos.
- **xterm.js** para o console (entrada/saída e `leia`).
- Layout: árvore de arquivos | editor com abas | painel de problemas/console.

### Backend (comandos Tauri)

```rust
#[tauri::command]
fn parse(source: String) -> ParseResult;

#[tauri::command]
fn analyze(path: String, source: String) -> AnalysisResult;

#[tauri::command]
fn run(path: String) -> RunHandle;

#[tauri::command]
fn debug_step(session: SessionId, action: DebugAction) -> DebugState;
```

Eventos para output em streaming:

```rust
app.emit("run:stdout", OutputChunk { session, text })?;
app.emit("run:diagnostics", diagnostics)?;
```

### Editor e análise incremental

Primeira versão: reanalisar o arquivo inteiro com _debounce_ (ex.: 150 ms).
Depois, se necessário, caching por arquivo e parsing incremental.

## 5. Fases e estimativas

Estimativas para **um desenvolvedor experiente, tempo integral**. Variam muito
com o nível de paridade desejado.

| Fase | Entrega | Estimativa |
|---|---|---|
| 0. Especificação e corpus | gramática, exemplos, testes dourados | 1 semana |
| 1. Sintaxe | lexer, parser, AST, diagnósticos | **concluída** |
| 2. Semântica | símbolos, tipos, checagens | **concluída** |
| 3. Runtime | bytecode, VM, stdlib de console | 4–6 semanas |
| 4. IDE MVP | Tauri, CodeMirror, abas, console, run | 3–4 semanas |
| 5. Depurador | breakpoints, step, inspeção | 2–3 semanas |
| 6. Paridade | gráficos/jogos, ajuda, plugins, packaging | 2–4 meses |

MVP utilizável (editar, analisar e executar programas de console): **~3 meses**.
Paridade funcional completa: **6–9 meses**.

## 6. Estratégia de validação

1. **Testes dourados**: cada `.por` do repositório original + saída esperada.
2. **Testes diferenciais**: executar o mesmo programa no Java original e na VM
   Rust, comparando stdout/stderr. Requer JDK 8 para rodar a referência.
3. **Fuzzing** do parser (não deve entrar em pânico nem travar).
4. **Testes de recuperação**: um erro não pode esconder os demais.

## 7. Riscos

| Risco | Mitigação |
|---|---|
| Semântica sutil (conversões, referências) | testes diferenciais desde o início |
| Gráficos/jogos (Java2D) | fase separada; canvas via host functions |
| Performance em programas grandes | bytecode + análise sob demanda |
| WebView inconsistente | testar nos 3 SOs; evitar APIs específicas |
| Escopo grande demais | portar por camadas, com MVP executável cedo |
| Licença | manter GPL-3.0 e atribuição ao projeto original |

## 8. Licença

O Portugol Studio original é GPL-3.0. O port é obra derivada; todo o código
deve permanecer GPL-3.0 (`GPL-3.0-only`) com atribuição ao projeto original.
