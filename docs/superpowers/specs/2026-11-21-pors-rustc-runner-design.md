# Design: executar o subset Portugol-RS com Rust

## Intenção

Dar ao aluno um caminho executável em arquivos `.pors`: validar o subset suportado, compilar o Rust emitido pelo compilador do projeto com `rustc` e mostrar stdout/stderr e código de saída reais na IDE.

## Objetivos

- Usar exclusivamente o Rust gerado por `portugol-rust`; rejeitar sintaxe/semântica que o frontend não suporta.
- Verificar a presença de `rustc` e explicar a limitação se a toolchain não estiver instalada.
- Chamar processos por `std::process::Command` com argumentos fixos, sem shell, Cargo, dependências externas ou conteúdo arbitrário do editor como executável.
- Executar o subset atual, sem loops/funções recursivas, em um diretório temporário e limpar os artefatos ao final.
- Habilitar Executar para `.pors`, preservar console real e impedir execução repetida enquanto a compilação/execução estiver pendente.
- Manter depuração, entrada interativa e `.rs` independentes desabilitados.

## Fora de escopo

- Executar Rust escrito pelo usuário, projetos Cargo ou crates externos.
- Depuração passo a passo, `leia`, loops, funções auxiliares e outros recursos ainda ausentes da gramática `.pors`.
- Prometer sandbox de sistema operacional: a origem do binário é o emissor controlado do subset, que atualmente não aceita chamadas arbitrárias nem laços.

## Abordagens consideradas

1. Interpretar a HIR `.pors` dentro do `portugol-service`: evitar ferramenta externa, mas cria um segundo runtime e pode divergir da tradução Rust que a ponte quer ensinar; rejeitada.
2. Gerar Rust validado e invocar `rustc` + o executável em processos com argumentos fixos: usa a semântica real da ponte, mantém o runtime clássico intacto e é escolhido para o subset finito atual.
3. Usar `cargo run` em projeto criado pelo aluno: ampla flexibilidade, porém executa scripts/builds e crates arbitrários, fora do contrato de segurança; rejeitada.

## Decisões

- Um serviço Rust no `portugol-service` retorna `stdout`, `stderr` e `exitCode`; erros de toolchain/compilação retornam mensagem de erro, sem fabricar resultados.
- `rustc --version` é consultado por comando separado para a IDE poder desabilitar Executar com explicação quando ausente.
- A fonte não é passada como argumento de processo; primeiro precisa passar pelo parser/lowering e pelo emissor atual.
- O Tauri executa compilação/execução em `spawn_blocking`; resultados atrasados são descartados se aba ou buffer mudarem.
- Limpeza usa RAII do diretório temporário mesmo em falhas de compilação ou execução.

## Riscos e rollback

Risco moderado: subprocessos e filesystem temporário; sem alterações de permissões Tauri ou do runtime clássico. O subset não tem loops/funções e não pode emitir chamadas escolhidas pelo aluno. Rollback: remover serviço/comandos/API e restaurar a mensagem “ainda não executa”; o compilador/exportador e o runtime clássico permanecem.
