# ADR 0022: Projetos de jogos 2D PORS com ciclo estruturado e exportação Macroquad

- **Status:** Proposta
- **Data:** 2026-11-27
- **Responsável:** mantenedor do produto
- **Emendas:** Emenda 1 (2026-11-27)
- **Relacionados:** [ADR 0015](0015-evolucao-visual-orientada-a-aprendizagem.md), [ADR 0016](0016-pors-como-linguagem-autoral-e-retirada-do-por.md), [ADR 0017](0017-modelo-execucao-e-estado-ui-visual-pors.md), [ADR 0018](0018-projetos-desktop-pors-com-designer-visual.md), [ADR 0019](0019-contrato-minimo-de-tipos-pors.md), [ADR 0020](0020-ajustes-pedagogicos-da-sintaxe-pors.md), [ADR 0021](0021-tela-como-forma-unica-de-interface-pors.md)

> **Emenda 1 (2026-11-27).** O manifesto `.ferproj` usa a mesma marcação do
> `.design`, como definido na seção 1 do ADR 0018 e implementado no crate
> `portugol-tela` (PR #259), e não TOML. O exemplo da seção 1 foi reescrito
> nessa marcação: o tipo do projeto é o atributo `Tipo="Jogo"` e os parâmetros
> da janela ficam no elemento filho `<Janela>`. As demais decisões não mudam.

> **Nota (2026-11-27).** A extensão do manifesto passou a ser `.ferproj`
> ([ADR 0023](0023-identidade-e-marca-ferro-studio.md), Emenda 1).

## Contexto

O Portugol Studio RS já executa programas gráficos e jogos 2D na VM através dos
módulos `Graficos`, `Teclado`, `Mouse` e `Sons`, mas essa área ainda carece de
estrutura de projeto e de ponte para Rust:

1. hoje existem apenas arquivos avulsos em loop imperativo (`funcao inicio()`
   com `Graficos.renderizar()` em um laço manual), sem manifesto de projeto
   para título de janela, resolução, taxa de quadros alvo, ícone ou pasta de
   recursos (`sprites`, `sons`, `fontes`);
2. o `portugol-rust` não exporta programas que usam `Graficos`, `Teclado`,
   `Mouse` e `Sons` para um projeto Cargo compilável;
3. o [ADR 0018](0018-projetos-desktop-pors-com-designer-visual.md) e o
   [ADR 0021](0021-tela-como-forma-unica-de-interface-pors.md) estabeleceram as
   telas (`.design` + code-behind `.pors`) com exportação para **Slint** como
   forma única de interfaces de formulário/desktop, deixando explícito que
   *"`Graficos` continua sendo o caminho para desenho 2D e jogos, que não são
   telas de formulário"*;
4. a avaliação da lista curada da comunidade Rust
   (`dasifefe/rust-game-development-frameworks`) mostrou que misturar um *game
   engine* dentro de uma janela Slint cria conflito de *event loop*, contexto
   gráfico e foco, enquanto adotar um padrão único, leve e dedicado para jogos
   2D fecha a tríade didática da IDE (**Console → `std`**, **Desktop `.design` → Slint**,
   **Jogos 2D → Macroquad**).

O objetivo é oferecer na IDE uma experiência de projeto de jogo 2D educacional:
estrutura de pastas organizada, recursos inspecionáveis no Explorador, ciclo de
jogo limpo com depuração por quadro (*frame*), e exportação Cargo para um
framework Rust simples, rápido de compilar e compatível com Desktop e Web
(`wasm32-unknown-unknown`).

## Decisão

### 1. Projeto de Jogo 2D PORS (`.ferproj` unificado)

O mesmo formato de manifesto `.ferproj` do ADR 0018 ganha `Tipo="Jogo"` no
modelo **Jogo 2D PORS** em *Novo projeto*:

```
MeuJogo/
  meujogo.ferproj       # manifesto: Tipo="Jogo", janela, fps alvo, ícone
  principal.pors         # ciclo iniciar / atualizar / desenhar
  cenas/                 # módulos ou estados de cena opcionais (.pors)
  recursos/
    sprites/             # imagens (.png)
    sons/                # efeitos e músicas (.wav, .mp3, .ogg)
    fontes/              # fontes opcionais (.ttf)
```

O manifesto declara os parâmetros iniciais da janela e recursos usados pelo
runtime da IDE e pelo exportador Cargo:

```xml
<Projeto Tipo="Jogo" Nome="MeuJogo" Versao="0.1.0" Entrada="principal.pors" Formato="1">
    <Janela Titulo="Meu Primeiro Jogo" Largura="800" Altura="600" Fps="60" />
</Projeto>
```

- O manifesto é lido pelo mesmo parser e validador do ADR 0018, com as mesmas
  mensagens em português. Os nomes seguem o PascalCase da marcação.
- `Tipo` vale `Desktop` (o padrão, quando o atributo é omitido) ou `Jogo`.
- Em projetos `Desktop`, `JanelaInicial` indica a tela de partida e não há
  `<Janela>` no manifesto, porque cada tela tem o próprio `.design`.
- Em projetos `Jogo`, `Entrada` indica o `.pors` com o ciclo do jogo, e o
  filho `<Janela>` traz `Titulo`, `Largura`, `Altura` e `Fps`. `JanelaInicial`
  não se aplica.
- `Icone` vale para os dois tipos.

- Os modelos iniciais em *Novo projeto → Jogo 2D PORS* incluem: **Jogo mínimo**
  (jogador controlável + placar), **Pong / Arcade** e **Desvio de obstáculos**.

### 2. Ciclo de jogo estruturado em PORS

Para projetos `Tipo="Jogo"`, o padrão recomendado adota três funções claras —
seguindo o mesmo princípio de estado explícito dos ADRs 0017, 0018 e 0021, e já
alinhado à sintaxe dos ADRs 0019 e 0020:

```pors
enum Cena {
    Jogando,
    FimDeJogo
}

registro Jogo {
    cena: Cena,
    x: real,
    y: real,
    pontos: inteiro
}

funcao iniciar() -> Jogo {
    retorne Jogo { cena: Cena::Jogando, x: 400.0, y: 300.0, pontos: 0 };
}

funcao atualizar(jogo: Jogo, dt: real) -> Jogo {
    seja mutavel j: Jogo = jogo;
    escolha j.cena {
        caso Cena::Jogando => {
            se (Teclado.tecla_pressionada(Teclado.SETA_DIREITA)) {
                j.x = j.x + 220.0 * dt;
            }
            se (Teclado.tecla_pressionada(Teclado.SETA_ESQUERDA)) {
                j.x = j.x - 220.0 * dt;
            }
        }
        caso Cena::FimDeJogo => {
            se (Teclado.tecla_pressionada(Teclado.ENTER)) {
                retorne iniciar();
            }
        }
    }
    retorne j;
}

funcao desenhar(jogo: Jogo) -> vazio {
    Graficos.definir_cor(Graficos.PRETO);
    Graficos.limpar();
    Graficos.definir_cor(Graficos.AZUL);
    Graficos.desenhar_retangulo(jogo.x, jogo.y, 40.0, 40.0, falso, verdadeiro);
    Graficos.definir_cor(Graficos.BRANCO);
    Graficos.desenhar_texto(16.0, 24.0, "Pontos: {jogo.pontos}");
}
```

- **Sem `&mut` nem ECS no início:** `atualizar(jogo, dt) -> Jogo` usa transição
  pura no código do aluno, mantendo consistência com os ADRs 0018 e 0021 e
  permitindo que a VM capture *snapshots* do estado a cada quadro.
- **`dt` protegido:** a VM (e o adaptador gerado) limita `dt` a no máximo
  `0.05` segundos por quadro para evitar saltos de física quando o jogo é
  pausado no depurador ou minimizado.
- **Compatibilidade mantida:** programas imperativos com `funcao inicio()` +
  `Graficos.renderizar()` continuam funcionando normalmente para exemplos curtos
  e material existente em `.pors`.

### 3. Experiência na IDE (execução, recursos e depuração por quadro)

A IDE **não** embute um motor de jogos nativo: ela executa o jogo na VM PORS
com a janela gráfica (`graphics.rs`) e o sistema de áudio (`rodio`) já
existentes, adicionando ferramentas próprias de desenvolvimento de jogos:

- **Explorador de projeto de jogo:** reconhece `.ferproj` (`Tipo="Jogo"`) e
  exibe a pasta `recursos/` com prévia em miniatura de imagens e botão de
  reprodução rápida para arquivos de áudio, além de inserção rápida do caminho
  relativo no editor.
- **Controles de execução de jogo:**
  - **Executar**, **Pausar / Retomar**;
  - **Avançar 1 quadro (*Step Frame*):** executa exatamente uma chamada de
    `atualizar` + `desenhar` e pausa novamente — ideal para depurar colisão e
    movimento;
  - **Reiniciar (`iniciar()`):** recria o estado inicial sem fechar a janela;
  - indicador de **FPS** e tempo de quadro (`ms`).
- **Inspetor de estado e histórico circular:** quando pausado, o Inspetor de
  Variáveis exibe os campos do `registro Jogo` (posição, cena, placar, vetores
  de entidades). A VM mantém um histórico circular curto (por exemplo, últimos
  120 quadros com orçamento de memória) para inspecionar quadros anteriores sem
  reexecutar o jogo.
- **Limites de execução:** os orçamentos da VM (ADR 0003) passam a ser
  aplicados **por quadro** no modo jogo (em vez de um teto único para toda a
  sessão), garantindo interrupção limpa em caso de laço infinito dentro de
  `atualizar` ou `desenhar`.

### 4. Exportação para Rust com Macroquad (Desktop e Web)

A exportação Cargo de projetos `Tipo="Jogo"` (e, progressivamente, de
programas `.pors` que usam `Graficos`/`Teclado`/`Mouse`/`Sons`) usa o
**[Macroquad](https://macroquad.rs)** (`macroquad = "0.4"`) como padrão oficial:

| Conceito PORS | Mapeamento Macroquad gerado |
|---|---|
| `<Janela>` do `.ferproj` | `fn window_conf() -> Conf` + `#[macroquad::main(window_conf)]` |
| `iniciar` / `atualizar` / `desenhar` | `let mut jogo = iniciar(); loop { let dt = get_frame_time().min(0.05); jogo = atualizar(jogo, dt); desenhar(&jogo); next_frame().await; }` |
| `Graficos.limpar`, formas, texto e imagens | `clear_background`, `draw_rectangle`, `draw_circle`, `draw_line`, `draw_text`, `load_texture` / `draw_texture` |
| `Teclado` / `Mouse` | `is_key_down`, `is_key_pressed`, `mouse_position`, `is_mouse_button_down` |
| `Sons` | `macroquad::audio::{load_sound, play_sound, stop_sound}` |

Macroquad foi escolhido pelos seguintes critérios:

1. **Correspondência 1:1 com `Graficos`:** o modelo *immediate-mode* em um laço
   `async` simples produz código Rust curto, legível e didático, sem
   *boilerplate* de motor complexo.
2. **100% Rust puro (zero fricção no PC do aluno):** não exige CMake nem
   compilador C++ instalado (ao contrário de bindings C), compila em ~5–15
   segundos em máquinas modestas e gera binários de poucos megabytes.
3. **WebAssembly compatível com o projeto:** compila diretamente para
   `wasm32-unknown-unknown` sem Emscripten nem `wasm-bindgen`, permitindo
   exportar o jogo do aluno também para o navegador.
4. **Licença permissiva:** distribuído sob `MIT OR Apache-2.0`, sem restrições
   comerciais ou copyleft para os jogos exportados pelos alunos.

### 5. Relação com os ADRs 0018 e 0021 (`.design` + Slint)

Fica estabelecida a separação definitiva entre os três alvos da ponte para Rust:

| Tipo de projeto | Uso principal | Execução na IDE | Alvo Cargo |
|---|---|---|---|
| **Console / Biblioteca** | Lógica, algoritmos, estruturas | Console integrado | Rust padrão (`std`) |
| **Aplicativo Desktop** (ADRs 0018 e 0021) | Formulários, janelas (`.design` + `.pors`), serviços do SO | Janela Tauri + `.design` | **Slint** |
| **Jogo 2D** (este ADR) | Jogos, simulações, áudio e entrada em tempo real | Janela gráfica (`graphics.rs`) + `rodio` | **Macroquad** |

- Nenhum game framework é acoplado dentro de janelas Slint. Caso um projeto
  `.design` precise no futuro de um controle simples de desenho 2D, ele deve usar
  um *buffer* RGBA (`slint::Image`) dentro do próprio Slint.

## Alternativas consideradas

### Bevy como alvo de exportação PORS
Rejeitada para tradução automática a partir do PORS. O paradigma ECS (`World`,
`Query`, `Commands`, `ResMut`, sistemas agendados) não tem correspondência
direta com código estruturado em Portugol-RS, além de exigir tempos de
compilação e uso de disco (`target/`) elevados para laboratórios escolares. O
Bevy deve ser citado na trilha avançada como próximo passo quando o aluno já
estiver escrevendo Rust puro.

### `raylib-rs`
Rejeitada. Embora tenha API 2D igualmente direta, depende de compilação C/CMake
para `raylib` no ambiente do usuário e de Emscripten para Web, aumentando a
chance de erro na máquina do aluno ao clicar em **Exportar Cargo**.

### `ggez`, `Notan` e `Comfy`
Rejeitadas. `ggez` e `Comfy` carregam pilha `wgpu` mais pesada e impõem traits
ou convenções mais opinativas; `Notan` é competente, mas tem comunidade,
documentação e exemplos educacionais muito menores que o Macroquad.

### `Fyrox`
Fora de escopo. É uma engine 2D/3D completa com editor de cenas próprio,
excessiva para o objetivo pedagógico do Portugol Studio RS.

### `Amethyst`, `Tetra`, `Piston`, `Coffee`
Rejeitadas por estarem arquivadas ou com baixa manutenção.

### Embutir Macroquad na IDE ou acoplá-lo ao Slint
Rejeitada. Duplicaria o ciclo de janela da IDE, dificultaria o depurador passo
a passo e a inspeção de estado na VM e criaria conflitos de *event loop* entre
Slint e Macroquad.

## Consequências

- **Fase 1 (manifesto e ciclo de jogo):** suporte a `Tipo="Jogo"` no
  `.ferproj` (compartilhando o parser com o ADR 0018), execução do ciclo
  `iniciar`/`atualizar`/`desenhar` na VM com `dt` limitado e orçamento por
  quadro.
- **Fase 2 (IDE para jogos):** reconhecimento de `recursos/` no Explorador com
  prévia de sprites e sons, botão *Avançar 1 quadro (Step Frame)*, medidor de
  FPS e histórico circular de quadros no depurador.
- **Fase 3 (exportação Macroquad Desktop):** gerador no `portugol-rust` para
  `Cargo.toml` (`macroquad`) + `src/main.rs` + cópia da pasta `recursos/`, com
  testes de paridade entre VM e binário Macroquad para formas, texto, entrada e
  áudio.
- **Fase 4 (exportação Web e catálogo):** opção de gerar pacote Web
  (`wasm32-unknown-unknown` + HTML mínimo para Macroquad) e modelos iniciais de
  jogos 2D na janela *Novo projeto*.
- O `portugol-rust` passa a possuir dois backends opcionais de projeto Cargo
  além de `std`: **Slint** (projetos `.design`, ADRs 0018 e 0021) e **Macroquad**
  (projetos `Tipo="Jogo"`, este ADR), sem adicionar nenhuma dependência ao
  núcleo `portugol-vm --no-default-features` nem ao `portugol-wasm`.
