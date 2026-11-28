# Laboratório visual de ownership — Plano de implementação

> Implementação sequencial na worktree `feat/ownership-visual-lab`; seguir TDD e executar cada verificação descrita antes de avançar.

## Tarefas

1. **Modelo e testes vermelhos.** Criar `apps/desktop/tests/ownership-lab.test.mjs` primeiro. Exigir quatro IDs, fontes Rust completas, snapshot inicial/final válida, navegação/clamp bem definido, move invalidando `a`, dois `&T`, exclusividade de `&mut T`, liberação por escopo e provenance constante. Importar o futuro módulo; executar `npm test` e confirmar falha por export ausente. Adicionar fixtures em `tests/fixtures/ownership/` e teste de compilação/execução com `rustc`; o cenário inválido deve falhar com diagnóstico de uso de valor movido.
2. **Modelo mínimo.** Implementar `src/ownership-lab.ts` com tipos discriminados e dados imutáveis. Funções públicas: `ownershipScenarioIds()`, `getOwnershipScenario(id)`, `getOwnershipSnapshot(id, stepIndex)`. Consulta fora do intervalo retorna `undefined`; componente decide desabilitar navegação. Executar somente teste focado e confirmar verde.
3. **Markup sem interação ainda.** Acrescentar região em `index.html` sob o cartão Ownership: seletor de cenário, nav Anterior/Próximo/Reiniciar, status de passo, trecho de código, diagrama textual/visual e explicação/diagnóstico. IDs estáveis e labels; proveniência sempre visível. Atualizar teste para conferir `aria-live`, labels, status explícito e região inicialmente oculta fora de Ownership.
4. **Renderização e controles.** Em `main.ts`, implementar `renderOwnershipLab()` ao resolver conceito e ao mudar cenário/passo. Criar elementos via DOM, usar `textContent`, marcar linha atual por `data-current`, exibir owners/referências/heap com nomes acessíveis, atualizar botões disabled e contador. O handler da UI mantém cenário/passo; entrada manual do usuário só altera o cenário demonstrativo, nunca o buffer.
5. **Estilo.** Em `style.css`, fazer grid responsivo do código e diagrama; definir visuais diferentes para owner, referência, moved e dropped sem depender apenas de cor; foco visível e `prefers-reduced-motion`. Em viewport estreito, empilhar colunas.
6. **Relatório e matriz local.** Criar `docs/reports/FEATURE-OWNERSHIP-LAB.md` no formato do template de feature (IDs, estado anterior, arquivos, IPC/permissões sem mudança, testes e NOT_RUN visual/E2E), referenciar a pesquisa atual e a limitação de seleção manual. Atualizar `docs/PARIDADE.md` na linha já existente de aprendizagem/debugger, sem marcar ownership como trace real.
7. **Verificação total.** Executar `npm test`, `npm run build`, `cargo fmt --all --check`, `CARGO_TARGET_DIR=/tmp/ferro-target cargo test --offline --locked`, `CARGO_TARGET_DIR=/tmp/ferro-target cargo clippy --all-targets --offline -- -D warnings` e `git diff --check`. A suíte desktop Tauri e captura visual ficam NOT_RUN localmente se a dependência GTK continuar ausente; o CI valida a matriz Tauri.

## Arquivos

- Criar: `apps/desktop/src/ownership-lab.ts`, `apps/desktop/tests/ownership-lab.test.mjs`, `apps/desktop/tests/fixtures/ownership/{move-invalid,shared-borrow,mutable-borrow,scope}.rs`, `docs/reports/FEATURE-OWNERSHIP-LAB.md`.
- Alterar: `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/src/style.css`, `docs/PARIDADE.md`.
- Nenhuma alteração em crates Rust, comandos/capabilities Tauri, dependencies/manifests, corpus ou documentos de integração não rastreados.

## Foco de revisão

- Ao terminar o último uso de uma referência, a snapshot deixa de tratá-la como empréstimo ativo.
- `move-invalid.rs` falha por valor movido, enquanto os demais fixtures compilam.
- Navegação por teclado nos controles funciona e `aria-live` anuncia apenas mudança de passo, sem leitura redundante de todo o painel.
- O estado do visualizador nunca é descrito como runtime trace ou como semântica observada do buffer atual.
