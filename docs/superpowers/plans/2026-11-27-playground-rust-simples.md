# Rust mais simples no Playground — Plano de implementação

> **Para agentes executores:** implementar cada tarefa neste worktree seguindo este plano. As etapas usam caixas de seleção (`- [ ]`).

**Objetivo:** mostrar no Playground Rust autônomo, mais legível e sem nomes da marca anterior, preservando os resultados dos programas Ferro.

**Arquitetura:** o gerador Rust passará a identificar as referências aos helpers no corpo e nos módulos de suporte incluídos e montará um preâmbulo mínimo com as dependências necessárias. Todos os identificadores e protocolos que aparecem no Rust gerado serão renomeados para Ferro; o consumidor usado por `leia()` será atualizado junto.

**Stack:** Rust (HIR/codegen), serviço Rust, TypeScript e testes existentes do workspace.

**Spec:** `docs/superpowers/specs/2026-11-27-playground-rust-simples-design.md`

## Restrições globais

- O resultado continua sendo um único `.rs` autônomo e compilável.
- O comportamento observável de programas suportados permanece igual.
- O suporte emitido pelo gerador não contém `PORS`/`pors`, `Portugol` nem identificadores da marca antiga; nomes e strings escritos pelo usuário são preservados.
- Helpers e runtimes necessários para recursos realmente usados permanecem disponíveis.
- O escopo não renomeia APIs internas não exibidas pelo código gerado.

## Foco de revisão

- Aritmética inteira deve preservar o comportamento de overflow do Ferro.
- Programas de texto devem manter interpolação, concatenação e limites de memória.
- Reais devem manter a representação formatada equivalente, sem incluir formatador se não houver uso.
- `leia()` deve continuar operando pelo protocolo da IDE com nomes Ferro.
- Vetores, UI MVU e bibliotecas devem manter seus helpers quando forem usados.

---

### Task 1: remover nomes antigos do Rust gerado e do protocolo

**Arquivos:**
- Modificar: `crates/ferro-rust/src/lib.rs`
- Modificar: `crates/ferro-rust/src/codegen/{mod.rs,expr.rs,stmt.rs,types.rs,runtime.rs}`
- Modificar: `crates/ferro-service/src/{pors_run.rs,lib.rs}`
- Modificar: `apps/desktop/src/learning-context.ts`
- Atualizar expectativas existentes: `crates/ferro-rust/src/lib.rs`, `crates/ferro-service/src/lib.rs` e `apps/desktop/tests/learning-context.test.mjs`

**Interfaces:**
- O gerador emitirá nomes internos no formato `__ferro_*`, módulos `__ferro_*`, rótulos de laço `ferro_for_*` e protocolo `FERRO_STUDIO_INPUT_PROTOCOL`/`FERRO_INPUT`.
- O serviço deverá reconhecer os novos nomes de protocolo e marcador de entrada; os runtimes de biblioteca usarão nomes de variáveis `FERRO_*`.
- A IDE deverá reconhecer os novos rótulos de laço nos trechos de Rust.

- [ ] Renomear no texto gerado todos os helpers, módulos, rótulos de laço, nomes de protocolo, mensagens de ambiente e User-Agent que hoje expõem Portugol/PORS.
- [ ] Atualizar o serviço e a IDE para consumir os novos nomes sem alterar o fluxo de entrada.
- [ ] Atualizar expectativas de testes existentes para os novos nomes; não criar casos de teste novos nesta tarefa.

### Task 2: emitir somente os helpers necessários

**Arquivos:**
- Modificar: `crates/ferro-rust/src/codegen/mod.rs`
- Modificar, se necessário para coletar referências: `crates/ferro-rust/src/codegen/types.rs`
- Modificar: `crates/ferro-rust/src/lib.rs` (helper de formatação real)

**Interfaces:**
- O renderizador montará primeiro o corpo e os runtimes das bibliotecas selecionadas; depois calculará dependências de helpers gerados e comporá o preâmbulo.
- Imports (`io`, `Write`, `FromStr`) e helpers de leitura, formatação real, strings, concatenação, indexação, clonagem de vetores e orçamento serão incluídos apenas se referenciados pelo código montado ou por dependências transitivas.

- [ ] Remover a emissão incondicional do formatador de reais e dos helpers centrais que o programa não usa.
- [ ] Preservar dependências transitivas dos runtimes selecionados e a ordem válida das declarações Rust.
- [ ] Manter a exportação de módulos de biblioteca seletiva já existente.

### Task 3: integração e revisão estática

**Arquivos:**
- Revisar os arquivos alterados nas tarefas anteriores.
- Modificar `apps/web-playground/app.mjs` somente se a resposta do WASM exigir adaptação; manter `runtime.exportRust()` como a origem da saída.

- [ ] Confirmar por inspeção que o corpo simples não inclui helpers de operações ausentes e que recursos especiais continuam incluindo suporte.
- [ ] Rodar `cargo check -p ferro-rust -p ferro-service` e `git diff --check`.
- [ ] Buscar `PORS|pors|PORTUGOL|portugol` no Rust gerado a partir das strings do codegen e confirmar ausência nos nomes emitidos.
