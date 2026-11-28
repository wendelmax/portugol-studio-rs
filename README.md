# Ferro Studio

Ambiente educacional para programar em português, criar apps desktop e jogos 2D
e evoluir para Rust real, construído em Rust
(repositório `portugol-studio-rs`).

O programa reúne editor, console, depurador, designer visual, ajuda, exemplos e
exercícios corrigidos automaticamente com foco em **Ferro (`.fer`)** — um
dialeto didático em português com tipos explícitos, imutabilidade por padrão,
registros, enumerações e tratamento de erros inspirados em Rust — além de suporte
direto a arquivos **Rust (`.rs`)** e projetos Cargo. O código `.fer` pode ser
visualizado lado a lado em Rust e exportado como projeto Cargo, permitindo ao
aluno avançar do pseudocódigo em português para uma linguagem usada
profissionalmente.

## Para quem é

- **Estudantes** que estão começando em lógica de programação e querem escrever
  código em português, ver erros enquanto digitam e acompanhar a execução passo
  a passo.
- **Professores** que buscam uma trilha progressiva de lógica até Rust, com
  ajuda integrada, catálogo de 119+ exemplos `.fer`, janela gráfica/áudio,
  interfaces declarativas (`UI.*`) e exercícios `.licao` com correção
  automática.
- **Quem quer chegar ao Rust**: o Ferro ensina conceitos essenciais do
  Rust (variáveis imutáveis por padrão, `Opcao`, `Resultado`, `escolha`
  exaustivo, operador `?`) e mostra o Rust equivalente com mapeamento de origem.
- **Desenvolvedores** interessados em compiladores, máquinas virtuais, WASM ou
  IDEs educacionais: o núcleo da linguagem está em crates Rust independentes da
  interface.

## Instalação

> **Experimente no navegador sem instalar:** acesse o
> [**Ferro Studio Web Playground (WASM)**](https://wendelmax.github.io/portugol-studio-rs/)
> para validar, formatar e executar programas `.fer` e apps visuais `UI.*`
> diretamente no navegador.

Baixe o instalador da sua plataforma na página de
[Releases](https://github.com/wendelmax/portugol-studio-rs/releases):

| Sistema | Arquivo |
|---|---|
| Windows (x86_64 e ARM64) | `.exe` (instala sem pedir administrador; x64 e ARM64) ou `.msi` (x64) |
| Linux (x86_64 e ARM64) | `.deb`, `.rpm`, `.AppImage` ou `.pkg.tar.zst` (Arch Linux) |
| macOS (Apple Silicon ARM64 e Intel x86_64) | `.dmg` |

No **Arch Linux**, você pode instalar o pacote `.pkg.tar.zst` da página de Releases com `sudo pacman -U portugol-studio-rs-*.pkg.tar.zst` ou compilar localmente a partir do repositório com `bash packaging/arch/build-package.sh`.

Os instaladores ainda não são assinados. No Windows, o SmartScreen pode exibir
"O Windows protegeu o seu computador": clique em **Mais informações** e depois
em **Executar assim mesmo**. No macOS, abra o aplicativo pela primeira vez com
o botão direito e escolha **Abrir**.

Programas `.fer` executam e depuram diretamente na máquina virtual da IDE, sem
precisar instalar nada além do instalador. O Rust ([rustup.rs](https://rustup.rs))
só é necessário para executar arquivos `.rs`, projetos Cargo e a opção
**Executar como Rust** da paleta de comandos, que compila o Rust gerado a
partir de um `.fer`.

## O que a IDE oferece

- Interface desktop proprietária (Tauri v2 + CodeMirror 6) com barra superior de projeto/modo/execução, `ActivityRail`, `Explorer`, `Inspector` (`Aprender`, `Bibliotecas`/`Modelos` PORS, árvore estrutural e variáveis) e diálogo dedicado de `Configurações` (`Ctrl+,`) com restauração de layout.
- Editor com realce de sintaxe para Ferro (`.fer`) e Rust (`.rs`), autocompletar de palavras, funções e módulos PORS, dicas ao passar o mouse, dobramento de código, busca, renomeação de símbolos e formatação automática.
- Erros e avisos em português enquanto você digita, com indicação da linha.
- Console com entrada de dados (`leia`) e parâmetros de execução.
- Depurador com pontos de parada, execução passo a passo, pilha de chamadas e inspeção de variáveis, vetores, registros e enums.
- Janela gráfica para programas que usam `Graficos`, `Teclado` e `Mouse`, reprodução de áudio com `Sons` e prévia visual declarativa para apps `UI.*`.
- Ajuda integrada 100% PORS, catálogo de 119+ exemplos `.fer`, 8 dicas ilustradas da IDE e 14 exercícios `.licao` com correção automática.
- Ponte lado a lado Ferro ↔ Rust com mapeamento de origem (`MappingGutter`), diff de traduções e desvinculação segura para editar `.rs` independente.
- Execução de arquivos Rust avulsos e projetos Cargo (`cargo check` / `cargo test` / `cargo run`).
- Temas padronizados com contraste WCAG AA (`>= 4,5:1`): `Sistema` automático (VS Code Light/Dark Modern), `IntelliJ Rust Dark/Light`, `GitHub`, `One Dark Pro`, `Dracula`, `Catppuccin Mocha`, `Night Owl`, `Alto contraste` e editor de temas personalizados.

### Atalhos do depurador

Clique na margem esquerda do editor para criar ou remover um ponto de parada.

| Ação | Atalho |
|---|---|
| Depurar começando pausado | `Shift+F5` |
| Executar (para nos pontos de parada) | `Shift+F6` ou `Ctrl+Enter` |
| Pausar | `F9` |
| Continuar até o próximo ponto de parada | `F5` |
| Passo por cima (não entra em funções) | `Shift+F9` ou `F10` |
| Entrar na função chamada | `Alt+F11` |
| Voltar para quem chamou | `Shift+F11` |
| Parar | `Shift+F7` |
| Ir à declaração (no editor) | `F12` |
| Ajuda | `F1` |
| Lista de atalhos | `F11` |

## Ferro (`.fer`) e Rust (`.rs`)

O **Ferro** é o dialeto autoral deste projeto para ensinar lógica de
programação com palavras-chave em português e transição natural para Rust:

- variáveis com `seja`, imutáveis por padrão (`seja mutavel` para alterar);
- tipos explícitos: `inteiro`, `real`, `logico`, `cadeia`, `caracter`,
  `vetor<T>`;
- funções com parâmetros e retorno tipados, incluindo recursão;
- `registro` e `enum` com `escolha` exaustivo;
- `Opcao<T>` e `Resultado<T, E>` com o operador `?`;
- vetores dinâmicos (`adicionar`, `remover`, `obter`, `tamanho`);
- bibliotecas padrão e multimídia PORS: `Matematica`, `Texto`, `Tipos`, `Util`,
  `Calendario`, `Arquivos`, `Json`, `Internet`, `UI`, `Graficos`, `Teclado`,
  `Mouse` e `Sons`;
- interfaces visuais declarativas com `UI.*` no modelo
  `iniciar`/`atualizar`/`visualizar`.

```
funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    se (b == 0) {
        retorne Resultado::Erro("divisão por zero");
    }
    retorne Resultado::Ok(a / b);
}

funcao inicio() {
    seja nome = leia();
    seja a: inteiro = leia();
    seja b: inteiro = leia();
    escolha dividir(a, b) {
        caso Resultado::Ok(v) => {
            escreval("{nome}: resultado = ", v);
        }
        caso Resultado::Erro(e) => {
            escreval("{nome}: erro: {e}");
        }
    }
}
```

Programas `.fer` podem ser exportados como um arquivo Rust ou como um projeto
Cargo completo. Os exemplos comentados ficam em
[`crates/ferro-service/assets/exemplos-pors`](crates/ferro-service/assets/exemplos-pors)
e aparecem no menu de exemplos da IDE. Detalhes sobre a exportação e a execução
de Rust estão em [`docs/PORTUGOL-RUST.md`](docs/PORTUGOL-RUST.md).

## Linha de comando

O executável `portugol` valida, formata, executa, depura e exporta programas
`.fer` sem abrir a IDE. Com o repositório clonado:

```bash
cargo run -p ferro-cli -- programa.fer                       # valida (também aceita pastas)
cargo run -p ferro-cli -- executar programa.fer              # executa na máquina virtual
cargo run -p ferro-cli -- depurar programa.fer               # depurador interativo no terminal
cargo run -p ferro-cli -- novo exercicio.fer                 # cria um arquivo inicial
cargo run -p ferro-cli -- formatar programa.fer              # formata (--check só confere)
cargo run -p ferro-cli -- exportar-rust programa.fer         # imprime o Rust gerado
cargo run -p ferro-cli -- exportar-cargo programa.fer pasta  # gera um projeto Cargo
cargo run -p ferro-cli -- novo --desktop "Meu App"            # cria um projeto desktop Ferro
cargo run -p ferro-cli -- "Meu App"                           # valida .fer, .design e .ferproj
```

Cada projeto desktop Ferro tem um manifesto `.ferproj` e telas descritas por
pares de arquivos: `Nome.design` (o desenho em pseudo-XAML) e `Nome.fer` (o
code-behind). `formatar` também aceita `.design` e `.ferproj`.

`executar --bytecode` mostra as instruções geradas para a máquina virtual,
útil para quem estuda compiladores. `--quiet` lista apenas os arquivos com erro.

## Para desenvolvedores

### Organização

| Caminho | Conteúdo |
|---|---|
| `crates/ferro-hir` | representação intermediária tipada, spans e diagnósticos compartilhados |
| `crates/ferro-frontend` | lexer, parser, formatador e verificação de tipos do Ferro (`.fer`) |
| `crates/ferro-bytecode` | conjunto de instruções e compilador HIR → bytecode |
| `crates/ferro-vm` | máquina virtual, bibliotecas padrão/multimídia, protocolo gráfico, UI MVU e depurador |
| `crates/ferro-rust` | geração de código Rust, projetos Cargo e source maps |
| `crates/ferro-wasm` | núcleo do Ferro compilado para WebAssembly (`wasm32-unknown-unknown`) |
| `crates/ferro-service` | camada usada pela IDE: verificar, executar, depurar, exemplos e exercícios `.licao` |
| `crates/ferro-tela` | formato `.design` das telas desktop (pseudo-XAML) e manifesto `.ferproj`: parser, esquema, validação, formatação e modelo de projeto |
| `crates/ferro-cli` | executável `portugol` |
| `apps/desktop` | IDE em Tauri v2 com frontend TypeScript e CodeMirror 6 |
| `apps/web-playground` | playground web estático sem dependências usando `ferro-wasm` (`scripts/build-web-playground.sh`) |
| `docs` | arquitetura, design e distribuição |

A IDE usa Tauri porque o produto depende de um editor de código completo
(seleção, desfazer, busca, autocompletar, entrada de acentos). Com a webview, o
CodeMirror resolve essa parte, e toda a lógica da linguagem fica em Rust.

### Ambiente

As versões de Rust e Node estão fixadas em [`mise.toml`](mise.toml):

```bash
mise install
mise exec -- cargo test
```

Para compilar a IDE no Linux, instale também o WebKitGTK 4.1 e o ALSA. No
Ubuntu:

```bash
sudo apt install libwebkit2gtk-4.1-dev libasound2-dev libayatana-appindicator3-dev librsvg2-dev
```

### Compilar e testar

```bash
cargo test                                   # núcleo, CLI e WebAssembly
cargo clippy --all-targets -- -D warnings
cargo check --target wasm32-unknown-unknown -p ferro-wasm

cd apps/desktop
npm ci
npm test                                     # testes do frontend (unitários, contrato e contraste AA)
npm run build                                # compila TypeScript + bundle Vite
npm run test:e2e                             # verificação E2E/visual em 1536×960 e 1280×800
npm run tauri dev                            # abre a IDE em modo de desenvolvimento
npm run tauri build                          # gera os instaladores locais
```

A IDE fica fora dos membros padrão do workspace porque depende das bibliotecas
gráficas do sistema; use `cargo build -p ferro-desktop` para compilá-la
diretamente.

### Documentação

- [`CONTRIBUTING.md`](CONTRIBUTING.md): como escolher e reservar uma issue,
  trabalhar em paralelo sem conflito e entregar o PR.
- [`docs/ARQUITETURA.md`](docs/ARQUITETURA.md): estrutura do compilador, da VM
  e da IDE.
- [`docs/PORTUGOL-RUST.md`](docs/PORTUGOL-RUST.md): exportação para Rust,
  execução de `.rs` e projetos Cargo, limites de execução.
- [`docs/DISTRIBUICAO.md`](docs/DISTRIBUICAO.md): como gerar e assinar os
  instaladores.

## Segurança

Programas `.fer` rodam na máquina virtual da IDE, com limites de tempo e de
memória; `Arquivos.*` fica restrito à pasta do programa e `Internet.*` vem
desligado por padrão. Já os arquivos `.rs`, os projetos Cargo e a opção
**Executar como Rust** são compilados para código nativo e rodam com as
permissões do seu usuário, sem isolamento. A IDE pede confirmação antes da
primeira execução; rode apenas código em que você confia.

## Licença

O Ferro Studio é distribuído sob a licença LGPL-3.0 (`LGPL-3.0-only`); o texto
completo está em [`LICENSE`](LICENSE).
