# Verificação manual de acessibilidade

Este roteiro cobre a IDE desktop com teclado e leitor de tela. Execute-o antes
de uma versão que altere o shell, os temas ou os controles do editor.

## Fluxo sem mouse

1. Na tela inicial, pressione `Ctrl+O`, escolha um arquivo `.fer` no seletor
   de arquivos usando apenas teclado e confirme com `Enter`.
2. Navegue até o editor com `Tab` ou `Shift+Tab`. O leitor de tela deve anunciar
   “Editor de código”; digite uma alteração pequena.
3. Pressione `Ctrl+S`. Se o diálogo de salvar aparecer, escolha o destino e
   confirme usando `Tab`, as setas e `Enter`.
4. Pressione `Shift+F6` para executar o programa.
5. Use `Tab` até os painéis de saída e selecione “Console” com `Enter`. O painel
   focado deve permitir ler a saída do programa com o leitor de tela.
6. Repita com uma fonte que contenha um erro: o painel “Mensagens” deve
   anunciar o tipo e o texto do diagnóstico, e não depender apenas da cor.

## Preferências visuais

- Ative “Reduzir movimento” no sistema e confirme que a navegação não depende
  de animação ou rolagem suave.
- Selecione o tema “Alto contraste”. Verifique texto, editor, seleção, foco,
  erros e sucesso. Mensagens de estado devem continuar compreensíveis pelo
  texto mesmo quando as cores forem indistinguíveis.
- Em um sistema que ofereça cores forçadas, ative esse modo e confirme que
  fundo, texto, controles, seleção e foco usam as cores de contraste do sistema.

Anote o sistema operacional, o leitor de tela, o tema/preferência visual e o
passo em que qualquer problema ocorreu.
