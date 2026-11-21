# ADR-0001: usar o Webstudio como referência e evoluir Portugol para Rust por etapas

- **Status:** Aceita como direção de produto
- **Data:** 21/11/2026
- **Pesquisa:** [Benchmark Portugol Webstudio](../research/portugol-webstudio-benchmark-2026-11-21.md)

## Contexto

Portugol Studio RS já possui um compilador e runtime em Rust: lexer com `logos`, parser recursivo descendente, AST com spans, análise semântica de símbolos/escopos/tipos, compilador de AST para bytecode, VM, E/S, bibliotecas e depurador com passo a passo e variáveis.

A análise atual entrega diagnósticos e símbolos, porém ainda não produz uma HIR tipada compartilhada. O núcleo é do dialeto Portugol: não aplica regras de ownership/borrowing do Rust e não chama `rustc`. Referências a parâmetros e `Rc<RefCell>` implementam semântica do Portugol, não a do Rust.

O Portugol Webstudio serve de referência de experiência. A interface pública demonstrou exercício com casos escondidos, autorias, correção por valor/função, links temporários sem solução, compartilhamento de código, documentação, acessibilidade, atualização e modo offline. Seu README declara uma arquitetura Angular/ANTLR/transpilação JavaScript e licença GPL-3.0; a aplicação Rust é LGPL-3.0-only.

## Decisão

1. Preservar o dialeto e os arquivos `.por` atuais como modo de compatibilidade, com o runtime/VM Rust existente.
2. Introduzir a aprendizagem de Rust como modo opcional e explícito, sem alterar a interpretação de Portugol clássico.
3. Usar um modelo intermediário semântico comum para conceitos realmente partilhados, com frontends e regras de análise específicos por dialeto. Ownership, borrowing e mutabilidade pertencem ao analisador Rust-like e não devem ser simulados por conversões de tokens do Portugol.
4. Começar a ponte por uma conversão lado a lado de um subconjunto documentado e testado; só depois introduzir Portugol-RS, compilação Rust real e laboratório de ownership.
5. Gerar código Rust semanticamente correto e tratar construções não suportadas com diagnósticos explícitos. Não apresentar uma tradução aproximada como código correto.
6. Manter exercícios, progresso e rascunhos local/offline por padrão. Publicação em link requer decisão separada sobre serviço, retenção, revogação e aviso de dados.
7. Reimplementar os fluxos de forma independente com os crates do projeto. Não copiar código ou assets GPL-3.0 do Webstudio para este repositório sem revisão de licença e decisão de relicenciamento.
8. Dar crédito visível e preciso às origens UNIVALI/LITE, aos autores de recursos licenciados e aos mantenedores do projeto, respeitando o escopo de cada licença.

## Alternativas consideradas

### A. Portar o runtime/transpilador do Webstudio

Rejeitada. Criaria um segundo motor, repetiria capacidades já disponíveis na VM Rust e traria código GPL-3.0 para um projeto LGPL-3.0-only sem necessidade.

### B. Acrescentar Rust-like diretamente à gramática atual e executá-lo na VM Portugol

Rejeitada. Misturaria regras de ownership com a semântica dinâmica do Portugol, faria diagnósticos incorretos sobre posse e confundiria o modo de compatibilidade.

### C. Adicionar uma representação tipada compartilhada e frontends/analisadores próprios por dialeto

Escolhida. Preserva o motor clássico, cria uma base para uma sintaxe pedagógica nova e permite comparar conceitos sem fingir que a VM Portugol é o compilador Rust.

## Consequências

- Será necessário definir a HIR e os limites de equivalência antes de ampliar a gramática.
- O primeiro MVP deverá ser restrito e verificável; uma conversão parcial deve identificar claramente as construções sem suporte.
- Uma futura integração real com `rustc` deve ser opt-in, executar em processo separado com limites e reportar diagnósticos em português, sem esconder os erros do compilador.
- A autoria de exercícios deve começar com arquivos locais e preservar a solução do docente fora do material distribuído ao aluno.
- A referência do Webstudio pode guiar comportamentos e prioridade, mas a revisão de licença continua obrigatória antes de importar qualquer recurso.
