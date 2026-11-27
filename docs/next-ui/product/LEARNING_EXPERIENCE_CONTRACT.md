# Modelo pedagógico — modos e progressão

## Modos
- **Iniciante**: Portugol, variáveis/condicionais/loops, execução visual; Rust não imposto.
- **Ponte**: sintaxe Portugol-RS opcional e equivalência explícita com Rust; comparador e explicações.
- **Rust**: rustc/Cargo real (se disponível), mutabilidade, tipos, ownership, borrowing, Result/Option.

**Mudança de modo nunca modifica arquivos automaticamente.** Recomendação contextual não é bloqueio rígido; professor pode permitir ou restringir recursos em futuras extensões.

## Catálogo educacional
`conceptId` estável, pré-requisitos, objetivos, exemplos comparados, dicas graduais, exercícios, critérios, rubrica, locale, versão de conteúdo. O painel de aprendizado deve exibir fonte da explicação: AST, correspondência ou seleção explícita. Nunca alegar identificar conceitos sem evidência.

## Lições `.licao` e a aba Aprender
- Um arquivo `.licao` contém os metadados e critérios da atividade: `id`, `title`, `statement`, `starter_code`, `hints` e `cases`. Ele descreve a atividade; o aluno resolve o exercício em uma aba de código `.fer`.
- A trilha de Aprender referencia cada lição pelo `id` estável do arquivo, não pelo caminho. A organização e os rótulos curtos das etapas são editoriais; o catálogo fornece título, enunciado e critérios completos da atividade.
- Ao abrir uma lição pela trilha, a IDE cria uma aba `.fer` com o enunciado e o `starter_code`, mantendo a lição associada para correção. Abrir a mesma lição pelo catálogo de Exercícios ou por um arquivo `.licao` preserva esse vínculo.
- Abrir ou ler a lição não registra progresso. Só a aprovação de todos os casos da lição para a revisão atual da resposta `.fer` conclui a atividade.
- O progresso usa o identificador estável da lição; renomear ou mover o arquivo `.licao` não reinicia a atividade concluída.

## Validação de exercício
- Conclusão = resultados de testes educacionais reais vinculados a uma revisão de código.
- Testes de aluno são isolados de testes internos do app; relatórios distinguem falha do aluno e falha de infraestrutura.
- Dicas acessíveis e formativas; feedback não revela segredo de teste privado nem nota sem critério.
- Progresso tem identidade local/conta conforme produto; não coletar dados pessoais sem necessidade.

## Fases posteriores, fora do MVP
Modo Professor, criação de turmas, distribuição de avaliações, conteúdo remoto, sync e painel de classe requerem decisões de privacidade e governança específicas.
