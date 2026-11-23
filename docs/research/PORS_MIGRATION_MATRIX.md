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
| Expressões | 🟡 aritmética, comparação, igualdade, lógica e curto-circuito | Bitwise, shifts, operações/unários restantes; validar precedência e overflow |
| Condições | 🟡 `se`/`senao` e `escolha` exaustivo para enums próprios, com payloads posicionais; sem guards, wildcard ou matching como expressão | `Option`/`Result`, referências e padrões aninhados |
| Repetição | ✅ `enquanto`, `para`, `pare`, `continue` | 🔎 `faca-enquanto` e alternativas pedagógicas; revisar loop labels |
| Funções | 🟡 `inicio` e funções auxiliares com parâmetros tipados, chamadas, retorno, recursão, chamadas para frente e mutabilidade explícita; Rust gerado, debugger VM, outline, navegação, rename e formatador PORS | Faltam cobertura aprofundada de frames/limites de recursão e referências/borrowing explícitos |
| Coleções | 🟡 Vetores tipados homogêneos não vazios, leitura/escrita indexada, `tamanho`, cópia por valor e bounds; sem matrizes, vetores vazios, redimensionamento, slices/visões ou vetores aninhados | Matrizes, construção vazia/redimensionamento e outras coleções; slices/visões e iteração permanecem decisões futuras |
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
| Matematica | ⬜ | Funções numéricas equivalentes e tipos/precisão explícitos |
| Texto | ⬜ | Unicode, fatiamento, busca, substituição e alocações limitadas |
| Tipos | ⬜ | Conversões explícitas, parse e falhas em `Result` |
| Util | ⬜ | Funções utilitárias que continuam úteis ao currículo |
| Calendario | ⬜ | Datas/duração/timezone; escolher crate/contrato determinístico |
| Arquivos | ⬜ | API confinada ao workspace/consentimento; limites e tratamento de erro |
| Objetos (JSON/XML) | ⬜ | Preferir tipos/serialização Rust seguros e `Result` |
| Internet/ServicosWeb | ⬜ | Requer consentimento e limites explícitos; nunca bloquear debugger sem cancelar |
| Graficos | ⬜ | API de janela/Canvas mantida com testes nativos multiplataforma |
| Teclado/Mouse | ⬜ | Eventos e capacidades Tauri portados para tipos/ownership PORS claros |
| Sons | ⬜ | Operações assíncronas, ciclo de vida e host da IDE |

## IDE e aprendizagem

| Fluxo | Estado PORS/Rust | Critério para a futura remoção do clássico |
|---|---|---|
| Novo/abrir/salvar | 🟡 `.pors` e `.rs` existem; fallback desconhecido ainda seleciona Portugol | `.pors` default, linguagem desconhecida explícita, filtros `.pors`/`.rs` |
| Workspace/exportação | 🟡 aceita `.por`, `.pors`, `.rs`, `.pex` | Listagem, criação, rename, delete e export só PORS/Rust e exercícios novos |
| CLI | 🟡 CLI cria (`novo`), valida, formata (`formatar`), exporta Rust (`exportar-rust`) e executa `.pors` na VM, preservando `.por` | Depuração interativa `.pors` no CLI; remover suporte `.por` apenas no gate final |
| Ajuda/autocomplete/formatador | 🟡 Rust/PORS parcial; conteúdo dominante é Portugol clássico | Referência PORS, snippets, completions, hover e formatador do subset final |
| Exemplos | 🟡 corpus inicial próprio de 8 exemplos `.pors` (LGPL-3.0-only) validados, formatáveis e executados em testes (`pors_examples`) | Integrar na tela inicial da IDE e expandir corpus mantendo recursos rastreáveis |
| Exercícios | 🟡 formato JSON versionado (`schema_version = 1`) e avaliador executável com limites (`pors_exercises`, 4 exercícios iniciais); `.pex` clássico mantido | Integrar seletor/progresso versionado na IDE e expandir catálogo antes de retirar `.pex` |
| Modos | 🟡 Beginner/Bridge/Rust coexistem com `.por` | Beginner/Bridge editam PORS; Rust é `.rs`; troca não reescreve fontes |
| Prévia Rust/source maps | 🟡 PORS e clássico suportados | PORS sempre; retirar frontend classic apenas no gate final |
| Navegação de símbolos/outline/rename | 🟡 PORS e clássico têm serviços semânticos próprios | Preservar PORS; avaliar navegação/refactor real para `.rs` com spans e teste Rust |
| Ownership lab | 🟡 simulação editorial | Analisar código/diagnósticos reais ou rotular trace; sem cenário apresentado como execução real |
| Janela gráfica, temas, abas, atalhos e painéis | ✅ shell existente | Manter no app PORS/Rust; alterar UI somente para fluxos funcionais novos |
| Testes/resultados/diagnósticos | 🟡 execução e debugger existem; painel de testes avalia apenas `.pex` | Unificar estados reais de Rust/PORS, distinguir erro do aluno de falha de infraestrutura e associar revisão |
| Persistência de progresso | 🟡 local, baseada em hash do exercício e da fonte | Novo formato versionado; progresso só após todos os casos aprovados; migração/isolamento de histórico documentados |
| Editor: busca, abas, desfazer, teclas e preferências | ✅ genérico ou parcialmente ligado ao editor | Preservar; revisar apenas comandos exclusivos de sintaxe `.por` |
| Open/save, diálogo nativo e drag/drop | 🟡 roteiam `.por`/`.pex` além de `.pors`/`.rs` | Manter segurança e arquivos sujos; aceitar `.pors`/`.rs` e exercícios novos; erro claro para formatos legados |
| Snippets, autocomplete, hover e bibliotecas | 🟡 recursos clássicos ainda aparecem em caminhos por idioma | Catálogo completo derivado de PORS/Rust, sem autocompletar nomes do Portugol clássico |
| Formatação | 🟡 Portugol usa formatador clássico, Rust usa `rustfmt`, PORS possui formatador canônico UTF-16 preservando comentários/literais para o subset atual | Acompanhar novas construções do subset PORS (vetores, structs/enums) e manter `rustfmt` real |
| Sobre, licença, atualização, relatos e alertas | ✅ funções de produto/desktop | Manter identidade e atribuições corretas; verificar que links/créditos não alegam suporte clássico |
| Dicas, ajuda de atalhos e referência de sintaxe | 🟡 textos e diagramas são classic-first | Reescrever para PORS/Rust com exemplos executáveis e documentação de subset |
| Plugins Java | ➖ implementação atual usa classes Java | Avaliar modelo de extensão Rust/WASM como feature própria; não portar bytecode/classe Java literalmente |
| Cursos/carrossel e coletor UNIVALI | ➖ dependem de serviços externos específicos | Verificar disponibilidade, privacidade e autorização; não simular integração inexistente |
| Distribuição/OS | 🟡 instaladores Tauri existem e associam `.por`/`.pex` | Manter pacotes; registrar só `.pors`/`.rs` e novo formato de exercício após os gates |

## Gate final de `.por`/`.pex`

Cada célula `⬜`/`🟡` acima precisa estar fechada, adaptada com justificativa
aprovada ou marcada não aplicável com motivo. Além disso, usar `rg` para
confirmar que produção, CLI, workspace, Tauri, association, README, instalação,
help e menus não anunciam nem interpretam `.por`/`.pex`. A VM e os caminhos
PORS/Rust não são candidatos a remoção.
