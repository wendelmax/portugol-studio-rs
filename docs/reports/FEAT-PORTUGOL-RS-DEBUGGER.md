# FEAT — Depurador Portugol-RS

**Estado:** fatia funcional em implementação; ainda não pronta para release.

## Entregue nesta fatia

- Backend inicial HIR→bytecode para bindings, expressões primitivas, escrita/leitura, condição e laços com `pare`/`continue`.
- Serviço de execução depurável `.pors` que reutiliza a VM, breakpoints, snapshots e canal de entrada do debugger clássico.
- Comando Tauri e roteamento da UI para iniciar `.pors` pela VM. Depurar não requer `rustc`; Executar continua usando Rust gerado.
- Indicador de status `VM Portugol-RS` ao pausar.
- Testes de breakpoint e snapshot, entrada tipada sem pausa duplicada e comparação de saída VM/Rust para um programa com laço e condição.
- Step over atualiza o snapshot e stop interrompe a VM durante a espera de `leia`.

## Validação

- `cargo test -p portugol-bytecode --offline --locked`
- `cargo test -p portugol-service --offline --locked --test portugol_rs_debugger`
- `cargo check -p portugol-desktop --offline --locked`
- `npm test` (36/36) e `npm run build`

## Pendências conhecidas

- Paridade diferencial completa para todos os operadores/tipos e categorias de erro do subset.
- Limites de 3 s de runtime ativo, 1 MiB de saída e 16 MiB de strings no backend VM.
- Testes de cancelamento durante pausa/entrada e de diagnósticos de spans UTF-8.
- Smoke visual da aplicação Tauri no KDE; testes de screenshots/E2E continuam indisponíveis nesta fatia.
- Avaliar sintaxe PORS adicional suportada pelo frontend e recusar diagnósticos de backend com localização correta.
- Sombreamento de nome é recusado pelo debugger até que a VM possa mostrar somente bindings cujo escopo está ativo.
