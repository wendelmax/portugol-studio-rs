# Portugol Studio RS

Ambiente para aprender programação em português, reescrito em Rust a partir do
[Portugol Studio](https://github.com/UNIVALI-LITE/Portugol-Studio) da UNIVALI.

O programa reúne editor, console, depurador, ajuda, exemplos e exercícios
corrigidos automaticamente. Além do Portugol clássico (`.por`), traz o
**Portugol-RS** (`.pors`), um dialeto didático com tipos explícitos, registros,
enumerações e tratamento de erros inspirados em Rust. O código `.pors` pode ser
exportado como Rust, o que permite ao aluno passar do pseudocódigo em português
para uma linguagem usada profissionalmente.

## Para quem é

- **Estudantes** que estão começando em lógica de programação e querem escrever
  código em português, ver os erros enquanto digitam e acompanhar a execução
  passo a passo.
- **Professores** que já usam o Portugol Studio em aula: os programas `.por`, os
  exemplos e as bibliotecas do original continuam funcionando, e a IDE inclui
  exercícios com testes automáticos.
- **Quem quer chegar ao Rust**: o Portugol-RS usa conceitos do Rust (variáveis
  imutáveis por padrão, `Opcao`, `Resultado`, `escolha` exaustivo) e mostra o
  Rust equivalente ao programa escrito.
- **Desenvolvedores** interessados em compiladores, máquinas virtuais ou IDEs
  educacionais: o núcleo da linguagem está em crates Rust independentes da
  interface.

## Instalação

Baixe o instalador da sua plataforma na página de
[Releases](https://github.com/wendelmax/Portugol-Studio/releases):

| Sistema | Arquivo |
|---|---|
| Windows x64 | `.exe` (instala sem pedir administrador) ou `.msi` |
| Linux x64 | `.deb`, `.rpm` ou `.AppImage` |
| macOS (Apple Silicon e Intel) | `.dmg` |

Os instaladores ainda não são assinados. No Windows, o SmartScreen pode exibir
"O Windows protegeu o seu computador": clique em **Mais informações** e depois
em **Executar assim mesmo**. No macOS, abra o aplicativo pela primeira vez com
o botão direito e escolha **Abrir**.

Programas `.por` não precisam de nada além do instalador. Para **executar**
arquivos `.pors` e `.rs` na IDE é preciso ter o Rust instalado
([rustup.rs](https://rustup.rs)); sem ele, a depuração de `.pors` continua
disponível.

## O que a IDE oferece

- Editor com realce de sintaxe, autocompletar de palavras, funções e
  bibliotecas, dicas ao passar o mouse, dobramento de código, busca e
  formatação automática.
- Erros e avisos em português enquanto você digita, com indicação da linha.
- Console com entrada de dados (`leia`) e parâmetros de execução.
- Depurador com pontos de parada, execução passo a passo, pilha de chamadas e
  inspeção de variáveis, vetores e matrizes.
- Janela gráfica para programas que usam `Graficos`, `Teclado` e `Mouse`, e
  reprodução de áudio com `Sons`.
- Ajuda integrada sobre a linguagem e as bibliotecas, exemplos prontos (jogos,
  algoritmos, bibliotecas) e exercícios com casos de teste.
- Prévia do código Rust equivalente ao programa, com opção de copiar ou salvar
  o `.rs`.
- Execução de arquivos Rust avulsos e de projetos Cargo, para quem já está
  praticando Rust.
- Temas claro e escuro.

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

## As duas linguagens

### Portugol clássico (`.por`)

É a linguagem do Portugol Studio original, com a mesma sintaxe, as mesmas
mensagens de erro e as mesmas bibliotecas: Matematica, Texto, Tipos, Util,
Calendario, Arquivos, Objetos, Internet, ServicosWeb, Graficos, Teclado, Mouse e
Sons. Os 196 exemplos do original fazem parte dos testes automatizados, que
conferem a saída de cada um contra o resultado esperado.

```
programa
{
    funcao inicio()
    {
        cadeia nome
        escreva("Qual é o seu nome? ")
        leia(nome)
        escreva("Olá, ", nome, "!")
    }
}
```

Arquivos antigos em ISO-8859-1 (padrão dos exemplos do Portugol Studio) são
abertos corretamente, assim como arquivos em UTF-8.

### Portugol-RS (`.pors`)

Dialeto criado neste projeto para servir de ponte entre o Portugol e o Rust.
Mantém as palavras-chave em português e adota a estrutura do Rust:

- variáveis com `seja`, imutáveis por padrão (`seja mutavel` para alterar);
- tipos explícitos: `inteiro`, `real`, `logico`, `cadeia`, `caracter`,
  `vetor<T>`;
- funções com parâmetros e retorno tipados, incluindo recursão;
- `registro` e `enum` com `escolha` exaustivo;
- `Opcao<T>` e `Resultado<T, E>` com o operador `?`;
- vetores dinâmicos (`adicionar`, `remover`, `obter`, `tamanho`);
- bibliotecas Matematica, Texto, Tipos, Util, Calendario, Arquivos e Json;
- interfaces visuais declarativas com `UI.*` no modelo
  iniciar/atualizar/visualizar.

```
funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    se (b == 0) {
        retorne Resultado::Erro("divisão por zero");
    }
    retorne Resultado::Ok(a / b);
}

funcao inicio() {
    escolha dividir(20, 4) {
        caso Resultado::Ok(v) => {
            escreva("resultado = ", v);
        }
        caso Resultado::Erro(e) => {
            escreva("erro: ", e);
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

O executável `portugol` valida, formata, executa e exporta programas sem abrir
a IDE. Com o repositório clonado:

```bash
cargo run -p portugol-cli -- programa.por               # valida (também aceita pastas)
cargo run -p portugol-cli -- executar programa.pors     # executa na máquina virtual
cargo run -p portugol-cli -- executar programa.por -- a b   # parâmetros para inicio(cadeia params[])
cargo run -p portugol-cli -- novo exercicio.pors        # cria um arquivo inicial
cargo run -p portugol-cli -- formatar programa.pors     # formata (--check só confere)
cargo run -p portugol-cli -- exportar-rust programa.pors        # imprime o Rust gerado
cargo run -p portugol-cli -- exportar-cargo programa.pors pasta # gera um projeto Cargo
```

`executar --bytecode` mostra as instruções geradas para a máquina virtual,
útil para quem estuda compiladores. `--quiet` lista apenas os arquivos com erro.

## Para desenvolvedores

### Organização

| Caminho | Conteúdo |
|---|---|
| `crates/portugol-syntax` | lexer, parser, AST e diagnósticos do Portugol clássico |
| `crates/portugol-semantics` | símbolos, escopos, tipos e verificações semânticas |
| `crates/portugol-rs-frontend` | parser, formatador e verificação de tipos do Portugol-RS |
| `crates/portugol-hir` | representação intermediária compartilhada |
| `crates/portugol-bytecode` | compilação para bytecode |
| `crates/portugol-vm` | máquina virtual, bibliotecas, protocolo gráfico e depurador |
| `crates/portugol-rust` | geração de código Rust |
| `crates/portugol-wasm` | núcleo do Portugol-RS compilado para WebAssembly |
| `crates/portugol-service` | camada usada pela IDE: verificar, executar, depurar, exemplos e exercícios |
| `crates/portugol-cli` | executável `portugol` |
| `apps/desktop` | IDE em Tauri v2 com frontend TypeScript e CodeMirror 6 |
| `tests/corpus` | exemplos do Portugol Studio original com as saídas esperadas |
| `docs` | arquitetura, decisões técnicas (`docs/adr`) e distribuição |

A IDE usa Tauri porque o produto depende de um editor de código completo
(seleção, desfazer, busca, autocompletar, entrada de acentos). Com a webview, o
CodeMirror resolve essa parte, e toda a lógica da linguagem fica em Rust.

### Ambiente

As versões de Rust, Node e do JDK 8 (usado em testes comparativos com o
original) estão fixadas em [`mise.toml`](mise.toml):

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
cargo test -p portugol-vm --test regressao   # corpus do original com host gráfico sem tela

cd apps/desktop
npm ci
npm test                                     # testes do frontend
npm run tauri dev                            # abre a IDE em modo de desenvolvimento
npm run tauri build                          # gera os instaladores locais
```

A IDE fica fora dos membros padrão do workspace porque depende das bibliotecas
gráficas do sistema; use `cargo build -p portugol-desktop` para compilá-la
diretamente.

### Documentação

- [`docs/ARQUITETURA.md`](docs/ARQUITETURA.md): estrutura do compilador, da VM
  e da IDE, e as diferenças de semântica herdadas do Java.
- [`docs/PARIDADE.md`](docs/PARIDADE.md): comparação recurso a recurso com o
  Portugol Studio original.
- [`docs/PORTUGOL-RUST.md`](docs/PORTUGOL-RUST.md): exportação para Rust,
  execução de `.rs` e projetos Cargo, limites de execução.
- [`docs/DISTRIBUICAO.md`](docs/DISTRIBUICAO.md): como gerar e assinar os
  instaladores.
- [`docs/adr`](docs/adr): registro das decisões de arquitetura.

## Segurança

Programas `.por` e o modo de depuração rodam na máquina virtual da IDE, com
limites de tempo e de memória. Já os arquivos `.rs`, os projetos Cargo e a
execução normal de `.pors` são compilados para código nativo e rodam com as
permissões do seu usuário, sem isolamento. A IDE pede confirmação antes da
primeira execução; rode apenas código em que você confia.

## Licença

O Portugol Studio original é distribuído sob a LGPL-3.0. Este projeto é uma obra
derivada e usa a mesma licença (`LGPL-3.0-only`); o texto completo está em
[`LICENSE`](LICENSE). Os exemplos em `tests/corpus`, a ajuda e parte das
imagens da interface vêm do projeto original.
