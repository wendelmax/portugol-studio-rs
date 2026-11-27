# ADR 0023: Identidade visual e marca Ferro Studio

- **Status:** Aceita
- **Data:** 2026-11-27
- **Responsável:** mantenedor do produto
- **Emendas:** Emenda 1 (2026-11-27)
- **Relacionados:** [ADR 0015](0015-evolucao-visual-orientada-a-aprendizagem.md), [ADR 0016](0016-pors-como-linguagem-autoral-e-retirada-do-por.md), [ADR 0018](0018-projetos-desktop-pors-com-designer-visual.md), [ADR 0021](0021-tela-como-forma-unica-de-interface-pors.md), [ADR 0022](0022-projetos-de-jogos-2d-pors-e-exportacao-macroquad.md)

> **Emenda 1 (2026-11-27).** Revisa a seção 2 e a Fase 2 da seção 4: na
> transição completa da Fase 2, a linguagem e o conjunto oficial de extensões
> também adotam a identidade **Ferro** (`.fer`, `.design`, `.ferproj`, `.licao`,
> `.rs` e, futuramente, `.ferws`), usando anotações `@teste` no código em vez
> de uma extensão separada e definindo `.licao` como equivalente oficial ao
> `.pex` clássico (mantendo a retirada de `.por`/`.pex` do ADR 0016).

## Contexto

Com a sequência dos ADRs 0016 a 0022, o projeto distanciou-se definitivamente
do Portugol Studio clássico:

1. não abre nem converte arquivos legados `.por`/`.pex` (ADR 0016);
2. possui linguagem autoral própria em português (`.pors`) com tipos
   explícitos, `Opcao`, `Resultado` e evolução sintática própria (ADRs 0019 e
   0020);
3. oferece designer visual e projetos desktop com `.design` + code-behind
   `.pors` exportáveis para **Slint** (ADRs 0018 e 0021);
4. define projetos estruturados de jogos 2D exportáveis para **Macroquad**
   (ADR 0022), além da ponte progressiva para Rust real (`std`) e do Web
   Playground em WebAssembly.

Manter o nome **"Portugol Studio RS"** passou a gerar três problemas de
produto:

- **Expectativa incorreta:** quem conhece o Portugol Studio da UNIVALI espera
  compatibilidade com `.por`/`.pex`;
- **Subestimação do produto:** soa como uma simples reescrita em Rust, e não
  como uma IDE autoral para ensinar lógica, criar aplicativos e jogos em
  português e evoluir para Rust;
- **Ambiguidade de "RS":** no Brasil, "RS" remete simultaneamente a Rio Grande
  do Sul e a Rust;
- **Identidade visual desalinhada:** o ícone atual da IDE traz um quadrado azul
  com `RS`, enquanto o Web Playground traz um `P` roxo, sem unidade visual.

## Decisão

### 1. Marca oficial: Ferro Studio

O produto passa a adotar o nome oficial **Ferro Studio**, com a assinatura de
posicionamento:

> **Ferro Studio** — *Programe em português. Crie apps e jogos. Evolua para Rust.*

"Ferro" estabelece a metáfora direta com **Rust** (*ferrugem*): representa a
base sólida e estruturada em português antes da transição para Rust real,
evitando tanto a conotação negativa de "enferrujado/velho" quanto colisões com
projetos humorísticos ou buscas não relacionadas.

### 2. Extensões oficiais do Ferro Studio e estabilidade de crates internos *(Emenda 1)*

Durante a conclusão das entregas do designer `.design` (Fase 1), as extensões
atuais (`.pors`, `.porsproj`, `.pors.json`) permanecem intocadas. Na **Fase 2**,
com janela de compatibilidade de leitura/aviso para arquivos `.pors` existentes,
o Ferro Studio adota seu conjunto oficial de extensões:

| Extensão | Finalidade | Exemplo | Observações |
|---|---|---|---|
| **`.fer`** | Código-fonte **Ferro** (incluindo testes com `@teste`) | `principal.fer` | Evita `.fe` por colisão com a linguagem *Fe* (`fe-lang`). Testes unitários usam anotação `@teste` dentro do `.fer` (mapeando 1:1 para `#[test]` em Rust), sem criar extensão `.teste` separada. |
| **`.design`** | Layout visual de telas desktop (ADRs 0018 e 0021) | `JanelaPrincipal.design` | Mantida sem alteração, formando par com `.fer`. |
| **`.ferproj`** | Manifesto de projeto (`Tipo="Desktop"` ou `Tipo="Jogo"`) | `escola.ferproj` | Evita colisão no SO com `.projeto` genérico; usa a mesma marcação pseudo-XAML do `.porsproj`. |
| **`.licao`** | Aula / exercício interativo com correção automática | `variaveis.licao` | **Equivalente oficial ao `.pex` clássico** (substituindo `.pors.json`), em texto versionável no Git com enunciado, código inicial `.fer`, dicas progressivas, casos `entrada → saída`, testes `@teste` e, futuramente, validação de eventos em `.design`. |
| **`.rs`** | Código-fonte Rust real | `main.rs` | Ponte direta para Rust / Cargo. |
| *(Futuro)* **`.ferws`** | Workspace multi-projeto (turma) | `minha-turma.ferws` | Introduzida quando a IDE suportar múltiplos projetos abertos na mesma janela. |

- **Adoção antecipada de `.ferproj` (2026-11-27):** o manifesto `.porsproj`
  nunca saiu numa versão lançada (não está na `v0.7.0`), então foi trocado
  por `.ferproj` sem janela de transição. `.pors` e `.pors.json` continuam
  com a transição descrita acima.
- **Retirada de `.por`/`.pex` mantida (ADR 0016):** o Ferro Studio não
  reintroduz o compilador clássico `.por` nem pacotes legados `.pex`.
- **Crates internos:** os nomes dos crates internos (`crates/portugol-*`)
  permanecem estáveis durante as entregas dos ADRs 0018–0022.

### 3. Identidade visual oficial (`Fe` `26`)

Fica adotada uma identidade única para IDE Desktop, Web Playground,
instaladores e documentação:

- **Ícone oficial (`Fe` `26`):** *squircle* em grafite profundo (`#0F172A` →
  `#1E293B`) com borda metálica sutil, monograma **`Fe`** centralizado em
  cobre forjado (`#FB923C` → `#E05A2B` → `#C2410C`) e o número atômico **`26`**
  em azul-aço (`#38BDF8`) no canto superior direito — unindo o símbolo químico
  do ferro à estética moderna de ferramentas de engenharia.
- **Vetorização pura:** a matriz `apps/desktop/src-tauri/icons/icon.svg` é
  armazenada exclusivamente com `<path>` geométricos (sem elementos `<text>`),
  garantindo renderização idêntica em Linux, Windows, macOS e Web sem depender
  de fontes instaladas.
- **Paleta oficial da marca:**
  - **Cobre Forjado (primária / ação):** `#E05A2B` (hover `#C2410C`, destaque
    claro `#FB923C`)
  - **Azul Aço (secundária / foco / interatividade):** `#0284C7` (modo escuro
    `#38BDF8`)
  - **Grafite Profundo (superfícies escuras):** `#0B1220` / `#131C2E` /
    `#27354D`
  - **Papel Frio (superfícies claras):** `#F8FAFC` / `#FFFFFF` / `#0F172A`
  - Todo contraste entre texto e fundo continua respeitando **WCAG AA** (`>= 4,5:1`).
- **Tipografia recomendada:** `Inter` ou `Plus Jakarta Sans` para interface e
  títulos; `JetBrains Mono` para editor e blocos de código.

### 4. Adoção em duas fases

Para não gerar conflitos com as entregas paralelas do designer `.design` (ADRs
0018 e 0021):

- **Fase 1 (imediata — este ADR):** publicação desta decisão e atualização dos
  ícones oficiais (`icon.svg`, PNGs, `icon.ico` e `icon.icns`) em
  `apps/desktop/src-tauri/icons/`.
- **Fase 2 (após conclusão do fluxo `.design`):**
  - adoção das extensões oficiais `.fer`, `.ferproj` e `.licao` (com leitura de
    transição para `.pors`, `.porsproj` e `.pors.json`), suporte à anotação
    `@teste`, e atualização das associações MIME/desktop;
  - atualização de `productName`, títulos visíveis na IDE Desktop, Ajuda, Sobre
    e redesenho do Web Playground (alinhado ao ADR 0021);
  - renomeação do repositório GitHub para **`wendelmax/ferro-studio`** (e
    GitHub Pages para `https://wendelmax.github.io/ferro-studio/`), atualizando
    em conjunto:
    - **Runtime da IDE:** `apps/desktop/src-tauri/src/updates.rs`,
      `alerts.rs`, `lib.rs`, `apps/desktop/src/shell/project-dialogs.ts` e
      `apps/desktop/index.html`;
    - **Build, release e empacotamento:** `Cargo.toml`, `scripts/release.sh`,
      `.github/workflows/release.yml`, `packaging/arch/PKGBUILD`,
      `packaging/arch/build-package.sh` e arquivo `.desktop`;
    - **Web Playground e documentação:** `apps/web-playground/index.html`,
      `apps/web-playground/README.md`, `README.md`, `CONTRIBUTING.md`,
      `docs/DISTRIBUICAO.md` e `docs/UX.md`;
  - planejamento da transição de identificadores/pacotes (`br.ferro.studio` e
    pacote Arch `ferro-studio` com `replaces`/`conflicts`), mantendo o slug
    antigo `portugol-studio-rs` livre na conta do GitHub para preservar os
    redirecionamentos automáticos de clones, links e verificação de
    atualizações em instalações anteriores (como a `v0.7.0`).

## Alternativas consideradas

### Manter "Portugol Studio RS"
Rejeitada pelos motivos do contexto: gera expectativa de compatibilidade com
`.por`/`.pex`, ambiguidade de `RS` e imagem de clone em vez de produto próprio.

### "Ferrugem" / "Ferrugem Studio"
Rejeitada. Embora traduza literalmente *Rust*, o termo pode soar pejorativo
para iniciantes e coordenadores de curso, além de colidir com buscas populares e
repositórios humorísticos no GitHub.

### "Forja Studio" ou "Brasa Studio"
Boas alternativas semânticas, porém menos imediatas na associação com *Rust* e
sem o gancho visual do símbolo **`Fe 26`** que **Ferro Studio** proporciona.

### Nota sobre "FerroUI / FerroDesign"
Existe um framework XAML experimental em Rust chamado `FerroUI` /
`FerroDesign` (citado no ADR 0018). O nome **Ferro Studio** não conflita com
nenhuma IDE em produção e é direcionado ao público educacional em língua
portuguesa; qualquer referência técnica ao `FerroUI` em documentação interna
continua distinguindo os dois projetos claramente.

## Consequências

- Os ícones da IDE em `apps/desktop/src-tauri/icons/` passam a refletir a nova
  marca **Ferro Studio (`Fe 26`)** já na próxima compilação/release.
- A Fase 2 (mudança de nome visível na interface, renomeação do repositório
  para `wendelmax/ferro-studio`, Web Playground, `README.md` e empacotamento)
  será executada em PRs próprios após o fechamento das fases principais do
  `.design` (ADR 0018/0021).
- Ao renomear o repositório no GitHub, o slug `portugol-studio-rs` deve
  permanecer sem novo repositório na mesma conta para que os redirecionamentos
  automáticos do GitHub continuem válidos para usuários existentes.
