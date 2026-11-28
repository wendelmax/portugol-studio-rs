# Arquitetura do port para Rust

## 1. Decisão de framework: Tauri v2

### Contexto

O Ferro Studio é uma IDE com:

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
| Depurador/console | painéis DOM dedicados | manual | manual |
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

> **Migração concluída (ADR 0016):** os crates do dialeto clássico (`portugol-syntax`,
> `portugol-semantics`) e `tests/corpus/` foram removidos na Fase 6. Spans e
> diagnósticos agora vivem em `ferro-hir`.

```
crates/
  ferro-hir/          # representação intermediária tipada, spans e diagnósticos compartilhados
  ferro-frontend/  # lexer, parser, formatador e checagem do Ferro (.fer)
  ferro-bytecode/     # instruções e compilador HIR -> bytecode
  ferro-vm/           # VM, depurador, protocolo gráfico, UI MVU, stdlib e bibliotecas PORS
  ferro-rust/         # geração de código Rust, projetos Cargo e source maps
  ferro-wasm/         # núcleo PORS sem dependências externas para WebAssembly
  ferro-service/      # fachada da IDE (análise, execução, depuração, exemplos e exercícios .licao)
  ferro-cli/          # executável `portugol`
apps/
  desktop/
    src-tauri/           # comandos e eventos Tauri v2, workspace (cap-std), áudio/gráficos
    src/                 # frontend (Vite + TypeScript + CodeMirror 6)
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

#### Como está implementado (fase 3)

O original não interpreta: ele traduz o Portugol para Java
(`execucao/gerador/GeradorCodigoJava.java`) e roda na JVM. A semântica de
execução a reproduzir é, portanto, a do Java gerado.

- `ferro-bytecode`: `Op` é um `enum` Rust (não bytes). Cada `Function` tem
  `code`, `spans` paralelos (mapa de fonte) e `locals` com nome/tipo de cada
  slot, já pensando no inspetor de variáveis. Uma função sintética
  `<programa>` inicializa as globais em ordem, roda comandos soltos e chama
  `inicio`.
- O compilador só usa tipos estáticos onde o destino é conhecido
  (declaração, atribuição, parâmetro, retorno, `leia`) para emitir
  `Convert`; as operações são resolvidas pela VM pelas tags dos valores.
- Alvos de escrita (`v[i] += x`, `m[i][j]++`) avaliam vetor e índices uma vez,
  em temporários, na ordem do Java.
- `ferro-vm`: VM de pilha com quadros explícitos e execução em fatias
  (`Vm::step(n)`), sinal de interrupção (`interrupt_handle`) e erro com pilha
  de chamadas. E/S passa pelo trait `Io` (`StdIo`, `BufferIo`; a IDE terá a
  sua).
- `ferro-rust`: lowering tipado e geração de Rust somente para o subconjunto
  descrito em `docs/FERRO-RUST.md`; retorna diagnóstico com span no primeiro
  recurso sem suporte e não participa da execução clássica.
- Parâmetro `&` guarda um `Reference` (slot da pilha, global ou elemento);
  vetores e matrizes são `Rc<RefCell<..>>` compartilhados, como arrays Java.

Comportamentos do original mantidos de propósito:

| Comportamento | Origem no Java |
|---|---|
| `inteiro` de 32 bits com *wrap-around* | `int` |
| `real` impresso como `1.0`, `1.0E7`, `1.0E-4` | `Double.toString` |
| `"x" + verdadeiro` dá `"xtrue"` | `StringBuilder.append(boolean)` |
| divisão/resto inteiro por zero é erro; real dá `Infinity` | `ArithmeticException` |
| `real -> inteiro` trunca e satura | cast `(int)` |
| `escolha` cai no próximo caso sem `pare` | `switch` |
| `leia` lê uma linha; inteiro via `Integer.parseInt` (sem espaços) | `AbaConsole` |
| `Tipos.cadeia_e_logico` aceita "começa com verdadeiro ou termina com falso" | regex `^verdadeiro\|falso$` |
| `inicio(cadeia params[])` recebe os parâmetros de execução | opções de execução da IDE |

Diferença deliberada: `caracter + caracter` concatena (a análise semântica
tipa como `cadeia`; no Java gerado isso somaria os códigos e não compilaria).

Desempenho: ~7 ns por instrução da VM (laço de 10 milhões de iterações em
~1,2 s; `fib(30)` em ~0,4 s), na faixa do CPython. Próximos passos se
necessário para jogos: instruções tipadas a partir dos tipos estáticos,
superinstruções (`LoadLocal+PushInt+Lt+JumpIfFalse`) e quadros sem `Vec`
genérico.

#### Como está implementado (fase 5)

`ferro_vm::debug::DebugControl` é compartilhado entre a thread da IDE e a
thread da VM. A VM verifica o controle antes de cada instrução: se o span da
instrução estiver dentro de um ponto de parada, ou se houve pedido de pausa,
publica um `PausedInfo` (span, pilha e variáveis) e bloqueia numa `Condvar`.
O callback de publicação roda com o bloqueio do controle tomado, o que fecha a
janela de corrida entre avisar a IDE e registrar a espera: um comando enviado
pela IDE só é processado depois que a VM está efetivamente parada.

Comandos: `Continue` segue até o próximo ponto de parada, `StepInto` para na
primeira instrução fora do comando corrente (entrando em chamadas), `StepOver`
ignora quadros mais profundos e `StepOut` só para em um chamador. A comparação
com o span inteiro da linha evita parar várias vezes nas instruções de um mesmo
comando; o quadro sintético `<programa>` é ignorado porque usa o span de todo o
arquivo.

O inspetor lê globais de `Module::globals` e locais de `Function::locals`
usando `base` do quadro corrente; vetores e matrizes são expandidos até 100
elementos. `ferro-service` converte spans em posições UTF-16 do editor e
expõe `RunHandle::{pause, continue_run, step_*, set_breakpoints}`; a camada
Tauri traduz isso em comandos `debug_*` e no evento `run:paused`.

### Bibliotecas padrão

| Biblioteca original | Estratégia no port |
|---|---|
| Matematica, Texto, Util, Tipos | Rust puro (**feito**, `ferro-vm/src/stdlib`) |
| Calendario | Rust puro |
| Arquivos | `std::fs` com sandbox por workspace |
| Internet, ServicosWeb | `reqwest` + runtime async |
| Sons | `rodio` |
| Teclado, Mouse, Graficos | host functions ligadas à janela/canvas |

## 4. IDE (Tauri v2)

### Frontend

- **Vite + TypeScript** (sem framework pesado; estado e painéis organizados em módulos/shells).
- **CodeMirror 6**:
  - destaque de sintaxe para Portugol clássico, Ferro (`.fer`) e Rust (`@codemirror/lang-rust`);
  - `@codemirror/lint` para diagnósticos do backend;
  - autocomplete via catálogo/serviço;
  - busca, dobramento e multi-cursor nativos.
- **Console integrado em DOM** (entrada/saída e `leia`, sem dependência de xterm.js).
- Layout: barra de atividades | explorador/aprender | editor com abas e prévia Rust/UI | painel inferior (console, testes, mensagens, depuração).

### Backend (comandos Tauri)

```rust
#[tauri::command]
fn check(source: String) -> Vec<DiagnosticInfo>;

#[tauri::command]
fn run(source: String, args: Vec<String>, path: Option<String>, breakpoints: Vec<u32>) -> u32;

#[tauri::command]
fn send_input(id: u32, line: String);

#[tauri::command]
fn stop(id: u32);

#[tauri::command]
fn debug_continue(id: u32); // também debug_pause, debug_step_*, debug_set_breakpoints
```

Eventos de uma execução (todos com `id`): `run:output`, `run:clear`,
`run:input`, `run:paused` e `run:finished`. Diagnósticos são entregues pelo
próprio `check`, sem evento separado.

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
| 3. Runtime | bytecode, VM, stdlib de console | **concluída** |
| 4. IDE MVP | Tauri, CodeMirror, abas, console, run | **concluída** |
| 5. Depurador | breakpoints, step, inspeção | **concluída** |
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
| Licença | manter LGPL-3.0 e atribuição ao projeto original |

## 8. Licença

O Ferro Studio é distribuído sob a licença LGPL-3.0 (`LGPL-3.0-only`). O texto
completo está no arquivo [`LICENSE`](../LICENSE).
