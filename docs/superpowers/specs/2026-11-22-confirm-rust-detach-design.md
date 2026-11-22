# Confirmar desvinculação de Rust gerado — design

## Objetivo

Antes de transformar Rust gerado em uma aba `.rs` independente, explicar que a nova aba deixa de acompanhar a tradução e pedir confirmação. A origem Portugol e a prévia vinculada permanecem intactas.

## Abordagens

1. **Continuar criando a aba imediatamente:** rápido, mas contraria UI-017 e pode surpreender quem interpreta o botão como abrir/editar uma prévia.
2. **Confirmação nativa Tauri e verificação da revisão após o diálogo (escolhida):** reutiliza `ask`, comunica claramente a mudança de propriedade e descarta uma confirmação atrasada se aba/fonte/artefato mudaram.
3. **Desvincular dentro do painel atual:** mais intrusivo, exige uma nova máquina de estados de autoria e torna arriscado preservar a prévia original.

## Fluxo

1. O botão só está habilitado para artefato atual, sem geração pendente ou estado stale.
2. Ao clicar, abrir confirmação nativa em português: o Rust se tornará independente, futuras alterações da fonte não o atualizarão, e origem/prévia permanecem preservadas.
3. Cancelar não cria ou modifica abas.
4. Confirmar cria uma nova aba `.rs` com o texto puro capturado. Antes de efetivar, validar que a aba, `EditorState.doc` e artefato continuam iguais ao snapshot do início do diálogo e que a tradução continua atual.
5. Se a fonte mudou enquanto o diálogo estava aberto, não criar a aba e informar que é necessário gerar novamente.

## Não objetivos

- Não gravar automaticamente um arquivo no sistema; o novo documento continua em memória até Salvar/Salvar como.
- Não alterar o documento de origem, o artefato do preview ou o modo de aprendizagem.
- Não exigir confirmação adicional para edição no Rust standalone depois de desvincular.

## Validação

- Testar a guarda de revisão/tab/fonte ao retornar de um diálogo assíncrono.
- Manter o fluxo atual de nova aba e de salvar como `.rs`.
- Rodar suíte Node, build e CI Tauri.
