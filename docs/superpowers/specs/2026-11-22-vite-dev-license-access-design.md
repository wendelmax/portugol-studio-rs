# Acesso do Vite de desenvolvimento à licença — design

## Objetivo

Permitir que o frontend do Tauri em desenvolvimento carregue a licença localizada na raiz do repositório, sem enfraquecer a política de arquivos servidos pelo Vite.

## Estado atual

`apps/desktop/src/main.ts` importa `../../../LICENSE?raw`. O Vite inicia com a raiz em `apps/desktop`, e `vite.config.ts` não inclui a raiz do workspace em `server.fs.allow`; o WebView recebe erro ao carregar esse recurso no `tauri dev`.

## Abordagens

1. **Adicionar somente a raiz do repositório a `server.fs.allow` (escolhida).** Mantém a licença como fonte única e limita o acesso de desenvolvimento aos arquivos do próprio projeto.
2. Copiar a licença para `apps/desktop/public`. Evita ampliar a allowlist, mas duplica conteúdo legal e cria risco de divergência.
3. Desativar a proteção de filesystem do Vite. Não é aceitável: expõe caminhos fora do projeto sem necessidade.

## Limites

- Apenas configuração do servidor de desenvolvimento; não altera o build de produção, IPC, permissões Tauri ou conteúdo de licença.
- Não permitir acesso à home inteira ou a caminhos arbitrários.
- Teste automatizado deve importar a configuração e validar que a allowlist contém a raiz do repositório.

## Resultado esperado

`npm run tauri dev` compila e o WebView consegue importar `LICENSE?raw`; a allowlist continua restrita à raiz deste repositório. O smoke visual nativo permanece responsabilidade do CI Linux/ambiente KDE.
