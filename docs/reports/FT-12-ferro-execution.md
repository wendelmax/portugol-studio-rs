# FT-12 — Execução de Ferro com `rustc`

- **Escopo:** executar `.fer` compilando o código Rust produzido pelo exportador existente, sem alterar a VM clássica.
- **Fluxo:** o serviço valida e exporta o subset aceito, grava o Rust em uma pasta temporária privada, chama `rustc --edition=2021` com argumentos fixos e executa o binário. A pasta temporária é removida ao sair do escopo.
- **IDE:** consulta a disponibilidade de `rustc`; habilita Executar para `.fer` somente quando disponível e mostra stdout, stderr e código de saída reais. Resultados de uma aba/documento que mudou durante a execução são descartados.
- **Limites explícitos:** somente sintaxe aceita pelo exportador Ferro atual; sem execução de arquivos `.rs`, entrada interativa, depurador, pausa ou interrupção. Falta de toolchain, sintaxe não suportada e erros de compilação são informados ao usuário. O limite de origem é 1 MiB.
- **Segurança operacional:** não há shell, Cargo, nem execução direta de código Rust fornecido pelo usuário; só é compilado o código gerado pelo subset validado.
- **Validação local:** testes do serviço, frontend, build, formatação, Clippy do serviço e desktop e `git diff --check` passaram nesta máquina. A checagem desktop usa metadados `pkg-config` temporários porque o ambiente não expõe WebKitGTK; a validação em CI continua necessária.
- **Pendente:** CI do PR, teste interativo na aplicação Linux e validar instalação/descoberta da toolchain em Windows e macOS.
