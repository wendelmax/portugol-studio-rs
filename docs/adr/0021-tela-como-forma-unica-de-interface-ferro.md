# ADR 0021: Telas (`.design` + `.fer`) como forma única de interface em FERRO e aposentadoria do `UI.*`

- **Status:** Aceita
- **Data:** 2026-11-27
- **Responsável:** mantenedor do produto
- **Andamento:** Etapas A, B e C concluídas (2026-11-28); a Etapa C foi antecipada, ver a nota abaixo
- **Substitui parcialmente:** [ADR 0017](0017-modelo-execucao-e-estado-ui-visual-ferro.md), decisão 1 (interface declarativa por componentes `UI.*`)
- **Relacionados:** [ADR 0018](0018-projetos-desktop-ferro-com-designer-visual.md) (Emendas 1 e 2), [ADR 0020](0020-ajustes-pedagogicos-da-sintaxe-ferro.md)

## Contexto

O ADR 0017 criou o `UI.*`: uma árvore de componentes montada em código
(`UI.coluna`, `UI.linha`, `UI.texto`, `UI.botao`, `UI.entrada`, `UI.lista`)
dentro de `visualizar(estado)`, com eventos identificados por cadeias em
`atualizar(estado, alvo)`. O ADR 0018 criou as telas: uma descrição da janela
em marcação (`.design`), editada por um designer visual, com handlers tipados
no code-behind (`.fer`). Na versão inicial do ADR 0018, os dois conviveriam.

Manter os dois prejudica o aprendizado:

1. **Duas formas de fazer a mesma tela.** O aluno teria que escolher entre
   "modo código" e "modo design" antes de saber programar, e aprenderia dois
   vocabulários para os mesmos controles (`UI.texto` e `Rotulo`, `UI.entrada`
   e `CaixaTexto`).
2. **Eventos por cadeia não são verificados.** Em `se (alvo == "inc")`, um erro
   de digitação no identificador do botão passa em silêncio. É o mesmo tipo de
   falha silenciosa que o ADR 0020 corrigiu na interpolação. Numa tela, a
   ligação `AoClicar="btnSomar_clicar"` é verificada na análise.
3. **Dois protocolos e duas exportações.** A árvore do `UI.*` e a do `.design`
   exigiriam dois contratos entre VM e host, e a exportação Rust do `UI.*`
   (runtime JSON em `ferro-rust`) ficaria ao lado da exportação Slint.

O uso publicado do `UI.*` é pequeno: um exemplo
(`08-ui-contador.fer`), nenhum exercício `.fer`, a prévia visual da IDE,
o playground Web, a ajuda e as dicas. Unificar agora custa pouco; depois do
designer pronto, custaria muito.

## Decisão

### 1. Uma só forma de criar interfaces

Em FERRO, interfaces gráficas são criadas **somente** com telas: um `.design`
(pseudo-XAML, ADR 0018) e o code-behind `.fer` do mesmo nome. O `UI.*` é aposentado nas etapas da seção 3.

A biblioteca `Graficos` não muda: ela continua sendo o caminho para desenho 2D
e jogos, que não são telas de formulário.

### 2. O que permanece do ADR 0017

- O modelo de estado puro (MVU): a tela é derivada do estado, e cada evento é
  uma função `estado -> estado`. Numa tela, isso aparece como handler
  `JanelaPrincipal -> JanelaPrincipal`.
- O runtime de sessão visual da VM (`crates/ferro-vm/src/ui.rs` e
  `crates/ferro-service/src/ui_session.rs`): orçamento por evento, limite de
  nós e profundidade, histórico de estados e cancelamento pelo host. Ele passa
  a conduzir telas.
- A VM FERRO em WASM para prévia instantânea no navegador e no WebView, sem
  exigir `rustc`.
- O renderizador DOM da prévia, que evolui para desenhar a árvore do `.design`.

Sai apenas a superfície do `UI.*`: as funções `UI.*` da biblioteca padrão, a
convenção `iniciar`/`atualizar`/`visualizar` com eventos por cadeia e o runtime
JSON gerado pelo `ferro-rust`.

### 3. Etapas da aposentadoria

**Etapa A — imediata.**
- Nenhum conteúdo novo usa `UI.*`.
- Ajuda, dicas e autocompletar marcam o `UI.*` como obsoleto e apontam para os
  projetos desktop.
- O ADR 0017 passa a indicar que sua decisão 1 foi substituída por este ADR.

**Etapa B — quando a Fase 3 do ADR 0018 estiver entregue** (code-behind e
execução de telas na IDE e na prévia WASM):
- `08-ui-contador.fer` vira um projeto de exemplo com `.design` e `.fer`.
- A prévia visual da IDE passa a abrir telas.
- O playground Web ganha duas abas, *Design* (o `.design`, com prévia) e
  *Código* (o `.fer`), e o link de compartilhamento guarda as duas.
- Programas com `UI.*` ainda compilam, mas recebem o aviso `ferro.ui-obsoleto`
  com a explicação da troca e um exemplo equivalente em `.design`.

**Etapa C — uma versão publicada depois da Etapa B:**
- As funções `UI.*` saem da biblioteca padrão. O uso gera erro com a mesma
  explicação da Etapa B.
- Saem o runtime `FERRO_UI_RUNTIME` do `ferro-rust`, os construtores de nós
  `UI.*` da VM e os testes que só existem para eles.
- Os testes do runtime de sessão visual continuam, reescritos sobre telas.

> **Nota (2026-11-28): Etapa C antecipada.** Por decisão do mantenedor, a
> Etapa C entrou sem esperar uma versão publicada com o aviso da Etapa B
> (a v0.7.0 saiu antes do aviso). Também saiu o modo visual do playground Web
> (`ferro_ui_*` no `ferro-wasm`), que dependia do `UI.*`: o playground fica só
> com programas de console até ganhar as abas de telas da Etapa B. O uso de
> `UI.*` gera o erro `ferro.ui-removido`, com a tabela da seção 4 na ajuda, e
> a entrada `iniciar`/`atualizar`/`visualizar` deixou de substituir `inicio`.

### 4. Correspondência para a migração

| `UI.*` | `.design` e `.fer` |
|---|---|
| `UI.coluna([...])` | `<Pilha>` |
| `UI.linha([...])` | `<Pilha Orientacao="Horizontal">` |
| `UI.texto(...)` | `<Rotulo Texto="..." />`, com `{dados.campo}` para valores |
| `UI.botao("inc", "+1")` | `<Botao Nome="btnInc" Texto="+1" AoClicar="btnInc_clicar" />` |
| `UI.entrada("nome", valor, "dica")` | `<CaixaTexto Nome="txtNome" TextoVazio="dica" />` |
| `UI.lista([...])` | `<Pilha>` com os itens, ou `<Lista>` com `<Item>` |
| `atualizar(estado, alvo)` com `se (alvo == ...)` | um handler por evento |
| `registro Estado` | `registro` indicado em `Dados="..."` na `<Janela>` |

O texto do aviso `ferro.ui-obsoleto` e a ajuda usam esta tabela.

## Alternativas consideradas

### Manter `UI.*` e telas lado a lado (versão inicial do ADR 0018)
Rejeitada pelos motivos do contexto: dois vocabulários, dois modelos de evento
e duas exportações para o aluno e para a manutenção.

### Unificar como "modo código" e "modo design" do mesmo modelo
Rejeitada. Mesmo com um esquema comum, o aluno continuaria vendo duas formas de
escrever a mesma tela e teria que aprender a conversão entre elas. A separação
marcação + código, como no WPF, já é a forma única.

### Retirar o `UI.*` de imediato
Rejeitada. A prévia visual da IDE e o playground dependem dele hoje. Retirar
antes de as telas executarem na IDE e no navegador deixaria essas superfícies
sem conteúdo visual.

### Manter `UI.*` só no playground
Rejeitada. O playground é a porta de entrada de muitos alunos; ensinar lá uma
forma que não existe na IDE criaria exatamente a confusão que este ADR evita.

## Consequências

- O ADR 0018 (Emendas 1 e 2) passa a tratar a tela como a forma única de
  interface, e sua Fase 3 inclui a migração da prévia da IDE e do playground.
- Superfícies afetadas pela aposentadoria: `crates/ferro-rust-frontend`
  (`parser/stdlib.rs`), `crates/ferro-vm` (`ui.rs` e `tests/ui_runtime.rs`),
  `crates/ferro-service` (`ui_session.rs`, `ferro_examples.rs` e o exemplo
  `08-ui-contador.fer`), `crates/ferro-rust` (`codegen/runtime.rs`),
  `crates/ferro-wasm`, `crates/ferro-cli` (testes), a IDE
  (`ferro-ui-*.ts`, `ferro-wasm-host.ts`, `ferro-help.ts`,
  `ferro-stdlib-catalog.ts`, `ferro.ts`, dicas e testes) e o playground
  (`app.mjs`, `playground-core.mjs`, `index.html`).
- Programas `.fer` escritos por alunos com `UI.*` deixam de compilar na
  Etapa C. O aviso da Etapa B dá pelo menos uma versão publicada de transição.
- Nenhuma mudança na biblioteca `Graficos`, nos programas de console nem nos
  gates de `.por`/`.pex` do ADR 0016.
