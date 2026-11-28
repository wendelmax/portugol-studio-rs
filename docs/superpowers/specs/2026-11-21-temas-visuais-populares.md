# Temas visuais populares

## Intenção

Oferecer mais opções visuais aplicáveis à IDE, mantendo a escolha local e disponível sem rede. Pessoas que já usam VS Code devem reconhecer paletas conhecidas e conseguir trocar o tema pelo seletor existente.

## Objetivos

- Manter Ferro, Dark, Sistema e perfis personalizados.
- Adicionar GitHub Light, GitHub Dark, One Dark Pro, Dracula, Catppuccin Mocha e Night Owl como presets instalados.
- Aplicar a paleta ao editor, moldura, início, painéis, console e demais componentes que usam as variáveis de tema.
- Persistir a seleção com o mecanismo atual e manter a edição de perfis personalizados.
- Mostrar nome do tema e agrupamento claro no seletor.

## Fora de escopo

- Baixar ou importar extensões do Marketplace.
- Alterar comportamento, teclas ou recursos do editor.
- Incluir imagens ou fontes externas.

## Abordagens consideradas

1. Carregar temas do Marketplace em tempo de execução. Requer rede, adiciona dependência de serviço externo e não atende ao uso offline.
2. Incluir presets de cores no app e reutilizar o seletor e a persistência existentes. Mantém a troca offline, sem novas dependências e cobre a IDE inteira por variáveis CSS. **Escolhida.**
3. Criar uma galeria de temas e um novo mecanismo de aplicação. Duplica o seletor e a persistência existentes sem benefício necessário para seis presets.

## Decisões de implementação

- As cores serão paletas de integração próprias, inspiradas em referências populares; a aplicação não incorporará pacotes de temas externos.
- As referências foram escolhidas por popularidade no [ranking de temas do VS Code Marketplace](https://www.vsxatlas.com/rankings?category=Themes&registry=marketplace), com GitHub Theme, One Dark Pro, Dracula, Catppuccin e Night Owl entre os temas mais instalados.
- Cada preset terá cores para as variáveis de interface e sintaxe que já existem, com base clara ou escura para `color-scheme` e CodeMirror.
- Presets serão imutáveis; a pessoa ainda poderá criar perfis editáveis no editor de temas já existente.
- A lista terá grupos de temas originais, presets populares e perfis personalizados.

## Restrições

- Sem dependências de execução adicionais e sem requisições de rede.
- Compatibilidade com Linux, Windows e macOS.
- Reutilizar `localStorage` e variáveis CSS atuais.

## Aceitação

- Cada um dos seis presets pode ser selecionado, aparece de ponta a ponta na IDE e persiste após reiniciar.
- Ferro, Dark, Sistema e temas personalizados continuam aplicáveis.
- Os testes validam nomes, base e cobertura de cores de todos os presets; o build e a inspeção visual do app passam.
