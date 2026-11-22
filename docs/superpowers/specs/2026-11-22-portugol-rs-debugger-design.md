# Depurador real para Portugol-RS

## Intenção

Permitir que o estudante execute `.pors` pausado, avance por instruções, use breakpoints e inspecione valores que vieram da execução. A tela de Depuração existente já oferece essas interações para Portugol clássico; o objetivo é conectá-la ao subset real de Portugol-RS sem chamar a simulação pedagógica de trace.

## Estado atual

- `portugol-rs-frontend::lower` valida o dialeto e produz a HIR compartilhada, com identidade de bindings e spans UTF-8.
- `portugol-rust` gera Rust compilável a partir da HIR; o serviço usa `rustc` para a execução normal de `.pors`, aplica limites e encaminha `leia` pelo console.
- `portugol-bytecode` compila a AST clássica para bytecode; `portugol-vm` já implementa pausa, continue, step into/over/out, breakpoints, snapshots de variáveis, entrada e interrupção.
- O contrato Tauri `run:*`, `RunObserver`, `RunHandle` e o inspetor já estão ligados à VM clássica. `canDebugSource` ainda desabilita `.pors`.
- A experiência comum de execução `.pors` e Rust standalone permanece baseada em `rustc`; `.rs` continua sem depurador neste incremento.

## Abordagens consideradas

1. **GDB/LLDB sobre o binário Rust gerado.** Mantém o runtime compilado, mas depende de ferramentas de depuração instaladas e de integração distinta por plataforma. Isso torna a disponibilidade irregular em Windows, macOS e Linux.
2. **Instrumentar o Rust gerado com um protocolo de debug.** Preserva o backend `rustc`, mas acrescenta checkpoints, snapshots tipados, controle de pausa/step, multiplexação de `leia` e eventos do processo em um protocolo novo.
3. **Compilar a HIR `.pors` para o bytecode existente e reutilizar a VM.** Mantém um único motor de stepping/snapshot/entrada em todas as plataformas e usa os mesmos spans e bindings HIR; exige um backend HIR→bytecode e testes diferenciais que garantam equivalência observável com o Rust gerado.

**Decisão:** abordagem 3. O debugger executará o programa real pela HIR compilada ao bytecode e exibirá snapshots da VM. A execução normal de `.pors` continua compilando o Rust gerado. Testes diferenciais compararão a saída da VM e do Rust para construções suportadas. Se a equivalência falhar para uma construção, o debugger deve recusá-la com diagnóstico posicionado até que o backend seja corrigido; não deve mostrar um estado enganoso.

## Objetivos

- Depurar o subset atualmente aceito por `portugol-rs-frontend::lower`.
- Iniciar pausado, continuar, pausar na próxima instrução, step over/into/out e respeitar breakpoints por linha.
- Mostrar linha/coluna, valor, tipo e escopo de variáveis realmente avaliadas pela VM.
- Preservar `leia`, stdout, clear, stop e eventos identificados pelo mesmo `runId` usado pelo debugger clássico.
- Usar o `RunObserver`/`RunHandle` e os eventos Tauri `run:*` existentes, evitando um segundo protocolo de debugger na UI.
- Habilitar Depurar em `.pors` sem depender de `rustc`, pois o bytecode é executado pela VM interna; a ação Executar normal mantém a exigência atual da toolchain.
- Indicar claramente no console/status que aquela sessão está sendo depurada pela VM Portugol-RS.
- Aplicar limites de saída e de memória de strings equivalentes aos já aplicados à execução normal de `.pors`, além da interrupção explícita; pausas intencionais não contam como tempo de runtime.
- Manter fixtures do laboratório de ownership identificadas como `Simulação didática`; o novo trace não afirma observar ownership interno de Rust.

## Fora de escopo

- Depurador de arquivos `.rs` ou projetos Cargo.
- Alterar o backend de execução normal `.pors` ou a semântica da geração Rust.
- Inspecionar stack/heap/ownership da máquina Rust. O painel mostra variáveis do modelo de execução Portugol-RS.
- Acrescentar funções, bibliotecas, vetores ou outras construções ao subset da linguagem apenas para o debugger.
- Afirmar paridade semântica para construções que não estejam cobertas por testes diferenciais.

## Contratos e comportamento

- O frontend chama o novo ponto de entrada do serviço com fonte, breakpoints e `pause_on_start`; a análise é feita por `portugol_rs_frontend::lower`.
- Um compilador HIR→bytecode gera bindings locais nomeados e mapeia cada instrução para o span fonte correspondente. As identidades `BindingId` evitam confundir variáveis sombreadas.
- O runtime compartilhado cria `RunHandle` e `DebugControl`, instala os mesmos breakpoints e emite `paused`, `finished`, `output` e `input` no contrato atual.
- A pausa acontece antes de executar cada instrução com span, incluindo `leia`. Ao retomar `leia`, o programa emite `run:input` e espera sem emitir evento de pausa repetido; após aceitar a entrada, a pausa seguinte acontece antes da próxima instrução. `stop` interrompe tanto uma pausa do debugger quanto uma espera por entrada.
- Erros de lowering/compilação retornam diagnósticos Portugol-RS com offsets do fonte. O frontend descarta eventos de sessões antigas como já faz para outras execuções.
- O fluxo normal `Executar` usa `rustc` e mantém sua confirmação, limites e entrada atuais. `Depurar` explicita que usa a VM para mostrar valores reais daquele backend.
- Debugger pausado tem stop responsivo, inclusive durante `leia`; `stop` nunca é apresentado como conclusão normal.
- O orçamento de 3 segundos mede somente runtime ativo. Tempo gasto em uma pausa intencional e esperando resposta de `leia` fica excluído, como na execução `.pors` atual.

## Segurança, limites e riscos

- Código `.pors` continua sendo avaliado por um subset validado e sem shell nem processo externo no modo Depurar.
- O backend HIR deve respeitar o mesmo limite de saída de 1 MiB, orçamento cumulativo de cadeias de 16 MiB e runtime ativo de 3 segundos de `run_portugol_rs`; o projeto precisa de testes de parada por esses limites e por cancelamento.
- VM e Rust gerado são dois backends de execução. Os testes diferenciais são um gate do recurso e devem cobrir cada instrução HIR suportada, overflow, divisão por zero, curto-circuito, entrada tipada, loops e concatenação.
- O snapshot expõe valores de bindings ativos e não representa stack/heap de Rust, ownership ou empréstimos.

## Aceitação

1. Para um programa `.pors` com variáveis, condições e loops, F5/Depurar inicia pausado, step atualiza o snapshot antes de cada instrução e breakpoints param na linha selecionada.
2. Continue e Stop funcionam durante execução e pausa. Um programa interrompido termina como `stopped/cancelled`, sem mensagem de sucesso.
3. `leia` pede e valida um valor do tipo correto, depois o step seguinte mostra o valor lido.
4. Nomes sombreados permanecem distintos na HIR e o inspetor mostra apenas os bindings ativos.
5. Para fixtures de sucesso do subset atual, stdout e valores finais observáveis são iguais entre a VM e o Rust gerado. Para falhas de runtime, a categoria e o span do fonte devem coincidir; mensagem localizada e frames internos não precisam ser textualmente iguais. Limites de saída/memória/runtime devem produzir o mesmo resultado tipado. Todas as instruções executáveis mapeiam para spans dentro do fonte.
6. O debugger `.pors` funciona sem `rustc`; a execução normal continua indisponível quando a toolchain não existe.
7. Temas, Portugol clássico, saída normal `.pors`, argumentos Rust e o debugger clássico continuam passando pelas suítes existentes.

## Revisão visual e de produto

- Reutilizar controles, painel, cores, navegação e acessibilidade do debugger atual.
- Mostrar o backend `VM Portugol-RS` no status da sessão e manter a marcação de linha pausada.
- Manter `Simulação didática` no laboratório de ownership; a nova capacidade não altera a proveniência dos cenários editoriais.
- Captura E2E de tela nativa só será reportada como feita se houver um runner disponível. Nesta implementação, contratos de UI/Tauri e fluxos da VM são verificáveis automatizadamente sem screenshot.

## Decisões editoriais para o builder

- Step e breakpoints param antes da instrução associada ao span; em `leia`, a parada ocorre antes do pedido de entrada e a execução continua aguardando a interface sem classificar a espera como pausa.
- Testes de equivalência comparam observáveis de programa (saída, valores, resultado tipado e posição), não a redação das mensagens Rust versus Portugol.
