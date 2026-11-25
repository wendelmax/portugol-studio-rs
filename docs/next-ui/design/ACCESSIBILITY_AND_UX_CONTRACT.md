# UX e acessibilidade — condições de aceite

- Navegação pelo teclado em árvore de arquivos, abas, editor, painéis, command palette, execução e debugging.
- Foco visível, sequência de foco coerente, atalhos remapeáveis quando possível; não capturar atalhos do sistema operacional.
- Controles sem texto devem ter accessible name; status anunciado em canal acessível sem bombardear leitor de tela a cada caractere de stdout.
- Contraste WCAG AA: texto normal ≥4.5:1, componentes/indicadores essenciais ≥3:1; testar paleta final, não apenas presumir.
- Zoom 200%, escalabilidade tipográfica, fontes monoespaçadas com fallback, redação simples em pt-BR.
- Erros informam localização, causa e como corrigir; nunca apenas vermelho. Mesmo feedback para mouse e teclado.
- Painéis redimensionáveis por mouse e teclado; ao estreitar, colapsar inspector antes de sacrificar editor.
- Preferência de movimento reduzido e temas persistentes.
- Permitir reconhecimento de controle desabilitado e explicação de capacidade ausente.
- Captura de screenshots de regressão sem dados privados/caminhos locais.

## Critérios de teste UI
1. Fluxo completo sem mouse (abrir arquivo → editar → salvar → executar → ler resultado).
2. Screen reader: botões e erros têm nomes/descrições.
3. Alargamento/redução de janela não esconde editor ou RunButton.
4. Diálogos de confirmação prendem/restauram foco corretamente.
5. Em modo bridge, código gerado rotulado como gerado, desatualizado ou desvinculado.
