# Relatório — FEAT-PORS-INPUT-001

- Branch / base: `feat/pors-interactive-input` / `bff7b1a`.
- Escopo: UI-003/UI-021, UC-06, FT-03; entrada interativa `leia` no subset Portugol-RS.
- Arquitetura preservada: Rust + Tauri 2, CodeMirror 6, serviço de execução existente com `rustc`. Sem comandos de shell fornecidos pela UI e sem mudanças de permissões.
- Contrato: `leia(variavel_mutavel);` solicita uma linha no console. O parser só aceita bindings mutáveis já declarados; o tipo esperado acompanha o evento `run:input`. O código gerado lê stdin normalmente fora da IDE. Dentro da IDE, um protocolo interno solicita entrada pelo mesmo evento e comando `send_input` do runtime Portugol.
- Cancelamento e limites: a espera usa o token de cancelamento existente; o limite de três segundos mede execução ativa e pausa durante a digitação. O teto de saída, limite de 1 MiB por linha de entrada e orçamento de strings continuam aplicados.
- Arquivos alterados: HIR, parser Portugol-RS, gerador Rust, executor do serviço, comandos Tauri, API e console frontend, testes do parser/editor/serviço, `docs/PARIDADE.md`, matriz de rastreabilidade e este relatório.
- Compatibilidade: programas sem `leia` mantêm saída anterior; o executor Rust standalone preserva stdin padrão. Portugol clássico mantém o próprio canal de entrada. Apenas uma variável é aceita por chamada `leia`.
- Estados de UI: o console mostra a saída antes do pedido, apresenta o formulário com tipo esperado, ecoa a linha enviada e oculta o formulário ao terminar ou cancelar.

## Verificação

- TDD do parser: teste vermelho confirmou que `leia` era rejeitado; após a implementação, os testes do parser passaram.
- `cd apps/desktop && npm test` — PASS, 25 testes.
- `cd apps/desktop && npm run build` — PASS, TypeScript e Vite; Vite emitiu o aviso já existente de chunk JavaScript acima de 500 kB.
- Regressão de limite: `rejects_an_input_line_larger_than_one_mib` falhou antes da validação de tamanho e passou depois.
- `cargo test --workspace --offline --locked` — PASS, toda a suíte de testes, corpus de regressão e doctests. O teste Internet/Serviços Web passou com permissão de rede habilitada.
- `cargo check -p portugol-desktop --offline --locked` — PASS.
- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --offline --locked -- -D warnings` — PASS.
- Teste E2E na janela Tauri — NOT RUN; não existe runner de interação da IDE configurado para esse fluxo.

## Limitações e próximos passos

- Ainda não há teste de interação DOM/Tauri que digite no formulário real; testes automatizados cobrem o protocolo e o encaminhamento no nível dos contratos.
- `leia` aceita um binding por chamada; o laboratório de ownership, depuração Portugol-RS e depuração Rust seguem como lacunas independentes.
- Próxima etapa: rodar a suíte completa do workspace e CI Linux/Windows/macOS; depois avaliar chamada com múltiplos bindings e suporte a `leia` em Rust standalone.

## Rollback

Reverter as alterações de FEAT-PORS-INPUT-001 remove o novo nó HIR, parser, codegen, canal Tauri e controles editoriais de `leia`; a execução e o console Portugol clássico permanecem intactos.
