# Playground editor-like com arquivos locais e trilha — design

## Intenção
Dar ao playground web noção de arquivo, abas e sequência de aprendizado. O usuário abre um `.fer` local, edita, salva e avança por uma trilha de lições curtas, sem sair do navegador.

## Objetivos
- Abrir arquivos `.fer` do computador, criar novo e salvar no próprio arquivo quando o navegador permitir.
- Interface com três zonas: lateral (arquivos, exemplos, trilha), abas do editor e painel inferior com abas (Terminal, Prévia UI, Rust gerado, Referência).
- Ampliar de 4 para 12 exemplos, agrupados por tema, todos executáveis.
- Trilha com 6 lições: enunciado curto, código inicial, verificação por saída esperada, dica e progresso salvo no navegador.
- Manter autocomplete, terminal, temas e responsividade já existentes.

## Não-objetivos
- Backend, conta, nuvem ou sincronização entre dispositivos.
- CodeMirror, npm, bundler ou qualquer dependência JavaScript.
- Mudanças em `crates/ferro-wasm`, no protocolo de memória ou em `pors_run(source, stdin)`.
- E/S interativa durante a execução.

## Restrições
- `apps/web-playground` continua estático; build copia todos os módulos.
- Sem `innerHTML`: DOM via `createElement`/`textContent`.
- Textos em pt-BR; temas claro/escuro e breakpoints 980/720/420 preservados.
- Testes com `node --test` a partir da raiz, sem `node_modules`.

## Abordagens
1. Só aumentar a lista de exemplos: barato, mas mantém a tela sem fluxo. Descartada.
2. Casca de editor com lateral, abas e painel inferior (escolhida): organiza arquivo → edição → execução → aprendizado com CSS grid e um módulo de estado pequeno, sem dependências.
3. Portar a IDE desktop/Tauri para a web: resolve arquivos, mas traz backend, dependências e um build separado. Descartada.

## Decisões
- Arquivos locais: `showOpenFilePicker`/`showSaveFilePicker` + `createWritable()` quando disponíveis; fallback com `<input type="file">` e download por Blob. Módulo `file-system.mjs` recebe `window`/`document` por parâmetro para ser testável.
- Abas em memória (não reabertas automaticamente); progresso da trilha em `localStorage` sob `pg-lessons`.
- Primeira abertura: `boas-vindas.fer`, o exemplo de saudação do catálogo, já como aba ativa; trilha visível na lateral. Escolher outro exemplo ou arquivo abre uma aba nova, ou ativa a existente — nunca substitui a aba atual.
- Testes Node usam exports WASM falsos, como os atuais; a validação dos exemplos oficiais continua nos testes Rust existentes. Nenhum build de WASM é adicionado ao `node --test`.
- Exemplos: 12, escolhidos do catálogo oficial `crates/ferro-service/assets/exemplos-pors` (já cobertos por teste Rust de compilação): saudação, entrada, tipos, decisão, laços, funções, vetores, registro, Opção/Resultado, bibliotecas, UI e exercício aberto.
- Trilha: definições separadas em `lessons.mjs` (variáveis, decisões, laços, vetores, cadeias, Resultado), com `starter` próprio, `stdin` fixo e `expected` de stdout. Não são os exemplos prontos e não entram na contagem dos 12.
- Verificação de lição: usa o buffer atual do editor e o `stdin` fixo da lição; o campo de entrada fica bloqueado durante a lição. Compara linhas de stdout (sem contar `\n` final); erro de compilação conta como não aprovado. Sem novo export WASM.
- Navegação da trilha é livre: lições anteriores não são trancadas; o progresso só marca conclusão.
- Referência de sintaxe vira aba do painel inferior, mantendo o mesmo conteúdo de `ferro-language.mjs`.
- PR #294 foi mesclado em `main` (`228332c`); esta fatia nasce de `main` atualizado.

## O que fica para o implementador
- Distribuir os exemplos em `playground-core.mjs` com `id`, `title`, `group`, `mode`, `input`, `source` e, quando fizer parte da trilha, `lesson`.
- Implementar a casca em `index.html`/`style.css` e o estado de abas/arquivos em `app.mjs`, preservando os ids usados pelos testes atuais.
- Criar `file-system.mjs`, `lessons.mjs` e testes correspondentes; todos os exemplos da trilha devem passar na verificação com o runtime real.
