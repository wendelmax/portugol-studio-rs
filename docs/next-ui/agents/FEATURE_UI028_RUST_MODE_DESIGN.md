# Design — modo de aprendizagem Rust

## Intenção

Concluir o terceiro nível da progressão pedagógica usando o editor `.rs` e a execução Rust que já existem. O aluno poderá selecionar Rust, estudar fundamentos de Rust e abrir/criar arquivos `.rs` sem converter ou alterar um arquivo Portugol implicitamente.

## Evidência e restrições

- `apps/desktop/src/main.ts` já seleciona `@codemirror/lang-rust`, abre/salva `.rs`, consulta `rust_toolchain_available` e executa com `run_rust_source`.
- `apps/desktop/index.html` expõe o modo Rust como indisponível; a mensagem sobre editor e compilador é imprecisa.
- `apps/desktop/src/learning-mode.ts` e `learning-content.ts` oferecem apenas Beginner/Bridge.
- A execução Rust não é sandboxed e exige consentimento; esses controles e limites existentes devem permanecer.
- A mudança não altera Tauri, permissões, IPC, parser, runtime, linguagem de arquivos ou formato `.por`/`.pors`.

## Abordagem escolhida

Habilitar `rust` como terceiro `LearningMode`, condicionado à disponibilidade real de `rustc`. O modo controla o conteúdo pedagógico e as orientações; nunca transforma o documento ativo, troca silenciosamente o motor nem afirma que `.por` virou Rust. Se o toolchain faltar, a opção permanece desabilitada e explica que Rust não está configurado. Em arquivos `.rs`, continuam valendo o editor, o salvamento e o executor existentes, incluindo o consentimento de segurança.

Alternativa descartada: criar uma segunda tela/editor/compiler Rust. Duplicaria componentes que já funcionam e aumentaria o risco de regressão.

## Escopo

- Incluir Rust no tipo, carregamento e persistência de modos.
- Habilitar/desabilitar o seletor a partir da resposta real de `rust_toolchain_available`.
- Mostrar descrição pedagógica de Rust e conceitos existentes de mutabilidade/ownership sem alegar detecção pelo cursor.
- Manter o laboratório como simulação didática verificada e rotulada.
- Atualizar matriz, gap report e relatório de feature.

## Fora de escopo

Formatador Rust, suporte a argumentos, debugger Rust, Cargo/projetos, workspace persistente, novos conceitos sem validação curricular e mudanças no contrato de execução.

## Critérios de aceite

1. `rust` é um valor persistível válido; dados inválidos ainda caem em `beginner`.
2. Sem toolchain, o controle Rust fica desabilitado com instrução correta; com `rustc`, fica selecionável.
3. Trocar para Rust não altera texto, arquivo nem engine do documento ativo.
4. Os conceitos e laboratório Rust exibidos vêm do catálogo existente, com proveniência manual/simulação claramente identificada.
5. Arquivos `.rs` usam o caminho real existente e mantêm aviso/consentimento de execução.

## Riscos e rollback

O risco principal é misturar o nível pedagógico com a linguagem do arquivo. Mitigação: modo separado, sem mutação do documento, e teste de integração estático focado nas ações observáveis. Rollback consiste em retirar o terceiro modo e restaurar o bloqueio antigo; editor e execução `.rs` não dependem dele.
