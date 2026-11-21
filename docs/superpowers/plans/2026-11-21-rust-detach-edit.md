# Plano: desvincular e editar Rust real

1. Estender o modelo de linguagem documental e contratos de nomes `.rs`; adicionar testes primeiro.
2. Habilitar suporte `.rs` no backend Tauri: filtro de abertura, listagem, leitura, gravação, criação, renomeação e exclusão; adicionar testes Rust primeiro.
3. Integrar gramática Rust no CodeMirror e separar extensões/diagnósticos/recursos Portugol por linguagem.
4. Adicionar a ação de desvincular à prévia, criar aba `.rs` editável e garantir que ela não participe da geração de tradução.
5. Completar salvar, salvar como, criação no workspace, marcação visual e bloqueios honestos de execução/depuração.
6. Rodar suíte frontend, build, formatação e testes Rust disponíveis; revisar todas as transições entre `.por`, `.pors` e `.rs`.

## Revisão de risco

- Não alterar o contrato de execução Portugol.
- Não habilitar execução de arquivos arbitrários nem afirmar que Rust compila.
- Confirmar que o conteúdo de origem não muda ao desvincular ou editar a nova aba.
- Confirmar ações desabilitadas e explicações acessíveis em `.rs`.
