# ADR 0018: Projetos desktop PORS com designer visual, code-behind puro e exportação Slint

- **Status:** Aceita
- **Data:** 2026-11-27
- **Responsável:** mantenedor do produto
- **Relacionados:** [ADR 0015](0015-evolucao-visual-orientada-a-aprendizagem.md), [ADR 0016](0016-pors-como-linguagem-autoral-e-retirada-do-por.md), [ADR 0017](0017-modelo-execucao-e-estado-ui-visual-pors.md), [ADR 0020](0020-ajustes-pedagogicos-da-sintaxe-pors.md), [ADR 0021](0021-tela-como-forma-unica-de-interface-pors.md)
- **Emendas:** Emendas 1, 2 e 3 (2026-11-27)

> **Emenda 1 (2026-11-27).** O formato da tela passa a usar uma marcação de tags no
> estilo XAML (pseudo-XAML) em vez da sintaxe de chaves da versão inicial. O
> designer segue um caminho híbrido em três níveis: posição livre como no
> Windows Forms, layout com `Pilha` e `Grade` como no WPF, e vinculação de
> exibição com `{dados.campo}`. `Pilha` e `Grade` passam da última fase para
> a fase do designer. O `UI.*` deixa de ser um caminho paralelo e é aposentado
> pelo [ADR 0021](0021-tela-como-forma-unica-de-interface-pors.md). As seções
> 2, 3, 4, 6 e 7, as alternativas e as consequências foram reescritas; as
> decisões de handler puro, execução em janela própria e exportação Slint não
> mudaram.

> **Emenda 2 (2026-11-27).** Uma tela é a junção de dois arquivos com o mesmo
> nome: `JanelaPrincipal.design` (o pseudo-XAML) e `JanelaPrincipal.pors` (o
> code-behind). Não existe arquivo `.tela` em disco: "tela" é o nome do
> conceito e do nó que agrupa os dois arquivos no Explorador. Abrir o nó abre o
> modo visual; abrir o `.design` mostra o pseudo-XAML; abrir o `.pors` mostra o
> código. O modo visual ganha as vistas *Design*, *Dividido* e *Código*.

> **Emenda 3 (2026-11-27).** Registra quais conceitos do WPF o formato
> aproveita, em versão simplificada, e quais ficam de fora. Entram já: o
> contêiner `Encaixe`, os atributos `Preenchimento` e `Alinhamento`, a ordem de
> tabulação, as teclas de acesso e a `Descricao` para leitores de tela (Fase 2);
> a vinculação de mão dupla em controles de entrada, `Habilitado` vinculado e
> dados de exemplo no designer (Fase 4). Modelos de itens, estilos, validação
> junto do campo e diálogos entre telas ficam para depois. Eventos roteados,
> *templates* de controle, *triggers*, animações, conversores e `ICommand` não
> serão adotados. Ver a seção 8.

## Contexto

O ADR 0017 introduziu programas visuais em PORS no modelo MVU
(`iniciar`/`atualizar`/`visualizar`) com uma árvore declarativa pequena
(`UI.coluna`, `UI.linha`, `UI.texto`, `UI.botao`, `UI.entrada`, `UI.lista`),
renderizada no WebView Tauri e no DOM Web. Esse modelo atende bem à prévia e a
exercícios, mas não atende a aplicativos desktop:

1. a tela é escrita só em código, sem um editor visual;
2. não existe o conceito de **projeto** de aplicativo (janela inicial, ícone,
   recursos, manifesto);
3. o programa não abre uma janela própria do sistema operacional nem usa
   serviços do SO (diálogos de arquivo, caixas de mensagem, área de
   transferência);
4. eventos são identificados por cadeias (`alvo == "inc"`), sem ligação
   verificável entre o controle e o código que reage a ele;
5. a exportação Rust gera um runtime próprio que serializa a árvore em JSON,
   sem interface nativa no binário resultante.

O objetivo é uma experiência parecida com o Visual Studio + Windows Forms/WPF:
criar um projeto a partir de um modelo, montar a janela arrastando componentes
de uma caixa de ferramentas, editar propriedades, ligar eventos aos componentes
e escrever em PORS a lógica ligada ao visual, sem perder a depuração passo a
passo e a ponte para Rust real que o produto já oferece.

Foram avaliados frameworks de UI em Rust para servir de base. O FerroUI
(port de Avalonia/WinUI) foi descartado por ora: o projeto tem menos de dois
meses, API ainda instável, um único mantenedor, executáveis de 51–71 MB para um
"olá mundo" e uma pilha WASM (`wasm32-unknown-emscripten`) diferente da adotada
no ADR 0017. Iced foi considerado pelo modelo Elm/MVU, mas não oferece
posicionamento absoluto nem uma descrição de tela em marcação, que um designer
no estilo Windows Forms exige. Leptos, Yew, Sycamore e Dioxus resolvem a Web ou
reaproveitam WebView, não um binário desktop nativo.

## Decisão

### 1. Projeto desktop PORS

Um novo modelo **Aplicativo Desktop PORS** em *Novo projeto* cria:

```
MeuApp/
  meuapp.porsproj        # manifesto: nome, versão, ícone, janela inicial
  principal.pors         # ponto de entrada
  JanelaPrincipal.design # desenho da janela em pseudo-XAML, mantido pelo designer
  JanelaPrincipal.pors   # code-behind: handlers de eventos
  recursos/              # imagens e ícones referenciados pelas telas
```

O manifesto `.porsproj` é a raiz do projeto: a IDE o reconhece ao abrir a
pasta, como já faz com `Cargo.toml`. Um projeto pode ter várias janelas.

**Uma tela é a junção de dois arquivos**, ligados pelo mesmo nome: o
`.design`, que descreve como a janela é, e o `.pors`, que diz o que ela faz.
Não há arquivo `.tela` em disco. O compilador junta os dois pelo nome na
análise e na execução, e o Explorador da IDE os mostra agrupados num nó de
tela:

```
MeuApp/
├─ meuapp.porsproj
├─ principal.pors
├─ JanelaPrincipal           ← nó de tela: abre o modo visual
│   ├─ JanelaPrincipal.design  ← abre o pseudo-XAML
│   └─ JanelaPrincipal.pors    ← abre o code-behind
└─ recursos/
```

- Um `.design` sem `.pors` do mesmo nome é uma tela sem comportamento: abre e
  executa, e o primeiro duplo clique num controle cria o `.pors`.
- Um `.pors` só pertence a uma tela quando existe o `.design` do mesmo nome;
  sem ele, é um arquivo PORS comum.
- Renomear o nó de tela renomeia os dois arquivos juntos, e o nome da janela
  dentro do `.design`.
- Fora da IDE, como no CLI ou num gerenciador de arquivos, aparecem só os dois
  arquivos, o que deixa a separação explícita também ali.

Separar os arquivos deixa visível para o aluno que a tela é desenho mais
lógica, como no Visual Studio, em que o formulário do Windows Forms aparece
como um nó com o código e o arquivo do designer dentro.

### 2. Formato `.design`: pseudo-XAML, fonte de verdade do layout

O layout é descrito numa marcação de tags **própria do Portugol Studio RS**,
inspirada no XAML do WPF: o aluno reconhece a estrutura de HTML e XAML, e o que
aprende aqui vale depois em WPF, MAUI ou Avalonia. A tela fica na marcação e a
lógica fica no `.pors`, a mesma separação do XAML com C#.

```xml
<Janela Nome="JanelaPrincipal" Titulo="Olá" Largura="300" Altura="160">
    <Rotulo Nome="lblMensagem" X="16" Y="16" Texto="Clique no botão" />
    <Botao Nome="btnOla" X="16" Y="56" Texto="Dizer olá" AoClicar="btnOla_clicar" />
</Janela>
```

Regras da marcação:

- Elementos e atributos em PascalCase, só com letras sem acento, para facilitar
  a digitação: `<CaixaTexto>`, `TextoVazio`, `AoClicar`.
- Todo valor fica entre aspas; o esquema do controle define se o valor é
  número, lógico (`verdadeiro`/`falso`), texto, nome de handler, lista de
  bordas ou caminho de recurso, e a validação aponta o atributo errado.
- `Nome` identifica o controle e vira o nome do campo no code-behind. É o
  equivalente ao `x:Name` do XAML, sem prefixo.
- Comentários usam `<!-- ... -->` e são preservados pelo formatador.
- Só existem as entidades `&lt;`, `&gt;`, `&amp;`, `&quot;` e `&apos;`. Não há
  namespaces, `xmlns`, DTD, instruções de processamento nem extensões de
  marcação (`{Binding ...}`): é XML simples, com mensagens de erro em português
  ("Falta fechar `<Pilha>` aberta na linha 3").
- Itens de `Lista` e `Combo` são elementos filhos: `<Item>Somar</Item>`. Nenhum
  outro elemento aceita texto solto.
- A raiz é sempre `<Janela>`. O atributo `Formato="1"` indica a versão do
  esquema; sem ele, vale a versão atual, e o formatador canônico o escreve.

O designer e o editor de texto editam o mesmo arquivo, nos dois sentidos, e o
designer reescreve o arquivo com um formatador canônico que preserva
comentários e a ordem dos elementos. O formato é validado por um crate sem
dependências de UI, compilável para `wasm32-unknown-unknown`, e **não** é o de
nenhum framework, o que mantém o alvo de exportação trocável.

#### Controles

Controles: `Rotulo`, `Botao`, `CaixaTexto`, `CaixaSelecao`, `BotaoOpcao`,
`Lista`, `Combo` e `Imagem`. Componentes não visuais: `Temporizador` e
`DialogoArquivo`, que ficam direto na `<Janela>`. Contêineres: `Painel`,
`Pilha`, `Grade` e `Encaixe`, descritos abaixo.

Atributos comuns a todos os controles visuais, além de posição e tamanho:

- `Margem` (espaço por fora) e, nos contêineres e em `Botao`, `Preenchimento`
  (espaço por dentro), com um, dois ou quatro números (`"8"`, `"8,4"`,
  `"8,4,8,4"`);
- `Alinhamento` e `AlinhamentoVertical` (`Inicio`, `Centro`, `Fim`,
  `Esticar`), usados dentro de `Pilha`, `Grade` e `Encaixe`;
- `Descricao`, o texto lido por leitores de tela quando o controle não tem
  texto visível próprio (uma `Imagem`, por exemplo).

Acessibilidade de teclado sem atributos extras: a **ordem de tabulação** é a
ordem dos elementos no `.design`, e um `_` no texto define a **tecla de
acesso** (`Texto="_Salvar"` mostra "Salvar" com o S sublinhado e responde a
Alt+S; `__` escreve um `_` literal).

#### Três níveis: do Windows Forms ao WPF

O aluno sobe de nível sem trocar de ferramenta nem de formato:

1. **Posição livre (Windows Forms).** `Janela` e `Painel` posicionam os filhos
   por `X` e `Y`, com `Largura`, `Altura` e `Ancora="Cima,Esquerda"`
   opcionais. É o comportamento padrão de arrastar e soltar no designer e não
   exige entender layout.
2. **Layout que se adapta (WPF).** `Pilha` empilha os filhos
   (`Orientacao="Vertical"` ou `"Horizontal"`, `Espaco`) e `Grade` os distribui
   em linhas e colunas (`Linhas="Auto,*"`, `Colunas="120,*"`, com `Linha` e
   `Coluna` em cada filho). Dentro deles não há `X`/`Y`: a posição é automática,
   e o diagnóstico explica isso. Todo controle aceita `Margem`. Uma `Pilha` ou
   `Grade` que é o único filho de `Janela` ou `Painel` ocupa todo o espaço; caso
   contrário, é posicionada por `X`/`Y` como qualquer controle. O designer
   continua arrastando controles para dentro deles.

   `Encaixe` monta o layout clássico de aplicativo: cada filho declara
   `Encaixar="Cima"`, `"Baixo"`, `"Esquerda"` ou `"Direita"` e ocupa aquela
   borda, na ordem do arquivo; o último filho sem `Encaixar` preenche o espaço
   que sobra.

   ```xml
   <Encaixe>
       <Rotulo Encaixar="Cima" Texto="Cadastro de alunos" />
       <Pilha Encaixar="Esquerda" Largura="160"> ... </Pilha>
       <Grade> ... </Grade>
   </Encaixe>
   ```

   Os atributos que o filho declara para o contêiner ler (`Linha`, `Coluna`,
   `Encaixar`) são o equivalente às propriedades anexadas do WPF; a ajuda os
   apresenta como "propriedades que o pai usa para posicionar o filho".
3. **Vinculação de exibição (WPF simplificado).** Um atributo pode mostrar o
   estado do programa com `{dados.campo}`, a mesma sintaxe da interpolação de
   cadeias (ADR 0020):

   ```xml
   <Rotulo Nome="lblResultado" Texto="Resultado: {dados.resultado}" />
   <Botao Nome="btnSalvar" Texto="Salvar" Habilitado="{dados.pode_salvar}" />
   ```

   - Só são aceitos caminhos de campos de `dados` (`{dados.pessoa.nome}`), sem
     chamadas nem operações: a marcação mostra o estado, não calcula.
   - Em atributos de texto, o valor pode misturar texto e vínculos, com os tipos
     escalares do ADR 0020. Em atributos de outro tipo, o valor deve ser
     exatamente um vínculo `{...}` do tipo do atributo.
   - `\{` e `\}` escrevem chaves literais, como nas cadeias PORS.
   - Nos atributos de exibição, a vinculação é de mão única (estado → tela).
     Depois de cada handler, os vínculos são reavaliados.
   - Nos controles de entrada, a vinculação é de **mão dupla**: em
     `<CaixaTexto Texto="{dados.nome}" />`, digitar atualiza `dados.nome`, e
     uma mudança em `dados.nome` atualiza o campo. Vale para `Texto` de
     `CaixaTexto`, `Marcado` de `CaixaSelecao` e `BotaoOpcao` e `Selecionado`
     de `Lista` e `Combo`. O valor vinculado deve ser exatamente um caminho
     `{dados.campo}` do tipo do atributo. Com isso, um formulário funciona sem
     um handler por campo; o handler `AoAlterar` continua disponível para
     reagir à mudança.
   - Uma regra de negócio decide se uma ação está disponível com
     `Habilitado="{dados.pode_salvar}"`, no lugar dos comandos com
     `CanExecute` do WPF.
   - O designer mostra os vínculos com **dados de exemplo**: o aluno define
     valores de exemplo para os campos de `Dados` no painel de propriedades, e
     o modo visual exibe "Resultado: 42" em vez de `{dados.resultado}`. Os
     exemplos ficam no `.design` (`<DadosExemplo resultado="42" />`, filho da
     `<Janela>`), são ignorados na execução e na exportação, e só aceitam
     campos e tipos que existem no registro. É o único elemento cujos
     atributos não seguem o PascalCase: eles são os nomes dos campos do
     registro, como escritos no `.pors`.

### 3. Designer visual na IDE

Abrir um nó de tela no Explorador abre o **modo visual**, com três vistas:

| Vista | O que mostra |
|---|---|
| *Design* | Só a janela, com arrastar e soltar |
| *Dividido* | A janela em cima e o `.design` embaixo. Arrastar um controle muda o pseudo-XAML na hora, e editar o texto move o controle |
| *Código* (F7, como no Visual Studio) | O `.pors` da tela |

A vista *Dividido* é a que mais ensina: o aluno vê que o designer apenas
escreve o `.design` por ele. Abrir o arquivo `.design` diretamente mostra o
pseudo-XAML no editor de texto, com validação e autocompletar, e um botão para
ir ao modo visual.

O modo visual oferece:

- **Caixa de ferramentas** com os controles e contêineres da seção 2,
  arrastáveis para a janela, para um `Painel` ou para dentro de uma `Pilha` ou
  `Grade`;
- seleção, movimentação e redimensionamento com alças, grade com *snap*, guias
  de alinhamento, desfazer/refazer; em `Pilha` e `Grade`, arrastar reordena ou
  muda a célula em vez de mover por `X`/`Y`;
- **árvore de elementos**, que mostra o aninhamento da marcação e permite
  envolver uma seleção num `Painel`, `Pilha` ou `Grade`;
- **painel de propriedades** com as abas *Propriedades* e *Eventos*;
- **duplo clique** num controle cria o handler do evento padrão no code-behind
  (por exemplo `btnSomar_clicar`) e leva o cursor até ele;
- componentes não visuais aparecem numa faixa abaixo da janela.

O designer é implementado na própria IDE (Tauri + DOM), evoluindo o
renderizador da prévia visual atual. Nenhum framework de UI Rust é embutido na
IDE.

### 4. Code-behind com handlers puros

A partir de cada `.design`, o frontend PORS gera um `registro` com o nome da
janela, que tem um campo por controle nomeado e as propriedades editáveis de
cada um, além do campo `dados`. Os atributos em PascalCase viram campos em
minúsculas com `_` (`Texto` → `texto`, `TextoVazio` → `texto_vazio`). Um
handler recebe a janela e **devolve a janela atualizada**.

```xml
<Janela Nome="JanelaPrincipal" Titulo="Calculadora" Largura="360" Altura="240"
        Dados="Conta">
    <Pilha Margem="16" Espaco="8">
        <CaixaTexto Nome="txtA" TextoVazio="Primeiro número" />
        <CaixaTexto Nome="txtB" TextoVazio="Segundo número" />
        <Botao Nome="btnSomar" Texto="Somar" AoClicar="btnSomar_clicar" />
        <Rotulo Nome="lblResultado" Texto="{dados.mensagem}" />
    </Pilha>
</Janela>
```

```pors
registro Conta {
    mensagem: cadeia
}

funcao btnSomar_clicar(tela: JanelaPrincipal) -> JanelaPrincipal {
    seja mutavel t: JanelaPrincipal = tela;
    escolha Tipos.para_inteiro(t.txtA.texto) {
        caso Resultado::Ok(a) => {
            escolha Tipos.para_inteiro(t.txtB.texto) {
                caso Resultado::Ok(b) => {
                    t.dados.mensagem = "Resultado: {a + b}";
                }
                caso Resultado::Erro(_) => {
                    t.dados.mensagem = "O segundo valor não é um número";
                }
            }
        }
        caso Resultado::Erro(_) => {
            t.dados.mensagem = "O primeiro valor não é um número";
        }
    }
    retorne t;
}
```

- Para o aluno, o código se lê como Windows Forms ("altero o texto do
  rótulo") nos níveis 1 e 2, e como WPF ("altero os dados e a tela acompanha")
  no nível 3. Por baixo, continua sendo a transição pura do ADR 0017:
  `atualizar(estado, evento) -> estado`, sem `&mut` nem regras de empréstimo
  no caminho crítico.
- Uma propriedade vinculada na marcação é somente leitura no code-behind:
  atribuir `t.lblResultado.texto` quando `Texto` usa `{dados...}` é
  diagnóstico, para que a tela tenha uma única fonte de verdade. Na
  vinculação de mão dupla, o valor digitado é lido e alterado em `t.dados`, e
  não no campo do controle.
- `dados` e os handlers puros fazem o papel do *view model* do MVVM, sem a
  cerimônia: como o estado é recalculado a cada evento, não há notificação de
  mudança (`INotifyPropertyChanged`) para o aluno implementar.
- O registro de `Dados` começa com os valores padrão de cada campo (`0`, `""`,
  `falso`, vetor vazio). O handler opcional `AoAbrir` da janela ajusta o
  estado inicial.
- A VM registra a janela antes e depois de cada evento. Isso habilita a
  inspeção de estado e o histórico (*time-travel*) no depurador.
- A ligação entre `AoClicar="btnSomar_clicar"` e a função é verificada em
  tempo de análise: um handler ausente, com assinatura errada ou de outra
  janela gera diagnóstico em português com posição no `.design` e no `.pors`.
  O mesmo vale para vínculos com campo inexistente ou de tipo errado.
- O registro gerado não é escrito em disco. Ele é sintetizado na HIR a partir
  do `.design`, aparece no autocompletar e nas dicas, e é regenerado a cada
  mudança do layout.
- O ciclo de vida segue o ADR 0017, com orçamento por evento (tempo ativo do
  handler, memória de cadeias, número de controles) e cancelamento pelo host.

### 5. Execução em janela própria e serviços do SO

- **Prévia** no designer: renderizador DOM, sem executar handlers, ou com
  execução opcional na própria aba.
- **Executar**: abre uma **janela Tauri separada**, com título, tamanho e
  ícone do `.porsproj`/`.design`, e a VM PORS conduzindo os eventos. Fechar a
  janela encerra a execução.
- Serviços do SO entram como módulos nativos da VM (`Op::CallNative`), com
  permissões explícitas no manifesto: `Dialogo` (abrir, salvar, mensagem,
  confirmação), `Arquivos` (restrito à pasta do projeto, como hoje),
  `AreaTransferencia` e `Notificacao`. O comportamento observável de cada
  serviço é especificado para que a VM e o Rust exportado se comportem igual.

### 6. Exportação para Rust com Slint

A exportação Cargo de um projeto desktop usa o **[Slint](https://slint.dev)**
como framework de UI do binário gerado:

- cada `.design` é traduzido para um componente `.slint`. Posição livre vira
  `x`/`y`, `Pilha` vira `VerticalLayout`/`HorizontalLayout`, `Grade` vira
  `GridLayout`, `Encaixe` vira uma combinação desses layouts (o Slint não tem
  um equivalente direto), e vínculos `{dados.campo}` viram *bindings* de propriedade do
  Slint; propriedades e *callbacks* mapeiam quase um para um;
- o code-behind é traduzido pelo `portugol-rust`: o registro da janela vira uma
  `struct` Rust; cada handler vira uma função `fn(JanelaPrincipal) ->
  JanelaPrincipal`, ligada ao *callback* do Slint por um adaptador gerado que
  lê as propriedades dos controles, chama o handler e escreve o resultado de
  volta;
- o projeto exportado usa o estilo nativo do Slint para a plataforma (Fluent no
  Windows, Cupertino no macOS, Fluent ou Qt no Linux);
- a exportação continua opcional: executar e depurar dentro da IDE não exige
  `rustc`.

Slint foi escolhido porque: (a) a descrição de tela em marcação, com posição
absoluta, layouts e *bindings*, corresponde diretamente aos três níveis do
`.design`; (b) é estável e
mantido há anos; (c) gera executáveis de poucos megabytes; (d) roda em
Windows, macOS, Linux e na Web; (e) possui um interpretador
(`slint-interpreter`) que pode, no futuro, servir de prévia nativa sem
compilação.

**Licença.** O Slint é distribuído sob GPLv3, sob a *Royalty-free License* ou
sob licença comercial. O Portugol Studio RS **não** incorpora o Slint na IDE.
Ele é apenas uma dependência dos projetos que o aluno exporta. O modelo gerado
declara a dependência e inclui a atribuição exigida pela *Royalty-free
License*, e o README do projeto exportado explica as opções de licença. O texto
vigente da licença deve ser conferido antes da primeira versão publicada da
exportação.

### 7. Relação com `UI.*`

A tela (`.design` com code-behind `.pors`) é a **única** forma de criar
interfaces em PORS. O
`UI.*` do ADR 0017 é aposentado em etapas, conforme o
[ADR 0021](0021-tela-como-forma-unica-de-interface-pors.md). Do ADR 0017
permanecem o modelo de estado puro, o orçamento por evento, o histórico e a VM
em WASM para a prévia. Nada aqui antecipa os gates de retirada de `.por`/`.pex`
do ADR 0016.

### 8. Conceitos do WPF: o que entra e o que fica de fora

O critério é o que ajuda a pensar em estrutura de interface; o que existe para
construir frameworks ou interfaces muito elaboradas fica de fora.

| Conceito do WPF | No PORS | Quando |
|---|---|---|
| Separação XAML + code-behind | `.design` + `.pors` | Fase 1 |
| Árvore lógica | Aninhamento das tags e árvore de elementos do designer | Fase 2 |
| `StackPanel`, `Grid`, `DockPanel`, `Canvas` | `Pilha`, `Grade`, `Encaixe`, posição livre em `Janela`/`Painel` | Fase 2 |
| `Margin`, `Padding`, alinhamentos | `Margem`, `Preenchimento`, `Alinhamento`, `AlinhamentoVertical` | Fase 2 |
| Propriedades anexadas | `Linha`, `Coluna`, `Encaixar` | Fase 2 |
| `TabIndex`, teclas de acesso, `AutomationProperties.Name` | Ordem das tags, `_` no texto, `Descricao` | Fase 2 |
| Unidades independentes de resolução | Pixels lógicos | Fase 2 |
| Data binding de mão única e dupla | `{dados.campo}` | Fase 4 |
| `CanExecute` de comandos | `Habilitado="{dados...}"` | Fase 4 |
| Dados de tempo de design (`d:DataContext`) | `<DadosExemplo>` | Fase 4 |
| MVVM | `dados` + handlers puros, sem notificação de mudança | Fase 4 |
| `DataTemplate` | `<Lista Itens="{dados.alunos}">` com `<ModeloItem>` para cada `registro` | Depois |
| `Style`, `ResourceDictionary`, temas | `<Estilo Para="Botao" ... />` na janela ou no projeto, temas claro e escuro | Depois |
| Validação (`IDataErrorInfo`) | `Erro="{dados.erro_idade}"` junto do campo, ligado a `Resultado` e `Tipos.para_*` | Depois |
| Várias janelas e diálogos (`ShowDialog`) | Abrir outra tela como diálogo, com resultado em `Resultado` | Depois |

Não serão adotados: eventos roteados (*bubbling*/*tunneling*), dependency
properties declaradas pelo aluno, `ControlTemplate`, *triggers* e
`VisualStateManager`, *storyboards* e animações, conversores
(`IValueConverter`, substituídos pela interpolação e pela formatação do
`escreva`), `ICommand` (substituído por handler + `Habilitado`) e o modelo de
conteúdo arbitrário (o máximo previsto é um `Botao` com `Icone`). Cada um é um
conceito a mais antes de o primeiro programa funcionar, sem ganho
proporcional no aprendizado.

## Alternativas consideradas

### Sintaxe de chaves (versão inicial deste ADR)
Substituída pela Emenda 1. Era mais curta, mas não lembrava nenhuma
tecnologia que o aluno encontrará depois. A marcação de tags é reconhecível de
HTML e transfere diretamente para XAML.

### XAML completo, com namespaces e extensões de marcação
Rejeitada. `xmlns`, `x:Name`, `{Binding Path=..., Mode=...}` e conversores são
cerimônia que não ensina programação. O pseudo-XAML mantém a estrutura e
troca as extensões pela interpolação que o aluno já conhece.

### Um elemento `<Tela>` separado para posição livre
Rejeitada. `Janela` e `Painel` já posicionam livremente; um elemento a mais
seria um conceito a mais no nível 1.

### Handlers com `&mut JanelaPrincipal` ou objetos mutáveis compartilhados
Rejeitada. É mais fiel ao Windows Forms, mas exigiria referências mutáveis e
checagem de empréstimos no frontend PORS antes da Fase 2 do ADR 0016, além de
impedir os snapshots imutáveis que o depurador usa.

### Iced como alvo de exportação
Rejeitada para projetos desktop. O modelo Elm/MVU casa com o ADR 0017, mas o
layout é só em código, sem posicionamento absoluto nem marcação, o que
afastaria o binário exportado do que o designer mostra.

### FerroUI (XAML compilado, estilo WPF/Avalonia)
Adiada. É o mais parecido com WPF, mas está em 0.x com API instável, tem um
único mantenedor, gera binários grandes e usa outra pilha WASM. Pode virar um
segundo alvo de exportação, porque o `.design` não depende de framework.

### Usar o `.slint` diretamente como formato do designer
Rejeitada. Prenderia o formato do projeto, os diagnósticos e a prévia Web à
gramática e às versões do Slint, e exigiria o parser do Slint no núcleo WASM
da IDE.

### Embutir um framework de UI nativo na IDE para a prévia
Rejeitada. A IDE é Tauri + DOM. Um segundo renderizador dentro dela
duplicaria entrada, foco e acessibilidade sem ganho para o aluno.

### Exportar como aplicativo Tauri com a VM embutida
Rejeitada como caminho principal. Garantiria paridade total, mas não seria
"Rust real", carregaria WebView como dependência e contrariaria a ponte para
Rust do produto. Pode ser oferecida depois como "empacotar sem compilar".

## Consequências

- **Fase 1 (formato e projeto):** crate de parser, validação e formatação do
  `.design` em pseudo-XAML; manifesto `.porsproj`; modelo *Aplicativo Desktop
  PORS*; reconhecimento do projeto e dos nós de tela no Explorador.
- **Fase 2 (designer, níveis 1 e 2):** modo visual com as vistas *Design*,
  *Dividido* e *Código*, canvas, caixa de ferramentas, árvore de
  elementos, seleção, movimentação, redimensionamento, painel de propriedades,
  desfazer/refazer e sincronização com o texto; `Janela`/`Painel` livres,
  `Pilha`, `Grade` e `Encaixe`; `Margem`, `Preenchimento`, alinhamentos,
  ordem de tabulação, teclas de acesso e `Descricao`.
- **Fase 3 (code-behind e execução):** síntese do registro da janela na HIR,
  verificação das ligações de eventos, criação do handler por duplo clique,
  execução em janela Tauri separada e snapshots no depurador; prévia visual da
  IDE e playground passam a usar telas (ADR 0021).
- **Fase 4 (vinculação, nível 3):** `{dados.campo}` em atributos, de mão única
  na exibição e de mão dupla nos controles de entrada; `Habilitado`
  vinculado; `<DadosExemplo>` no designer; verificação de tipos e
  propriedades vinculadas somente leitura.
- **Fase 5 (serviços do SO):** `Dialogo`, `Arquivos`, `AreaTransferencia`,
  `Notificacao` e `Temporizador`, com permissões no manifesto.
- **Fase 6 (exportação Slint):** `.design` → `.slint`, adaptadores de
  *callbacks* e testes de paridade entre a VM e o binário exportado.
- **Depois:** modelos de itens (`<ModeloItem>`), estilos e temas, validação
  junto do campo, diálogos entre telas e células que ocupam várias linhas ou
  colunas.
- A HIR passa a aceitar tipos sintetizados de fora do código-fonte `.pors`,
  com *source maps* que apontam para o `.design`.
- Testes de paridade passam a cobrir a sequência de eventos → estado da janela
  na VM e no binário Slint, documentando diferenças inevitáveis de host
  (fontes, métricas de texto, aparência nativa).
- A exportação passa a depender de uma biblioteca externa (Slint). Atualizações
  de versão do Slint precisam de testes de regressão do gerador.
- O crate `portugol-tela` do PR #256, escrito com a sintaxe de chaves, é
  convertido para a marcação de tags e para a extensão `.design` antes da
  mesclagem. Esquema, validação e diagnósticos são mantidos; o nome do crate
  continua, porque ele trata do conceito de tela.
