# Design: navegação e renomeação semânticas em Ferro

## Intenção

Completar uma lacuna concreta da edição `.fer`: o editor já analisa o dialeto e conhece seus bindings, mas desativa ir à declaração e renomear símbolo. Esses comandos devem usar a identidade semântica do HIR, preservando escopos e referências sombreadas.

## Objetivos

- Resolver usos e declarações para o binding correto em `ferro-rust-front-end`/HIR.
- Oferecer ir à declaração, prévia do alvo e rename no mesmo fluxo já usado para `.por`.
- Preservar offsets UTF-16 do CodeMirror e recusar nomes inválidos ou que alterem a resolução de referências.
- Preservar todas as capacidades existentes do pseudocódigo clássico e a exportação Rust.

## Fora de escopo

- Execução/depuração de `.fer`, geração de source maps, funções/escopos que a gramática atual não suporta e mudança da gramática.
- Rename para Rust real ou renomeação entre arquivos.

## Abordagens consideradas

1. Encontrar ocorrências pelo texto do identificador: simples, mas incorreto para strings, comentários e bindings sombreados; rejeitado.
2. Reutilizar o HIR tipado do frontend `.fer`, coletar spans de declarações/usos por `BindingId` e validar o rename reanalisando o programa: escolhido, pois mantém a resolução semântica já estabelecida e dispensa parser paralelo.
3. Adicionar AST de referências e serviços de rename ao frontend: mais explícito, mas amplia a API do compilador além do necessário para a gramática atual.

## Decisões e limites

- Cada operação `.fer` baixa o source para HIR, mapeia `BindingId` para spans e rejeita erros de análise.
- O offset recebido do editor é UTF-16; spans do HIR são UTF-8. Converter por `LineIndex` antes de localizar o token e converter resultados ao retornar.
- Um rename só é aceito quando a nova análise mantém o mesmo mapa de usos para declarações; isso impede captura de nomes e alteração de escopo.
- A função `inicio` não é renomeável porque é o ponto de entrada obrigatório do dialeto.
- Comandos existentes são habilitados somente para `.fer`; os de `.rs` continuam indisponíveis com explicação acessível.

## Risco e rollback

Risco baixo a moderado: serviços novos no núcleo e integração com os botões/atalhos atuais. Sem mudanças em IPC sensível, runtime, engine clássica ou dependências externas. Rollback: remover os serviços `.fer`, testes e habilitação dos comandos; o restante do editor segue igual.
