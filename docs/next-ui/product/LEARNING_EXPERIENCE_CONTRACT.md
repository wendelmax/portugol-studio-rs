# Modelo pedagógico — modos e progressão

## Modos
- **Iniciante**: Portugol, variáveis/condicionais/loops, execução visual; Rust não imposto.
- **Ponte**: sintaxe Portugol-RS opcional e equivalência explícita com Rust; comparador e explicações.
- **Rust**: rustc/Cargo real (se disponível), mutabilidade, tipos, ownership, borrowing, Result/Option.

**Mudança de modo nunca modifica arquivos automaticamente.** Recomendação contextual não é bloqueio rígido; professor pode permitir ou restringir recursos em futuras extensões.

## Catálogo educacional
`conceptId` estável, pré-requisitos, objetivos, exemplos comparados, dicas graduais, exercícios, critérios, rubrica, locale, versão de conteúdo. O painel de aprendizado deve exibir fonte da explicação: AST, correspondência ou seleção explícita. Nunca alegar identificar conceitos sem evidência.

## Validação de exercício
- Conclusão = resultados de testes educacionais reais vinculados a uma revisão de código.
- Testes de aluno são isolados de testes internos do app; relatórios distinguem falha do aluno e falha de infraestrutura.
- Dicas acessíveis e formativas; feedback não revela segredo de teste privado nem nota sem critério.
- Progresso tem identidade local/conta conforme produto; não coletar dados pessoais sem necessidade.

## Fases posteriores, fora do MVP
Modo Professor, criação de turmas, distribuição de avaliações, conteúdo remoto, sync e painel de classe requerem decisões de privacidade e governança específicas.
