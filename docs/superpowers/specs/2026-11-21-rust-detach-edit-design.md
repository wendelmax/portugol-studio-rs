# Design: editar Rust real como documento independente

## Objetivo

Completar a ponte de aprendizagem permitindo que o aluno transforme a prévia gerada em um documento Rust editável, sem alterar o programa de origem. O projeto mantém `.por` para pseudocódigo clássico e `.fer` para Ferro; Rust real usa `.rs`.

## Experiência

- A prévia gerada permanece somente leitura e identifica o código como gerado.
- A ação **Desvincular e editar** cria uma nova aba `.rs` com o texto atual da prévia.
- A aba `.rs` é independente. Traduções posteriores não modificam seu conteúdo.
- Arquivos `.rs` podem ser abertos, editados, salvos, criados, renomeados e excluídos no Explorador.
- O editor usa realce de sintaxe Rust e oferece salvar como `.rs`.
- Rust ainda não é executado, depurado, inspecionado como Ferro nem traduzido novamente. Os controles incompatíveis ficam desabilitados com explicação acessível.

## Alternativas

1. Manter só copiar/salvar da prévia: simples, mas o aluno precisa sair e reabrir o arquivo para continuar o aprendizado.
2. Desvincular para uma aba `.rs` editável (escolhida): dá continuidade real ao fluxo e separa claramente origem e resultado.
3. Executar compilador Rust nesta etapa: amplia segurança, configuração e dependências do sistema; fica fora do escopo enquanto não houver contrato de compilação integrado.

## Regras e estados

- `.por`, `.fer` e `.pex` mantêm os comportamentos atuais.
- `.rs` nunca passa pelo interpretador Ferro nem pelo inspetor/formatador/completador Ferro.
- Desvincular sem código gerado disponível fica indisponível.
- Cancelar salvar como não altera caminho, linguagem ou estado salvo.
- A prévia só deve ser oferecida quando a aba ativa for Ferro ou Ferro.
- Workspace aceita `.rs` nas operações já existentes para fontes suportadas, com a mesma restrição de caminhos e operações exclusivas existente.

## Validação

Testes unitários devem cobrir detecção/extensão, classificação e nomes `.rs`, e contratos Rust do workspace/abertura de arquivos. Build TypeScript, suíte frontend e testes Rust/CI validam integração. A disponibilidade local de WebKitGTK pode impedir testes Tauri no Linux.
