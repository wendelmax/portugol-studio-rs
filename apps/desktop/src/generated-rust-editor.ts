import { EditorState, StateEffect, StateField, type Extension, type Transaction } from "@codemirror/state";
import { Decoration, EditorView, highlightActiveLineGutter, lineNumbers } from "@codemirror/view";
import { rust } from "@codemirror/lang-rust";

export interface GeneratedRustMapping {
  from: number;
  to: number;
  sourceOffset: number;
  sourceLine: number;
  generatedLine: number;
  quality: "exact" | "approximate";
}

export const generatedRustMappings = StateEffect.define<readonly GeneratedRustMapping[]>();

function validMappings(mappings: readonly GeneratedRustMapping[], documentLength: number): GeneratedRustMapping[] {
  const sorted = mappings
    .filter(({ from, to, sourceOffset }) =>
      Number.isInteger(from) && Number.isInteger(to) && Number.isInteger(sourceOffset)
      && from >= 0 && to > from && to <= documentLength && sourceOffset >= 0,
    )
    .slice()
    .sort((left, right) => left.from - right.from || left.to - right.to);
  const selected: GeneratedRustMapping[] = [];
  for (const mapping of sorted) {
    if (selected.some((current) => mapping.from < current.to && mapping.to > current.from)) continue;
    selected.push(mapping);
  }
  return selected;
}

export const generatedRustDecorations = StateField.define<readonly GeneratedRustMapping[]>({
  create: () => [],
  update(mappings, transaction: Transaction) {
    for (const effect of transaction.effects) {
      if (effect.is(generatedRustMappings)) return validMappings(effect.value, transaction.newDoc.length);
    }
    return transaction.docChanged ? [] : mappings;
  },
  provide: (field) => EditorView.decorations.from(field, (mappings) => Decoration.set(
    mappings.map((mapping) => Decoration.mark({
      class: "cm-rust-mapped-segment",
      attributes: {
        "data-source-offset": String(mapping.sourceOffset),
        "data-quality": mapping.quality,
      },
    }).range(mapping.from, mapping.to)),
    true,
  )),
});

export function mappedSourceOffsetAt(mappings: readonly GeneratedRustMapping[], position: number): number | null {
  if (!Number.isInteger(position) || position < 0) return null;
  const mapping = mappings.find(({ from, to, sourceOffset }) =>
    Number.isInteger(from) && Number.isInteger(to) && Number.isInteger(sourceOffset)
    && from <= position && position < to && sourceOffset >= 0,
  );
  return mapping?.sourceOffset ?? null;
}

export function mappedSourceOffsetAtSelection(state: EditorState): number | null {
  return mappedSourceOffsetAt(state.field(generatedRustDecorations), state.selection.main.head);
}

export function generatedRustEditorExtensions(
  onSourceOffset: (offset: number) => void,
  additionalExtensions: Extension[] = [],
): Extension[] {
  return [
    lineNumbers(),
    highlightActiveLineGutter(),
    rust(),
    EditorState.tabSize.of(4),
    EditorState.readOnly.of(true),
    EditorView.editable.of(false),
    EditorView.contentAttributes.of({
      "aria-label": "Código Rust gerado, somente leitura",
      "aria-readonly": "true",
      "aria-keyshortcuts": "F12",
    }),
    generatedRustDecorations,
    EditorView.domEventHandlers({
      keydown(event, editor) {
        if (event.key !== "F12") return false;
        const offset = mappedSourceOffsetAtSelection(editor.state);
        if (offset === null) return false;
        event.preventDefault();
        onSourceOffset(offset);
        return true;
      },
      click(event, editor) {
        if (event.button !== 0) return false;
        const position = editor.posAtCoords({ x: event.clientX, y: event.clientY });
        if (position === null) return false;
        const offset = mappedSourceOffsetAt(editor.state.field(generatedRustDecorations), position);
        if (offset === null) return false;
        onSourceOffset(offset);
        return true;
      },
    }),
    ...additionalExtensions,
  ];
}

export function createGeneratedRustEditorState(
  source = "",
  onSourceOffset: (offset: number) => void = () => undefined,
  additionalExtensions: Extension[] = [],
): EditorState {
  return EditorState.create({
    doc: source,
    extensions: generatedRustEditorExtensions(onSourceOffset, additionalExtensions),
  });
}
