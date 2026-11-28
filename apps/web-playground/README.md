# Ferro Studio — Web Playground

Playground estático, sem dependências JavaScript externas, movido pelo crate
[`ferro-wasm`](../../crates/ferro-wasm) compilado para
`wasm32-unknown-unknown`. Publicado em
[wendelmax.github.io/ferro-studio](https://wendelmax.github.io/ferro-studio/).

## Uso

- **Abrir .fer**: botão `Abrir .fer` na barra superior. Usa o seletor nativo
  quando o navegador permite; nos demais, abre um seletor comum.
- **Salvar**: `Salvar` grava no mesmo arquivo quando há permissão; caso
  contrário, baixa uma cópia. `Novo` cria `sem-titulo-N.fer`.
- **Abas**: exemplos, arquivos e lições abertos viram abas. Trocar de aba
  preserva o texto editado; `×` fecha a aba.
- **Exemplos**: doze programas organizados por grupo na lateral, incluindo
  entrada tipada, vetores, `Resultado` e prévia visual MVU.
- **Trilha**: seis lições curtas. `Verificar` compara a saída do seu programa
  com o esperado; o progresso fica no navegador. A entrada padrão é fixa nas
  lições.
- **Painéis inferiores**: Terminal, Prévia UI, Rust gerado e Referência de
  sintaxe (pesquisável; clique insere o trecho no editor).
- **Executar** ou `Ctrl/Cmd+Enter` roda a aba ativa. `Tema` alterna claro/escuro.

## Build local

```bash
rustup target add wasm32-unknown-unknown
./scripts/build-web-playground.sh
python3 -m http.server 8080 --directory apps/web-playground/dist
```

Abra `http://localhost:8080`. Para checar as soluções da trilha com o WASM
real, rode `node scripts/check-web-lessons.mjs` depois do build.
