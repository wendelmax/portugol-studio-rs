# ADR 0017: Modelo funcional de estado, UI declarativa e runtime WASM/Rust para programas visuais em FERRO

- **Status:** Aceita
- **Data:** 2026-11-22
- **Responsável:** mantenedor do produto
- **Relacionados:** [ADR 0015](0015-evolucao-visual-orientada-a-aprendizagem.md), [ADR 0016](0016-ferro-como-linguagem-autoral-e-retirada-do-por.md), [plano de migração FERRO](../plans/ferro-only-migration.md), [matriz de migração FERRO](../research/ferro_MIGRATION_MATRIX.md)
- **Substituída parcialmente por:** [ADR 0021](0021-tela-como-forma-unica-de-interface-ferro.md)

> **Revisão (ADR 0021):** a decisão 1, interface declarativa com `UI.*`, foi substituída: interfaces em FERRO passam a ser criadas somente com telas, um `.design` com o code-behind `.fer` (ADR 0018), e o `UI.*` foi aposentado em etapas, concluídas em 2026-11-28 (o uso agora é o erro `ferro.ui-removido`). As decisões 2 a 5 (estado puro, VM em WASM, orçamento por evento e preservação de contratos) continuam valendo.

## Contexto

O Ferro Studio RS permanece uma aplicação **Rust + Tauri** cuja linguagem
autoral é **Ferro (`.fer`)**, com ponte progressiva para Rust real
(`.rs`/Cargo). O dialeto `.fer` já conta com HIR tipada independente
(`ferro-hir`), frontend próprio (`ferro-rust-frontend`), geração de Rust
com source maps (`ferro-rust`) e compilação HIR → bytecode para a VM
compartilhada (`ferro-bytecode`/`ferro-vm`).

Para sustentar uma futura **interface visual de criação de programas desktop e
web** em FERRO — sem reescrever a arquitetura existente e sem antecipar a
retirada de `.por`/`.pex` fora dos gates do ADR 0016 — é necessário definir:

1. como a interface gráfica de um programa FERRO é descrita para Desktop (Tauri
   WebView) e Web (DOM);
2. como eventos da interface atualizam o estado do programa em FERRO;
3. como programas visuais FERRO executam na prévia interativa e no alvo Web sem
   exigir a toolchain `rustc` instalada localmente.

## Decisão

1. **Interface declarativa por componentes no primeiro protótipo:**
   programas visuais em FERRO descrevem telas como uma árvore declarativa de
   componentes derivada do estado (`visualizar(estado: Estado) -> Visao`),
   começando com um conjunto pequeno e portátil entre WebView Tauri e DOM Web
   (`Coluna`, `Linha`, `Texto`, `Botao`, `Entrada`, `Lista`). Um componente
   `Canvas` para desenho 2D e jogos fica previsto como extensão futura, fora do
   escopo mínimo do primeiro protótipo.
2. **Modelo funcional puro de estado e eventos (MVU):**
   o estado da tela é modelado por um tipo composto (`registro`/`struct`), os
   eventos por variantes tipadas (`enum`) e toda transição de estado ocorre por
   uma função pura e explícita:
   `atualizar(estado: Estado, evento: Evento) -> Estado`.
   Essa escolha facilita depuração passo a passo, inspeção de snapshots e
   histórico (*time-travel*) na VM educacional, além de retirar referências
   mutáveis (`&mut`) e regras de aliasing/borrowing do caminho crítico P0 do
   primeiro protótipo visual. Referências e borrowing seguem na evolução
   pedagógica da Fase 2 do ADR 0016.
3. **Prévia interativa e alvo Web via VM FERRO em WASM + exportação Rust/Cargo:**
   o núcleo puro de FERRO (`ferro-hir`, `ferro-rust-frontend`,
   `ferro-bytecode` e o núcleo da `ferro-vm` sem dependências de sistema
   operacional/rede) será compilável para `wasm32-unknown-unknown`, permitindo
   análise, execução e prévia visual instantânea no navegador e no WebView
   Tauri sem exigir `rustc` local. A geração de Rust (`ferro-rust`) e o fluxo
   Cargo permanecem o caminho oficial para build nativo/web real.
4. **Ciclo de vida orientado a eventos com orçamento por evento:**
   o limite atual de 3 segundos totais do runner *batch*
   (`crates/ferro-service/src/ferro_run.rs`) não se aplica ao tempo em que a
   janela de um programa visual permanece aberta aguardando interação. Apps
   visuais terão orçamento por *tick*/evento (tempo ativo de `atualizar` e
   `visualizar`, limite cumulativo de memória de cadeias e limite de nós da
   árvore de visão), além de cancelamento explícito pelo host.
5. **Preservação de arquitetura, contratos e gates:**
   - não renomear o produto nem reescrever o shell Rust + Tauri;
   - manter `.por`/`.pex` intactos até cumprir integralmente o gate da
     [matriz de migração](../research/ferro_MIGRATION_MATRIX.md);
   - manter equivalência observável e testes diferenciais entre VM FERRO e Rust
     gerado para todas as construções de linguagem publicadas, documentando
     explicitamente diferenças inevitáveis de host.

## Alternativas consideradas

### Estado mutável por referência (`&mut Estado`) nos handlers
Rejeitada para o primeiro protótipo visual. Exigiria introduzir referências
mutáveis, checagem de empréstimos e regras de aliasing no frontend FERRO antes
de validar a experiência visual, além de dificultar a captura imutável de
estados anteriores no depurador.

### Execução Web exclusivamente via compilação `rustc`/Cargo
Rejeitada. Obriga a instalação de toolchain Rust/WASM nas máquinas de alunos e
escolas e adiciona latência de compilação incompatível com feedback imediato em
um construtor visual.

### Modelo principal baseado na biblioteca imperativa `Graficos` (Java2D)
Rejeitada para aplicativos desktop/web. Embora a biblioteca `Graficos` atenda a
jogos 2D clássicos em Canvas, ela não oferece acessibilidade nativa, foco de
teclado, layout responsivo ou controles de formulário equivalentes em Desktop e
Web.

### Protótipo híbrido já com `Canvas` desde a primeira entrega
Rejeitada como requisito inicial. Introduzir duas superfícies de renderização e
dois protocolos de entrada ao mesmo tempo amplia o escopo antes de validar o
núcleo declarativo + estado funcional + runtime WASM. `Canvas` permanece como
extensão compatível com a árvore declarativa após o primeiro protótipo.

## Consequências

- **P0.1 (linguagem):** após a integração de vetores em FERRO, a prioridade de
  linguagem passa a ser `registro`/`struct`, `enum`, `escolha` exaustivo e
  acesso a campos (`.`) na HIR, frontend, bytecode/VM e gerador Rust.
- **P0.2 (portabilidade WASM):** isolar dependências de SO e rede em
  `crates/ferro-vm/Cargo.toml` (`ureq`, `chrono/clock`, `std::fs`, threads
  e sincronização desktop) atrás de feature flags opcionais, com verificação de
  compilação `wasm32-unknown-unknown` no CI sem regressão no desktop.
- **P0.3 (contrato Host/UI):** habilitar módulos/chamadas nativas na HIR e em
  `compile_ferro` (`Op::CallNative`), definir o protocolo serializável de árvore
  de visão e eventos entre FERRO e host (Tauri WebView / DOM Web) e aplicar
  orçamento por evento.
- **P1 (edição bidirecional e ergonomia):** formatador FERRO determinístico,
  recuperação básica de erros no parser, `senao se`, `Option`/`Result` e núcleo
  mínimo de `Texto`/`Tipos` para conversões seguras em formulários.
- **Escopo explícito:** FERRO continua sendo uma linguagem educacional autoral;
  esta decisão não promete compatibilidade com todo o ecossistema Rust nem
  substitui projetos Cargo reais em `.rs`.
