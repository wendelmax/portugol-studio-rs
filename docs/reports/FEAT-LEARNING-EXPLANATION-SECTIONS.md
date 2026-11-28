# FEAT — Seções progressivas no inspetor Aprender

- Requisito: UI-023/024, UC-13; completar o cartão com “O que mudou?”, “Por que isso importa?” e “Praticar”.
- Implementação: cada conceito do catálogo local contém conteúdo editorial próprio; `renderLearningPanel` preenche as regiões nomeadas com `textContent`. Seleção manual, pin por aba/nível, contexto mapeado, exemplos e laboratório de ownership preservam seus fluxos.
- Compatibilidade: nenhuma alteração a Rust/Tauri, arquivos do aluno, tradução, execução ou progresso. A seção “Praticar” é uma sugestão manual, não um exercício avaliado.
- Proveniência: comparações, snippets e texto são conteúdo editorial; a feature não infere progresso ou domínio.
- A revisão independente apontou risco de anúncios longos na região live; agora somente o título breve do conceito é anunciado automaticamente, e o restante pode ser navegado pelo leitor de tela.

## Validação

- RED: `node --experimental-strip-types --test --test-name-pattern='cada cartão|cartão Aprender apresenta' tests/learning-content.test.mjs` — os dois testes novos falharam pelos campos e regiões ainda ausentes.
- GREEN: mesmo comando após implementação — PASS, 2/2.
- `npm test --prefix apps/desktop` — PASS, 184/184.
- `npm run build --prefix apps/desktop` — PASS; Vite reporta o aviso preexistente de chunk acima de 500 kB.
- `cargo test -p ferro-service --offline --locked` — PASS, 57 testes.
- `cargo fmt --all --check` e `git diff --check` — PASS.
- Interação visual/E2E Tauri: NOT_RUN até existir runner dedicado no projeto.
