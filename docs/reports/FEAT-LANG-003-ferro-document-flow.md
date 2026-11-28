# FEAT-LANG-003 — Fluxo de documentos Ferro

- Escopo: arquivos `.fer` na IDE Tauri, com detecção de linguagem por aba, análise e preview Rust pelo frontend Ferro e associação para abertura pelo sistema.
- Arquitetura preservada: mesmo Tauri v2, CodeMirror 6, serviço Rust e backend HIR compartilhado. Arquivos `.por` continuam usando o parser e a VM clássicos.
- `.fer` agora abre por diálogo, argumento de inicialização, evento do sistema e drag-and-drop; pode ser salvo com o sufixo correto e tem associação de arquivo Tauri. `Save As` só atualiza caminho/linguagem após `write_file` concluir com sucesso.
- O editor ativa realce/completar específico do dialeto, mostra diagnósticos em português com offsets UTF-16 e lista bindings. Preview gera Rust pela HIR compartilhada.
- Execução, depuração, formatador clássico, inclusão de bibliotecas, navegação semântica e renomeação ficam desativados para `.fer` com explicação. O dialeto não é enviado à VM clássica e não há invocação de `rustc`/shell.
- Extensão `.fer` foi adotada conforme decisão do produto; `.por` permanece a extensão clássica.

## Validação executada

- `npm test` — passou, 6 arquivos/suítes de teste, incluindo detecção `.fer`, Unicode e sufixo de salvamento.
- `npm run build` — passou (`tsc` e build Vite). O Vite reporta o aviso existente de bundle principal acima de 500 kB.
- `cargo test -p ferro-service --offline` — passou: 17 testes unitários e 10 de serviço, incluindo spans UTF-16 após emoji, símbolos e exportação.
- `cargo fmt --all -- --check` — passou.
- `git diff --check` — passou.
- `tauri.conf.json` — JSON válido.
- Teste do crate `ferro-desktop` — não executável neste ambiente: falta `webkit2gtk-4.1`/`javascriptcoregtk-4.1.pc` requerido pelo Tauri WebKit no Linux. O teste nativo de filtro de arquivos está incluído, mas precisa rodar no CI Tauri.
- O PR #40 (frontend Ferro) foi mesclado em `main` após os seis checks ficarem verdes; esta implementação foi rebaseada sobre a main resultante. O PR desta fatia e seus checks são publicados após a validação local.

## Limitações e próximos passos

- CLI ainda aceita apenas `.por`; executar `.fer` não foi habilitado.
- O subset atual de Ferro não tem ownership/borrowing, loops, funções extras, bibliotecas, VM, execução ou depuração. Esses recursos devem entrar por incrementos sem simular suporte.
- Após conexão com GitHub: confirmar o PR da frontend Ferro (#40), garantir que ele esteja integrado, rebasear esta branch em `main`, abrir PR desta fatia, aguardar checks verdes e mesclar/deletar a branch conforme instrução do usuário.
- Rollback: reverter o commit desta branch; não requer migração de dados.
