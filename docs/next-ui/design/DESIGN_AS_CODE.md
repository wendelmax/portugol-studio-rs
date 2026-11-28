# Portugol Studio Next — Design as Code

> Version: 0.1.0 | Status: Proposed | Language: pt-BR | Audience: designers, frontend engineers, AI coding agents

## 0. Contract

This is the source of truth for the **visual and interaction design** of Portugol Studio Next, an educational IDE evolving Portugol Studio toward progressive Rust learning.

**Scope:** desktop-first IDE, adaptable to 1280px+ viewports. Smaller screens must reflow panels; do not hide core editor functionality. The reference mockup is illustrative, not a pixel-perfect implementation mandate. Code snippets, execution times, and test statuses in the mockup are demo data and **must not be treated as live results**.

**Non-goals (v0.1):** full browser/mobile editor, cloud synchronization, real-time collaboration, complete Rust compiler implementation, automatic equivalence for arbitrary programs.

### Product principles

1. Education First: every complex concept has contextual, non-intrusive explanation.
2. Progressive Disclosure: advanced panels are optional and mode-sensitive.
3. Dual Language: inspect original and translated source together, but never imply translation is infallible.
4. Visual Debugging: show execution, variable values, scope and, where supported, memory movement.
5. IDE-grade UX: familiar keyboard shortcuts, resizeable panes, tabs, command palette, accessible contrasts.
6. Student control: generated Rust is clearly identified and is not silently overwritten.

## 1. Learning modes

| Mode | Primary editor | Secondary editor | Education panel | Runtime |
|---|---|---|---|---|
| `beginner` / Iniciante | Classic Portugol | Hidden by default | Basic hints | Portugol interpreter |
| `bridge` / Ponte para Rust | Portugol or Ferro | Generated Rust | Active explanations and mapping | Interpreter or Rust (explicit choice) |
| `rust` / Rust | Rust | Optional source comparison | Advanced types, ownership, borrowing | Cargo / rustc |

Mode affects defaults, hints and visible controls; it **must not change source code automatically**. Persist mode per workspace.

## 2. Desktop information architecture

```text
APP SHELL
├── TitleBar: logo · workspace selector · Run / Debug · Share · window controls
├── WorkArea
│   ├── ActivityRail (56px): Projeto · Aprender · Exemplos · Debug · Configurações
│   ├── ExplorerPanel (220–280px; collapse/resize)
│   ├── EditorWorkspace (flex)
│   │   ├── EditorTabs
│   │   ├── EditorSplit (resizable)
│   │   │   ├── SourceEditor: Portugol / Ferro / Rust
│   │   │   ├── MappingGutter: source-to-generated correspondences
│   │   │   └── GeneratedRustEditor: read-only initially, editable after explicit detach
│   │   └── BottomPanel: Console · Tests · Output · Problems
│   └── LearningInspector (300–380px; collapse/resize)
│       ├── ContextualExplanation
│       ├── LearningProgress
│       └── VariablesMemoryPanel
└── StatusBar: diagnostics · mode · Rust edition · cursor · indentation · encoding
```

Suggested desktop ratios at 1536×960: ActivityRail 56px, Explorer 232px, Inspector 336px, remaining central editor area. Split source and generated editors 50:50 within available workspace. Minimum readable editor width 320px: collapse inspector first, explorer second, then use editor tabs instead of forcing narrow code columns.

### Breakpoints

- `>= 1440px`: full layout with editor split and learning inspector.
- `1200–1439px`: inspector defaults collapsed or as drawer; split editors on demand.
- `900–1199px`: explorer collapsible; source/Rust editor switcher instead of permanent split.
- `< 900px`: unsupported for full desktop IDE; provide a readable compact view, avoid unusable nested horizontal scrolling.

All side/bottom panes resizable; store pane sizes locally per workspace. Restore defaults action required.

## 3. Design tokens

Use semantic tokens rather than hard-coded colors in components. Theme below is **dark default**, with a future light theme driven by the same semantic keys.

```yaml
product:
  name: Portugol Studio Next
  locale: pt-BR
  direction: ltr
  defaultTheme: dark

color:
  bg:
    app: '#0C1118'
    panel: '#111923'
    elevated: '#18232F'
    editor: '#0D141D'
    selection: '#143D37'
  border:
    subtle: '#273340'
    strong: '#405364'
    focus: '#55D6AA'
  text:
    primary: '#E7EFF5'
    secondary: '#A7B8C5'
    muted: '#8495A2'
  accent:
    primary: '#34D399'
    primaryHover: '#55E4AD'
    amber: '#F4BD55'
    blue: '#65C7FF'
    purple: '#C99CFF'
  status:
    success: '#46D99A'
    warning: '#F4BD55'
    error: '#FB7185'
    info: '#65C7FF'

typography:
  ui: 'Inter, system-ui, sans-serif'
  code: 'JetBrains Mono, ui-monospace, monospace'
  uiBody: 13
  uiLabel: 12
  heading: 15
  editor: 14
  editorLineHeight: 1.6

spacing:
  unit: 4
  scale: [4, 8, 12, 16, 24, 32]

radius:
  sm: 6
  md: 10
  lg: 14

size:
  titleBarHeight: 56
  activityRailWidth: 56
  statusBarHeight: 30
  editorTabHeight: 40
  controlMinHeight: 32
  hitTarget: 40

motion:
  shortMs: 120
  standardMs: 180
  reducedMotion: 'disable nonessential movement'
```

Ensure WCAG AA contrast for normal text (4.5:1), non-text controls/indicators (3:1 where applicable). Validate all token combinations in implementation; the palette is a starting point, not proof of conformance. Do not communicate errors, success, or learning progress by color alone.

## 4. Component contracts

### `AppShell`
- Owns global layout and resize boundaries.
- States: `theme`, `mode`, `workspace`, `activeView`, `panels`.
- Keyboard: `Ctrl/Cmd+Shift+P` command palette; `Ctrl/Cmd+S` save; `F5` run; `F9` toggle breakpoint; `F10` step over; `F11` step into; prevent conflicts based on host platform.

### `TitleBar`
- Product identity at left, current project in center-left, primary `Executar` and secondary `Depurar` at right.
- `Executar` dropdown chooses Portugol interpreter vs generated Rust where supported. Label execution engine in console.
- While running: visible busy state, cancel/stop control, repeated-run prevention.

### `ActivityRail`
- Navigation: Projeto, Aprender, Exemplos, Depurar, Configurações.
- Active route has shape/indicator as well as color; tooltips for icon-only mode.

### `ExplorerPanel`
- File tree supports keyboard navigation, nesting, rename, create, delete with confirmation, and unsaved indicators.
- Source extension `.por` is a proposal, not a guaranteed existing compatibility format. Preserve native legacy format when importing existing Portugol Studio projects.
- Generated `.rs` files live under a visible `generated/` view or generated artifacts group, never silently mixed with authored files.

### `SourceEditor`
- Syntax highlight, line numbers, brackets, diagnostics, completion, hover help, find/replace, keyboard-first editing.
- Editors support independent scroll, optional synced scroll by semantic mapping, and accessible focus switching.
- The source editor is always editable unless a read-only file is intentionally opened.

### `GeneratedRustEditor`
- Default read-only and marked `Rust gerado`.
- Source changes mark generated text `desatualizado` until regeneration finishes.
- `Desvincular e editar` converts generated text into a student-owned Rust file with confirmation and preserved history; never overwrite manually edited code.
- `Mostrar diferenças` displays changes between prior and newly generated output.

### `MappingGutter`
- Semantic mappings between source and generated code, not naïve line-number equality.
- Hover/click highlights relevant ranges and opens an explanation; show `sem correspondência direta` for runtime helpers or generated boilerplate.
- If translation unavailable or incomplete, show `tradução parcial` and a diagnostic; no misleading green status.

### `BottomPanel`
- Tabs: Terminal / Testes / Saída da Execução / Problemas.
- Terminal distinguishes stdout, stderr, compiler output and test output.
- Show success only if the selected engine really ran successfully; status badges derive from runtime events.
- Problems are searchable and navigable, with error messages translated into Brazilian Portuguese when feasible while retaining original diagnostic metadata.

### `LearningInspector`
- Context-sensitive education card keyed by cursor/selection + learning mode.
- `O que mudou?`, `Por que isso importa?`, `Ver exemplo`, `Praticar` sections.
- Students can hide hints, pin a topic, or request more detail. Never obscure the editor with unsolicited overlays.
- Learning progress must distinguish completed exercises from inferred mastery; no claims of competency based solely on opening a panel.

### `VariablesMemoryPanel`
- At minimum: variable name, type, displayed value, scope, last update.
- Advanced visualization: stack frames, heap objects, owner, moved-from values, immutable/mutable loans where a real trace or verified simulation supports it.
- Show `Simulação didática` label when memory states are pedagogical models rather than runtime observations.

## 5. User journeys

### Journey A: First program

1. Student opens `Meu primeiro algoritmo` in Beginner mode.
2. Explorer opens `inicio.por` and code editor receives focus.
3. Student runs program with F5.
4. Bottom panel shows selected runtime, output, exit status and genuine diagnostics.
5. Inspector highlights the current concept (variable and output) without requiring Rust knowledge.

**Acceptance:** a beginner can complete the path without showing Rust concepts or compiler complexity.

### Journey B: Transition to Rust

1. Student switches to `Ponte para Rust`.
2. Source remains unchanged; generated Rust appears beside it.
3. Student clicks an assignment and sees mapped Rust ranges with explanation of `let mut`.
4. Student edits source. Generated output becomes stale and regenerates safely.
5. Student runs Rust explicitly and sees `cargo` / `rustc` diagnostics if compilation fails.

**Acceptance:** no silent source edits; translated code has provenance; errors never masquerade as success.

### Journey C: Ownership lab

1. Student opens a guided lesson using `String`.
2. Simulator walks through owner creation, move, invalid use.
3. Diagram shows source variables and heap allocation with clearly labeled simulated state.
4. Equivalent Rust compile error is shown alongside a plain-language explanation.
5. Student modifies code to compile (for example, use `&` or clone where appropriate).

**Acceptance:** a moved-from variable cannot be depicted as a valid owner; simulations avoid teaching incorrect ownership behavior.

## 6. State model

```ts
type LearningMode = 'beginner' | 'bridge' | 'rust';
type RuntimeEngine = 'portugol' | 'rust';
type TranslationStatus = 'idle' | 'generating' | 'ready' | 'stale' | 'partial' | 'error';
type RunStatus = 'idle' | 'running' | 'succeeded' | 'failed' | 'cancelled';

type IDEState = {
  workspaceId: string;
  mode: LearningMode;
  activeFileId: string | null;
  secondaryFileId: string | null;
  selectedRuntime: RuntimeEngine;
  translationStatus: TranslationStatus;
  runStatus: RunStatus;
  activeBottomTab: 'terminal' | 'tests' | 'output' | 'problems';
  activeRailView: 'project' | 'learn' | 'examples' | 'debug' | 'settings';
  inspectorOpen: boolean;
  explorerOpen: boolean;
  sourceDirty: boolean;
  generatedDetached: boolean;
};
```

No fake `38 ms`, `1/1 passou`, or `Sem erros` on actual app screens unless backed by real execution results. Empty states should say `Ainda não executado`, `Nenhum teste configurado`, or `Aguardando tradução`.

## 7. Accessibility and usability

- Full keyboard navigation; predictable tab/focus order across file tree, editors, console, inspector.
- Resize grips accessible by keyboard, at least 40px interactive targets where practical.
- Screen-reader labels for icon-only actions, diagnostics with text semantics, not just colored squiggles.
- Optional high-contrast theme, reduced motion, scalable code fonts, responsive panel toggles.
- Distinguish editor content from generated content for screen-reader users.
- Avoid animated teaching interruptions and reward mechanics that distract from work.

## 8. Suggested implementation boundaries

Keep framework choice open until repository inspection. Prefer design tokens and components independent of frontend framework. Suggested packages if the existing project can support a workspace structure:

```text
apps/desktop/                 # existing desktop shell and UI integration
packages/design-tokens/      # semantic palette, spacing, typography, themes
packages/ui-components/      # tabs, tree, buttons, splitters, overlays, panels
packages/editor-adapter/     # language editor engine integration
packages/learning-ui/        # explanation cards, progress, exercises
packages/memory-visualizer/  # validated traces / simulated conceptual states
crates/language-portugol/    # optional: parser / semantic analysis
crates/language-portugol-rs/ # optional: educational dialect
crates/transpiler-rust/      # optional: semantics-aware Rust generation
crates/runtime/              # optional: execution and diagnostic interfaces
```

**Important:** The package layout is a proposed architecture, not permission to rewrite existing workspace infrastructure. Inspect actual repository, framework, project formats, dependencies and parser/runtime semantics before deciding whether to adopt these boundaries.

## 9. Engineering and design acceptance criteria

- [ ] Visually recognizable evolution of Portugol Studio, with reduced density in beginner mode.
- [ ] At 1536×960, split editors + console + learning inspector usable without overlapping content.
- [ ] At 1280×800, core editor and run button remain visible and inspector can collapse.
- [ ] Explorer, console and inspector are collapsible and resizable, including keyboard controls.
- [ ] Core editor is independently keyboard navigable; line numbers and diagnostics are accessible.
- [ ] Generated Rust visibly distinguished, with a stale state and safe detach action.
- [ ] Runtime status reflects actual process outcome; demo mock data never leaks into production.
- [ ] Ownership visualization explicitly separates model simulation and actual debugging evidence.
- [ ] Localized pt-BR terminology is consistent across UI and lessons.
- [ ] New screens implement loading, empty, success, error and offline/unavailable states.
- [ ] Visual snapshots at 1536×960 and 1280×800 for Beginner, Bridge and Rust modes.
- [ ] Component tests for mode switch, editor resize, run state and translation status.

## 10. Implementation order for coding agents

1. **Repository reconnaissance:** inventory current UI technology, application shell, parser/runtime, legacy project format, test/build scripts. Do not replace existing code blindly.
2. **Foundations:** semantic token layer, app shell, keyboard behavior, panel splitters, responsive rules.
3. **Core IDE:** ActivityRail, Explorer, EditorTabs, SourceEditor, BottomPanel and genuine execution statuses.
4. **Learning mode:** contextual explanations, mode switch, progressive panel defaults.
5. **Dual-language:** Rust generated view, provenance, stale status, source mappings and safe code ownership.
6. **Memory lab:** strictly labeled simulations followed by evidence-based runtime integration.
7. **Quality gate:** a11y, screenshot comparisons, regression tests, packaging, performance assessment.

### Agent instructions

- Treat this document as **design specification**, not evidence that any feature currently works.
- Reuse existing technologies and components if adequate; justify any new dependency.
- Produce small reviewable commits and screenshots before/after per milestone.
- Never invent parser capabilities, translation correctness, compiler output or test results.
- Preserve existing Portugol programs and education workflows.
- When encountering a semantic mismatch between Portugol and Rust, create an explicit design/engineering decision record rather than silently changing behavior.

## 11. Open design decisions

- Final product naming and whether `Ferro` is a visible language name or only a course mode.
- Existing Portugol Studio source/project compatibility scope.
- UI framework, windowing shell and editor engine.
- Whether Rust code generation happens incrementally or on explicit request.
- Backend execution sandboxing and compiler/toolchain distribution model.
- How lessons and progress are stored, exported and reset.
- Scope of native debugging versus pedagogical simulation.
