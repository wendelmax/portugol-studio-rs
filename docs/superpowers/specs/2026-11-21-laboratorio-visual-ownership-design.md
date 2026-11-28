# Laboratório visual de ownership — Design

## Intenção

Evoluir o cartão estático de ownership da aba Aprender para uma aula navegável que mostre como valores `String` são movidos, emprestados e liberados. Isso conecta a ponte didática já existente ao conceito Rust que mais diferencia a linguagem, sem confundir o aluno sobre o runtime atual da IDE.

## Requisitos e limites

- Rastreabilidade: UI-023/024/027, UC-13/15, FT-07, AC-13.
- Quatro cenários: move; empréstimo compartilhado `&T`; empréstimo exclusivo `&mut T`; encerramento de empréstimo/ownership por escopo.
- Cada passo apresenta trecho Rust, variável/owner/referência, alocação quando aplicável, explicação em português e posição do passo.
- Move inválido aparece como rejeição de compilação, não como execução que continua.
- O painel identifica permanentemente **Simulação didática — não é execução nem trace de memória Rust**.
- Não analisar o buffer do aluno, executar código vindo da UI, alterar a VM/debugger Ferro, trocar o modo de aprendizado ou registrar progresso.
- Nenhum novo processo de produção, comando Tauri, permissão ou dependência.

## Abordagens consideradas

1. **Trace Rust real:** exigiria editor/runtime Rust integrado e eventos de compilador/runtime; esses componentes não existem. Não cabe nesta fatia e não deve ser imitado.
2. **Interpretar código do aluno com regras próprias:** parece contextual, mas cria um parser/analisador incompleto que pode ensinar regras erradas. Rejeitada para esta etapa.
3. **Cenários curriculares explícitos com transições verificadas e fixtures Rust:** recomendada. Mantém exemplos editoriais como hoje, acrescenta interação visual e testa cada exemplo contra `rustc`, sem afirmar execução real.

## Proposta de interação

O laboratório aparece somente quando o conceito `Ownership e movimento` está selecionado no modo Bridge. Um seletor muda entre quatro cenários. Anterior, Próximo e Reiniciar percorrem passos do cenário. A tela mostra o código com a linha didática atual, um diagrama semântico de stack/heap/referências, o texto do passo e o diagnóstico final quando o programa é inválido. Os botões desabilitam corretamente nos limites; contador e estado são anunciados por `aria-live="polite"`. A visualização usa texto, símbolos e contornos além de cor.

## Arquitetura e arquivos

- `apps/desktop/src/ownership-lab.ts`: catálogo imutável, tipos e consulta de cenário/passo; nenhum parser nem efeito colateral.
- `apps/desktop/index.html`: controles e regiões acessíveis dentro do painel existente.
- `apps/desktop/src/main.ts`: conecta seleção do conceito, cenário e controles; cria nós DOM e usa `textContent` para todo conteúdo.
- `apps/desktop/src/style.css`: diagrama responsivo com tokens atuais; sem dependência visual nova.
- `apps/desktop/tests/ownership-lab.test.mjs`: teste-first do catálogo, limites e invariantes das snapshots, markup/labels e compilação das fixtures.
- `apps/desktop/tests/fixtures/ownership/*.rs`: exemplos executáveis/compiláveis correspondentes aos cenários, sem dependência da aplicação.
- `docs/research/REPOSITORY_DISCOVERY_CURRENT.md`: inventário atual e lacunas observadas.
- `docs/plans/ownership-visual-lab.md`: plano incremental desta fatia.
- `docs/reports/FEATURE-OWNERSHIP-LAB.md`: execução, validação e limitações após implementação.

## Estados e conteúdo

- Move: `a` possui a `String`; move transfere a posse para `b` e invalida `a`; tentativa de usar `a` recebe diagnóstico de compile time.
- `&T`: um owner pode ter múltiplas referências compartilhadas; o owner segue válido.
- `&mut T`: uma referência mutável exclusiva permite alteração; após o último uso do empréstimo, o owner volta a ser acessível.
- Escopo: a referência não sobrevive ao bloco; o owner permanece até sair de seu próprio escopo e então o heap é liberado.

As snapshots representam passos pedagógicos verificados, não instruções observadas no programa aberto. Texto define exatamente se há owner, empréstimo, valor movido, referência ativa, liberação ou rejeição do compilador.

## Critérios de aceite

- Cada cenário selecionado mostra código/estado/texto correspondentes ao mesmo índice de passo.
- Nenhuma snapshot permite que dois bindings sejam owners simultâneos da mesma `String` após move.
- Empréstimos compartilhados coexistem; empréstimo mutável é exclusivo; acesso ao owner enquanto o empréstimo é usado não é mostrado como válido.
- Fixtures shared/mutable/scope compilam e produzem saída esperada; fixture move falha por uso depois de move.
- Navegação teclado/mouse e estados primeiro/último passo são acessíveis e claros.
- A proveniência da simulação continua visível em todos os cenários e passos.
- Exercícios, progresso, preview de exportação e debugger Ferro continuam inalterados.

## Riscos e rollback

Risco pedagógico principal é desenhar um estado impossível de ownership. O modelo é estático e os testes de fixture com `rustc` confirmam que exemplos válidos/inválidos correspondem às regras ilustradas. O risco de integração é baixo por não tocar runtime/IPC. Rollback remove o modelo, fixtures, região visual, estilos, testes e relatório; o cartão editorial que existe hoje permanece.
