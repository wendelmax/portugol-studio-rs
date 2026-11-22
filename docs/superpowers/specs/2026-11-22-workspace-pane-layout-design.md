# Layout de painéis por workspace — design

## Objetivo

Fazer cada workspace lembrar as proporções do explorador, painel inferior e painel de utilitários. Ao alternar entre abas de projetos diferentes, a IDE restaura o layout associado à aba. Isso aproxima o comportamento do contrato de `DESIGN_AS_CODE.md` sem introduzir um novo store ou alterar os limites de resize.

## Abordagens

1. **Uma chave global (atual):** simples, mas alternar projetos muda a configuração de todos; não atende ao contrato.
2. **Store separado em Tauri:** centraliza preferências no backend, mas aumenta comandos, persistência e migração sem necessidade para uma preferência local.
3. **Chaves namespaced no `localStorage` (escolhida):** reaproveita a identidade SHA-256 já entregue por `WorkspaceListing.preferenceKey`; mantém valores proporcionais e usa a chave legada como fallback.

## Comportamento

- `loadPaneLayout(storage, preferenceKey?)` lê o layout específico do workspace e, se ausente/inválido, o layout global legado.
- `savePaneLayout(storage, layout, preferenceKey?)` grava no workspace ativo quando fornecido; fora de workspace mantém a chave legada.
- Ao ativar uma aba, `main.ts` troca o estado `paneLayout` para a chave daquela aba antes de reaplicar os tamanhos renderizados.
- As proporções continuam normalizadas pelos limites já existentes; o modo de aprendizagem, a fonte e a execução permanecem independentes.
- Preferências continuam locais ao perfil. Não sincronizam entre máquinas. Mover a pasta gera chave nova e herda o layout global como padrão.

## Não objetivos

- Não adicionar botão de reset nesta mudança.
- Não alterar breakpoints, limites de tamanho, CSS, aparência das divisórias ou armazenamento das outras preferências.
- Não migrar os valores globais para todos os workspaces automaticamente; o layout global serve como fallback até o primeiro resize local.

## Restrições e validação

- Preservar os contratos Tauri existentes e a API de `WorkspaceListing`.
- Cobrir layout por workspace, independência, fallback global, aba standalone, armazenamento indisponível e aplicação ao alternar abas.
- Executar testes Node, build frontend, formatação Rust e CI Tauri multiplataforma.
