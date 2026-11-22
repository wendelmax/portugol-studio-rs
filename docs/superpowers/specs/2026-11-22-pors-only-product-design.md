# Direção PORS/Rust e retirada do Portugol clássico

## Intenção

Concentrar este produto no aprendizado da linguagem autoral Portugol-RS
(`.pors`) e na passagem para Rust real (`.rs`), deixando arquivos e conteúdo
Portugol clássico para o Portugol Studio original. O projeto atual usa Rust +
Tauri; a decisão não muda seu nome.

## Objetivos

- Levar as capacidades úteis hoje disponíveis apenas no Portugol clássico para
  uma linguagem PORS tipada, testável e com exportação Rust correta para o
  subset suportado.
- Acrescentar conceitos de Rust ensináveis com semântica, diagnósticos ou traces
  verificáveis: ownership, moves, borrowing, structs/enums, pattern matching,
  `Option` e `Result`, além de `rustc`/Cargo reais.
- Manter a IDE desktop, segurança, temas, editor, workspace, debugger, exercícios
  e materiais que continuem necessários a PORS/Rust.
- Remover de vez suporte de produção para `.por`/`.pex` somente depois de os
  gates da matriz de migração serem satisfeitos.

## Fora de objetivo

- Renomear o produto por causa do rótulo `portugol-studio-next`.
- Implementar toda a linguagem Rust em PORS ou afirmar compatibilidade universal
  com `rustc`.
- Remover a VM inteira: o debugger PORS depende do backend HIR → bytecode → VM.
- Alterar, converter ou apagar arquivos legados dos usuários.
- Manter um segundo interpretador clássico permanentemente no produto final.

## Estado observado

PORS já tem um frontend independente, HIR, geração Rust e uma rota de debugger
pela VM compartilhada. Seu subset atual contém tipos primitivos, variáveis,
expressões, entrada/saída, condicionais e loops. O Portugol clássico ainda
oferece o restante do catálogo de linguagem/bibliotecas e sustenta grande parte
dos exemplos, ajuda, exercícios e CLI.

## Abordagens

1. **Manter `.por` indefinidamente:** rejeitada por duplicar linguagem e
   currículo e tirar foco de PORS/Rust.
2. **Remover `.por` imediatamente:** rejeitada porque há lacunas em linguagem,
   bibliotecas, conteúdo, exercícios, CLI e fluxos de arquivos.
3. **Portar e remover sob gates:** escolhida pelo usuário e recomendada pela
   auditoria. Cada feature é entregue/testada primeiro em PORS/Rust; a remoção
   acontece como fase final.

## Restrições e escolhas

- PORS é autoral, sintaxe em português, imutabilidade padrão e mutabilidade
  explícita; Rust real fica disponível por `.rs`/Cargo.
- A execução normal e a depuração devem ter semântica alinhada. Divergências
  deliberadas e diferenças de backend serão documentadas e testadas.
- Código clássico só é retirado depois da matriz PORS completa e da substituição
  dos seus caminhos visíveis ao usuário.
- O nome do produto e a identidade visual atual são preservados; a UI muda para
  apoiar a experiência PORS/Rust conforme as features precisarem.

## Critérios de sucesso

- Os fluxos de aprendizado mantidos não pedem arquivos `.por` nem exercícios
  `.pex`.
- Funções, coleções, bibliotecas e ferramentas que serão mantidas têm
  equivalente PORS/Rust e testes determinísticos.
- Conceitos Rust ensinados no editor refletem código e revisão reais; simulações
  são sempre rotuladas como tal.
- A aplicação, CLI, instaladores e associação de arquivos reconhecem `.pors`,
  `.rs`, Cargo e o formato de exercício progressivo e não interpretam formatos
  legados.
- Nenhuma capacidade do debugger PORS ou recurso Rust é perdida na remoção do
  frontend clássico.

## Próximo trabalho

Usar `docs/plans/pors-only-migration.md` para completar a matriz de capacidades
e dividir cada domínio em PRs pequenas. A primeira implementação de linguagem
deve ser escolhida pela matriz depois da auditoria de PORS e das dependências
`.por`, com TDD e testes diferenciais de VM/Rust gerado.
