# FT-16 — Loop contado no Portugol-RS

- **Sintaxe aceita:** `para (seja mutavel i: inteiro = 0; i < 10; i += 1) { ... }`; controle `inteiro` ou `real`, condição lógica e atualização composta do mesmo binding.
- **Semântica:** o cabeçalho exige `seja`; inicializador tem escopo pelo cabeçalho e corpo; o controle não escapa do loop; o corpo ganha bloco próprio antes do step; shadowing em loops aninhados mantém binding IDs distintos.
- **Geração:** inicialização + `while` + atualização após cada corpo dentro de bloco Rust local; atualizações inteiras usam as operações de wrapping existentes. `rustc` executa o resultado.
- **Mapeamento:** source maps exatos cobrem a declaração, condição, atualização e corpo; teste com `é` valida fronteiras UTF-8, ranges dentro da origem/Rust, conteúdo mapeado e preâmbulo `generated_only`.
- **Limites:** `pare`, `continue`, `faca-enquanto`, `escolha`, funções adicionais e cabeçalhos opcionais continuam fora do subset. A execução continua com timeout/limites tipados existentes; `.por` clássico não foi alterado.
- **Validação local:** `cargo fmt --all -- --check`, `cargo test --locked --workspace --exclude portugol-desktop` (passou com corpus, execução de `.pors`, shadowing no corpo, divisão/resto por zero e regressões clássicas), `cargo clippy --locked --workspace --exclude portugol-desktop --all-targets -- -D warnings`, `PKG_CONFIG_PATH=/tmp/ft10-pkgconfig cargo check --locked --manifest-path apps/desktop/src-tauri/Cargo.toml --tests` e `git diff --check` passaram. O CI validará as plataformas suportadas.
