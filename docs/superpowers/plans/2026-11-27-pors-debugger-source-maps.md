# PORS Debugger Source Maps Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expor, para a instrução pausada no debugger da IDE, o nó HIR, o offset do bytecode e a linha do Rust gerado, usando source maps internos já existentes e sem criar VM nova, painel novo ou API WASM nova.

**Architecture:** O compilador PORS passa a gravar um rótulo de nó HIR paralelo a cada opcode; a VM publica esse rótulo junto com `ip` em `PausedInfo`; o serviço converte para um campo opcional `mapping` em `DebugInfo` (contrato Tauri); a IDE usa esse campo e o `RustExport.sourceMap` já existente para calcular a linha gerada. O preview Rust continua somente leitura, sem formatação “bonita” e sem equivalência conceitual separada.

**Tech Stack:** Rust (`portugol-bytecode`, `portugol-vm`, `portugol-service`), TypeScript/CodeMirror (`apps/desktop`), `node:test`.

**Spec:** `docs/superpowers/specs/2026-11-26-pors-syntax-evolution-design.md:167-178` e decisão de contorno `:187`.

**Precondition:** `docs/superpowers/plans/2026-11-27-pors-nucleo-sintaxe.md` Tasks 1–5 concluídas; as novas construções já emitem spans próprios.

## Global Constraints

- Nenhuma VM nova, nenhum opcode novo, nenhum painel novo: usar `#debug`, `renderDebugInfo()` e o `run:paused` atuais.
- Nenhum campo novo em JSON WASM; `export_rust_pors_json()` continua com `ok`, `code`, `diagnostics`.
- Spans internos são UTF-8 em bytes; `Position` da IDE é UTF-16 — toda conversão passa por `LineIndex`.
- O preview Rust exibe `GeneratedRust.code` compilável; não introduzir tradução “bonita” nem tipo de equivalência conceitual nesta fatia.
- `escreval` não ganha opcode nem rótulo próprio: o rótulo aparece no `Write` sintético do `\n`.
- Commits sem `Co-Authored-By`, sem menção a Claude/IA/Anthropic, sem emoji.

## Review Focus

- Os rótulos por opcode precisam corresponder aos spans de cada parte da interpolação, não ao span da cadeia inteira — teste na Task 1.
- `leia()` dentro de expressão precisa ser rotulado `Read`, sem `StoreLocal` — teste na Task 1.
- Offsets UTF-8 da VM não podem ser exibidos como offsets UTF-16 da IDE — teste na Task 2.
- Se não houver segmento de source map para o trecho, o painel deve mostrar `—`, não uma linha inventada — teste na Task 2.
- A fachada WASM não pode ganhar campo novo — verificação na Task 3.

---

### Task 1: Rótulos de nó HIR no bytecode e na pausa da VM

**Files:**
- Modify: `crates/portugol-bytecode/src/op.rs:210-220`
- Modify: `crates/portugol-bytecode/src/pors.rs:206-210`, `:390-401`, `:641-820`
- Modify: `crates/portugol-vm/src/debug.rs:24-48`, `:241`, `:671-697`
- Modify: `crates/portugol-vm/src/vm.rs:619-629`, `:740`
- Update constructors: `crates/portugol-vm/tests/pors_core.rs:106`, `crates/portugol-vm/tests/ui_runtime.rs`, `crates/portugol-wasm/tests/wasm_facade.rs`
- Test: `crates/portugol-bytecode/tests/pors_compiler.rs`, `crates/portugol-vm/tests/pors_core.rs`

**Interfaces:**
- Consumes: `emit(op, span)`; `Function.spans`; `PausedInfo`.
- Produces: `InstructionMapping { node: &'static str, start: u32, end: u32 }`; `Function.nodes: Vec<Option<InstructionMapping>>`; `PausedInfo.ip` e `PausedInfo.mapping: Option<InstructionMapping>`.

- [ ] **Step 1: Escrever os testes que falham**

Em `crates/portugol-bytecode/tests/pors_compiler.rs`:

```rust
#[test]
fn maps_interpolation_parts_and_read_expression_to_hir_nodes() {
    let source = "funcao inicio() { escreva(\"Olá, {leia()}!\"); }";
    let program = portugol_rs_frontend::lower(source).expect("fonte válida");
    let module = portugol_bytecode::compile_pors(&program).expect("bytecode válido");
    let entry = &module.functions[module.entry as usize];
    let labels: Vec<_> = entry.nodes.iter().flatten().map(|n| n.node).collect();
    assert!(labels.contains(&"InterpolatedString.Literal"));
    assert!(labels.contains(&"InterpolatedString.Expression"));
    assert!(labels.contains(&"Read"));
}
```

Em `crates/portugol-vm/tests/pors_core.rs` (`entry` é o índice da função sintética em `Module::entry`; a pausa passa por `DebugControl`, cujo callback é chamado com o bloqueio interno tomado — por isso o `Continue` é enviado por outra thread):

```rust
#[test]
fn paused_info_exposes_the_instruction_ip_and_its_hir_mapping() {
    use std::sync::mpsc;
    use portugol_vm::debug::{DebugCommand, DebugControl};

    let source = "funcao inicio() { escreva(leia()); }";
    let module = compile_pors_source(source);
    let entry = &module.functions[module.entry as usize];
    let read_ip = entry.nodes.iter().position(|n| n.is_some_and(|n| n.node == "Read")).unwrap();
    let breakpoint = entry.spans[read_ip];

    let captured = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&captured);
    let (tx, rx) = mpsc::channel();
    let debug = DebugControl::new(move |info| {
        sink.lock().unwrap().push((info.ip, info.mapping.clone()));
        tx.send(()).expect("canal da pausa");
    });
    debug.set_breakpoints(vec![breakpoint]);
    let cont = debug.clone();
    let worker = thread::spawn(move || {
        rx.recv().expect("pausa no Read");
        cont.command(DebugCommand::Continue);
    });

    let mut io = BufferIo::new("Ada\n");
    Vm::new(&module, &mut io)
        .with_debug(debug)
        .run()
        .expect("execução com pausa");
    worker.join().expect("thread de continue");

    let found = captured.lock().unwrap().clone();
    assert!(found.iter().any(|(ip, mapping)| {
        *ip == read_ip && mapping.as_ref().is_some_and(|m| m.node == "Read")
    }));
}
```

Nota: `spans[i]` e `nodes[i]` são paralelos a `code[i]`, então `read_ip` serve tanto para o breakpoint quanto para a asserção do `ip`.

- [ ] **Step 2: Rodar e confirmar a falha**

Run: `cargo test -p portugol-bytecode --test pors_compiler maps_interpolation_parts_and_read_expression_to_hir_nodes && cargo test -p portugol-vm --test pors_core paused_info_exposes_the_instruction_ip_and_its_hir_mapping`
Expected: FAIL por campos inexistentes.

- [ ] **Step 3: Gravar rótulos no compilador**

Definir `InstructionMapping` em `op.rs` com `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. Acrescentar `nodes: Vec<Option<InstructionMapping>>` a `Function`, mantendo `nodes[i]` paralelo a `code[i]`/`spans[i]`. Em `emit()`, acrescentar `None`; criar `emit_node(op, span, node: InstructionMapping)` para as emissões rotuladas. Rotular: `Op::PushStr` de literal de interpolação com `InterpolatedString.Literal` e o span da parte; `Op::Add` que introduz uma parte com `InterpolatedString.Expression` e o span da parte; `Op::Read` vindo de `ExprKind::Read` com `Read` e o span da chamada; `Op::Return` da cauda com `Return.tail` e o span da expressão. Atualizar todos os construtores de `Function` nos testes listados em **Files** com `nodes: vec![None; code.len()]` (ou os rótulos apropriados nos testes que os verificam).

- [ ] **Step 4: Publicar na pausa da VM**

Acrescentar `ip: usize` e `mapping: Option<InstructionMapping>` a `PausedInfo`. Em `vm.rs::debug_info()`, preencher a partir do quadro atual: `ip = frame.ip` e `mapping = function.nodes.get(frame.ip).copied().flatten()`. Atualizar os dois construtores de `PausedInfo` (`debug.rs:241`, `vm.rs:740`). Nenhuma outra API muda: a pausa continua passando pelo callback de `DebugControl::new()`.

- [ ] **Step 5: Rodar e confirmar o passe**

Run: `cargo test -p portugol-bytecode -p portugol-vm -p portugol-wasm && cargo fmt --check`
Expected: PASS, inclusive os testes de UI MVU que constroem `Function` manualmente.

- [ ] **Step 6: Commit**

```bash
git add crates/portugol-bytecode crates/portugol-vm
git commit -m "feat(bytecode): rotular instruções com o nó HIR de origem"
```

---

### Task 2: Contrato Tauri e painel do debugger

**Files:**
- Modify: `crates/portugol-service/src/run.rs:42-55`, `:280-306`
- Modify: `apps/desktop/src/api.ts:37-53`
- Modify: `apps/desktop/src/run/debug-view.ts`, `apps/desktop/src/run/pause-presentation.ts`, `apps/desktop/src/run/debugger-shell.ts`, `apps/desktop/src/run/execution-presentation.ts`, `apps/desktop/src/main.ts`
- Modify: `apps/desktop/index.html:240-244`
- Test: `crates/portugol-service/tests/portugol_rs_debugger.rs`, `apps/desktop/tests/debug-mapping.test.mjs`, `apps/desktop/tests/pause-presentation.test.mjs`

**Interfaces:**
- Consumes: `PausedInfo.ip`/`mapping` (Task 1); `SourceMapInfo[]`, `sourceMapLinks()` → `SourceMapLink[]`.
- Produces: `DebugMappingInfo` serializado em `camelCase`; `DebugInfo.mapping: Option<DebugMappingInfo>`; `debugMappingText(info, links): string | null`.

- [ ] **Step 1: Escrever os testes que falham**

Em `crates/portugol-service/tests/portugol_rs_debugger.rs`, acrescentar um teste que inicia sessão com `escreva(leia());`, fornece `Ada` quando receber `Event::Input("cadeia")` e, no primeiro `Event::Paused(info)`, asserta:

```rust
let mapping = info.mapping.expect("pausa deve trazer o mapeamento");
assert_eq!(mapping.hir_node, "Read");
assert!(mapping.bytecode_offset < u32::MAX);
assert_eq!(mapping.hir_start.line, mapping.hir_end.line);
```

Em `apps/desktop/tests/debug-mapping.test.mjs` (criar):

```js
import test from "node:test";
import assert from "node:assert/strict";
import { debugMappingText } from "../src/run/debug-view.ts";

const info = {
  position: { offset: 0, line: 1, column: 1 },
  stack: [], variables: [],
  mapping: { hirNode: "Read", hirStart: { offset: 20, line: 2, column: 3 },
             hirEnd: { offset: 26, line: 2, column: 9 }, bytecodeOffset: 7 },
};
const links = [{ sourceLine: 2, generatedLine: 9, sourceOffset: 18, generatedOffset: 300, quality: "exact" }];

test("debugMappingText shows HIR node, bytecode offset and generated Rust line", () => {
  assert.equal(debugMappingText(info, links), "HIR Read@20..26 · bytecode 7 · Rust linha 9");
});

test("debugMappingText does not invent a Rust line without a mapped segment", () => {
  assert.equal(debugMappingText(info, [{ ...links[0], sourceOffset: 40, generatedLine: 99 }]),
    "HIR Read@20..26 · bytecode 7 · Rust linha —");
  assert.equal(debugMappingText({ ...info, mapping: null }, links), null);
});
```

Em `apps/desktop/tests/pause-presentation.test.mjs`, estender o fake `renderDebugInfo` para receber um terceiro argumento e afirmar que `links` é repassado.

- [ ] **Step 2: Rodar e confirmar a falha**

Run (na raiz): `cargo test -p portugol-service --test portugol_rs_debugger` e, em `apps/desktop`, `node --experimental-strip-types --test tests/debug-mapping.test.mjs tests/pause-presentation.test.mjs`
Expected: FAIL por `info.mapping` ausente e `debugMappingText` inexistente.

- [ ] **Step 3: Serializar o mapeamento no serviço**

Definir `DebugMappingInfo` com `#[derive(Debug, Clone, PartialEq, Eq, Serialize)] #[serde(rename_all = "camelCase")]`, campos `hir_node: String`, `hir_start: Position`, `hir_end: Position`, `bytecode_offset: u32`. Acrescentar `mapping: Option<DebugMappingInfo>` a `DebugInfo` (sem alterar os campos existentes). No callback de `run.rs:283-305`, converter `info.mapping` com `index.position(start)`/`index.position(end)` e `info.ip as u32`. Atualizar `apps/desktop/src/api.ts` com `DebugMapping` e `DebugInfo.mapping?: DebugMapping`.

- [ ] **Step 4: Renderizar no painel existente**

Em `debug-view.ts`, exportar `debugMappingText(info: DebugInfo, links: SourceMapLink[]): string | null`: sem `info.mapping`, devolver `null`; com mapeamento, escolher o link com maior `sourceOffset <= mapping.hirStart.offset` (usar `—` quando não houver) e montar exatamente `HIR {hirNode}@{start}..{end} · bytecode {offset} · Rust linha {linha}`. Em `renderDebugInfo()`, aceitar terceiro parâmetro opcional `sourceMap: SourceMapInfo[] = []`, converter com `sourceMapLinks(sourceMap)` e escrever o resultado em `#debug-mapping`, escondendo o elemento quando `null`. Usar `textContent`, nunca `innerHTML`. Em `index.html`, acrescentar `<div id="debug-mapping" class="debug-mapping" hidden></div>` dentro de `#debug`, ao lado de `#debug-stack`; estilizar em `apps/desktop/src/editor/theme` ou CSS existente do painel sem alterar a estrutura do painel.

- [ ] **Step 5: Repassar o source map já carregado**

Estender `PausePresentationDependencies` e `DebuggerShellDependencies` com `rustSourceMap?: () => SourceMapInfo[]`. Em `pause-presentation.ts`, chamar `renderDebugInfo(info, callback, dependencies.rustSourceMap?.() ?? [])`. Em `main.ts`/`execution-presentation.ts`, ligar o callback ao `sourceMap` do `RustExport` já usado pelo preview — nenhum comando Tauri novo.

- [ ] **Step 6: Rodar e confirmar o passe**

Run (na raiz): `cargo test -p portugol-service --test portugol_rs_debugger && cargo fmt --check`; em `apps/desktop`: `node --experimental-strip-types --test tests/debug-mapping.test.mjs tests/pause-presentation.test.mjs && npm run build`
Expected: PASS; `DebugInfo` antigo sem `mapping` continua aceito pelo frontend como `undefined`.

- [ ] **Step 7: Commit**

```bash
git add crates/portugol-service/src/run.rs crates/portugol-service/tests/portugol_rs_debugger.rs apps/desktop/src/api.ts apps/desktop/src/run apps/desktop/src/main.ts apps/desktop/index.html apps/desktop/tests/debug-mapping.test.mjs apps/desktop/tests/pause-presentation.test.mjs
git commit -m "feat(debugger): mostrar nó HIR, bytecode e linha Rust na pausa"
```

---

### Task 3: Verificação final e limites da superfície

**Files:**
- Modify: `docs/research/PORS_MIGRATION_MATRIX.md` (linha `Debugger`)
- Test: WASM e runner PORS já existentes

**Interfaces:**
- Consumes: Tasks 1–2; planos de núcleo e realce.
- Produces: fatia pronta para revisão, sem expansão do contrato público.

- [ ] **Step 1: Verificar os contratos inalterados**

Run:

```bash
cargo test -p portugol-wasm --test wasm_facade
grep -n "sourceMap" crates/portugol-wasm/src/lib.rs
```

Expected: testes PASS; nenhuma ocorrência de `sourceMap`/campo novo na fachada WASM.

- [ ] **Step 2: Rodar a suíte completa**

Run:

```bash
cargo fmt --check && cargo test && cargo clippy --all-targets -- -D warnings
```

Em `apps/desktop`: `npm test && npm run build && npm run test:e2e`.

Expected: 0 falhas.

- [ ] **Step 3: Atualizar a matriz**

Registrar na linha `Debugger` que a pausa passa a exibir nó HIR, offset de bytecode e linha do Rust gerado para as novas construções, mantendo a ressalva de que WASM não expõe source map e que a equivalência conceitual resumida continua fora desta fatia.

- [ ] **Step 4: Commit**

```bash
git add docs/research/PORS_MIGRATION_MATRIX.md
git commit -m "docs: registrar mapeamento do debugger para novas sintaxes"
```
