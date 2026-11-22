# ADR 0012 — Confirmar ao desvincular Rust gerado

- **Status:** aceita
- **Data:** 2026-11-22

## Contexto

UI-017 do contrato Next pede confirmação ao transformar código derivado em código autoral. O botão atual criava uma aba Rust imediatamente. O diálogo Tauri é assíncrono: entre abrir e confirmar, a fonte pode mudar ou outra aba pode se tornar ativa.

## Decisão

Reutilizar `ask` com aviso claro, capturar identidade da aba/documento/artefato e verificar o snapshot após confirmação. Só então criar uma nova aba standalone com o texto puro; cancelar não produz efeitos. Se o snapshot mudou, descartar a resposta tardia.

## Consequências

- O aluno escolhe explicitamente a transferência de autoria.
- Fonte e preview permanecem preservados; arquivo não é gravado sem pedido explícito do usuário.
- Confirmações atrasadas não podem criar uma aba a partir de conteúdo que já não corresponde ao documento ativo.
