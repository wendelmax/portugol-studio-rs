# Ir à declaração — Plano de implementação

> Worktree isolada: `feat/go-to-declaration`; implementação nativa neste turno. IDs: UI-036, UC-21, AC-14, FT-08.

## Tarefas (TDD)

1. **RED no serviço:** adicionar unit tests para resolver referência local e função à declaração; testar shadowing e UTF-16 após emoji; testar erro em posição sem símbolo/erro semântico. Executar `cargo test -p portugol-service resolve_symbol` e confirmar falha por API inexistente.
2. **GREEN no serviço:** reaproveitar `bindings`/`locate` de rename para retornar span da declaração em UTF-16; exportar a API no crate; passar os testes focados e todos os testes do serviço.
3. **Contrato Tauri:** registrar comando síncrono `resolve_symbol`, sem processo ou permissão nova; adicionar tipo/result wrapper em `api.ts`; rodar `cargo fmt`, testes do serviço e typecheck.
4. **RED/GREEN de UI:** criar helper testável para política F12 conforme foco e Ctrl/Cmd+clique; cobrir F12 fora do editor preservando Sobre e distinguir modificadores de plataforma. Executar teste focado antes de adicionar handlers.
5. **Integração do editor:** comando F12 no keymap CodeMirror; Ctrl/Cmd+clique via handler de evento; obter offset do evento; pedir resolução; selecionar span de declaração, rolar para a view e focar. Capturar tab e `EditorState`; ignorar resposta atrasada se mudaram.
6. **Feedback e acessibilidade:** em falha, apresentar mensagem pt-BR na barra de status; expandir lista F11 com ações contextuais e atualizar dica/tooltip; sem alteração no buffer.
7. **Evidência/documentação:** atualizar `docs/PARIDADE.md`; registrar arquivos, testes e limites em `docs/reports/FEATURE-GO-TO-DECLARATION.md`.
8. **Gate total:** executar `cargo test -p portugol-service`, `cargo fmt --all --check`, `cargo clippy -p portugol-service --all-targets -- -D warnings`, `npm test`, `npm run build` (Vite com outDir `/tmp` se worktree read-only), e `git diff --check`. Native Tauri/E2E não declarado PASS sem ambiente e runner disponíveis.

## Regras de segurança e compatibilidade

- Resolver apenas a revisão capturada; nunca aplicar resultado antigo em outra aba ou documento.
- Usar o resolvedor semântico existente, não busca textual no frontend.
- Não editar a fonte, iniciar execução, mudar progresso, nem abrir outro arquivo.
- Não alterar `.gitignore`, documentos de integração fornecidos pelo usuário, capabilities ou permissões.

## Risco principal

Offset do CodeMirror é UTF-16; o tokenizer Rust opera offsets de bytes. A conversão deve passar por `LineIndex` e ser testada com caractere suplementar anterior à referência.
