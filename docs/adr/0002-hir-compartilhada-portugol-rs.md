# ADR 0002: HIR tipada compartilhada para Portugol e Portugol-RS

> A política inicial de manter `.por` permanentemente foi supersedida pelo
> [ADR 0016](0016-pors-como-linguagem-autoral-e-retirada-do-por.md). A HIR
> independente de frontend e backend e a separação entre parser, semântica,
> bytecode, VM e gerador Rust continuam válidas para a evolução PORS/Rust.

- Status: arquitetura implementada; política de manter `.por` permanentemente supersedida pelo ADR 0016
- Data: 2026-11-21

## Contexto

O projeto já possui parser, AST, análise semântica, bytecode e VM para Portugol clássico. O exportador em `portugol-rust` contém uma HIR privada que armazena expressões como strings de Rust e conhece apenas o subconjunto exportável. Ela é uma representação de destino, não uma IR semântica que dois frontends possam compartilhar.

Portugol-RS será um dialeto educacional opcional. Arquivos `.por` continuam compatíveis com o dialeto clássico; `.pors` será a extensão proposta para o dialeto novo, mas só deve ser registrada na IDE após parser, análise e persistência desse formato funcionarem de ponta a ponta.

## Decisão

Introduzir uma crate `portugol-hir` sem dependência de frontend nem backend. A HIR conterá tipos de domínio, spans neutros de bytes, identificadores estáveis de bindings, mutabilidade e nós de expressão/comando. Adaptadores de frontend baixam suas ASTs para a HIR; geradores/runtime consomem HIR sem colocar texto do destino dentro dela.

A primeira fatia adiciona o modelo, testes, API `lower_to_hir(&str)` para o subconjunto clássico suportado e migra `export(&str)` para emitir código a partir da HIR. A validação semântica clássica continua ocorrendo antes do lowering. Construções fora do subconjunto retornam diagnóstico localizado. A implementação anterior permanece compilada somente em testes como oráculo de equivalência até a migração ser estabilizada.

Portugol-RS terá um parser/adaptador independente em uma fatia posterior. Não se altera o significado de `.por`, o parser clássico, a VM ou os comandos Tauri nesta decisão.

## Consequências

- Frontends podem compartilhar uma representação sem acoplamento a sintaxe.
- Exportadores deixam de usar strings de código como semântica e podem manter mapeamentos de origem.
- A primeira fatia tem escopo limitado e não implica suporte ao dialeto `.pors`.
- A HIR deve crescer por nós tipados e testes; não deve virar cópia da AST clássica.
- O contrato futuro de `.pors` exige detecção explícita de linguagem e testes de abrir, analisar, salvar e executar/exportar.

## Alternativas rejeitadas

- Reusar a HIR privada atual: mistura semântica com sintaxe Rust e impede múltiplos destinos.
- Criar um segundo runtime completo: duplica regras de tipos, diagnósticos e execução.
- Tratar `.pors` como `.por`: não representa mutabilidade explícita ou regras novas.
