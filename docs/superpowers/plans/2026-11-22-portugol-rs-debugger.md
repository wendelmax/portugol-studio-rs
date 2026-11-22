# Depurador Portugol-RS Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Depurar o subset `.pors` suportado com stepping, breakpoints, entrada, cancelamento e snapshots reais da VM.

**Architecture:** Lowering continuará produzindo a HIR compartilhada. Um backend HIR→bytecode alimentará `portugol-vm` e os contratos `RunHandle`/`RunObserver` existentes; a execução normal permanece em `rustc`. Testes diferenciais bloqueiam construções cuja semântica VM não corresponda ao Rust gerado.

**Tech Stack:** Rust workspace, Portugol-RS HIR/frontend, bytecode e VM existentes, Tauri, TypeScript, Node `node:test`.

**Spec:** `docs/superpowers/specs/2026-11-22-portugol-rs-debugger-design.md`

## Global Constraints

- Não alterar a execução normal `.pors` baseada em Rust gerado.
- Não oferecer debugger para `.rs` ou Cargo nesta fatia.
- Pausa ocorre antes da instrução mapeada; `leia` pausa antes do pedido e aguarda entrada sem pausa repetida.
- Depuração respeita runtime ativo de 3 s, stdout de 1 MiB e strings cumulativas de 16 MiB; espera de pausa/entrada não consome runtime.
- Snapshots representam valores Portugol-RS da VM, não memória/ownership Rust.
- Preservar contratos existentes de `run:*`, `runId`, cancelamento, UI e debugger clássico.

## Review Focus

- Identidades HIR sombreadas devem mapear para slots locais distintos; teste de shadowing no compilador.
- Operações com divergência potencial (overflow, divisão por zero, curto-circuito e concatenação) precisam de testes diferenciais por categoria/span.
- `leia` pausado e cancelamento durante pausa/entrada devem ser testados no runtime.
- Instruções com spans UTF-8 devem mapear para linhas/colunas corretas em caracteres multibyte.
- Fonte obsoleto ou eventos de sessão anterior não podem alterar o snapshot atual; testar descarte pelo fluxo existente.

## Arquivos e responsabilidades

- `crates/portugol-bytecode/src/` e testes: compilação de HIR PORS para bytecode, preservando spans e bindings.
- `crates/portugol-vm/src/`: limites e relógio ativo necessários ao runtime depurável, sem contar pausas/entrada.
- `crates/portugol-service/src/` e `tests/`: novo ponto de entrada de depuração PORS, criação de sessão e equivalência diferencial contra o backend Rust.
- `apps/desktop/src-tauri/src/`: comando Tauri que conecta o serviço ao contrato atual de debugger/eventos.
- `apps/desktop/src/`: capacidade, roteamento `.pors`, rótulo de backend e tratamento UI usando controles existentes.
- `docs/adr/`, `docs/PARIDADE.md`, `docs/reports/`: decisão de backend e estado/evidência do recurso.

## Plano

### Task 1: HIR PORS → bytecode com spans e bindings

**Files:**
- Modify: `crates/portugol-bytecode/src/*`
- Test: testes unitários do crate bytecode

**Interface:** adicionar uma função pública que aceite HIR baixada e produza `Program` bytecode ou diagnóstico de lowering; preservar IDs de binding e span por instrução.

- [x] Escrever testes para declaração/uso, mutação, condicionais, laços `enquanto`/`para`, `pare`, `continue`, concatenação, shadowing e spans UTF-8.
- [x] Executar os testes focados e verificar falha por ausência do backend.
- [x] Implementar lowering HIR→bytecode, recusando construções não representáveis com span do `.pors`.
- [x] Rodar testes focados e toda a suíte do crate.

### Task 2: Runtime de debug com entrada, limites e tempo ativo

**Files:**
- Modify: `crates/portugol-vm/src/*`
- Test: testes de runtime/controle da VM

**Interface:** expor execução instrumentada que usa `DebugControl`, breakpoints e snapshots, respeitando saída/string/runtime e cancelamento responsivo.

- [x] Testar pausa antes de `leia`, espera sem evento de pausa repetido, retomada após entrada válida e cancelamento durante espera de entrada.
- [x] Testar limite de saída, orçamento cumulativo de strings e runtime ativo no modo instrumentado.
- [x] Verificar que o relógio de runtime exclui pausa do debugger e espera de entrada.
- [x] Implementar limites tipados na VM e passar os testes focados.

### Task 3: Serviço de depuração e gate diferencial

**Files:**
- Modify: `crates/portugol-service/src/*`
- Create/modify: `crates/portugol-service/tests/portugol_rs_debugger.rs`

**Interface:** `debug_portugol_rs(source, breakpoints, pause_on_start, observer) -> Result<RunHandle, Diagnostics>` usando `lower`, backend bytecode e o runtime VM existente.

- [x] Adicionar fixture diferencial inicial; comparar stdout em declaração, mutação, `para`, condição e `escreva`.
- [x] Executar primeiro testes focados e confirmar que falham por falta do serviço/backend.
- [x] Implementar serviço, limites tipados e teste de erro de lowering.
- [x] Rodar suíte do serviço e do workspace offline; documentar o teste que depende de rede.

### Task 4: Comando Tauri e eventos

**Files:**
- Modify: `apps/desktop/src-tauri/src/commands.rs` (ou módulo atual de comandos) e allowlist/registro Tauri
- Test: testes Rust disponíveis para despacho/contratos

- [x] Estender o roteamento do comando para preservar `runId`, eventos `run:*`, entrada e stop.
- [x] Ligar o comando ao serviço e reutilizar o protocolo atual.
- [x] Rodar `cargo check -p portugol-desktop` e testes direcionados.

### Task 5: UI `.pors` e regressão visual/funcional

**Files:**
- Modify: `apps/desktop/src/runtime-capabilities.ts`, `apps/desktop/src/main.ts`, `apps/desktop/src/index.html` ou módulos de debugger existentes
- Test: testes Node existentes e novos casos de capacidade/roteamento

- [x] Testar capacidade de Depurar `.pors` independentemente de `rustc`, enquanto Executar normal conserva a exigência de toolchain.
- [x] Roteiar Depurar para o comando VM, manter os controles e snapshots existentes e mostrar `VM Portugol-RS` na sessão.
- [ ] Testar eventos antigos e alternância para Portugol/Rust sem vazamento de estado.
- [x] Rodar `npm test` e `npm run build`; smoke visual nativo continua pendente.

### Task 6: Documentar contratos e estado entregue

**Files:**
- Create: `docs/adr/0014-debugger-portugol-rs-vm.md`
- Modify: `docs/PARIDADE.md`
- Create: `docs/reports/FEAT-PORTUGOL-RS-DEBUGGER.md`

- [x] Documentar escolha VM, limites, subset real e diferença da simulação de ownership.
- [x] Atualizar paridade apenas com comportamento coberto por testes e registrar pendências de UI/E2E.
- [ ] Rodar verificação final completa, revisar cobertura spec→testes e registrar comandos/resultados.

