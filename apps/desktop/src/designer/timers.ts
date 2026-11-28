/**
 * Temporizadores de uma tela em execução (ADR 0018, Fase 5): cada
 * `<Temporizador>` nomeado e ligado a `AoDisparar` dispara enquanto
 * `ativo` for verdadeiro, no `intervalo` atual (o código pode mudar os dois).
 * Sem DOM: o relógio é injetado, para testar sem esperar.
 */
import type { DesignNode } from "../api.ts";
import { attributeOf } from "./canvas-model.ts";
import type { ScreenSnapshot } from "./live-tree.ts";

/** Menor intervalo aceito pelo esquema, em milissegundos. */
export const MIN_INTERVAL_MS = 10;

export type TimerSpec = { name: string; interval: number; active: boolean };

/** Temporizadores da tela com o estado atual de cada um. */
export function timerSpecs(root: DesignNode, snapshot: ScreenSnapshot): TimerSpec[] {
  const specs: TimerSpec[] = [];
  const visit = (node: DesignNode) => {
    const name = attributeOf(node, "Nome");
    if (node.tag === "Temporizador" && name && attributeOf(node, "AoDisparar")?.trim()) {
      const state = snapshot.controles[name] ?? {};
      const interval = typeof state.intervalo === "number" ? state.intervalo : Number(attributeOf(node, "Intervalo") ?? 0);
      specs.push({ name, interval: Math.max(MIN_INTERVAL_MS, interval || 1000), active: state.ativo === true });
    }
    node.children.forEach(visit);
  };
  visit(root);
  return specs;
}

export interface TimerClock {
  setInterval: (callback: () => void, ms: number) => unknown;
  clearInterval: (handle: unknown) => void;
}

/**
 * Liga, desliga e reajusta os relógios conforme o estado. Um disparo só
 * acontece se o anterior do mesmo temporizador já terminou, para que um
 * handler lento não acumule uma fila de eventos.
 */
export function createTimers(clock: TimerClock, fire: (name: string) => Promise<unknown>) {
  const running = new Map<string, { interval: number; handle: unknown }>();
  const busy = new Set<string>();

  function stop(name: string) {
    const timer = running.get(name);
    if (timer) clock.clearInterval(timer.handle);
    running.delete(name);
  }

  function tick(name: string) {
    if (busy.has(name)) return;
    busy.add(name);
    void fire(name).finally(() => busy.delete(name));
  }

  return {
    sync(specs: readonly TimerSpec[]) {
      const wanted = new Map(specs.filter((spec) => spec.active).map((spec) => [spec.name, spec]));
      for (const name of [...running.keys()]) {
        const spec = wanted.get(name);
        if (!spec || spec.interval !== running.get(name)?.interval) stop(name);
      }
      for (const [name, spec] of wanted) {
        if (running.has(name)) continue;
        running.set(name, { interval: spec.interval, handle: clock.setInterval(() => tick(name), spec.interval) });
      }
    },
    /** Para todos (janela fechando). */
    stopAll() {
      for (const name of [...running.keys()]) stop(name);
    },
    /** Temporizadores ligados agora, para testes. */
    active: () => [...running.entries()].map(([name, timer]) => ({ name, interval: timer.interval })),
  };
}
