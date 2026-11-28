# Prompt de implementação — Ferro Studio Next

Você está trabalhando no repositório existente do Ferro Studio reimplementado em Rust.

Leia, nesta ordem:
1. `DESIGN_AS_CODE.md` — princípios, visual, layout, tokens e experiência.
2. `UI_FUNCTIONAL_CONTRACT.md` — IDs dos componentes, funcionalidades, contratos, fluxos e critérios.
3. `features/*.feature` — cenários BDD executáveis/adaptáveis.

## Missão

Evoluir o produto existente para que cada componente desenhado tenha comportamento funcional, estado observável, tratamento de falhas e testes. O desenho é uma referência; não assuma que todas as capacidades já existem.

## Procedimento obrigatório

1. Inspecione a árvore de diretórios, dependências, CI, frameworks GUI, parser/AST, runtime, testes, formatos de arquivo e integração de processos. NÃO escolha outra stack antes desta análise.
2. Crie `IMPLEMENTATION_GAP_REPORT.md` com a matriz `UI-ID | existente | parcialmente existente | ausente | dependência | proposta | teste | risco`.
3. Preserve compatibilidade dos programas originais do Ferro Studio e suas extensões reais; não presuma extensão `.por` até confirmar.
4. Mapeie cada componente `UI-*` a um componente real e cada `UC-*` a um caso de uso. Declare explicitamente funcionalidades indisponíveis; não invente resultados.
5. Implemente vertical slices pequenas, seguindo a seção 9 do contrato. Para cada uma, adicione unitários, integração, componente e BDD apropriados.
6. Mostre visualmente estados loading/empty/error/success; use `data-testid` ou identificadores equivalentes de teste estáveis.
7. Use rastreio por `requestId`/`revision` nas operações assíncronas, para evitar condições de corrida; não sobrescreva Rust desvinculado.
8. A execução de código não confiável deve passar por restrições e isolamento compatíveis com a plataforma. Documente claramente o que o sandbox oferece e não oferece.
9. Registre os comandos realmente executados e respectivos resultados; se testes dependem de Rust instalado/debugger/sistema operacional, marque como skipped/pending com razão.
10. Gere capturas de tela em 1536×960 e 1280×800, modos beginner/bridge/rust, e valide acessibilidade e regressões.

## Entregáveis

- `IMPLEMENTATION_GAP_REPORT.md`
- Componentes e adapters funcionais com interfaces documentadas
- Testes associados aos IDs `FT-*` e critérios `AC-*`
- Log de evidências de testes e screenshots
- Registro de decisões e bloqueios técnicos

**Definição de êxito:** não basta parecer com a imagem. O usuário deve conseguir navegar, editar, salvar, executar, comparar o código, inspecionar erros e aprender com feedback verdadeiro; cada recurso indisponível deve ser identificado com transparência.
