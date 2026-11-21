# FEAT-WORKSPACE-002 — criação de programas no Explorador

## Entregue

- Diálogo acessível para criar Portugol clássico (`.por`) ou Portugol-RS (`.pors`) na raiz do workspace aberto.
- O arquivo aparece na árvore e abre em uma nova aba, usando o fluxo de salvamento já existente.
- Nome vazio, separadores de caminho, extensões incompatíveis, formatos não suportados e colisões são rejeitados.
- Rust cria exclusivamente; arquivo existente e symlink nunca são truncados nem seguidos.

## Validação local

- `cd apps/desktop && npm test` — as sete suítes de teste passaram.
- `cd apps/desktop && npm run build` — TypeScript e Vite passaram; aviso existente de chunk acima de 500 kB.
- `cargo fmt --all` — aplicado; conferir com `cargo fmt --all -- --check`.
- Testes Rust do desktop — não executáveis neste host porque `webkit2gtk-4.1` e `javascriptcoregtk-4.1` estão ausentes. O CI Linux/macOS/Windows deve validar o módulo Tauri e os testes novos.
- Instalação frontend — `npm ci --ignore-scripts`, sem vulnerabilidades reportadas.

## Ainda fora desta fatia

Criar arquivos em subpastas, renomear, excluir, criar pastas e restaurar o último workspace após reiniciar.
