# Execução de arquivos Rust no Ferro Studio RS

## Intenção

Concluir o terceiro nível de aprendizagem permitindo compilar e executar um arquivo `.rs` independente no editor atual. O aluno deve ver saída real e diagnósticos reais do `rustc`, sem alterar ou salvar silenciosamente o buffer.

## Como funciona hoje

Arquivos `.rs` já são reconhecidos, abertos, editados e salvos com realce Rust. `rustc` é detectado para o executor `.fer`, mas `canRunSource` e `runProgram` recusam arquivos `.rs`. O serviço `.fer` já demonstra a fronteira necessária: comandos Tauri fechados, pasta temporária, compilação e processo supervisionados, limites de saída/tempo.

## Abordagens consideradas

1. **Escolhida — executar um arquivo independente com `rustc`:** reutilizar os limites e o diretório temporário do serviço atual, fixar os argumentos do compilador, capturar diagnósticos JSON e executar o binário resultante. É adequada ao documento `.rs` já suportado, não pede um modelo de projeto nem expõe shell ao frontend.
2. **Cargo/projetos:** permitir dependências e módulos exige manifestos, identidade de projeto, seleção de toolchain e sandbox mais amplo. Não cabe no modelo atual de arquivo isolado.
3. **WebAssembly:** restringiria chamadas nativas e teria uma experiência diferente da linguagem Rust real; também introduziria um runtime e uma toolchain novos.

## Decisões

- Compilar somente o texto do buffer atual como um binário Rust 2021 independente. Não executar Cargo, scripts de build, comandos fornecidos pelo frontend ou dependências externas.
- Requerer `rustc` instalado no sistema. Quando ausente, desabilitar Executar com explicação; não baixar ou instalar toolchains sem consentimento.
- Limitar código-fonte a 1 MiB, compilação a 20 segundos, execução a 3 segundos e saída combinada a 1 MiB. Recolher o processo filho ao atingir um limite e remover a pasta temporária após a sua terminação.
- Exibir erros e spans `rustc` no painel de Mensagens; exibir stdout/stderr e exit code reais no Console. Tratar timeout e excesso de saída como falhas, nunca como sucesso.
- Antes da primeira execução Rust em cada sessão da IDE, pedir confirmação explícita. Explicar que código Rust roda com as permissões da conta local e **não** é sandbox. O aluno inicia toda execução manualmente.
- Não oferecer stdin interativo, depuração, Cargo, dependências, módulos externos ou promessa de isolamento de subprocessos nesta fatia. Documentar esses limites.
- Aceitar resultados somente se a mesma aba e revisão ainda estiverem ativas. Edição durante compilação não deve apresentar resultado obsoleto como pertencente ao código atual.

## Critérios de aceite

- Um `.rs` que compila mostra stdout, stderr e exit code do binário real.
- Um `.rs` inválido mostra diagnóstico clicável na linha/coluna real do código Rust.
- Loop infinito e excesso de saída são encerrados por limites e identificados na UI.
- O botão respeita detecção de `rustc` e não muda o comportamento de `.por` ou `.fer`.
- A confirmação de execução informa a fronteira de segurança; cancelar não inicia compilação.
- Builds e testes multiplataforma existentes continuam sem adicionar permissões Tauri ou dependências frontend.
