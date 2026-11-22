# Análise Cargo com diagnósticos no editor — design

## Objetivo

Permitir que a pessoa solicite `cargo check` do projeto Cargo ativo e veja os diagnósticos do arquivo aberto no editor, sem executar Cargo automaticamente durante digitação.

## Fluxo

1. A paleta oferece **Analisar projeto Cargo** apenas para `.rs` em projeto detectado.
2. O comando salva o arquivo Rust ativo e requer que outros arquivos Rust alterados do workspace sejam salvos.
3. Se ainda não houve consentimento Cargo na sessão, pergunta antes de iniciar qualquer compilação ou macro.
4. Tauri valida workspace e arquivo, atribui ID para cancelamento e chama serviço Rust com argumentos fixos.
5. Serviço roda `cargo check --message-format=json`, com timeout, limite de saída e cancelamento.
6. Diagnósticos do arquivo aberto viram lint do CodeMirror; mensagens de outros arquivos seguem no console.

## Limites

- Sem diagnóstico automático ao digitar, sem escolha de alvo e sem debugger Rust.
- Cargo pode acessar a rede e executar código de build com permissões locais; não há sandbox.
- Dependências não resolvidas podem falhar por rede, toolchain ou limites do supervisor.
