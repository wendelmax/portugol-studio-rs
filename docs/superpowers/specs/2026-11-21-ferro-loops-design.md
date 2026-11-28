# Repetição `enquanto` no Ferro

## Intenção

Permitir que estudantes expressem repetição no dialeto Ferro e vejam a
mesma construção gerada em Rust, preservando a HIR compartilhada e os source
maps. Como loops podem não terminar ou produzir saída continuamente, a execução
do Rust gerado precisa ter limites explícitos.

## Estado atual

- O parser Ferro reconhece `enquanto` como construção ainda não suportada
  e devolve um diagnóstico localizado.
- A HIR contém `se`, mas não contém repetição.
- O backend Rust gera o subset da HIR e o serviço executa o binário por meio de
  `Command::output`, que espera sem prazo e captura stdout/stderr sem limite.
- O frontend mostra sucesso, falha de compilação e saída de runtime, mas não um
  limite de runtime.

## Contrato da linguagem

```text
enquanto (condicao_logica) {
    comandos
}
```

- Condição deve ter tipo `logico`.
- O bloco tem escopo próprio, igual ao bloco de `se`.
- Declarações e atribuições seguem as regras atuais de binding e mutabilidade.
- `pare`, `continue`, `faca-enquanto`, `para` e rótulos não fazem parte desta
  fatia; continuam recebendo diagnósticos localizados.
- O gerador emite `while <condição> { <corpo> }` em Rust e preserva segmentos
  de source map da condição, comandos internos e statement completo.
- O lowering clássico de `.por` não muda sua compatibilidade nesta entrega.

## Limites de execução

- O prazo máximo de execução gerada é 3 segundos, contado depois que o binário
  começa a executar (a compilação Rust não consome esse orçamento).
- stdout e stderr compartilham um limite de captura de 1 MiB por execução.
- O código gerado conta alocações cumulativas de strings e para ao atingir
  16 MiB; cada resultado de concatenação é verificado antes de alocar. Os
  literais continuam limitados pelo limite de 1 MiB da fonte.
- O runner drena os pipes em threads enquanto acompanha o processo; ao exceder
  o limite ou o prazo, encerra e recolhe o processo e retorna o texto capturado.
- Uma interrupção por limite é um resultado IPC tipado `runtime_limit`, com a
  razão `time`, `output` ou `string_memory`; falhas de spawn/pipe continuam
  erros de transporte.
- Não há shell, processo arbitrário, nova dependência, nem alteração na VM do
  pseudocódigo clássico.

## Abordagens consideradas

1. **Permitir o loop e continuar com `Command::output`:** menor mudança, mas um
   loop infinito congela o worker e uma saída grande pode consumir memória sem
   limite. Rejeitada.
2. **Adicionar um interpretador FERRO com timeout:** permite controle fino, mas
   duplica a semântica que o `rustc` já verifica e cria um segundo backend.
   Rejeitada nesta fatia.
3. **Estender HIR/codegen e supervisionar o processo Rust gerado:** preserva a
   semântica do compilador Rust e adiciona prazo, captura limitada e orçamento
   de strings sem nova dependência. Escolhida.

## Arquitetura e arquivos

- `crates/ferro-hir`: nó `StmtKind::While` com condição e corpo.
- `crates/ferro-rust-frontend/src/parser.rs`: parser do statement, regra de
  tipo e escopo.
- `crates/ferro-rust/src/codegen.rs`: emissão/source maps do loop e guardas
  de string apenas na saída Ferro, sem alterar a geração clássica.
- `crates/ferro-rust/src/hir.rs`: manter lowering clássico recusando loops
  nessa entrega, se o AST clássico ainda não puder representar a mesma HIR sem
  alterar a compatibilidade.
- `crates/ferro-service/src/ferro_run.rs`: resultado IPC `runtime_limit`,
  captura concorrente limitada e supervisão do processo.
- `apps/desktop/src/api.ts` e `apps/desktop/src/main.ts`: tipo e mensagem de
  limite em português preservando o texto capturado no console.
- `docs/PARIDADE.md` e relatório FT-14: registrar gramática e limites.

## Não objetivos

- Loops no interpretador clássico, `pare`/`continue`, `para`, funções
  adicionais, vetores ou ownership.
- Cancelamento manual do runner FERRO.
- Prazo de compilação `rustc`, quota RSS do sistema operacional e isolamento
  total do processo.
- Alterar associação do arquivo `.fer` ou nomes de comandos Tauri.

## Verificação

- Parse e HIR de um loop que atualiza binding mutável e termina.
- Diagnóstico localizado para condição não lógica e comandos ainda não
  suportados dentro do loop.
- Compilação/execução real com `rustc` e source map nos spans do loop.
- Resultados tipados de timeout, saída excessiva e orçamento de strings; captura
  total <= 1 MiB e alocações rastreadas <= 16 MiB.
- Regressões clássicas e todos os testes de workspace suportados no host/CI.
