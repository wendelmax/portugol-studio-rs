# Design: layout de painéis ajustável e persistente

## Objetivo

Permitir que estudantes ajustem o Explorador, o painel inferior e o painel Utilitários com mouse ou teclado. Restaurar proporções ao reabrir a IDE sem deixar painéis fora da janela.

## Comportamento

- Divisórias têm nome acessível, foco por Tab, orientação e valores ARIA atualizados.
- Setas ajustam a largura/altura em passos de 24 px; Home e End vão aos limites.
- Mouse preserva o arraste existente.
- Proporções ajustadas ficam em `localStorage`; valores inválidos ou indisponibilidade de storage retornam aos padrões.
- Cada painel é limitado pela área disponível e seus mínimos atuais; redimensionar a janela recalcula os pixels a partir da proporção persistida.
- O Explorador e sua divisória somem juntos quando ocultos ou quando o editor é expandido.

## Alternativas consideradas

1. Persistir pixels exatos: simples, mas restaura mal quando a janela muda de resolução.
2. Persistir proporções com limites e fallback local: escolhido para preservar a intenção e se adaptar ao tamanho da janela.
3. Adicionar um LayoutStore Rust/Tauri: seria mais complexo e não necessário para preferência visual local nesta fase.

## Limites

Layout é uma preferência local da aplicação, não parte do workspace e não é sincronizado entre máquinas. A mudança não cria painel de Testes nem altera a execução.

## Validação

Testes unitários cobrem limites, valores inválidos e storage indisponível. Testes de contrato conferem atributos de teclado/ARIA; build, suite frontend e CI validam integração desktop.
