# ADR 0007 — Diagnosticar projetos Cargo sob demanda

- **Status:** aceito
- **Data:** 2026-11-22

## Contexto

O linter `rustc` do editor compila um buffer isolado e é deliberadamente desativado para arquivos identificados em projetos Cargo, pois imports e módulos do projeto causariam falsos erros. `cargo check` pode executar build scripts e macros procedurais, além de baixar dependências; portanto, não deve rodar implicitamente enquanto o usuário digita.

## Decisão

Adicionar o comando explícito **Analisar projeto Cargo** à paleta de comandos quando o arquivo Rust ativo pertence a um projeto Cargo. A confirmação de execução Cargo da sessão é pré-requisito; a permissão é distinta do Rust standalone. O backend invoca somente `cargo check --message-format=json --manifest-path <manifest>` no projeto validado pelo workspace, sob supervisão, timeout de 20 s, limite de saída de 1 MiB e cancelamento. Diagnósticos cujo span pertence ao arquivo aberto são mapeados para offsets CodeMirror UTF-16; demais mensagens renderizadas permanecem disponíveis no painel de saída.

## Consequências

- Nenhum build script, macro procedural ou resolução de dependências roda apenas por digitar.
- O aluno precisa invocar o comando explicitamente; o primeiro uso mostra o mesmo aviso Cargo sobre rede e execução local sem sandbox.
- O linter standalone permanece inalterado fora do Cargo.
- O comando não executa o programa e não seleciona alvos.
