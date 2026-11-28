# Design — edição de laços em Ferro

## Intenção

Deixar `enquanto` e `para` reconhecíveis e sugeridos no editor de arquivos `.fer`. Ambos já são aceitos pelo frontend e executados pelo runner; hoje o editor os apresenta como identificadores comuns.

## Abordagens

1. **Adicionar os dois termos às listas FERRO existentes (escolhida):** mantém parser de realce e catálogo de conclusão locais ao módulo, sem dependências ou alterações no runtime.
2. Compartilhar uma lista de keywords entre editor e compilador: criaria acoplamento entre pacotes para uma diferença pequena e com ciclos de versão distintos.
3. Habilitar todas as keywords clássicas: sugeriria recursos ainda não suportados pelo dialeto FERRO.

## Escopo e restrições

- Adicionar somente `enquanto` e `para` ao realce e ao autocomplete de Ferro.
- Manter o realce e o autocomplete clássicos inalterados.
- Não sugerir `faca`, `pare` ou outras construções FERRO ainda não implementadas.
- Sem mudanças em Rust, IPC, permissões, formato `.fer` ou dependências.

## Validação

Testes Node usam o parser real do CodeMirror para verificar os nós `keyword` e o provedor real de autocomplete para verificar as duas sugestões.
