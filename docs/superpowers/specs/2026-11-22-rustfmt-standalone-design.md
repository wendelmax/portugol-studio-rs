# Formatador Rust para arquivos `.rs`

## Intenção

Habilitar `Formatar código-fonte` em buffers Rust standalone para que a IDE progressiva permita editar, formatar e executar `.rs` sem sair do fluxo desktop existente.

## Estado observado

- CodeMirror já oferece o botão/atalho e `formatCode()` chama o formatador Ferro.
- `.rs` desabilita o botão com a explicação `Formatador Rust não configurado.`
- O serviço já supervisiona processos com limites de saída/tempo e usa diretórios temporários; `rustfmt` não faz parte da capability atual (`rust_toolchain_available` verifica apenas `rustc`).
- A especificação Next exige ações reais, respostas obsoletas descartadas e código gerado/fonte preservados em erro.

## Objetivo e limites

- Usar `rustfmt` instalado no sistema, com edição Rust 2021 e configuração padrão determinística.
- Não iniciar shell, Cargo, projeto, dependência ou executável informado pela UI.
- Preservar texto do buffer se rustfmt estiver ausente, falhar, exceder limite ou se o documento/aba mudar durante a chamada.
- Habilitar o botão em `.rs` somente após detectar `rustfmt`; Ferro continua sem formatador e pseudocódigo clássico mantém seu formatador.
- Aplicar a substituição como uma única transação de undo.

## Abordagens consideradas

1. **Formatter Rust próprio:** duplicaria gramática e regras oficiais; risco de reformatar incorretamente Rust real.
2. **Rodar `rustfmt` por shell ou no arquivo do usuário:** amplia a superfície de comando e pode sobrescrever dados fora do buffer.
3. **Recomendação — serviço Tauri com comando fixo `rustfmt`:** gravar a revisão em diretório temporário, usar `--edition 2021 --emit stdout --quiet` e um arquivo de configuração vazio explícito; aplicar somente o texto de stdout após sucesso e validação de aba/revisão.

## Restrições e estados

- Fonte: máximo 1 MiB; stdout/stderr combinados: máximo existente de 1 MiB; limite de execução: 5 segundos.
- Capability de `rustfmt` independente da capability `rustc`.
- Erros retornam motivo legível em português; ausência de ferramenta mantém controle desabilitado com instrução `rustup component add rustfmt`.
- Um resultado só é aplicado se a mesma aba e o mesmo objeto de documento continuarem ativos.

## Não objetivos

Não formatar Ferro, código gerado no preview, arquivos em disco fora do buffer, projetos Cargo, macros executadas, nem instalar toolchains automaticamente.

## Rollback

Remover comando/capability Tauri, API e ramificação `.rs` em `formatCode`; o formatador pseudocódigo clássico permanece intacto.
