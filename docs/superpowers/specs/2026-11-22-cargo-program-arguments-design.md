# Argumentos de programa em projetos Cargo

O estudante que abre um projeto Cargo deve poder informar os mesmos `argv` que já são aceitos para `.rs` standalone. O campo atual permanece na barra de opções; em Cargo, os argumentos são enviados somente ao binário executado, depois de `--`.

## Abordagens

- **Reusar o parser `parseProcessArguments` e a validação Rust já existente (escolhida):** mantém a notação e os limites conhecidos, sem shell ou novos pacotes.
- **Criar sintaxe específica de Cargo:** duplicaria UX e parser sem necessidade; rejeitada.
- **Aceitar flags de Cargo no campo:** ampliaria a superfície de comando e confundiria flags de build e `argv`; rejeitada.

## Limites

Não alterar a seleção de alvo, consentimento Cargo, execução standalone, entrada stdin ou parâmetros de Portugol. Revalidar tudo no backend; não confiar no campo da UI. Argumentos vazios preservam a execução atual.

## Aceitação

- Cargo recebe espaços, Unicode, aspas literais e argumento vazio intactos em `std::env::args()`.
- A UI habilita o campo em `.rs` dentro de Cargo e explica a notação sem shell.
- O serviço recusa NUL, mais de 256 argumentos e os limites de bytes por plataforma antes de iniciar Cargo.
- Build não recebe argv do programa; sem shell e sem flags livres de Cargo.
