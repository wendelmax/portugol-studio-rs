# Portugol Studio Next — Shell visual

## Implementado

- Tema padrão `Portugol Studio Next` para instalações sem preferência salva, preservando temas e preferências anteriores.
- Barra superior identificada, rail de atividades e abas no workspace do editor.
- Atividades ligadas a explorador, aprendizado, exemplos, depuração e configurações existentes.
- Aprendizado e laboratório de ownership no inspector; a simulação mantém sua proveniência didática explícita.
- Inspector e explorador abertos por padrão a partir de 1440 px; recolhidos no primeiro uso abaixo desse limite.
- Controles de bibliotecas, modelos, estrutura, variáveis, execução, salvamento e atalhos mantidos.

## Validação

- `npm test`: 21 testes passaram.
- `npm run build`: TypeScript e Vite passaram; permanece o aviso existente de chunk JavaScript acima de 500 kB.
- `cargo test --workspace`: passou, incluindo testes unitários, regressão do corpus Portugol Studio e doctests.
- `npm run tauri dev`: compilação nativa e inicialização do processo Tauri completaram na sessão KDE. O GTK emitiu o aviso `gtk_window_set_titlebar() called on a realized window`.
- Captura e inspeção visual: `NOT_RUN`. A captura do desktop mostrou somente a tela de bloqueio do KDE, impedindo conferir a janela e suas interações nos tamanhos 1536×960 e 1280×800.

## Pendências de verificação

- Revisar a janela nativa desbloqueada em 1536×960 e 1280×800, incluindo recolhimento/redimensionamento das laterais e navegação por teclado.
- Atualizar este relatório com capturas e resultados quando a sessão gráfica permitir essa inspeção.
