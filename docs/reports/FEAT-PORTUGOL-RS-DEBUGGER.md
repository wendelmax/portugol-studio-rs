# FEAT — Depurador Portugol-RS

**Estado:** fatia funcional implementada; revisão e smoke visual nativo pendentes.

## Entregue nesta fatia

- Backend inicial HIR→bytecode para bindings, expressões primitivas, escrita/leitura, condição e laços com `pare`/`continue`.
- Serviço de execução depurável `.pors` que reutiliza a VM, breakpoints, snapshots e canal de entrada do debugger clássico.
- Comando Tauri e roteamento da UI para iniciar `.pors` pela VM. Depurar não requer `rustc`; Executar continua usando Rust gerado.
- Indicador de status `VM Portugol-RS` ao pausar.
- Testes de breakpoint e snapshot, entrada tipada sem pausa duplicada e comparação de saída VM/Rust para laços `para`/`enquanto`, `continue`, curto-circuito, overflow inteiro e concatenação de cadeias. Divisão por zero confirma erro na mesma linha fonte.
- Step over atualiza o snapshot e stop interrompe a VM durante a espera de `leia`.
- O compilador preserva escopos lexicais e bindings sombreados em slots distintos; snapshots mostram somente o binding visível e reativam o anterior ao sair do bloco.
- Regressões verificam shadowing em blocos condicionais e restauração após o término de um `para`.
- A VM aplica limites tipados de 3 s de runtime ativo, 1 MiB de saída e 16 MiB cumulativos de cadeias; pausa e espera de entrada são excluídas do relógio.

## Validação

- `cargo test -p portugol-bytecode --offline --locked`
- `cargo test -p portugol-service --offline --locked --test portugol_rs_debugger`
- `cargo test --workspace --offline --locked -- --skip bibliotecas_internet_e_servicos_web`
- `cargo check -p portugol-desktop --offline --locked`
- Clippy (`portugol-bytecode`, `portugol-vm`, `portugol-service`, `-D warnings`)
- `npm test` (36/36) e `npm run build`
- Validação final desta revisão: suíte do workspace, clippy com `-D warnings`, `cargo check -p portugol-desktop`, `cargo fmt --check`, `git diff --check` e build frontend passaram. O único teste omitido requer acesso externo à internet.

## Pendências conhecidas

- Paridade diferencial completa para todos os operadores/tipos e categorias de erro do subset.
- Paridade diferencial mais ampla para categorias de erro e spans de todas as construções do subset.
- Testes automatizados de troca de documento e eventos de sessão obsoletos.
- Smoke visual da aplicação Tauri no KDE; testes de screenshots/E2E continuam indisponíveis nesta fatia.
- Avaliar sintaxe PORS adicional suportada pelo frontend e recusar diagnósticos de backend com localização correta.
