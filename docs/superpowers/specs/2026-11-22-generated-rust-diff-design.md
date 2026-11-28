# Comparar traduções Rust geradas — design

## Intenção

Ao editar Ferro em modo Bridge, a pessoa precisa entender o que mudou na tradução. A prévia atual mostra apenas a versão mais recente e descarta visualmente o conteúdo anterior.

## Objetivos

- Preservar o último artefato válido e reter o imediatamente anterior após uma nova tradução bem-sucedida.
- Oferecer “Mostrar diferenças” e “Voltar ao código atual” na prévia Rust.
- Mostrar linhas removidas/adicionadas lado a lado, contagens e números de linha; a comparação é informativa e somente leitura.
- Desabilitar a comparação enquanto a tradução estiver pendente ou stale, e quando não houver duas versões diferentes.
- Manter o texto gerado e seus source maps atuais como fonte de verdade; o diff nunca altera a fonte ou o editor.

## Fora de escopo

- Resolução de conflitos, edição inline, merge ou aplicação seletiva de mudanças.
- Dependência nova ou editor de merge completo.
- Comparação de artefato desvinculado, histórico de sessões ou versões persistidas.

## Abordagens

1. Adicionar `@codemirror/merge` e uma MergeView. Dá navegação e alinhamento sofisticados, mas aumenta dependências e integrações para um diff somente informativo.
2. Renderizar uma comparação local somente leitura em duas colunas, usando um diff de linhas testado e limitado por orçamento de células; se o orçamento for excedido, apresentar um bloco agregado de alteração. **Escolhida:** atende ao uso pedagógico, evita nova dependência e mantém o CodeMirror existente dedicado ao código atual.
3. Abrir outra aba Rust com o código antigo. Perde contexto da comparação e pode confundir a pessoa sobre qual documento é autoral.

## Comportamento

- Uma geração bem-sucedida com código diferente promove o artefato anterior válido a `previousArtifact`; falhas e estados stale preservam ambos sem apresentá-los como comparação atual.
- Uma nova geração idêntica não oferece um diff antigo como se fosse da revisão corrente.
- O botão alterna a visualização atual e a comparação. A prévia continua somente leitura; o status informa claramente quando não há revisão anterior.
- O diff usa contexto, linhas removidas e adicionadas, com rótulos textuais e `aria-label`; cor nunca é o único indicador.
- Cancelamento/edição durante a geração continuam sob as guardas de revisão já existentes.

## Validação

Testar transições do estado (primeira geração, nova saída diferente, saída idêntica, stale e erro), diff de inserção/remoção/alteração/contexto e fallback de grande entrada. Rodar suíte frontend, build, format gates e CI desktop multiplataforma. Interação nativa/visual segue pendente sem runner Tauri E2E.
