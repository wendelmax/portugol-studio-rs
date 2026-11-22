# Seleção de binários Cargo — design

## Objetivo

Permitir que a pessoa execute cada binário de um workspace Cargo com múltiplos bins, mantendo o comportamento padrão de projetos simples e usando apenas alvos descobertos do próprio projeto.

## Fluxo

1. A pessoa inicia **Executar** em um arquivo `.rs` de workspace Cargo.
2. A IDE obtém metadata local do projeto com Cargo offline e sem dependências/build scripts.
3. Se existe um binário padrão inequívoco, executa-o como hoje.
4. Se existem vários bins sem padrão, apresenta seletor acessível com pacote e nome do binário e guarda a escolha na aba.
5. A IDE solicita consentimento Cargo antes do build/run, se necessário.
6. Tauri revalida a raiz do workspace e os nomes do alvo via metadata; executa `cargo build --package <pacote> --bin <bin>` e `cargo run --quiet --package <pacote> --bin <bin>` sem shell.

## Limites

- Seleciona somente alvos `bin`; exemplos e testes não são executáveis nesta etapa.
- O Cargo arguments field continua desabilitado; não aceitar comandos ou flags livres.
- `cargo check` permanece sob demanda e sem seleção de alvo.
- Um workspace que não tenha binário exibe mensagem localizada e não inicia o build.
