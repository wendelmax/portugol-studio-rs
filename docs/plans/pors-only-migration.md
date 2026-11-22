# Migração para PORS e Rust, com retirada de `.por`

Este plano implementa o [ADR 0016](../adr/0016-pors-como-linguagem-autoral-e-retirada-do-por.md).
O nome do produto não muda. As pastas `portugol-studio-next` e
`portugol-studio-next-integration` servem como referências de requisitos e
design e serão conciliadas/removidas conforme a solicitação existente.

## Baseline auditado em 2026-11-22

### Já disponível em PORS/Rust

- Arquivos `.pors`, parser próprio, cinco tipos primitivos, bindings imutáveis
  por padrão e `mutavel` explícito.
- Declaração, atribuição, expressões, `escreva`, `leia`, `se/senao`,
  `enquanto`, `para`, `pare` e `continue`.
- Execução normal através de Rust gerado e `rustc`; fonte mapeada a Rust com
  diagnósticos do subset suportado.
- Depurador `.pors` sobre HIR → bytecode → VM compartilhada, com escopos,
  breakpoints, stepping, entrada, limites tipados e cancelamento.
- Rust standalone e Cargo reais, `rustfmt`, `rustc` diagnostics e argumentos.
- Shell Tauri, CodeMirror, workspace, temas, painéis, source maps e preferências
  por workspace.

### Lacunas que bloqueiam a retirada do dialeto clássico

| Domínio | Estado PORS/Rust | Trabalho para fechar |
|---|---|---|
| Linguagem | Um `inicio`, primitivos e controle básico | Funções, parâmetros/retornos, chamadas, coleções, indexação, comandos ausentes e diagnósticos completos |
| Conceitos de Rust | Mutabilidade explícita; modo Rust real; ownership em simulação editorial | Ownership/borrowing verificáveis no código do aluno, estruturas/enums, `Option`/`Result`, pattern matching e explicações ligadas a diagnósticos reais |
| Bibliotecas | I/O primitiva; execução e limites próprios | Portar/adaptar bibliotecas úteis com contratos seguros em PORS, Rust gerado e debugger |
| Conteúdo | Conteúdo Progressivo parcial | Exemplos, ajuda, onboarding, soluções de referência e explicações próprias para PORS/Rust |
| Exercícios | Correção e progresso de `.pex` clássico | Formato versionado e avaliador PORS/Rust; progresso só por testes aprovados |
| IDE | Muitos fluxos PORS e Rust já existem | PORS como idioma padrão em novo arquivo, detecção estrita, formatador, completions, snippets, explorer, importação/exportação e mensagens sem fallback para `.por` |
| CLI e distribuição | CLI clássico; Tauri reconhece `.por`/`.pex`/`.pors`/`.rs` | CLI PORS/Rust, associação e filtros só PORS/Rust, instaladores e documentação atualizados |

Inventário de recursos originais: [`PARIDADE.md`](../PARIDADE.md). A matriz é
histórica e não é prova de paridade PORS; cada linha precisa ganhar status
próprio na próxima revisão da documentação.

## Ordem e gates

### Fase 0 — Inventário verificável

- Reconciliar cada linha da matriz de paridade e os requisitos `DESIGN_AS_CODE`,
  contratos funcionais e features Gherkin existentes.
- Marcar cada capacidade como `PORS pronto`, `PORS parcial`, `ausente`,
  `adaptado` ou `não aplicável`, com teste e owner do próximo incremento.
- Registrar formatos, bibliotecas, recursos com licença e caminhos de produção
  que ainda chamam parser/semântica/runtime clássicos.

**Gate:** não iniciar remoção de código clássico enquanto a matriz tiver linha
sem decisão, evidência ou critério de aceite.

### Fase 1 — Fechar a linguagem PORS

1. Expandir HIR e frontend para funções declaradas pelo aluno, parâmetros,
   retornos e chamadas; definir erros, ordem, escopo e recursão.
2. Adicionar coleções e acesso/indexação com limites e spans; depois completar
   controles de fluxo de uso educacional, mantendo sintaxe PORS em português e
   mutabilidade explícita.
3. Introduzir structs/enums, correspondência de padrões e tipos algébricos,
   incluindo `Option`/`Result` com propagação segura e diagnósticos didáticos.
4. Para cada fatia: teste falhando primeiro, lower à HIR, execução VM e Rust
   gerado quando aplicável, limites, diagnósticos em português, source maps e
   comparação observável.

**Gate:** todos os recursos publicados como suportados funcionam em execução
normal e debugger; gramática fora do subset gera erro localizado.

### Fase 2 — Ganhos de Rust com evidência

1. Substituir cenários roteirizados do laboratório de ownership por análise do
   buffer PORS/Rust ou por trace real da execução identificada com sua
   proveniência.
2. Ensinar ownership, moves, empréstimos imutáveis/mutáveis, escopo, cópia e
   clone com exemplos progressivos compilados/testados por `rustc`.
3. Acrescentar módulos, organização de projeto e ferramentas Cargo conforme
   cada conceito tiver um caminho seguro no produto.

**Gate:** UI nunca apresenta simulação como estado real; explicação e diagnóstico
apontam para código e revisão atuais.

### Fase 3 — Bibliotecas e capacidades da IDE

- Portar bibliotecas do catálogo clássico em grupos verificáveis, com API PORS
  idiomática e adaptadores Rust; funções sem equivalente seguro recebem decisão
  de produto em vez de stub enganoso.
- Reimplementar em PORS/Rust os fluxos mantidos de gráficos, teclado, mouse,
  som, arquivos, calendários, JSON/XML, rede e serviços, observando capabilities,
  consentimento e limites atuais.
- Fechar editor: autocompletion, hover/help, outline, rename/go-to-definition,
  formatador, source maps, debugging, workspace e testes.

**Gate:** matriz por biblioteca/API com assinatura, semântica, efeito, limites,
backend normal/debug e testes multiplataforma.

### Fase 4 — Aprendizagem e conteúdo PORS/Rust

- Fazer `.pors` o template e idioma padrão no onboarding e na criação de arquivo.
- Substituir exemplos e ajuda do Portugol clássico por material PORS original,
  com créditos/licenças mantidos para qualquer conteúdo reaproveitado.
- Criar exercícios em formato próprio e versionado, com starter, conceitos,
  casos de teste, dicas e origem; avaliá-los no serviço sem comando de shell do
  frontend. Suportar PORS primeiro, Rust quando houver runner isolado e aceito.
- Migrar progressão de iniciante → ponte → Rust sem mudar fonte ao trocar de
  modo. Persistir domínio só após resultado verificável da versão avaliada.

**Gate:** todas as ações de prática levam a exercícios executáveis; nenhum
progresso nasce de abrir conteúdo, executar sem aprovação ou conteúdo editorial.

### Fase 5 — CLI, projetos e experiência PORS/Rust por padrão

- Tornar o CLI capaz de criar, verificar, executar e depurar `.pors` e projetos
  `.rs`/Cargo; remover o default implícito para Portugol clássico.
- Fazer detecção de linguagem estrita: `.pors`, `.rs` e formatos de exercício
  conhecidos; extensão desconhecida não vira `.por` por fallback.
- Migrar explorer, exportação, diálogo de arquivos, OS associations, drag/drop,
  mensagens, docs e instaladores.

**Gate:** nenhum fluxo de produto interpreta silenciosamente `.por`/`.pex`.

### Fase 6 — Remover legado e lançar

- Confirmar todos os gates das Fases 1–5 e revisar a busca de referências.
- Excluir comandos Tauri clássicos, analisador/parser/compiler sem consumidores,
  CLI clássico, exercícios `.pex`, assets/corpus clássicos e associação `.por`.
- Manter apenas estruturas da VM compartilhada requeridas por PORS/debugger.
- Atualizar README curto para usuários, créditos, licença, PARIDADE histórica,
  distribuição e notas de release. Avisar que `.por`/`.pex` não são abertos e
  indicar o Portugol Studio original para esses arquivos.

**Gate:** CI Linux/Windows/macOS, smoke Tauri no Linux/KDE, CLI offline, criação
de PORS, execução/debug, exercícios, Rust standalone/Cargo e ausência de
referências de suporte a `.por`/`.pex` em runtime/docs voltados ao usuário.

## Políticas transversais

- Não renomear o produto baseado em `portugol-studio-next`.
- Não apagar arquivos do usuário nem tentar convertê-los silenciosamente.
- Não ampliar o dialeto para imitar sintaxe Rust sem benefício pedagógico
  explicável. PORS é linguagem autoral; `.rs` é Rust real.
- Não copiar código ou assets GPL do Webstudio; manter revisões de licença.
- Preservar Rust + Tauri desktop e os contratos de workspace/processos seguros.
- Criar PRs por feature independente; só mesclar CI completamente verde, sem
  coautoria ou trailers “Generated by”.
