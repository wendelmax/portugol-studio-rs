# Design — Navegação bidirecional no editor Ferro–Rust

## Intenção

Na aprendizagem bridge, mostrar de forma visível a correspondência entre a linha selecionada em Ferro/Ferro e as linhas de Rust gerado. O aluno deve conseguir partir da origem e chegar ao trecho gerado, e continuar podendo voltar do Rust para a origem. O recurso usa os source maps reais já retornados pelo backend; não tenta inferir alinhamento por texto ou linha parecida.

## Estado e restrições atuais

- `apps/desktop/src/main.ts` apresenta a origem em CodeMirror e a prévia Rust em um `<pre>` somente leitura, lado a lado no modo bridge.
- Trechos Rust com source map são botões focáveis que navegam de volta para a origem.
- `SourceMapInfo` inclui linhas, posições UTF-16 e qualidade; o contrato Rust/Tauri já é usado por exportação `.por` e `.fer`.
- Respostas antigas da tradução são descartadas por geração; o código gerado não substitui o documento da origem.
- O frontend é TypeScript + Vite, e os testes UI atuais usam Node test runner e exercitam utilitários puros e contratos estáticos.

## Objetivos

- Mostrar no vão entre os editores os pares de linhas que têm mapeamento verificável para a linha atual da origem.
- Ao ativar um par, focar e rolar até o trecho correspondente no Rust gerado.
- Preservar o caminho já existente de um trecho Rust para sua localização na origem.
- Identificar mapeamentos aproximados e declarar quando a linha selecionada não possui relação direta.
- Manter navegação por teclado, nomes acessíveis e escopo limitado ao modo bridge.

## Não objetivos

- Não alterar comandos Tauri, geração Rust, semântica, sintaxe, associação `.fer` ou mapa IPC.
- Não editar a origem nem transformar a prévia em arquivo independente.
- Não alegar correspondência em segmentos `generated_only` ou não mapeados.
- Não adicionar dependência, editor novo, alinhamento textual heurístico ou resultado simulado.

## Abordagens

1. **Trilho de correspondência por source maps (escolhida):** um gutter estreito lista `origem ↔ Rust` para a linha ativa e aponta ao botão mapeado no painel gerado. Usa a infraestrutura atual, é explícito e não ocupa o editor com uma segunda instância CodeMirror.
2. **Realçar automaticamente linhas dos dois editores:** exigiria controlar seleção/decoração em CodeMirror na origem e montar uma navegação equivalente no `<pre>`; mais estado e foco implícito sem acrescentar informação aos source maps.
3. **Mostrar uma tabela completa de toda a tradução:** pode crescer proporcionalmente ao programa e desfocar a relação com o trecho que o aluno está estudando.

## Decisão de interação

O gutter é visível somente quando a prévia Rust está aberta. Ele observa a linha atual do cursor/seleção e exibe pares únicos extraídos de source maps com origem válida. O par move foco para o botão correspondente no Rust gerado; não move nem altera o cursor de edição. Se não houver par, uma mensagem de estado anuncia que a linha não tem correspondência direta. A qualidade `approximate` fica explícita no nome/tooltip do botão.

## Arquivos previstos

- `apps/desktop/index.html`: gutter com `nav`, status acessível e `data-testid` estável.
- `apps/desktop/src/source-map-ui.ts`: derivação e deduplicação dos links a partir de `SourceMapInfo`.
- `apps/desktop/src/main.ts`: atualização da linha ativa, rendering de links, foco/scroll bidirecional e vínculo com a geração atual.
- `apps/desktop/src/style.css`: trilho compacto coerente com temas e layout flexível.
- `apps/desktop/tests/source-map.test.mjs`: casos de mapeamentos exatos, aproximados, sem origem, deduplicados e contrato visual/teclado.
- `docs/PARIDADE.md` e `docs/reports/FT-15-ferro-mapping-gutter.md`: registrar escopo e validação.

## Critérios de aceite

- Uma linha com um ou mais source maps mostra somente pares reais, únicos e ordenados.
- Linhas sem origem ou com `generated_only` nunca criam botão de navegação.
- Ativar link no gutter foca e rola o trecho de Rust correspondente; ativar trecho Rust ainda navega à origem.
- Alterar documento, trocar aba ou fechar o preview não mantém mapa de uma geração antiga.
- UI informa a ausência de correspondência e a localização aproximada sem prometer equivalência semântica.
- Testes Node, build frontend, check Tauri, suíte Rust e CI permanecem verdes.
