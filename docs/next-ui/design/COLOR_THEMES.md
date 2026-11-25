# Paletas Padronizadas de Cores — Portugol Studio RS

Este documento consolida o sistema de temas do **Portugol Studio RS**, combinando a proposta visual do [`DESIGN_AS_CODE.md`](DESIGN_AS_CODE.md) com referências consagradas de **IntelliJ Rust (RustRover)** e os temas claro e escuro mais utilizados do **VS Code**.

## Regras de Arquitetura e Acessibilidade

1. **Tokens semânticos únicos:** todas as cores literais vivem exclusivamente em `apps/desktop/src/styles/tokens.css` (bases clara/escura e alto contraste do sistema) e `apps/desktop/src/theme-presets.ts` (catálogo de presets).
2. **Contraste WCAG AA obrigatório (>= 4,5:1):** verificado automaticamente por `apps/desktop/tests/theme-presets.test.mjs` em todos os pares de texto, superfícies, abas, console, destaque, erro, linha atual e seleção de código.
3. **Sem dependência apenas de cor:** diagnósticos, foco, seleção e estado de execução combinam ícones, rótulos textuais e bordas.

## Catálogo Padronizado de Temas

| Grupo no seletor | ID estável | Nome exibido | Base | Referência de cores |
|---|---|---|---|---|
| **Portugol Studio RS** | `preset:portugol-studio-next` *(padrão)* | Portugol Studio Next | `dark` | Proposta `DESIGN_AS_CODE.md` (`#0c1118` / `#111923` / `#34d399`) |
| **Portugol Studio RS** | `system`, `light`, `dark` | Sistema / Claro padrão / Escuro padrão | `light`/`dark` | Bases nativas em `tokens.css` |
| **IntelliJ / RustRover** | `preset:intellij-rust-dark` | IntelliJ Rust Dark | `dark` | JetBrains Islands Dark / Darcula Rust (`#1e1f22`, `#2b2d30`, palavras-chave laranja `#e09f7d`, strings verde `#82c78a`, funções azul `#7cbcff`, tipos roxo `#d896d0`) |
| **IntelliJ / RustRover** | `preset:intellij-rust-light` | IntelliJ Rust Light | `light` | JetBrains Light / RustRover Light (`#ffffff`, `#f7f8fa`, palavras-chave azul-marinho `#0033b3`, strings verde `#066c14`, funções petróleo `#005a70`, tipos roxo `#7c1089`) |
| **VS Code — Escuros** | `preset:vscode-dark-modern` | VS Code Dark Modern | `dark` | VS Code Dark Modern / Dark+ (`#1f1f1f`, `#181818`, `#75b7f0`, `#6ad9c3`, `#dcdcaa`, `#e0a58d`) |
| **VS Code — Escuros** | `preset:github-dark` | GitHub Dark | `dark` | GitHub Dark Default (`#0d1117`, `#161b22`, `#ff7b72`, `#79c0ff`, `#d2a8ff`) |
| **VS Code — Escuros** | `preset:one-dark-pro` | One Dark Pro | `dark` | Atom / VS Code One Dark Pro (`#282c34`, `#21252b`, `#61afef`, `#98c379`) |
| **VS Code — Escuros** | `preset:dracula` | Dracula | `dark` | Dracula Official (`#282a36`, `#21222c`, `#bd93f9`, `#50fa7b`, `#f1fa8c`) |
| **VS Code — Escuros** | `preset:catppuccin-mocha` | Catppuccin Mocha | `dark` | Catppuccin Mocha (`#1e1e2e`, `#181825`, `#cba6f7`, `#a6e3a1`, `#89dceb`) |
| **VS Code — Escuros** | `preset:night-owl` | Night Owl | `dark` | Night Owl (`#011627`, `#0b253a`, `#82aaff`, `#c792ea`, `#ecc48d`) |
| **VS Code — Claros** | `preset:vscode-light-modern` | VS Code Light Modern | `light` | VS Code Light Modern / Light+ (`#ffffff`, `#f8f8f8`, `#005fb8`, `#0000df`, `#126378`, `#a31515`) |
| **VS Code — Claros** | `preset:github-light` | GitHub Light | `light` | GitHub Light Default (`#ffffff`, `#f6f8fa`, `#0969da`, `#cf222e`, `#0550ae`) |
| **Acessibilidade** | `preset:high-contrast` | Alto contraste | `dark` | Fundo preto puro `#000000`, bordas `#ffffff` e destaque `#ffff00` |
