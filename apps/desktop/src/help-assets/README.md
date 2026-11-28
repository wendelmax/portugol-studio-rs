# Origem e curadoria da ajuda

Os tópicos didáticos e diagramas de fluxo em `topicos/` e `recursos/imagens/` foram originalmente derivados da tag [v2.7.5 do Portugol Studio](https://github.com/UNIVALI-LITE/Portugol-Studio/tree/v2.7.5/ide/src/main/assets/ajuda) (licença LGPL-3.0) e adaptados para refletir integralmente a sintaxe e a semântica de **Ferro (`.fer`)**.

- Exemplos interativos em `recursos/exemplos-pors/` (77 arquivos `.fer` válidos, sem sintaxe legada `.por`).
- A documentação completa das 13 bibliotecas nativas e suas constantes é servida pela fonte única em `apps/desktop/src/pors-stdlib-catalog.ts`, validada automaticamente contra `crates/ferro-frontend/src/parser/stdlib.rs` e `constants.rs`.
