# ADR 0018: Projetos desktop PORS com designer visual, code-behind puro e exportação Slint

- **Status:** Aceita
- **Data:** 2026-11-27
- **Responsável:** mantenedor do produto
- **Relacionados:** [ADR 0015](0015-evolucao-visual-orientada-a-aprendizagem.md), [ADR 0016](0016-pors-como-linguagem-autoral-e-retirada-do-por.md), [ADR 0017](0017-modelo-execucao-e-estado-ui-visual-pors.md)

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
  JanelaPrincipal.tela   # layout da janela, mantido pelo designer
  JanelaPrincipal.pors   # code-behind: handlers de eventos
  recursos/              # imagens e ícones referenciados pelas telas
```

O manifesto `.porsproj` é a raiz do projeto: a IDE o reconhece ao abrir a
pasta, como já faz com `Cargo.toml`. Um projeto pode ter várias janelas; cada
uma é um par `.tela` + `.pors` com o mesmo nome.

### 2. Formato `.tela`: fonte de verdade do layout

O layout é descrito num formato **próprio do Portugol Studio RS**, textual,
determinístico e legível, para que diffs e revisões sejam úteis e o aluno possa
editá-lo à mão:

```
janela JanelaPrincipal {
    titulo: "Calculadora"
    largura: 360
    altura: 240

    caixa_texto txtA { x: 16  y: 16  largura: 120 }
    caixa_texto txtB { x: 16  y: 52  largura: 120 }
    botao btnSomar { x: 16  y: 92  texto: "Somar"  ao_clicar: btnSomar_clicar }
    rotulo lblResultado { x: 16  y: 132  texto: "Resultado: -" }
}
```

- O designer e o editor de texto editam o mesmo arquivo, nos dois sentidos.
  O designer reescreve o arquivo com um formatador canônico que preserva
  comentários e a ordem dos controles.
- O formato tem esquema versionado e é validado por um crate novo, sem
  dependências de UI, compilável para `wasm32-unknown-unknown` como o restante
  do núcleo PORS.
- O primeiro conjunto de controles é: `rotulo`, `botao`, `caixa_texto`,
  `caixa_selecao`, `botao_opcao`, `lista`, `combo`, `imagem` e `painel`, além
  dos componentes não visuais `temporizador` e `dialogo_arquivo`.
- O posicionamento inicial é absoluto (`x`, `y`, `largura`, `altura`), com
  âncoras simples. Contêineres com layout (pilha, grade, encaixe), estilos e
  binding declarativo ficam para uma fase posterior, sem quebrar arquivos
  existentes.
- O formato **não** é o de nenhum framework. Isso mantém o alvo de exportação
  trocável.

### 3. Designer visual na IDE

A IDE ganha um editor de design para arquivos `.tela`, com as abas
*Design* e *Código*:

- **Caixa de ferramentas** com os controles da seção 2, arrastáveis para a
  janela;
- seleção, movimentação e redimensionamento com alças, grade com *snap*, guias
  de alinhamento, desfazer/refazer;
- **painel de propriedades** com as abas *Propriedades* e *Eventos*;
- **duplo clique** num controle cria o handler do evento padrão no code-behind
  (por exemplo `btnSomar_clicar`) e leva o cursor até ele;
- componentes não visuais aparecem numa faixa abaixo da janela.

O designer é implementado na própria IDE (Tauri + DOM), reaproveitando o
renderizador da prévia `UI.*`. Nenhum framework de UI Rust é embutido na IDE.

### 4. Code-behind com handlers puros

A partir de cada `.tela`, o frontend PORS gera um `registro` com o nome da
janela, que tem um campo por controle e as propriedades editáveis de cada um.
Um handler recebe a janela e **devolve a janela atualizada**:

```pors
funcao btnSomar_clicar(tela: JanelaPrincipal) -> JanelaPrincipal {
    seja mutavel t: JanelaPrincipal = tela;
    escolha Tipos.para_inteiro(t.txtA.texto) {
        caso Resultado::Ok(a) => {
            escolha Tipos.para_inteiro(t.txtB.texto) {
                caso Resultado::Ok(b) => {
                    t.lblResultado.texto = "Resultado: " + Tipos.inteiro_para_cadeia(a + b);
                }
                caso Resultado::Erro(_) => {
                    t.lblResultado.texto = "B não é um número";
                }
            }
        }
        caso Resultado::Erro(_) => {
            t.lblResultado.texto = "A não é um número";
        }
    }
    retorne t;
}
```

- Para o aluno, o código se lê como Windows Forms ("altero o texto do
  rótulo"). Por baixo, continua sendo a transição pura do ADR 0017:
  `atualizar(estado, evento) -> estado`, sem `&mut` nem regras de empréstimo
  no caminho crítico.
- A VM registra a janela antes e depois de cada evento. Isso habilita a
  inspeção de estado e o histórico (*time-travel*) no depurador.
- O estado do programa que não pertence a nenhum controle fica num
  `registro` declarado pelo aluno e referenciado pela janela (campo `dados`,
  declarado no `.tela`), para não misturar estado de domínio com propriedades
  visuais.
- A ligação entre `ao_clicar: btnSomar_clicar` e a função é verificada em
  tempo de análise: um handler ausente, com assinatura errada ou de outra
  janela gera diagnóstico em português com posição no `.tela` e no `.pors`.
- O registro gerado não é escrito em disco. Ele é sintetizado na HIR a partir
  do `.tela`, aparece no autocompletar e nas dicas, e é regenerado a cada
  mudança do layout.
- O ciclo de vida segue o ADR 0017, com orçamento por evento (tempo ativo do
  handler, memória de cadeias, número de controles) e cancelamento pelo host.

### 5. Execução em janela própria e serviços do SO

- **Prévia** no designer: renderizador DOM, sem executar handlers, ou com
  execução opcional na própria aba.
- **Executar**: abre uma **janela Tauri separada**, com título, tamanho e
  ícone do `.porsproj`/`.tela`, e a VM PORS conduzindo os eventos. Fechar a
  janela encerra a execução.
- Serviços do SO entram como módulos nativos da VM (`Op::CallNative`), com
  permissões explícitas no manifesto: `Dialogo` (abrir, salvar, mensagem,
  confirmação), `Arquivos` (restrito à pasta do projeto, como hoje),
  `AreaTransferencia` e `Notificacao`. O comportamento observável de cada
  serviço é especificado para que a VM e o Rust exportado se comportem igual.

### 6. Exportação para Rust com Slint

A exportação Cargo de um projeto desktop usa o **[Slint](https://slint.dev)**
como framework de UI do binário gerado:

- cada `.tela` é traduzido para um componente `.slint`. Posição absoluta,
  propriedades e *callbacks* mapeiam quase um para um;
- o code-behind é traduzido pelo `portugol-rust`: o registro da janela vira uma
  `struct` Rust; cada handler vira uma função `fn(JanelaPrincipal) ->
  JanelaPrincipal`, ligada ao *callback* do Slint por um adaptador gerado que
  lê as propriedades dos controles, chama o handler e escreve o resultado de
  volta;
- o projeto exportado usa o estilo nativo do Slint para a plataforma (Fluent no
  Windows, Cupertino no macOS, Fluent ou Qt no Linux);
- a exportação continua opcional: executar e depurar dentro da IDE não exige
  `rustc`.

Slint foi escolhido porque: (a) a descrição de tela em marcação com
posicionamento absoluto corresponde diretamente ao `.tela`; (b) é estável e
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

`UI.*` continua existindo para exercícios, exemplos curtos e a Web, sem
mudanças. Projetos desktop são um caminho separado, que reaproveita o mesmo
modelo de estado, a mesma VM e o mesmo renderizador DOM. Nada aqui antecipa
os gates de retirada de `.por`/`.pex` do ADR 0016.

## Alternativas consideradas

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
segundo alvo de exportação, porque o `.tela` não depende de framework.

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
  `.tela`; manifesto `.porsproj`; modelo *Aplicativo Desktop PORS*;
  reconhecimento do projeto no Explorador.
- **Fase 2 (designer mínimo):** canvas, caixa de ferramentas, seleção,
  movimentação, redimensionamento, painel de propriedades, desfazer/refazer e
  sincronização com o texto.
- **Fase 3 (code-behind):** síntese do registro da janela na HIR, verificação
  das ligações de eventos, criação do handler por duplo clique, execução em
  janela Tauri separada e snapshots no depurador.
- **Fase 4 (serviços do SO):** `Dialogo`, `Arquivos`, `AreaTransferencia`,
  `Notificacao` e `temporizador`, com permissões no manifesto.
- **Fase 5 (exportação Slint):** `.tela` → `.slint`, adaptadores de
  *callbacks* e testes de paridade entre a VM e o binário exportado.
- **Fase 6 (recursos estilo WPF):** contêineres com layout, estilos/temas e
  binding declarativo.
- A HIR passa a aceitar tipos sintetizados de fora do código-fonte `.pors`,
  com *source maps* que apontam para o `.tela`.
- Testes de paridade passam a cobrir a sequência de eventos → estado da janela
  na VM e no binário Slint, documentando diferenças inevitáveis de host
  (fontes, métricas de texto, aparência nativa).
- A exportação passa a depender de uma biblioteca externa (Slint). Atualizações
  de versão do Slint precisam de testes de regressão do gerador.
