# Design: Rust mais simples no Playground

## Intenção

Ao clicar em **Ver Rust**, quem está aprendendo deve conseguir reconhecer a
tradução do próprio programa Ferro. O código deve continuar compilável e
produzir os mesmos resultados observáveis, mas não deve carregar helpers de
suporte que o programa não usa. O Rust exibido também deve refletir a marca
Ferro: o código de suporte emitido pelo gerador não deve conter `PORS`/`pors`,
`Portugol` ou a marca anterior em identificadores, comentários, nomes de
protocolo ou textos embutidos como o User-Agent HTTP. Identificadores e strings
escritos pelo usuário permanecem intactos para preservar seu programa.

## Estado atual

`apps/web-playground/app.mjs` mostra diretamente o texto retornado por
`runtime.exportRust()`. O gerador em `crates/ferro-rust/src/codegen/mod.rs`
acrescenta helpers comuns de formatação numérica e runtime PORS
independentemente das operações usadas; os runtimes específicos de módulos já
são selecionados por uso.

## Abordagens

1. **Gerar somente o suporte necessário (escolhida):** analisar as operações
   do programa e emitir apenas helpers referenciados, mantendo um único `.rs`
   autônomo e preservando a saída/semântica existente. Isso reduz o código
   incidental sem criar uma tradução diferente para o Playground.
2. **Mostrar uma tradução didática separada:** produzir uma versão curta
   voltada ao ensino. Ela pode omitir validações e detalhes necessários à
   execução autônoma, então não garante o mesmo comportamento em todos os
   casos.

## Escopo e limites

- Otimizar o Rust gerado para programas simples e exemplos do Playground.
- Preservar as semânticas já implementadas, incluindo limites e diagnósticos
  quando forem usados; atualizar de forma coordenada o protocolo gerado para
  `leia()` e seu consumidor, se necessário.
- Remover marcas antigas do Rust gerado e das interfaces que consomem seus
  protocolos, sem renomear globalmente as APIs internas do compilador ou o
  README de projeto Cargo separado.
- Não mudar a linguagem Ferro, o WASM, nem o fluxo de execução do Playground.
- Recursos realmente usados (por exemplo, UI MVU ou módulos de biblioteca)
  ainda podem exigir suporte maior; esse código não será removido só para
  encurtar a listagem.

## Critério de sucesso

Para programas que não usam leitura, reais, vetores ou módulos especiais, o
Rust exibido contém apenas o `main`, funções do programa e os helpers
estritamente necessários, não contém menções a PORS e compila mantendo a saída
do programa Ferro.
