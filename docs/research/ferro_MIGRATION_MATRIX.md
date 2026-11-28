# Matriz de migração para FERRO/Rust

Snapshot-base: `main` em `140a846` (PR #97; 22/11/2026), já com funções FERRO.
Esta matriz é o gate vivo do
[ADR 0016](../adr/0016-ferro-como-linguagem-autoral-e-retirada-do-por.md).

Legenda: ✅ já coberto em FERRO/Rust · 🟡 parcial · ⬜ ausente · 🔎 auditar e
decidir equivalência/escopo. Um ✅ exige testes na implementação FERRO/Rust,
não apenas suporte ao pseudocódigo clássico.

## Linguagem e execução

| Capacidade | FERRO/Rust hoje | Lacuna / critério para fechar |
|---|---|---|
| Arquivo e entrada | ✅ `.fer`, `funcao inicio()` sem argumentos | Criar novo arquivo `.fer` por padrão; CLI FERRO ainda ausente |
| Tipos | 🟡 inteiro, real, lógico, cadeia, caracter; imutabilidade padrão e inferência em `seja nome = expressao` (testada em frontend, bytecode, VM e Rust gerado) | 🔎 Semântica explícita de tamanhos, overflow, Unicode, unit/void; inferência permanece local ao inicializador |
| Variáveis | ✅ `seja`, `seja mutavel`, atribuição e shadowing testados | Validar contrato comum em VM e Rust gerado em todos os caminhos |
| Expressões | ✅ aritmética, bitwise/shifts (`&`, `|`, `^`, `~`, `<<`, `>>`, `&=`, `|=`, `^=`), comparação, igualdade, lógica e curto-circuito com paridade VM/Rust; interpolação `{expressao}` com spans próprios, `leia()` com conversão implícita ao tipo esperado e `escreval` com quebra final (frontend, bytecode, VM, WASM e Rust gerado) | Manter alinhamento se novos tipos numéricos forem expostos; 🔎 largura/overflow, Unicode e heap da interpolação continuam adiados |
| Condições | 🟡 `se`/`senao` e `escolha` exaustivo para enums, `Opcao`/`Resultado` e ramo coringa (`caso _ =>`) com detecção de casos inalcançáveis; sem guards ou matching como expressão | Referências, padrões aninhados e guards |
| Repetição | ✅ `enquanto`, `faca-enquanto`, `para`, `pare`, `continue` com paridade VM/Rust | Revisar loop labels apenas se o currículo demandar |
| Funções | 🟡 `inicio` e funções auxiliares com parâmetros tipados, chamadas, `retorne`, retorno implícito apenas na expressão final do corpo de função tipada, recursão, chamadas para frente e mutabilidade explícita; Rust gerado, debugger VM, outline, navegação, rename e formatador FERRO | Faltam cobertura aprofundada de frames/limites de recursão e referências/borrowing explícitos; RFC 007 (`tente` como expressão) não implementada nesta fatia |
| Coleções | 🟡 Vetores tipados homogêneos com construção vazia tipada (`[]`), redimensionamento dinâmico (`adicionar`, `remover` -> `Opcao<T>`), leitura segura (`obter` -> `Opcao<T>`), leitura/escrita indexada, `tamanho`, cópia por valor e bounds; sem matrizes, slices/visões ou vetores aninhados | Matrizes e outras coleções (mapas/conjuntos); slices/visões e iteração permanecem decisões futuras |
| Tipos compostos | 🟡 Registros/enums nominais, campos, construtores, vetores, mutação sob binding mutável, cópia por valor e `escolha` exaustivo; VM/debugger expande campos e payload ativo; Rust gerado usa `struct`/`enum` | Sem recursão de tipos, `Box`, vetores aninhados, mutação indexada/nested, campos mutáveis individuais ou guards; validar mais profundamente a equivalência de limites e diagnósticos |
| Rust `Option`/`Result` | ✅ `Opcao<T>` (`Algum`/`Nenhum`), `Resultado<T, E>` (`Ok`/`Erro`), `escolha` exaustivo, propagação com `?` e geração nativa para `Option<T>`/`Result<T, E>` em Rust, testados em frontend, VM e `rustc` | Expandir combinadores/métodos auxiliares quando biblioteca padrão FERRO evoluir |
| Ownership/borrowing | 🟡 painel editorial com fixtures; sem analisar buffer do aluno | Moves, empréstimos `&T`/`&mut T`, regras de aliasing/lifetimes do subset; comparar com `rustc` |
| Concorrência Rust | ⬜ | Introduzir threads e compartilhamento seguro (`Arc`/`Send`/`Sync`), depois `Mutex<T>` para estado composto e `Atomic*` para operações simples em valores compatíveis; explicar `Ordering` só após sincronização básica |
| Diagnósticos | 🟡 códigos e spans localizados em subset, incluindo `ferro.interpolation-invalid`, `ferro.interpolation-non-string`, `ferro.read-arguments-unsupported`, `ferro.implicit-return-without-type` e `ferro.return-type-mismatch` (spans do trecho ofensor); leitura que não converte ao tipo esperado é erro de execução com mensagem sobre o tipo, não valor padrão silencioso nem erro estático de tipo | Cobertura de cada sintaxe/semântica; português, ajuda contextual e source maps; nomes não resolvidos em interpolação permanecem literais por compatibilidade |
| Execução normal | 🟡 gera Rust subset e executa `rustc` | Igualdade observável entre todas as construções publicadas e casos diferenciados |
| Debugger | 🟡 HIR → bytecode → VM; breakpoints, stepping, escopos, input, limites; FERRO mostra campos de registros e variante/payload de enum; a pausa passa a exibir nó HIR, offset de bytecode e linha do Rust gerado para as novas construções (`escreval`, interpolação, `leia()`, retorno implícito) | Cobrir a gramática que for expandida; documentar diferenças inevitáveis de backend; WASM não expõe source map e a equivalência conceitual resumida continua fora desta fatia |
| Segurança/limites | ✅ supervisor de Rust gerado e limites do debugger FERRO | Manter saída/tempo/memória/cancelamento e consentimento em todo recurso novo |

## Migração para a sintaxe evoluída

| Ponto | Registro |
|---|---|
| Leitura | As duas formas convertem implicitamente: a forma com variável-alvo converte ao tipo declarado da variável; `leia()` converte ao tipo esperado pelo contexto (declaração anotada, argumento tipado, atribuição, retorno e campo) e resulta em `cadeia` sem contexto. Entrada que não converte gera erro de execução com mensagem sobre o tipo esperado, sem valor padrão silencioso. |
| Preferência nos exemplos | Exemplos migrados preferem `leia()` — `seja x: tipo = leia();` ou `x = leia();`. A forma com variável-alvo permanece válida e fica restrita à compatibilidade e a laços com leitura repetida. |
| Conversão de `caracter`/`logico` | `caracter` exige exatamente um caractere, não apenas o primeiro da linha; `logico` aceita espaços ao redor. Leituras escalares passam por `trim` antes da conversão; `cadeia` permanece bruta. Exemplos que dependiam do comportamento antigo foram ajustados. |
| Fronteira de contexto | O tipo esperado é propagado a inicializador, argumento, atribuição, retorno e campo. Em operação binária como `1 + leia()`, o lado direito é analisado sem contexto (`target=None` → `cadeia`); `target=Some(Integer)` ocorre em `seja n: inteiro = leia();`, não dentro da soma. Exemplos não usam `1 + leia()`. |
| Rótulo divergente pré-existente | Em `crates/ferro-rust/src/codegen/stmt.rs:216`, a leitura com variável declara `lógico` na mensagem de erro; em `crates/ferro-rust/src/codegen/types.rs:100`, a leitura como expressão declara `logico`. A divergência é pré-existente, registrada aqui e não corrigida nesta fatia: `stmt.rs` não é alterado. |
| Exceção `UI.entrada` | Campo de texto bruto continua usando `Tipos.para_*` com `Resultado`/`escolha` quando o objetivo é ensinar validação explícita; a conversão implícita da leitura não substitui esse caminho. |
| Interpolação | A RFC 004 aceita só `cadeia` e `caracter` dentro de `{...}`. Números, lógicos, vetores, registros, enums e `vazio` geram `ferro.interpolation-non-string`; valores não textuais em `escreval` são argumentos extras, não interpolação. |

### Cobertura do gate e allowlist

- O gate percorre os 211 arquivos `.fer` e os exemplos embutidos. Desses, 172 programas completos são compilados; os outros 39 são fragmentos referenciados por blocos `data-type="sintaxe"` da ajuda, sem `funcao inicio()`, e por isso não são compilados isoladamente.
- O marcador `escreva-com-quebra-explicita` detecta apenas chamadas de `escreva` cuja lista de argumentos contém uma cadeia com `\n` explícito. `escreva` sem quebra continua sintaxe válida e foi preservado byte a byte. A allowlist mantém 15 arquivos do catálogo e 1 exemplo de ajuda (16 entradas, 32 chamadas) porque `escreval` acrescentaria uma quebra final e alteraria o stdout esperado — inclusive quando a quebra está no início ou no meio da cadeia.
- Quatro laços do catálogo e dois laços da ajuda com leitura repetida usam a forma com variável-alvo de `leia` e estão na allowlist, junto com duas entradas HTML de compatibilidade (`entrada_saida/index.html` e `entrada_saida/leia.html`). Nenhuma entrada existente foi removida ou enfraquecida, e nenhuma entrada documental nova foi acrescentada.

## Bibliotecas do inventário clássico

Para cada grupo, revisar a API completa em `docs/PARIDADE.md`, escolher a API
FERRO idiomática, seu backend (VM e/ou Rust), efeitos, consentimentos e limites.
Não considerar uma biblioteca portada por implementar apenas o nome do módulo.

| Biblioteca clássica | Estado em FERRO/Rust | Decisão pendente |
|---|---|---|
| Matematica | ✅ `Matematica` em FERRO/Rust (constantes `PI`/`E` e 16 funções tipadas sem coerção implícita, em VM mínima, WASM e Rust gerado) | Expandir funções trigonométricas inversas/hiperbólicas se o currículo demandar |
| Texto | ✅ `Texto` em FERRO/Rust (13 funções Unicode-safe por `char`, incluindo `dividir` -> `vetor<cadeia>`, `Opcao`/`Resultado` em busca/indexação/subtexto e orçamento de memória verificado antes de alocar, em VM mínima, WASM e Rust gerado) | Manter alinhamento com futuros utilitários de formatação quando necessários |
| Tipos | ✅ `Tipos` em FERRO/Rust (19 funções explícitas de conversão e validação com `Resultado<T, cadeia>` para parses falíveis e bases 2/10/16, em VM mínima, WASM e Rust gerado) | Manter alinhamento com futuros tipos numéricos se `i64`/`u32` forem expostos |
| Util | ✅ `Util` em FERRO/Rust (`sorteia` -> `Resultado<inteiro, cadeia>`, `definir_semente`, `tempo_decorrido`, `aguarde` cancelável e limitado a 60 s, em VM mínima, WASM e Rust gerado) | Manter fora funções dependentes do host/OS (como `obter_diretorio_usuario`) até `Arquivos` com consentimento |
| Calendario | ✅ `Calendario` em FERRO/Rust (contrato UTC determinístico zero-dependência com 19 constantes e 15 funções incluindo `dia_semana_*`, `nome_mes`, `eh_bissexto`, `dias_no_mes` e `data_iso_atual`, em VM mínima, WASM e Rust gerado) | Avaliar formatação localizada/fusos opcionais apenas se o currículo exigir |
| Arquivos | ✅ `Arquivos` em FERRO/Rust confinado ao workspace (`ler_texto`, `ler_linhas`, `escrever_texto`, `acrescentar_texto`, `existe`, `remover`, `listar`), bloqueando caminhos absolutos/`..`/escape por symlink, limite de 2 MiB + orçamento de cadeias e erros em `Resultado<T, cadeia>` (VM mínima, WASM com erro tipado e Rust gerado) | Integrar seletor de arquivo da IDE via diálogo explícito apenas quando o currículo demandar |
| Objetos (JSON/XML) | ✅ `Json` em FERRO/Rust zero-dependência (RFC 8259, 19 funções puras/imutáveis: `valido`, `formatar`, `minificar`, `escapar_cadeia`, `contem_chave`, `obter_chaves`, `obter_*`, `obter_lista_*`, `definir_*`, `remover_chave`) com profundidade máxima 64, limite de 2 MiB + orçamento de cadeias e erros em `Resultado<T, cadeia>` (VM mínima, WASM e Rust gerado) | Avaliar XML apenas se o currículo demandar |
| Internet/ServicosWeb | ✅ `Internet` em FERRO/Rust (`definir_tempo_limite`, `obter_texto`, `enviar_json`, `endereco_disponivel`) negada por padrão com consentimento explícito (`with_ferro_network_enabled` / `FERRO_FERRO_ALLOW_NETWORK=1`), timeout 100..30000 ms, cancelamento cooperativo <= 25 ms no debugger, limite de 1 MiB + orçamento de cadeias e erros em `Resultado<T, cadeia>` (VM, WASM com erro tipado e Rust gerado) | Avaliar cabeçalhos/métodos HTTP adicionais se o currículo demandar |
| Graficos | ✅ `Graficos` em FERRO via ponte tipada para o host nativo da VM (`stdlib` + `DesktopHost`), com constantes `COR_*`/`GRADIENTE_*`/`CANAL_*`, polígonos via `vetor<inteiro>`, ativos confinados ao workspace, `Resultado<T, cadeia>` nas operações falíveis e erro tipado explícito em `--no-default-features`/WASM | Avaliar backend gráfico nativo em Rust gerado se o currículo exigir exportação standalone |
| Teclado/Mouse | ✅ `Teclado` e `Mouse` em FERRO via ponte tipada para o host de entrada da VM (`stdlib` + `DesktopHost`), com constantes `TECLA_*`/`BOTAO_*`, consultas diretas e esperas canceláveis `ler_tecla`/`ler_botao` em `Resultado<inteiro, cadeia>` (com erro explícito sem `stdlib`/WASM) | Avaliar eventos tipados de alto nível integrados ao runtime MVU no futuro |
| Sons | ✅ `Sons` em FERRO via ponte tipada para o host de áudio da VM (`stdlib` + `DesktopHost`), com carregamento confinado ao workspace, controle de volume/posição e `Resultado<T, cadeia>` nas operações falíveis (com erro explícito sem `stdlib`/WASM) | Avaliar backend de áudio em Rust gerado apenas se exportação standalone exigir |

## IDE e aprendizagem

| Fluxo | Estado FERRO/Rust | Critério para a futura remoção do clássico |
|---|---|---|
| Novo/abrir/salvar | 🟡 Novo Arquivo e criação no workspace agora usam `.fer` por padrão, filtros priorizam `.fer`/`.rs` e exemplos `.fer` da Ajuda/exercícios salvam com `.fer`; `.por` legado mantido acessível | Linguagem desconhecida explícita e retirada de `.por` apenas no gate final |
| Workspace/exportação | 🟡 aceita `.por`, `.fer`, `.rs`, `.pex` | Listagem, criação, rename, delete e export só FERRO/Rust e exercícios novos |
| CLI | ✅ CLI cria (`novo`), valida, formata (`formatar`), exporta Rust/Cargo (`exportar-rust`/`exportar-cargo`), executa (`executar`) e depura interativamente (`depurar` com breakpoints, stepping e variáveis) arquivos `.fer`, preservando `.por` | Remover suporte `.por` apenas no gate final |
| Ajuda/autocomplete/formatador | 🟡 Rust/FERRO parcial; conteúdo dominante é pseudocódigo clássico. Entrada/saída FERRO documenta `leia()`, a forma com variável-alvo de `leia`, `escreval` e interpolação lado a lado, com `leia()` como forma preferida nos exemplos | Referência FERRO, snippets, completions, hover e formatador do subset final |
| Exemplos | ✅ catálogo completo em `.fer` (14 destaques FERRO + 119 exemplos clássicos migrados = 133 exemplos sob `LGPL-3.0-only`, cobrindo fundamentos, controle, funções, vetores, registros/enums, `Opcao`/`Resultado`/`?`, todas as bibliotecas padrão e multimídia, música, colisões, jogos e UI MVU), integrados na tela inicial sem carregar `.por` e validados em testes (`lower`, `format`, `compile_ferro` e execução) | Manter exemplos atualizados conforme novas capacidades FERRO entrarem |
| Exercícios | ✅ formato JSON versionado (`schema_version = 1`), avaliador com limites (`ferro_exercises`, 10 exercícios cobrindo fundamentos, funções, `Resultado`/`?`, vetores dinâmicos, `Texto`, `Tipos`, `Calendario` e `Json`) e integração no catálogo e painel de Testes da IDE; `.pex` clássico mantido | Retirar `.pex` apenas no gate final |
| Modos | 🟡 Beginner/Bridge/Rust coexistem com `.por` | Beginner/Bridge editam FERRO; Rust é `.rs`; troca não reescreve fontes |
| Prévia Rust/source maps | 🟡 FERRO e clássico suportados | FERRO sempre; retirar frontend classic apenas no gate final |
| Navegação de símbolos/outline/rename | 🟡 FERRO e clássico têm serviços semânticos próprios | Preservar FERRO; avaliar navegação/refactor real para `.rs` com spans e teste Rust |
| Ownership lab | 🟡 simulação editorial | Analisar código/diagnósticos reais ou rotular trace; sem cenário apresentado como execução real |
| Janela gráfica, temas, abas, atalhos e painéis | ✅ shell existente | Manter no app FERRO/Rust; alterar UI somente para fluxos funcionais novos |
| Testes/resultados/diagnósticos | ✅ execução, debugger e painel de Testes unificado avaliam exercícios `.fer`, projetos Cargo/Rust (`cargo test` supervisionado com consentimento explícito) e `.pex` legado, distinguindo erro de compilação, falha de teste/infraestrutura e revisão exata | Retirar `.pex` apenas no gate final |
| Persistência de progresso | 🟡 local, baseada em hash do exercício e da fonte | Novo formato versionado; progresso só após todos os casos aprovados; migração/isolamento de histórico documentados |
| Editor: busca, abas, desfazer, teclas e preferências | ✅ genérico ou parcialmente ligado ao editor | Preservar; revisar apenas comandos exclusivos de sintaxe `.por` |
| Open/save, diálogo nativo e drag/drop | 🟡 roteiam `.por`/`.pex` além de `.fer`/`.rs` | Manter segurança e arquivos sujos; aceitar `.fer`/`.rs` e exercícios novos; erro claro para formatos legados |
| Snippets, autocomplete, hover e bibliotecas | 🟡 `.fer` possui realce, snippets, autocomplete contextual (`UI.`, `Matematica.`, `Texto.`, `Tipos.`, `Util.`, `Calendario.`, `Arquivos.`, `Json.`, `Internet.`, `Graficos.`, `Teclado.`, `Mouse.`, `Sons.`, `Opcao::`, `Resultado::`, `adicionar`/`remover`/`obter`, enums e símbolos locais) e hover sem vazar símbolos do pseudocódigo clássico; `.por` clássico permanece isolado | Manter catálogo FERRO sincronizado à medida que novas bibliotecas/coleções FERRO entrarem e retirar caminho `.por` apenas no gate final |
| Realce na IDE e no playground | ✅ CodeMirror e Web Playground realçam `escreval` como builtin, `leia()` e trechos `{...}` de cadeias com cor de tema própria; o playground usa spans seguros (sem `innerHTML`) e mantém o `<pre>` com texto idêntico ao `<textarea>` | Preservar taxonomia compartilhada e estender a novas construções FERRO sem alterar backends |
| Formatação | 🟡 Ferro usa formatador clássico, Rust usa `rustfmt`, FERRO possui formatador canônico UTF-16 preservando comentários/literais para o subset atual | Acompanhar novas construções do subset FERRO (vetores, structs/enums) e manter `rustfmt` real |
| Sobre, licença, atualização, relatos e alertas | ✅ funções de produto/desktop | Manter identidade e atribuições corretas; verificar que links/créditos não alegam suporte clássico |
| Dicas, ajuda de atalhos e referência de sintaxe | 🟡 ajuda F1 abre em Ferro (FERRO) com guias executáveis, referência das 13 bibliotecas FERRO e todos os 77 exemplos dos tópicos de ajuda portados para `.fer` (`apps/desktop/src/help-assets/recursos/exemplos-ferro/`) | Atualizar dicas iniciais e retirar referências de bibliotecas clássicas apenas no gate final |
| Plugins Java | ➖ implementação atual usa classes Java | Avaliar modelo de extensão Rust/WASM como feature própria; não portar bytecode/classe Java literalmente |
| Cursos/carrossel e coletor UNIVALI | ➖ dependem de serviços externos específicos | Verificar disponibilidade, privacidade e autorização; não simular integração inexistente |
| Distribuição/OS | 🟡 instaladores Tauri existem e associam `.por`/`.pex` | Manter pacotes; registrar só `.fer`/`.rs` e novo formato de exercício após os gates |

## Gate final de `.por`/`.pex`

Cada célula `⬜`/`🟡` acima precisa estar fechada, adaptada com justificativa
aprovada ou marcada não aplicável com motivo. Além disso, usar `rg` para
confirmar que produção, CLI, workspace, Tauri, association, README, instalação,
help e menus não anunciam nem interpretam `.por`/`.pex`. A VM e os caminhos
FERRO/Rust não são candidatos a remoção.
