# Ferro Studio Next — shell visual

## Objetivo

Levar a tela desktop existente em Rust + Tauri para a composição do mockup e dos contratos de design fornecidos pelo usuário: barra superior, rail de atividade, explorador, workspace de editores, inspector pedagógico, painel inferior e status bar.

## Abordagem

Reorganizar o HTML/CSS e os handlers da aplicação existente, preservando CodeMirror, serviços Rust, comandos Tauri, IDs funcionais dos controles e as convenções de build. Não criar outro frontend nem trocar framework. O plano é uma fatia de shell: os serviços e capacidades ausentes permanecem explicitamente indisponíveis.

## Requisitos visuais e funcionais

- Aparência escura baseada nos tokens semânticos do design: fundo `#0C1118`, painéis `#111923`, editor `#0D141D`, acento `#34D399`; manter os temas existentes selecionáveis.
- Em 1536×960: rail 56 px, explorador ~232 px, inspector ~336 px; fonte e Rust gerado lado a lado quando o modo Ponte fornece uma tradução.
- Em 1280×800: editor e Executar permanecem visíveis; inspector recolhe por padrão e o Rust é alternável, sem colunas ilegíveis.
- Abas de arquivo aparecem sobre o workspace central; `btn-run`, `btn-debug`, `btn-stop`, save/open, navegação, teclado e estados dirty continuam ligados aos handlers atuais.
- Aprender reutiliza o catálogo didático já existente; depuração e variáveis reutilizam a VM atual; ownership continua identificado como simulação didática.
- Console, testes, saída e problemas mostram somente dados reais. O modo Rust permanece desabilitado enquanto runtime e editor próprios não existirem.
- Sem compartilhamento em nuvem, novos comandos Tauri, novos pacotes ou mudanças na semântica das linguagens nesta fatia.

## Aceite

As composições de 1536×960 e 1280×800 não sobrepõem nem cortam o editor, o botão Executar ou os painéis. Rail, tabs e controles essenciais têm nomes acessíveis e funcionam por teclado. Testes verificam markup/contratos estáveis; build e smoke Tauri validam o shell nativo. Qualquer limitação de screenshot automation é registrada como `NOT_RUN`.
