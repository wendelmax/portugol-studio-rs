# Cartões de aprendizagem com explicação progressiva

## Objetivo

Completar UI-024 no inspetor Aprender. Cada conceito disponível deve apresentar, além da definição, três blocos editoriais: “O que mudou?”, “Por que isso importa?” e “Praticar”. O conteúdo deve corresponder ao conceito e ao nível ativo.

## Comportamento

- Cada `LearningConcept` possui texto explícito para mudança, relevância e prática; não derivar esses textos do código nem inferi-los do cursor.
- Os exemplos existentes permanecem com sua proveniência e rótulos atuais.
- Os blocos são renderizados como texto seguro (`textContent`) e não modificam fonte, execução ou progresso.
- Todos os conceitos de Beginner, Bridge e Rust têm os três textos não vazios.

## Não objetivos

Sem nova navegação/serviço Tauri, exercício avaliável, progresso, tradução automática, inferência semântica ou mudança de arquitetura.

## Aceite

Teste unitário confirma completude por conceito nos três níveis e UI possui regiões nomeadas para as três seções; `npm test` e `npm run build` passam.
