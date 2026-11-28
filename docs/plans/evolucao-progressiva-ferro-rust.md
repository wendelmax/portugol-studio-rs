# Plano de evolução progressiva: Ferro → Rust

**ADR:** [0001 — Webstudio como referência e ponte progressiva](../adr/0001-webstudio-benchmark-e-ponte-progressiva-rust.md)
**Pesquisa:** [benchmark Ferro Webstudio](../research/ferro-webstudio-benchmark-2026-11-21.md)
**Status:** proposta técnica inicial; cada etapa deve ser entregue em PR próprio com critérios de aceite.

> **Atualização em 2026-11-22:** este documento registra o plano inicial e seu
> estado descritivo não é mais o inventário vigente. A implementação avançou
> para runtime `.fer`, modos por workspace, Rust standalone/Cargo, source maps,
> exercícios validados e debugger FERRO. Consulte
> [`docs/research/REPOSITORY_DISCOVERY_CURRENT.md`](../research/REPOSITORY_DISCOVERY_CURRENT.md),
> [`docs/PARIDADE.md`](../PARIDADE.md) e
> [`docs/plans/repriorizacao-produto-progressivo-2026-11-22.md`](repriorizacao-produto-progressivo-2026-11-22.md)
> antes de escolher uma etapa. A sequência antiga abaixo é histórica e foi
> substituída pelo plano de repriorização.

## Resultado visado

A pessoa aprende um conceito em Ferro, vê a tradução equivalente e gradualmente passa a usar uma sintaxe explícita inspirada em Rust, até conseguir compilar código Rust verdadeiro. A IDE preserva a compatibilidade do Ferro Studio, oferece diagnósticos em português e visualiza posse/empréstimos somente onde as regras do Rust realmente se aplicam.

## Estado atual confirmado

- Lexer Rust (`logos`), parser recursivo, AST com spans e recuperação de erros.
- Analisador Ferro de símbolos, escopos, tipos e diagnósticos.
- Bytecode e VM próprios com debugger, step e inspeção de variáveis.
- Não há HIR tipada compartilhada, análise de ownership Rust ou chamada de `rustc`.
- A IDE já tem exemplos, documentação, ajuda, temas, 54 exercícios corrigidos e importação `.pex`.
- O benchmark Webstudio anuncia 75 exercícios, progresso local, autoria e compartilhamento de exercícios sem a solução.

## Fase 0 — consolidar o produto Ferro

**Objetivo:** remover fricção educacional antes de introduzir outra linguagem.

1. Revisar o `Sobre`: nomes e links de autores/mantenedores, UNIVALI/LITE, licença do app e créditos/licenças dos recursos. Atribuir separadamente fontes originais e inspiração de UX.
2. Reutilizar os símbolos semânticos para tooltip de declaração/comentário, F12 e Ctrl/Cmd+clique até a declaração.
3. Comparar os 54 exercícios locais com os 75 do benchmark e decidir quais conteúdos faltam após deduplicação e validação de correção.
4. Projetar autoria local de `.pex`: enunciado, código inicial, solução privada, exemplos, testes, entradas, critérios de texto/valor/função e geração de resultados pela VM.
5. Validar progresso persistente e recuperação de estado para exercícios próprios/importados.
6. Fazer uma revisão acessível com teclado e leitor de tela nos diálogos, editor, sugestões, debugger e correção.

**Critérios de aceite:** os créditos estão rastreáveis e corretamente atribuídos; a navegação por símbolo respeita escopo; exercício criado localmente contém testes reproduzíveis e não distribui a solução por acidente; os testes continuam offline.

## Fase 1 — infraestrutura semântica comum

**Objetivo:** permitir mais de um frontend sem fragmentar o compilador.

1. Especificar uma HIR tipada para programas, declarações, funções, fluxo de controle, expressões, tipos e spans.
2. Criar lowering de AST Ferro para HIR mantendo todos os testes do corpus e sem alterar resultados da VM.
3. Anotar tipos e bindings estáveis na HIR; separar erros sintáticos/semânticos dos dados consumidos pela IDE.
4. Definir conceitos curriculares e IDs estáveis, como variável, mutabilidade, função, controle, tipo e coleção, para relacionar exercício, Ferro, Ferro e Rust.

**Critérios de aceite:** os programas atuais geram HIR tipada; os resultados dos testes diferenciais não mudam; todos os diagnósticos preservam spans/fontes.

## Fase 2 — ponte Ferro → Rust real

**Objetivo:** entregar o primeiro caminho verificável de um programa existente a código Rust compilável.

1. Definir um subconjunto inicial explícito: bloco principal, inteiros/reais/lógicos/cadeias, declaração, atribuição, expressões aritméticas/lógicas, `escreva` e condicionais simples.
2. Implementar geração Rust a partir da HIR, não substituição de palavras. Traduzir semântica de `escreva`, strings, tipos numéricos, entrada/saída e variáveis com regras documentadas.
3. Retornar o código gerado e um mapa de fonte Rust ↔ Ferro.
4. Mostrar Ferro e Rust lado a lado, com diferenças sincronizadas por declaração/expressão/controle e indicação de construções sem tradução.
5. Compilar casos gerados com `rustc` em testes e apresentar diagnósticos do compilador em português, ligados ao trecho Ferro quando possível.

**Critérios de aceite:** nenhum caso fora do subconjunto gera Rust como se fosse válido; programas suportados compilam; testes executam as duas versões e comparam resultados nos limites semânticos especificados.

## Fase 3 — dialeto opcional Ferro

**Objetivo:** introduzir conceitos de Rust de forma gradual sem mudar arquivos existentes.

1. Escrever a gramática e os exemplos de Ferro separadamente, incluindo `seja`, `mutavel`, tipos explícitos e referências ensináveis.
2. Criar frontend próprio e lower para HIR; diferenciar mutabilidade ausente, explícita e imutável.
3. Criar regras pedagógicas por modo de aula, progressivas e com diagnósticos acionáveis em português.
4. Mapear cada exercício progressivo para versões de Ferro, Ferro e Rust.

**Critérios de aceite:** `programa .por` mantém comportamento e parsing; a sintaxe Ferro só ativa no modo escolhido; cada diagnóstico tem teste positivo, negativo e span exato.

## Fase 4 — laboratório de ownership e borrowing

**Objetivo:** tornar visível por que um valor pode ser movido ou emprestado.

**Primeira fatia UI-027:** o painel Aprender oferece cenários editoriais roteirizados para `String`, move, `&T`, `&mut T` e escopo. Os exemplos são conferidos com fixtures `rustc`, mas o estado exibido é uma simulação didática, não um trace nem uma análise do código aberto. A fase continua aberta até existir análise de eventos/spans para o subconjunto de Rust definido abaixo.

1. Definir primeiro um subconjunto explícito de Rust (String, bindings, escopos, `&T`, `&mut T`, chamadas e retornos).
2. Criar análise de eventos de memória e spans: criação, move, empréstimo compartilhado, empréstimo mutável, uso, fim de escopo e erro.
3. Apresentar timeline/visualizador junto do código e controles passo/reiniciar.
4. Comparar cada diagnóstico/sucesso com `rustc` em casos de referência; deixar claros os limites do subconjunto.
5. Não simular ownership no runtime Ferro nem declarar que o simulador executa Rust completo.

**Critérios de aceite:** cada exemplo do laboratório tem um caso Rust compilável ou um erro de `rustc` equivalente; movimentos e empréstimos são mostrados pelo span certo; exemplos inválidos não chegam a execução como se fossem seguros.

## Fase 5 — compilação/exportação Rust

**Objetivo:** permitir continuar o aprendizado fora da IDE.

1. Exportar `.rs` com origem, imports, `main` e mapa de regiões geradas.
2. Oferecer verificação/execução local se `rustc` estiver instalado, com opção de cancelar e limites de tempo/tamanho.
3. Explicar instalação do toolchain e apresentar versão de `rustc` sem descarregar binários automaticamente.
4. Manter a exportação funcionando offline.

**Critérios de aceite:** o arquivo exportado compila por conta própria para todos os programas dentro do subset; a ausência de `rustc` não quebra a IDE nem impede a prévia.

## Requisitos transversais

- Preservar corpus, semântica e atalhos do pseudocódigo clássico.
- Sem telemetria ou envio de código obrigatório. Compartilhamento de conteúdo é ação explícita, com aviso de público, prazo e conteúdo removido.
- Exercícios docentes não publicam a solução.
- Português é o idioma de diagnósticos pedagógicos; oferecer o texto original de `rustc` como detalhe recolhível.
- Cada release separa recursos prontos de protótipos limitados.
- Revisar licenças de assets, snippets e conteúdo antes de reutilizar materiais externos.
- Linux, Windows e macOS continuam suportados no núcleo e na IDE.

## Ordem recomendada de entrega

1. Terminar as mudanças visuais atuais e publicar a revisão da janela Sobre/créditos como uma pequena PR.
2. Implementar navegação por declaração com os símbolos existentes.
3. Entregar inventário de exercícios restantes e definição de formato/autor local.
4. Abrir PRs pequenos da HIR, lowering e testes, preservando o corpus em cada etapa.
5. Só então começar gerador de Rust e comparação lado a lado.
6. Desenvolver Ferro e o laboratório de ownership como trilhas opt-in após o MVP de tradução compilar.
