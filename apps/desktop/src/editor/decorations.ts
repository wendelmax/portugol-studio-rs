// Estado e decorações do editor ligados à execução: linha do erro de
// execução, pontos de parada e linha onde o depurador está parado.

import { RangeSet, StateEffect, StateField, type EditorState } from "@codemirror/state";
import { Decoration, EditorView, GutterMarker, gutter, type DecorationSet } from "@codemirror/view";
import type { Diagnostic } from "@codemirror/lint";
import type { DiagnosticInfo } from "../api";

/** Destaque da linha onde ocorreu o erro de execução. */
export const setRuntimeError = StateEffect.define<{ from: number; to: number } | null>();
export const runtimeError = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setRuntimeError)) {
        if (!effect.value) return Decoration.none;
        const line = tr.state.doc.lineAt(Math.min(effect.value.from, tr.state.doc.length));
        return Decoration.set([Decoration.line({ class: "cm-runtime-error" }).range(line.from)]);
      }
    }
    return tr.docChanged ? Decoration.none : value.map(tr.changes);
  },
  provide: (field) => EditorView.decorations.from(field),
});

/** Offsets dos pontos de parada; acompanham as edições do documento. */
export const toggleBreakpoint = StateEffect.define<number>();
export const breakpoints = StateField.define<Set<number>>({
  create: () => new Set<number>(),
  update(value, tr) {
    let next = value;
    for (const effect of tr.effects) {
      if (effect.is(toggleBreakpoint)) {
        next = new Set(next);
        if (next.has(effect.value)) next.delete(effect.value);
        else next.add(effect.value);
      }
    }
    if (!tr.docChanged) return next;
    const mapped = new Set<number>();
    for (const pos of next) mapped.add(tr.changes.mapPos(pos, -1));
    return mapped;
  },
});

/** Linhas (a partir de 1) com ponto de parada, em ordem. */
export function breakpointLines(state: EditorState): number[] {
  return [...state.field(breakpoints)]
    .map((pos) => state.doc.lineAt(Math.min(pos, state.doc.length)).number)
    .sort((a, b) => a - b);
}

/**
 * Margem clicável dos pontos de parada. `onToggle` recebe o novo estado;
 * `markerIcon` cria o ícone de cada marcador.
 */
export function createBreakpointGutter(onToggle: (state: EditorState) => void, markerIcon: () => HTMLElement) {
  const breakpointMarker = new (class extends GutterMarker {
    toDOM() {
      const dot = markerIcon();
      dot.classList.add("cm-breakpoint");
      dot.title = "Ponto de parada — clique para remover";
      return dot;
    }
  })();
  return gutter({
    class: "cm-breakpoint-gutter",
    markers: (view) => {
      const positions = [...view.state.field(breakpoints)]
        .map((pos) => view.state.doc.lineAt(Math.min(pos, view.state.doc.length)).from)
        .sort((a, b) => a - b);
      return RangeSet.of(positions.map((pos) => breakpointMarker.range(pos)));
    },
    initialSpacer: () => breakpointMarker,
    domEventHandlers: {
      mousedown(view, line, event) {
        if ((event as MouseEvent).button !== 0) return false;
        view.dispatch({ effects: toggleBreakpoint.of(line.from) });
        onToggle(view.state);
        return true;
      },
    },
  });
}

/** Destaque da linha onde a VM está parada no depurador. */
export const setDebugLine = StateEffect.define<number | null>();
export const debugLine = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setDebugLine)) {
        if (effect.value === null) return Decoration.none;
        const line = tr.state.doc.line(Math.max(1, Math.min(effect.value, tr.state.doc.lines)));
        return Decoration.set([Decoration.line({ class: "cm-debug-line" }).range(line.from)]);
      }
    }
    return tr.docChanged ? Decoration.none : value.map(tr.changes);
  },
  provide: (field) => EditorView.decorations.from(field),
});

/** Converte um diagnóstico do serviço para o formato do CodeMirror. */
export function toCodeMirror(info: DiagnosticInfo, length: number): Diagnostic {
  const from = Math.min(info.start.offset, length);
  const to = Math.min(Math.max(info.end.offset, from + 1), length);
  return {
    from,
    to: Math.max(to, from),
    severity: info.severity,
    source: info.code,
    message: info.help ? `${info.message}\nAjuda: ${info.help}` : info.message,
  };
}
