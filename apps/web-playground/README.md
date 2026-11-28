# Ferro Studio — Web Playground (WASM)

Playground estático, sem dependências JavaScript externas, movido pelo crate
[`ferro-wasm`](../../crates/ferro-wasm) compilado para
`wasm32-unknown-unknown`.

Versão online publicada automaticamente via GitHub Pages:
[wendelmax.github.io/ferro-studio](https://wendelmax.github.io/ferro-studio/).

## Recursos

- Validação instantânea de código `.fer` com diagnósticos em português;
- Formatação canônica UTF-16;
- Execução na VM Ferro em memória linear com suporte a `leia` e orçamentos de
  passos, saída e memória de cadeias;
- Prévia visual interativa para apps declarativos MVU (`UI.*` com
  `iniciar`/`atualizar`/`visualizar`), renderizados sem `innerHTML`;
- Interface responsiva com tema claro/escuro explícito (preferência salva no
  navegador) e realce de sintaxe leve no editor `.fer`.

## Como compilar e executar localmente

1. Instale o alvo WebAssembly (uma única vez):

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. Gere o bundle estático pronto para servir em `apps/web-playground/dist/`:

   ```bash
   ./scripts/build-web-playground.sh
   ```

3. Sirva a pasta `apps/web-playground/dist/` com qualquer servidor HTTP estático:

   ```bash
   python3 -m http.server 8080 --directory apps/web-playground/dist
   ```

   Depois abra `http://localhost:8080`.
