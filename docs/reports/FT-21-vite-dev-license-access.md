# FT-21 — Acesso do Vite de desenvolvimento à licença

## Resultado

O frontend Tauri importa `LICENSE?raw`, que fica na raiz do repositório enquanto o root do Vite é `apps/desktop`. O allowlist explícito do Vite agora contém somente a raiz do repositório calculada a partir da localização de `vite.config.ts`; o acesso não foi ampliado para a home nem para diretórios pais. Build de produção, IPC, permissões Tauri e o arquivo da licença não mudaram.

## Validação executada

- `node --experimental-strip-types --test tests/vite-config.test.mjs` — PASS.
- `npm test` em `apps/desktop` — PASS, 20/20 testes.
- `npm run build` em `apps/desktop` — PASS; Vite alerta que o chunk `ide` tem 656.22 kB, acima do limite padrão de 500 kB.
- `npm run tauri dev` na worktree Linux/KDE — o processo nativo `portugol-desktop` iniciou a partir de `target/debug` da worktree e permaneceu ativo. O servidor Vite iniciou em `http://localhost:5173`; o módulo `/@fs/.../LICENSE?raw` da worktree respondeu HTTP 200 e o erro de allowlist não reapareceu.
- O GTK emitiu um aviso de `gtk_window_set_titlebar()` para uma janela já realizada; não impediu a inicialização.

## Teste e desvio registrados

O teste Node verifica o contrato da configuração no código-fonte: raiz derivada de `import.meta.url` e lista explícita `[repositoryRoot]`. Um import direto da configuração no worker do `node:test` falhou de forma inconsistente; por isso, o comportamento efetivo foi verificado também pelo smoke test do Tauri/Vite e pela resposta HTTP 200 do recurso da licença. O smoke test é feito na máquina Linux/KDE, não no CI.

Ruling: manter o teste de contrato estático junto do smoke test nativo, em vez de insistir no import da configuração dentro do worker — o lançamento Tauri carrega a configuração pelo caminho de produção do Vite. Custo se a decisão estiver errada: alterações futuras podem quebrar sem modificar a forma textual coberta pelo teste; o smoke nativo é a cobertura complementar.

## Limitação

Esse smoke confirma o carregamento do frontend na janela Tauri, não a paridade visual completa do Portugol Studio Next.
