# Portugol Studio RS

Ambiente para aprender programação em português, construído em Rust a partir da
inspiração do [Portugol Studio](https://github.com/UNIVALI-LITE/Portugol-Studio)
da UNIVALI.

O programa reúne editor, console, depurador, ajuda, exemplos e exercícios
corrigidos automaticamente com foco em **Portugol-RS (`.pors`)** — um dialeto
didático em português com tipos explícitos, imutabilidade por padrão, registros,
enumerações e tratamento de erros inspirados em Rust — além de suporte direto a
arquivos **Rust (`.rs`)** e projetos Cargo. O código `.pors` pode ser visualizado
lado a lado em Rust e exportado como projeto Cargo, permitindo ao aluno avançar
do pseudocódigo em português para uma linguagem usada profissionalmente.

> **Arquivos `.por` e `.pex` legados:** conforme o
> [ADR 0016](docs/adr/0016-pors-como-linguagem-autoral-e-retirada-do-por.md), o
> suporte ao Portugol clássico (`.por`/`.pex`) foi retirado. O Portugol Studio
> RS não abre nem converte arquivos `.por`/`.pex` automaticamente; para projetos
> e exercícios nesse formato legado, utilize o
> [Portugol Studio original](https://github.com/UNIVALI-LITE/Portugol-Studio).

## Para quem é

- **Estudantes** que estão começando em lógica de programação e querem escrever
  código em português, ver erros enquanto digitam e acompanhar a execução passo
  a passo.
- **Professores** que buscam uma trilha progressiva de lógica até Rust, com
  ajuda integrada, catálogo de 119+ exemplos `.pors`, janela gráfica/áudio,
  interfaces declarativas (`UI.*`) e exercícios `.pors.json` com correção
  automática.
- **Quem quer chegar ao Rust**: o Portugol-RS ensina conceitos essenciais do
  Rust (variáveis imutáveis por padrão, `Opcao`, `Resultado`, `escolha`
  exaustivo, operador `?`) e mostra o Rust equivalente com mapeamento de origem.
- **Desenvolvedores** interessados em compiladores, máquinas virtuais, WASM ou
  IDEs educacionais: o núcleo da linguagem está em crates Rust independentes da
  interface.

## Instalação

> **Experimente no navegador sem instalar:** acesse o
> [**Portugol-RS Web Playground (WASM)**](https://wendelmax.github.io/portugol-studio-rs/)
> para validar, formatar e executar programas `.pors` e apps visuais `UI.*`
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

Programas `.pors` executam e depuram diretamente na máquina virtual da IDE, sem
precisar instalar nada além do instalador. O Rust ([rustup.rs](https://rustup.rs))
só é necessário para executar arquivos `.rs`, projetos Cargo e a opção
**Executar como Rust** da paleta de comandos, que compila o Rust gerado a
partir de um `.pors`.

## O que a IDE oferece

- Interface desktop proprietária (Tauri v2 + CodeMirror 6) com barra superior de projeto/modo/execução, `ActivityRail`, `Explorer`, `Inspector` (`Aprender`, `Bibliotecas`/`Modelos` PORS, árvore estrutural e variáveis) e diálogo dedicado de `Configurações` (`Ctrl+,`) com restauração de layout.
- Editor com realce de sintaxe para Portugol-RS (`.pors`) e Rust (`.rs`), autocompletar de palavras, funções e módulos PORS, dicas ao passar o mouse, dobramento de código, busca, renomeação de símbolos e formatação automática.
- Erros e avisos em português enquanto você digita, com indicação da linha.
- Console com entrada de dados (`leia`) e parâmetros de execução.
- Depurador com pontos de parada, execução passo a passo, pilha de chamadas e inspeção de variáveis, vetores, registros e enums.
- Janela gráfica para programas que usam `Graficos`, `Teclado` e `Mouse`, reprodução de áudio com `Sons` e prévia visual declarativa para apps `UI.*`.
- Ajuda integrada 100% PORS, catálogo de 119+ exemplos `.pors`, 8 dicas ilustradas da IDE e 14 exercícios `.pors.json` com correção automática.
- Ponte lado a lado Portugol-RS ↔ Rust com mapeamento de origem (`MappingGutter`), diff de traduções e desvinculação segura para editar `.rs` independente.
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

## Portugol-RS (`.pors`) e Rust (`.rs`)

O **Portugol-RS** é o dialeto autoral deste projeto para ensinar lógica de
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

Programas `.pors` podem ser exportados como um arquivo Rust ou como um projeto
Cargo completo. Os exemplos comentados ficam em
[`crates/portugol-service/assets/exemplos-pors`](crates/portugol-service/assets/exemplos-pors)
e aparecem no menu de exemplos da IDE. Detalhes sobre a exportação e a execução
de Rust estão em [`docs/PORTUGOL-RUST.md`](docs/PORTUGOL-RUST.md).

## Linha de comando

O executável `portugol` valida, formata, executa, depura e exporta programas
`.pors` sem abrir a IDE. Com o repositório clonado:

```bash
cargo run -p portugol-cli -- programa.pors                       # valida (também aceita pastas)
cargo run -p portugol-cli -- executar programa.pors              # executa na máquina virtual
cargo run -p portugol-cli -- depurar programa.pors               # depurador interativo no terminal
cargo run -p portugol-cli -- novo exercicio.pors                 # cria um arquivo inicial
cargo run -p portugol-cli -- formatar programa.pors              # formata (--check só confere)
cargo run -p portugol-cli -- exportar-rust programa.pors         # imprime o Rust gerado
cargo run -p portugol-cli -- exportar-cargo programa.pors pasta  # gera um projeto Cargo
```

`executar --bytecode` mostra as instruções geradas para a máquina virtual,
útil para quem estuda compiladores. `--quiet` lista apenas os arquivos com erro.

## Para desenvolvedores

### Organização

| Caminho | Conteúdo |
|---|---|
| `crates/portugol-hir` | representação intermediária tipada, spans e diagnósticos compartilhados |
| `crates/portugol-rs-frontend` | lexer, parser, formatador e verificação de tipos do Portugol-RS (`.pors`) |
| `crates/portugol-bytecode` | conjunto de instruções e compilador HIR → bytecode |
| `crates/portugol-vm` | máquina virtual, bibliotecas padrão/multimídia, protocolo gráfico, UI MVU e depurador |
| `crates/portugol-rust` | geração de código Rust, projetos Cargo e source maps |
| `crates/portugol-wasm` | núcleo do Portugol-RS compilado para WebAssembly (`wasm32-unknown-unknown`) |
| `crates/portugol-service` | camada usada pela IDE: verificar, executar, depurar, exemplos e exercícios `.pors.json` |
| `crates/portugol-cli` | executável `portugol` |
| `apps/desktop` | IDE em Tauri v2 com frontend TypeScript e CodeMirror 6 |
| `apps/web-playground` | playground web estático sem dependências usando `portugol-wasm` (`scripts/build-web-playground.sh`) |
| `docs` | arquitetura, decisões técnicas (`docs/adr`) e distribuição |

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
cargo check --target wasm32-unknown-unknown -p portugol-wasm

cd apps/desktop
npm ci
npm test                                     # testes do frontend (unitários, contrato e contraste AA)
npm run build                                # compila TypeScript + bundle Vite
npm run test:e2e                             # verificação E2E/visual em 1536×960 e 1280×800
npm run tauri dev                            # abre a IDE em modo de desenvolvimento
npm run tauri build                          # gera os instaladores locais
```

A IDE fica fora dos membros padrão do workspace porque depende das bibliotecas
gráficas do sistema; use `cargo build -p portugol-desktop` para compilá-la
diretamente.

### Documentação

- [`CONTRIBUTING.md`](CONTRIBUTING.md): como escolher e reservar uma issue,
  trabalhar em paralelo sem conflito e entregar o PR.
- [`docs/ARQUITETURA.md`](docs/ARQUITETURA.md): estrutura do compilador, da VM
  e da IDE.
- [`docs/PARIDADE.md`](docs/PARIDADE.md): inventário histórico de comparação
  com o Portugol Studio original.
- [`docs/PORTUGOL-RUST.md`](docs/PORTUGOL-RUST.md): exportação para Rust,
  execução de `.rs` e projetos Cargo, limites de execução.
- [`docs/DISTRIBUICAO.md`](docs/DISTRIBUICAO.md): como gerar e assinar os
  instaladores.
- [`docs/adr`](docs/adr): registro das decisões de arquitetura.

## Segurança

Programas `.pors` rodam na máquina virtual da IDE, com limites de tempo e de
memória; `Arquivos.*` fica restrito à pasta do programa e `Internet.*` vem
desligado por padrão. Já os arquivos `.rs`, os projetos Cargo e a opção
**Executar como Rust** são compilados para código nativo e rodam com as
permissões do seu usuário, sem isolamento. A IDE pede confirmação antes da
primeira execução; rode apenas código em que você confia.

## Licença

O Portugol Studio original é distribuído sob a LGPL-3.0. Este projeto é uma obra
derivada e usa a mesma licença (`LGPL-3.0-only`); o texto completo está em
[`LICENSE`](LICENSE).
