# ADR 0017: Modelo funcional de estado, UI declarativa e runtime WASM/Rust para programas visuais em PORS

- **Status:** Aceita
- **Data:** 2026-11-22
- **Responsável:** mantenedor do produto
- **Relacionados:** [ADR 0015](0015-evolucao-visual-orientada-a-aprendizagem.md), [ADR 0016](0016-pors-como-linguagem-autoral-e-retirada-do-por.md), [plano de migração PORS](../plans/pors-only-migration.md), [matriz de migração PORS](../research/PORS_MIGRATION_MATRIX.md)
- **Substituída parcialmente por:** [ADR 0021](0021-tela-como-forma-unica-de-interface-pors.md)

> **Revisão (ADR 0021):** a decisão 1, interface declarativa com `UI.*`, foi substituída: interfaces em PORS passam a ser criadas somente com telas, um `.design` com o code-behind `.pors` (ADR 0018), e o `UI.*` é aposentado em etapas. As decisões 2 a 5 (estado puro, VM em WASM, orçamento por evento e preservação de contratos) continuam valendo.

## Contexto

O Portugol Studio RS permanece uma aplicação **Rust + Tauri** cuja linguagem
autoral é **Portugol-RS (`.pors`)**, com ponte progressiva para Rust real
(`.rs`/Cargo). O dialeto `.pors` já conta com HIR tipada independente
(`portugol-hir`), frontend próprio (`portugol-rs-frontend`), geração de Rust
com source maps (`portugol-rust`) e compilação HIR → bytecode para a VM
compartilhada (`portugol-bytecode`/`portugol-vm`).

Para sustentar uma futura **interface visual de criação de programas desktop e
web** em PORS — sem reescrever a arquitetura existente e sem antecipar a
retirada de `.por`/`.pex` fora dos gates do ADR 0016 — é necessário definir:

1. como a interface gráfica de um programa PORS é descrita para Desktop (Tauri
   WebView) e Web (DOM);
2. como eventos da interface atualizam o estado do programa em PORS;
3. como programas visuais PORS executam na prévia interativa e no alvo Web sem
   exigir a toolchain `rustc` instalada localmente.

## Decisão

1. **Interface declarativa por componentes no primeiro protótipo:**
   programas visuais em PORS descrevem telas como uma árvore declarativa de
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
3. **Prévia interativa e alvo Web via VM PORS em WASM + exportação Rust/Cargo:**
   o núcleo puro de PORS (`portugol-hir`, `portugol-rs-frontend`,
   `portugol-bytecode` e o núcleo da `portugol-vm` sem dependências de sistema
   operacional/rede) será compilável para `wasm32-unknown-unknown`, permitindo
   análise, execução e prévia visual instantânea no navegador e no WebView
   Tauri sem exigir `rustc` local. A geração de Rust (`portugol-rust`) e o fluxo
   Cargo permanecem o caminho oficial para build nativo/web real.
4. **Ciclo de vida orientado a eventos com orçamento por evento:**
   o limite atual de 3 segundos totais do runner *batch*
   (`crates/portugol-service/src/pors_run.rs`) não se aplica ao tempo em que a
   janela de um programa visual permanece aberta aguardando interação. Apps
   visuais terão orçamento por *tick*/evento (tempo ativo de `atualizar` e
   `visualizar`, limite cumulativo de memória de cadeias e limite de nós da
   árvore de visão), além de cancelamento explícito pelo host.
5. **Preservação de arquitetura, contratos e gates:**
   - não renomear o produto nem reescrever o shell Rust + Tauri;
   - manter `.por`/`.pex` intactos até cumprir integralmente o gate da
     [matriz de migração](../research/PORS_MIGRATION_MATRIX.md);
   - manter equivalência observável e testes diferenciais entre VM PORS e Rust
     gerado para todas as construções de linguagem publicadas, documentando
     explicitamente diferenças inevitáveis de host.

## Alternativas consideradas

### Estado mutável por referência (`&mut Estado`) nos handlers
Rejeitada para o primeiro protótipo visual. Exigiria introduzir referências
mutáveis, checagem de empréstimos e regras de aliasing no frontend PORS antes
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

- **P0.1 (linguagem):** após a integração de vetores em PORS, a prioridade de
  linguagem passa a ser `registro`/`struct`, `enum`, `escolha` exaustivo e
  acesso a campos (`.`) na HIR, frontend, bytecode/VM e gerador Rust.
- **P0.2 (portabilidade WASM):** isolar dependências de SO e rede em
  `crates/portugol-vm/Cargo.toml` (`ureq`, `chrono/clock`, `std::fs`, threads
  e sincronização desktop) atrás de feature flags opcionais, com verificação de
  compilação `wasm32-unknown-unknown` no CI sem regressão no desktop.
- **P0.3 (contrato Host/UI):** habilitar módulos/chamadas nativas na HIR e em
  `compile_pors` (`Op::CallNative`), definir o protocolo serializável de árvore
  de visão e eventos entre PORS e host (Tauri WebView / DOM Web) e aplicar
  orçamento por evento.
- **P1 (edição bidirecional e ergonomia):** formatador PORS determinístico,
  recuperação básica de erros no parser, `senao se`, `Option`/`Result` e núcleo
  mínimo de `Texto`/`Tipos` para conversões seguras em formulários.
- **Escopo explícito:** PORS continua sendo uma linguagem educacional autoral;
  esta decisão não promete compatibilidade com todo o ecossistema Rust nem
  substitui projetos Cargo reais em `.rs`.
