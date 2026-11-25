# Plano de implementação: seletor de níveis de aprendizagem

## Contexto e objetivo

Implementar UI-028/UC-05 com o menor incremento compatível com o app: Beginner mantém a IDE Portugol, Bridge expõe a prévia real do exportador atual, e Rust fica desabilitado até existir toolchain/editor Rust. A escolha não edita fonte nem muda o runtime.

## Mudanças por arquivo

1. `apps/desktop/tests/learning-mode.test.mjs`: primeiro, adicionar testes para default, persistência válida, valor corrompido/storage indisponível, Rust não habilitado e acessibilidade do seletor.
2. `apps/desktop/src/learning-mode.ts`: tipos e funções puras de validação/carregamento/salvamento com tratamento de falha de localStorage.
3. `apps/desktop/index.html`: seletor rotulado dentro das opções existentes; usar IDs/data-testid estáveis; opção Rust disabled com descrição associada.
4. `apps/desktop/src/main.ts`: sincronizar modo e painel preview; invalidar geração/timer ao sair de Bridge; ativar export no Bridge e em tab ativa; botão Rust escolhe Bridge, fechar preview volta para Beginner.
5. `apps/desktop/src/style.css`: estilo alinhado ao padrão de opções e foco acessível.
6. `apps/desktop/package.json`: incluir novo teste no script atual mantendo Node test runner/dependências.
7. `docs/portugol-studio-next-integration/docs/quality/TRACEABILITY_MATRIX.yaml`: registrar FEAT-LEARN-001 e status real.
8. `docs/portugol-studio-next-integration/docs/agents/FEATURE_REPORT_UI028.md`: preencher resultados após execução.

## Contratos e compatibilidade

- Serviço: módulo TS pequeno para preferência do modo.
- Tauri/IPC/Rust: sem mudança. Bridge chama o `export_rust` existente; Portugol segue executado pela VM atual.
- Estado inicial: beginner; valor local inválido ou storage bloqueado usa beginner sem crash.
- Persistência: global, provisória por ausência de WorkspaceStore; migração futura quando houver workspace ID.
- Rust: disabled com motivo explícito; não abre falso editor nem executa processo.

## Verificação

- Teste primeiro: `cd apps/desktop && npm test` deve falhar por módulo/markup ausentes, depois passar.
- `cd apps/desktop && npm run build`.
- `cargo fmt --all -- --check` e `cargo test --workspace --offline --locked` para confirmar que não houve regressão Rust.
- Clippy e CI completa conforme toolchain/ambiente disponíveis.
- Screenshot/E2E e Gherkin: não há runner configurado; reportar `NOT_RUN` sem alegar cobertura.

## Riscos e rollback

Risco baixo: estado visual novo pode divergir ao usar atalhos/botões; concentrar mudanças em uma função de sincronização e testar transições. Rollback remove seletor, helper, testes e linha do script; `btn-rust` e exportador preservam funcionalidade clássica.
