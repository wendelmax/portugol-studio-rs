# Projeto Cargo no editor Rust — design

## Objetivo

Permitir compilar e executar um projeto Cargo existente ao abrir um de seus arquivos Rust pelo explorador do workspace, mantendo os fluxos de Portugol, Portugol-RS e `.rs` standalone.

## Fluxo

1. O backend verifica o arquivo `.rs` relativamente ao workspace registrado e procura o `Cargo.toml` ancestral mais próximo.
2. A UI identifica o modo Cargo, desativa argumentos ainda não suportados e não aplica o linter de buffer standalone.
3. A execução exige consentimento para rede, build scripts, macros procedurais e dependências locais, sem prometer sandbox.
4. O usuário salva arquivos Rust alterados no projeto.
5. Um comando Tauri tipado usa executáveis/argumentos fixos, compila com eventos JSON e executa com `cargo run` para manter o ambiente e a seleção de alvo padrão do Cargo.
6. O supervisor aplica timeout, limite de saída e cancelamento; diagnósticos e saída chegam ao console.

## Limites desta entrega

- Sem seleção manual de múltiplos binários, args de runtime, `cargo check` em tempo real ou debugger Rust.
- Não instala Cargo; informa quando a toolchain não está disponível.
- A execução herda permissões locais. Não há isolamento forte nem alteração de permissões Tauri.
- Fora de um projeto Cargo, `.rs` continua usando `rustc` standalone.

## Verificação

- Testes Rust de projeto funcional, erro rustc renderizado, manifestos confinados e caso standalone.
- Teste frontend do contrato UI → API Tauri → executor, build TypeScript/Vite e suíte Node.
- Testes Tauri do workspace e validação de formato/Clippy.
