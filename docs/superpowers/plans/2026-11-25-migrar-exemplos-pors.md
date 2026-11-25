# Migração de exemplos Portugol clássico para PORS — Plano de implementação

> **Para agentes de execução:** execute cada lote nesta worktree, seguindo TDD e validando os exemplos PORS com o serviço Rust antes de alterar as telas que os expõem.

**Objetivo:** substituir os 119 programas `.por` do catálogo e as 77 fontes `.por` usadas pela ajuda por exemplos Portugol-RS `.pors` equivalentes e executáveis.

**Arquitetura:** manter a fonte de verdade dos exemplos do catálogo no serviço Rust, que já expõe metadados e valida os 14 exemplos PORS atuais. Manter exemplos próprios da ajuda junto aos assets de ajuda do frontend; a ajuda resolve somente arquivos `.pors`. O catálogo da tela inicial e a ajuda deixam de carregar o corpus clássico de testes, que continua isolado para regressões do dialeto compatível.

**Tecnologias:** Rust, Tauri 2, TypeScript, Vite, testes Node e suíte de integração do `portugol-service`.

**Especificação:** issue [#225](https://github.com/wendelmax/portugol-studio-rs/issues/225), [ADR 0016](../../adr/0016-pors-como-linguagem-autoral-e-retirada-do-por.md), [plano PORS-only](../pors-only-migration.md) e [matriz de migração](../research/PORS_MIGRATION_MATRIX.md).

## Restrições globais

- A extensão dos exemplos editáveis passa a ser `.pors`; não alterar silenciosamente os arquivos `.por` dos usuários.
- Não remover o parser clássico nem os fixtures `.por` que ainda dão cobertura à compatibilidade.
- Todos os exemplos novos precisam usar PORS real e APIs implementadas; não apresentar pseudocódigo como programa executável.
- Preservar categoria, intenção didática, créditos e cabeçalhos de licença dos exemplos de origem adaptados.
- A implementação permanece Rust + Tauri desktop e não adiciona dependências.

## Foco de revisão

- Exemplo usa API que existe só no frontend desktop: validar a execução no host nativo correspondente.
- Exemplo pede entrada: testar entradas válidas, inválidas e saída esperada.
- Exemplo manipula arquivos ou rede: executar sob os limites/capacidades PORS existentes.
- Exemplo gráfico, de áudio, teclado ou mouse: testar host interativo ou registrar explicitamente a adaptação didática.
- Referências de ajuda relativas a caminhos: confirmar que todos resolvem um `.pors` existente e que o botão abre a extensão correta.

---

## Mapa de arquivos

- `crates/portugol-service/assets/exemplos-pors/`: fontes PORS portadas para o catálogo inicial.
- `crates/portugol-service/src/pors_examples.rs`: metadados, lista de fontes e testes de compilação/execução.
- `crates/portugol-service/assets/exemplos/` e `apps/desktop/src-tauri/src/catalog.rs`: recursos e API do catálogo clássico a retirar da UI após substituir seus consumidores.
- `apps/desktop/src/examples.ts` e `apps/desktop/src/pors-catalog.ts`: composição/abertura do catálogo PORS da tela inicial.
- `apps/desktop/src/help-assets/recursos/exemplos-pors/`: fontes PORS usadas por tópicos de ajuda.
- `apps/desktop/src/help-assets/topicos/linguagem_portugol/`: tópicos e referências relativas adaptados para exemplos PORS.
- `apps/desktop/src/help.ts`: importação e abertura dos exemplos incluídos na ajuda.
- `apps/desktop/tests/` e `crates/portugol-service/tests/`: inventário, roteamento, referências, execução e cobertura dos exemplos.
- `tests/corpus/exemplos/` e `tests/corpus/ajuda/recursos/exemplos/`: fixtures `.por` preservadas para testes de regressão, sem importação para a aplicação.

## Lotes de trabalho

### Lote 1 — Catálogo PORS da tela inicial

1. Adicionar testes que exijam equivalência de inventário para os 119 programas e verifiquem que o frontend não chama `example_catalog()` para fontes `.por`.
2. Portar programas por grupo didático: algoritmos, entrada/saída, condicionais, laços, funções, vetores/matrizes, bibliotecas, gráficos/interação e jogos. Usar APIs PORS já testadas; adaptar para UI MVU ou recursos visuais quando o exemplo clássico depender do host gráfico.
3. Registrar nomes, categorias, descrições, fontes e atribuição em PORS; estender os testes do serviço para compilar e executar os exemplos com entradas e saídas esperadas.
4. Fazer a tela inicial compor o catálogo somente com `listPorsExamples()`, abrindo cada fonte com nome `.pors` e sem diretório temporário de `.por`.
5. Remover os comandos e recursos do catálogo clássico que perderem todos os consumidores, mantendo o runtime clássico somente para sua cobertura isolada até o gate ADR 0016.
6. Validar: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `npx tsc --noEmit`, `npm test` e `npm run build`.

### Lote 2 — Exemplos dos tópicos de ajuda

1. Adicionar testes para contar os 77 arquivos de referência e exigir que cada trecho ativo aponte para um `.pors` existente.
2. Criar fontes PORS originais/adaptadas junto aos assets de ajuda, mantendo autoria/licença de cada adaptação.
3. Atualizar os tópicos para demonstrar a sintaxe PORS correspondente ao conceito e apontar para as novas fontes; tópicos que ensinam conceitos sem equivalente publicado devem receber uma versão didática PORS (ou ser reescritos como explicação PORS equivalente), nunca código `.por` em tela.
4. Fazer `help.ts` importar somente fontes `.pors`; botões abrem arquivos `.pors`.
5. Validar links relativos e cobertura completa dos 77 exemplos; rodar testes da IDE, build frontend e validações Rust tocadas.

### Lote 3 — Fechamento

1. Atualizar o catálogo de aprendizagem e a documentação de migração para informar a substituição dos exemplos visíveis.
2. Buscar caminhos de produção/UI que ainda carreguem exemplos `.por`; separar referências permitidas de teste/regressão.
3. Fazer revisão de categoria, fidelidade didática, créditos, limites e suporte desktop antes de considerar a issue completa.

## Estado

- [x] Lote 1 — catálogo da tela inicial (119 exemplos clássicos portados para `.pors` + 14 destaques PORS = 133 no catálogo; tela inicial somente PORS).
- [x] Lote 2 — tópicos de ajuda (77 exemplos de ajuda portados para `.pors`, 48 HTMLs e `help.ts` atualizados).
- [x] Lote 3 — fechamento e busca final.
