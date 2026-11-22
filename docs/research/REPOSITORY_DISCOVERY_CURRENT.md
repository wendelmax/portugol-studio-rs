# Descoberta atual do repositório

Snapshot: `main` em `660a779` (22/11/2026), após PR #95. A direção atual é o
[ADR 0016](../adr/0016-pors-como-linguagem-autoral-e-retirada-do-por.md):
PORS/Rust como produto, sem renomear a aplicação; `.por`/`.pex` só saem depois
do gate em [`PORS_MIGRATION_MATRIX.md`](PORS_MIGRATION_MATRIX.md).

## Arquitetura observada

- **Desktop:** Tauri v2, `tauri-plugin-dialog`, comandos e capacidades em
  `apps/desktop/src-tauri/`. O produto permanece desktop; Vite serve ao bundle
  frontend.
- **Frontend:** HTML/CSS/TypeScript sem framework SPA, Vite, CodeMirror 6 e
  Node `node:test`. Editor e workspace já distinguem `.por`, `.pors`, `.rs` e
  `.pex` em vários fluxos.
- **Núcleo Rust:** workspace edition 2024 com crates de syntax, HIR, semântica,
  bytecode, VM, geração Rust e service. Dependências clássicas e PORS convivem
  neste snapshot.
- **Portugol clássico:** `portugol-syntax` → `portugol-semantics` → compilador
  bytecode/VM; mantém APIs públicas `check`, `start`, `inspect`, `export_rust`,
  CLI, corpus, bibliotecas e 54 exercícios XML `.pex`.
- **PORS:** `portugol-rs-frontend` → HIR; geração Rust via `portugol-rust` para
  execução normal; `portugol-bytecode::compile_pors` → VM compartilhada para
  debugger. O frontend aceita entrypoint `funcao inicio()` sem parâmetros,
  cinco tipos primitivos, bindings explícitos, expressões, I/O, condições,
  loops, `pare` e `continue`. Não tem funções do usuário, coleções ou
  bibliotecas. A gramática não é Rust completo.
- **Rust:** `.rs` standalone usa `rustc`/`rustfmt`, supervisão, cancelamento,
  diagnósticos e argumentos. Projetos com `Cargo.toml` têm discovery offline,
  build/run do alvo selecionado e `cargo check` sob demanda; debugger Rust e lint
  contínuo Cargo ainda não existem. Scripts/dependências Cargo não são
  sandboxed e exigem consentimento.
- **Tauri e processos:** frontend não invoca shell arbitrário. Chamadas de Rust
  usam processos supervisionados/limites e APIs Tauri; a execução standalone
  mantém aviso de que não é sandbox.
- **Aprendizagem:** modos Beginner/Bridge/Rust, source maps, conteúdos e
  progresso de exercícios. Ownership está implementado como simulação
  editorial com fixtures; não analisa o buffer nem é trace do processo Rust.

## Dependências clássicas a portar ou retirar

O parser/analisador/compiler clássico sustenta `portugol-service`, CLI,
corpus/regressões, bibliotecas, exportador classic, ajuda e exemplos. Tauri,
`workspace.rs`, `open_files.rs`, configuração de associação de arquivos e
frontend também codificam `.por`/`.pex`. Catálogo de exemplos, ajuda e checker
de exercícios refletem o produto Java. Detalhes e ordem estão na matriz PORS.

A VM não é exclusivamente clássica: `compile_pors` usa `Module`/`Vm` para o
debugger. A remoção deverá procurar consumidores e deixar esse caminho intacto.

## Lacunas de migração prioritárias

1. Funções, parâmetros/retornos, chamadas e política de recursão.
2. Coleções, indexação, tipos compostos, controle/matching e conceitos
   ownership/borrowing verificáveis.
3. Bibliotecas e contratos para execução PORS e debugger.
4. Conteúdo, ajuda e exercícios PORS/Rust; avaliador próprio versionado.
5. CLI PORS e fluxos PORS/Rust sem fallback implícito para linguagem clássica.
6. Retirada `.por`/`.pex` apenas após todos os gates da matriz.

## Evidência recente

- PR #93: roteamento por ID e descarte/replay ordenado de eventos de execução.
- PR #94: comparação diferencial VM/Rust gerado para operadores e span de
  operador não suportado.
- PR #95: diagnóstico de nome desconhecido com offsets UTF-16 após Unicode e
  rejeição semântica de incompatibilidade de tipos.
- PRs #93–#95 passaram CI de núcleo e Tauri em Linux, Windows e macOS. Consulte
  os checks de cada PR para logs; não extrapolar esta evidência a recursos
  PORS ainda não implementados.

## Registro de auditoria de worktrees

Em 22/11/2026, 21 worktrees antigas foram auditadas: 15 limpas de trabalho já
mesclado foram removidas; seis foram preservadas por alterações, documentos
não rastreados ou dependências compartilhadas. As worktrees preservadas
precisam de nova conferência antes da limpeza documental final. Documentos
untracked do usuário devem continuar intactos.
