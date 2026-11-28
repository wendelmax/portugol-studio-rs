# FT-18 — Realce e autocomplete dos laços Ferro

- **Entrega:** o editor `.fer` classifica `enquanto` e `para` como keywords e oferece ambas no autocomplete.
- **Precisão:** `faca` e `pare` continuam ausentes porque ainda não são construções suportadas pelo subset FERRO.
- **Compatibilidade:** o dialeto clássico, runtime, IPC e arquivos existentes não foram alterados; nenhuma dependência nova.
- **Validação:** teste focused `node --experimental-strip-types apps/desktop/tests/ferro-editor-loops.test.mjs` (2/2); `npm --prefix apps/desktop test` (17/17); `npm --prefix apps/desktop run build` (PASS, aviso Vite para chunk principal acima de 500 kB); `git diff --check` (PASS).
