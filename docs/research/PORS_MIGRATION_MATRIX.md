# Matriz de migração para PORS/Rust

Snapshot-base: `main` em `140a846` (PR #97; 22/11/2026), já com funções PORS.
Esta matriz é o gate vivo do
[ADR 0016](../adr/0016-pors-como-linguagem-autoral-e-retirada-do-por.md).

Legenda: ✅ já coberto em PORS/Rust · 🟡 parcial · ⬜ ausente · 🔎 auditar e
decidir equivalência/escopo. Um ✅ exige testes na implementação PORS/Rust,
não apenas suporte ao Portugol clássico.

## Linguagem e execução

| Capacidade | PORS/Rust hoje | Lacuna / critério para fechar |
|---|---|---|
| Arquivo e entrada | ✅ `.pors`, `funcao inicio()` sem argumentos | Criar novo arquivo `.pors` por padrão; CLI PORS ainda ausente |
| Tipos | 🟡 inteiro, real, lógico, cadeia, caracter; imutabilidade padrão | 🔎 Semântica explícita de tamanhos, overflow, inferência, Unicode, unit/void |
| Variáveis | ✅ `seja`, `seja mutavel`, atribuição e shadowing testados | Validar contrato comum em VM e Rust gerado em todos os caminhos |
| Expressões | ✅ aritmética, bitwise/shifts (`&`, `|`, `^`, `~`, `<<`, `>>`, `&=`, `|=`, `^=`), comparação, igualdade, lógica e curto-circuito com paridade VM/Rust | Manter alinhamento se novos tipos numéricos forem expostos |
| Condições | 🟡 `se`/`senao` e `escolha` exaustivo para enums, `Opcao`/`Resultado` e ramo coringa (`caso _ =>`) com detecção de casos inalcançáveis; sem guards ou matching como expressão | Referências, padrões aninhados e guards |
| Repetição | ✅ `enquanto`, `faca-enquanto`, `para`, `pare`, `continue` com paridade VM/Rust | Revisar loop labels apenas se o currículo demandar |
| Funções | 🟡 `inicio` e funções auxiliares com parâmetros tipados, chamadas, retorno, recursão, chamadas para frente e mutabilidade explícita; Rust gerado, debugger VM, outline, navegação, rename e formatador PORS | Faltam cobertura aprofundada de frames/limites de recursão e referências/borrowing explícitos |
| Coleções | 🟡 Vetores tipados homogêneos com construção vazia tipada (`[]`), redimensionamento dinâmico (`adicionar`, `remover` -> `Opcao<T>`), leitura segura (`obter` -> `Opcao<T>`), leitura/escrita indexada, `tamanho`, cópia por valor e bounds; sem matrizes, slices/visões ou vetores aninhados | Matrizes e outras coleções (mapas/conjuntos); slices/visões e iteração permanecem decisões futuras |
| Tipos compostos | 🟡 Registros/enums nominais, campos, construtores, vetores, mutação sob binding mutável, cópia por valor e `escolha` exaustivo; VM/debugger expande campos e payload ativo; Rust gerado usa `struct`/`enum` | Sem recursão de tipos, `Box`, vetores aninhados, mutação indexada/nested, campos mutáveis individuais ou guards; validar mais profundamente a equivalência de limites e diagnósticos |
| Rust `Option`/`Result` | ✅ `Opcao<T>` (`Algum`/`Nenhum`), `Resultado<T, E>` (`Ok`/`Erro`), `escolha` exaustivo, propagação com `?` e geração nativa para `Option<T>`/`Result<T, E>` em Rust, testados em frontend, VM e `rustc` | Expandir combinadores/métodos auxiliares quando biblioteca padrão PORS evoluir |
| Ownership/borrowing | 🟡 painel editorial com fixtures; sem analisar buffer do aluno | Moves, empréstimos `&T`/`&mut T`, regras de aliasing/lifetimes do subset; comparar com `rustc` |
| Concorrência Rust | ⬜ | Introduzir threads e compartilhamento seguro (`Arc`/`Send`/`Sync`), depois `Mutex<T>` para estado composto e `Atomic*` para operações simples em valores compatíveis; explicar `Ordering` só após sincronização básica |
| Diagnósticos | 🟡 códigos e spans localizados em subset | Cobertura de cada sintaxe/semântica; português, ajuda contextual e source maps |
| Execução normal | 🟡 gera Rust subset e executa `rustc` | Igualdade observável entre todas as construções publicadas e casos diferenciados |
| Debugger | 🟡 HIR → bytecode → VM; breakpoints, stepping, escopos, input, limites; PORS mostra campos de registros e variante/payload de enum | Cobrir a gramática que for expandida; documentar diferenças inevitáveis de backend |
| Segurança/limites | ✅ supervisor de Rust gerado e limites do debugger PORS | Manter saída/tempo/memória/cancelamento e consentimento em todo recurso novo |

## Bibliotecas do inventário clássico

Para cada grupo, revisar a API completa em `docs/PARIDADE.md`, escolher a API
PORS idiomática, seu backend (VM e/ou Rust), efeitos, consentimentos e limites.
Não considerar uma biblioteca portada por implementar apenas o nome do módulo.

| Biblioteca clássica | Estado em PORS/Rust | Decisão pendente |
|---|---|---|
| Matematica | ✅ `Matematica` em PORS/Rust (constantes `PI`/`E` e 16 funções tipadas sem coerção implícita, em VM mínima, WASM e Rust gerado) | Expandir funções trigonométricas inversas/hiperbólicas se o currículo demandar |
| Texto | ✅ `Texto` em PORS/Rust (13 funções Unicode-safe por `char`, incluindo `dividir` -> `vetor<cadeia>`, `Opcao`/`Resultado` em busca/indexação/subtexto e orçamento de memória verificado antes de alocar, em VM mínima, WASM e Rust gerado) | Manter alinhamento com futuros utilitários de formatação quando necessários |
| Tipos | ✅ `Tipos` em PORS/Rust (19 funções explícitas de conversão e validação com `Resultado<T, cadeia>` para parses falíveis e bases 2/10/16, em VM mínima, WASM e Rust gerado) | Manter alinhamento com futuros tipos numéricos se `i64`/`u32` forem expostos |
| Util | ✅ `Util` em PORS/Rust (`sorteia` -> `Resultado<inteiro, cadeia>`, `definir_semente`, `tempo_decorrido`, `aguarde` cancelável e limitado a 60 s, em VM mínima, WASM e Rust gerado) | Manter fora funções dependentes do host/OS (como `obter_diretorio_usuario`) até `Arquivos` com consentimento |
| Calendario | ✅ `Calendario` em PORS/Rust (contrato UTC determinístico zero-dependência com 19 constantes e 15 funções incluindo `dia_semana_*`, `nome_mes`, `eh_bissexto`, `dias_no_mes` e `data_iso_atual`, em VM mínima, WASM e Rust gerado) | Avaliar formatação localizada/fusos opcionais apenas se o currículo exigir |
| Arquivos | ✅ `Arquivos` em PORS/Rust confinado ao workspace (`ler_texto`, `ler_linhas`, `escrever_texto`, `acrescentar_texto`, `existe`, `remover`, `listar`), bloqueando caminhos absolutos/`..`/escape por symlink, limite de 2 MiB + orçamento de cadeias e erros em `Resultado<T, cadeia>` (VM mínima, WASM com erro tipado e Rust gerado) | Integrar seletor de arquivo da IDE via diálogo explícito apenas quando o currículo demandar |
| Objetos (JSON/XML) | ✅ `Json` em PORS/Rust zero-dependência (RFC 8259, 19 funções puras/imutáveis: `valido`, `formatar`, `minificar`, `escapar_cadeia`, `contem_chave`, `obter_chaves`, `obter_*`, `obter_lista_*`, `definir_*`, `remover_chave`) com profundidade máxima 64, limite de 2 MiB + orçamento de cadeias e erros em `Resultado<T, cadeia>` (VM mínima, WASM e Rust gerado) | Avaliar XML apenas se o currículo demandar |
| Internet/ServicosWeb | ✅ `Internet` em PORS/Rust (`definir_tempo_limite`, `obter_texto`, `enviar_json`, `endereco_disponivel`) negada por padrão com consentimento explícito (`with_pors_network_enabled` / `PORTUGOL_PORS_ALLOW_NETWORK=1`), timeout 100..30000 ms, cancelamento cooperativo <= 25 ms no debugger, limite de 1 MiB + orçamento de cadeias e erros em `Resultado<T, cadeia>` (VM, WASM com erro tipado e Rust gerado) | Avaliar cabeçalhos/métodos HTTP adicionais se o currículo demandar |
| Graficos | ⬜ | API de janela/Canvas mantida com testes nativos multiplataforma |
| Teclado/Mouse | ⬜ | Eventos e capacidades Tauri portados para tipos/ownership PORS claros |
| Sons | ⬜ | Operações assíncronas, ciclo de vida e host da IDE |

## IDE e aprendizagem

| Fluxo | Estado PORS/Rust | Critério para a futura remoção do clássico |
|---|---|---|
| Novo/abrir/salvar | 🟡 Novo Arquivo e criação no workspace agora usam `.pors` por padrão, filtros priorizam `.pors`/`.rs` e exemplos `.pors` da Ajuda/exercícios salvam com `.pors`; `.por` legado mantido acessível | Linguagem desconhecida explícita e retirada de `.por` apenas no gate final |
| Workspace/exportação | 🟡 aceita `.por`, `.pors`, `.rs`, `.pex` | Listagem, criação, rename, delete e export só PORS/Rust e exercícios novos |
| CLI | ✅ CLI cria (`novo`), valida, formata (`formatar`), exporta Rust/Cargo (`exportar-rust`/`exportar-cargo`), executa (`executar`) e depura interativamente (`depurar` com breakpoints, stepping e variáveis) arquivos `.pors`, preservando `.por` | Remover suporte `.por` apenas no gate final |
| Ajuda/autocomplete/formatador | 🟡 Rust/PORS parcial; conteúdo dominante é Portugol clássico | Referência PORS, snippets, completions, hover e formatador do subset final |
| Exemplos | ✅ corpus próprio de 14 exemplos `.pors` (LGPL-3.0-only) cobrindo fundamentos, controle, funções, vetores dinâmicos, registros/enums, `Opcao`/`Resultado`/`?`, `Matematica`/`Texto`/`Tipos`/`Util`/`Calendario`/`Json`/`Arquivos` e UI MVU, integrados na IDE e validados em testes | Expandir conforme novas bibliotecas (ex.: `Internet`) entrarem antes de retirar `.por` |
| Exercícios | ✅ formato JSON versionado (`schema_version = 1`), avaliador com limites (`pors_exercises`, 10 exercícios cobrindo fundamentos, funções, `Resultado`/`?`, vetores dinâmicos, `Texto`, `Tipos`, `Calendario` e `Json`) e integração no catálogo e painel de Testes da IDE; `.pex` clássico mantido | Retirar `.pex` apenas no gate final |
| Modos | 🟡 Beginner/Bridge/Rust coexistem com `.por` | Beginner/Bridge editam PORS; Rust é `.rs`; troca não reescreve fontes |
| Prévia Rust/source maps | 🟡 PORS e clássico suportados | PORS sempre; retirar frontend classic apenas no gate final |
| Navegação de símbolos/outline/rename | 🟡 PORS e clássico têm serviços semânticos próprios | Preservar PORS; avaliar navegação/refactor real para `.rs` com spans e teste Rust |
| Ownership lab | 🟡 simulação editorial | Analisar código/diagnósticos reais ou rotular trace; sem cenário apresentado como execução real |
| Janela gráfica, temas, abas, atalhos e painéis | ✅ shell existente | Manter no app PORS/Rust; alterar UI somente para fluxos funcionais novos |
| Testes/resultados/diagnósticos | ✅ execução, debugger e painel de Testes unificado avaliam exercícios `.pors`, projetos Cargo/Rust (`cargo test` supervisionado com consentimento explícito) e `.pex` legado, distinguindo erro de compilação, falha de teste/infraestrutura e revisão exata | Retirar `.pex` apenas no gate final |
| Persistência de progresso | 🟡 local, baseada em hash do exercício e da fonte | Novo formato versionado; progresso só após todos os casos aprovados; migração/isolamento de histórico documentados |
| Editor: busca, abas, desfazer, teclas e preferências | ✅ genérico ou parcialmente ligado ao editor | Preservar; revisar apenas comandos exclusivos de sintaxe `.por` |
| Open/save, diálogo nativo e drag/drop | 🟡 roteiam `.por`/`.pex` além de `.pors`/`.rs` | Manter segurança e arquivos sujos; aceitar `.pors`/`.rs` e exercícios novos; erro claro para formatos legados |
| Snippets, autocomplete, hover e bibliotecas | 🟡 `.pors` possui realce, snippets, autocomplete contextual (`UI.`, `Matematica.`, `Texto.`, `Tipos.`, `Util.`, `Calendario.`, `Arquivos.`, `Json.`, `Internet.`, `Opcao::`, `Resultado::`, `adicionar`/`remover`/`obter`, enums e símbolos locais) e hover sem vazar símbolos do Portugol clássico; `.por` clássico permanece isolado | Manter catálogo PORS sincronizado à medida que novas bibliotecas/coleções PORS entrarem e retirar caminho `.por` apenas no gate final |
| Formatação | 🟡 Portugol usa formatador clássico, Rust usa `rustfmt`, PORS possui formatador canônico UTF-16 preservando comentários/literais para o subset atual | Acompanhar novas construções do subset PORS (vetores, structs/enums) e manter `rustfmt` real |
| Sobre, licença, atualização, relatos e alertas | ✅ funções de produto/desktop | Manter identidade e atribuições corretas; verificar que links/créditos não alegam suporte clássico |
| Dicas, ajuda de atalhos e referência de sintaxe | 🟡 ajuda F1 agora abre em Portugol-RS (PORS) com guias executáveis e referência das 9 bibliotecas PORS (`Matematica`, `Texto`, `Tipos`, `Util`, `Calendario`, `Arquivos`, `Json`, `Internet`, `UI`), mantendo conteúdo clássico como legado | Atualizar dicas iniciais e retirar tópicos clássicos apenas no gate final |
| Plugins Java | ➖ implementação atual usa classes Java | Avaliar modelo de extensão Rust/WASM como feature própria; não portar bytecode/classe Java literalmente |
| Cursos/carrossel e coletor UNIVALI | ➖ dependem de serviços externos específicos | Verificar disponibilidade, privacidade e autorização; não simular integração inexistente |
| Distribuição/OS | 🟡 instaladores Tauri existem e associam `.por`/`.pex` | Manter pacotes; registrar só `.pors`/`.rs` e novo formato de exercício após os gates |

## Gate final de `.por`/`.pex`

Cada célula `⬜`/`🟡` acima precisa estar fechada, adaptada com justificativa
aprovada ou marcada não aplicável com motivo. Além disso, usar `rg` para
confirmar que produção, CLI, workspace, Tauri, association, README, instalação,
help e menus não anunciam nem interpretam `.por`/`.pex`. A VM e os caminhos
PORS/Rust não são candidatos a remoção.
