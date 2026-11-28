// Ponte TypeScript sem dependências externas para a crate WebAssembly `ferro-wasm`
// (ADR 0017 / crates/ferro-wasm/src/lib.rs).
// Instancia `ferro_wasm.wasm`, gerencia alocação/liberação UTF-8 em memória linear
// e conecta sessões MVU ao renderizador DOM seguro `ui-preview-renderer.ts`.

import type {
  UiEvent,
  UiNode,
  UiPreviewController,
  UiRenderLimits,
} from "./ui-preview-renderer";

export interface PorsWasmExports {
  readonly memory: WebAssembly.Memory;
  pors_alloc(len: number): number;
  pors_dealloc(ptr: number, len: number): void;
  pors_result_ptr(): number;
  pors_result_len(): number;
  pors_validate(srcPtr: number, srcLen: number): number;
  pors_format(srcPtr: number, srcLen: number): number;
  pors_run(
    srcPtr: number,
    srcLen: number,
    inputPtr: number,
    inputLen: number,
    maxSteps: number,
    maxStringBytes: number,
    maxOutputBytes: number,
  ): number;
  pors_ui_start(
    srcPtr: number,
    srcLen: number,
    maxSteps: number,
    maxNodes: number,
    maxDepth: number,
  ): number;
  pors_ui_dispatch(
    sessionId: number,
    kindPtr: number,
    kindLen: number,
    targetPtr: number,
    targetLen: number,
    valuePtr: number,
    valueLen: number,
  ): number;
  pors_ui_snapshot(sessionId: number): number;
  pors_ui_close(sessionId: number): number;
}

export interface PorsDiagnostic {
  readonly code: string;
  readonly message: string;
  readonly start: number;
  readonly end: number;
}

export interface PorsValidateResult {
  readonly ok: boolean;
  readonly diagnostics: readonly PorsDiagnostic[];
}

export type PorsFormatResult =
  | {
      readonly ok: true;
      readonly changed: boolean;
      readonly formatted: string;
    }
  | {
      readonly ok: false;
      readonly error: string;
      readonly code?: string;
      readonly diagnostics: readonly PorsDiagnostic[];
    };

export interface PorsRunBudget {
  readonly maxSteps?: number;
  readonly maxStringBytes?: number;
  readonly maxOutputBytes?: number;
}

export type PorsRunResult =
  | {
      readonly ok: true;
      readonly stdout: string;
      readonly diagnostics: readonly PorsDiagnostic[];
    }
  | {
      readonly ok: false;
      readonly stdout?: string;
      readonly error: string;
      readonly code?: string;
      readonly limit?: string | null;
      readonly diagnostics: readonly PorsDiagnostic[];
    };

export interface PorsUiBudget {
  readonly maxSteps?: number;
  readonly maxNodes?: number;
  readonly maxDepth?: number;
}

export interface PorsUiSuccess {
  readonly ok: true;
  readonly session: number;
  readonly step: number;
  readonly history_len: number;
  readonly tree: UiNode;
  readonly stdout: string;
}

export interface PorsUiFailure {
  readonly ok: false;
  readonly error: string;
  readonly code?: string;
  readonly stdout?: string;
  readonly diagnostics?: readonly PorsDiagnostic[];
}

export type PorsUiResult = PorsUiSuccess | PorsUiFailure;

export interface PorsWasmRuntime {
  validate(source: string): PorsValidateResult;
  format(source: string): PorsFormatResult;
  run(source: string, input?: string, budget?: PorsRunBudget): PorsRunResult;
  startUi(source: string, budget?: PorsUiBudget): PorsUiResult;
  dispatchUi(sessionId: number, event: UiEvent): PorsUiResult;
  snapshotUi(sessionId: number): PorsUiResult;
  closeUi(sessionId: number): boolean;
  closeAllUiSessions(): number;
  readonly activeUiSessionIds: readonly number[];
}

export type MountUiPreviewFn = (
  container: HTMLElement,
  onEvent: (event: UiEvent) => void,
  limits?: Partial<UiRenderLimits>,
) => UiPreviewController;

export interface PorsUiSessionHostOptions {
  readonly runtime: PorsWasmRuntime;
  readonly mount: MountUiPreviewFn;
  readonly container: HTMLElement;
  readonly limits?: Partial<UiRenderLimits>;
  readonly onSnapshot?: (snapshot: PorsUiSuccess) => void;
  readonly onError?: (error: PorsUiFailure) => void;
}

export interface PorsUiSessionHostController {
  start(source: string, budget?: PorsUiBudget): PorsUiResult;
  restart(): PorsUiResult | null;
  dispatch(event: UiEvent): PorsUiResult | null;
  snapshot(): PorsUiResult | null;
  cancel(): void;
  destroy(): void;
  readonly sessionId: number | null;
  readonly step: number;
  readonly active: boolean;
}

const REQUIRED_EXPORTS = [
  "memory",
  "pors_alloc",
  "pors_dealloc",
  "pors_result_ptr",
  "pors_result_len",
  "pors_validate",
  "pors_format",
  "pors_run",
  "pors_ui_start",
  "pors_ui_dispatch",
  "pors_ui_snapshot",
  "pors_ui_close",
] as const;

function assertWasmExports(raw: WebAssembly.Exports): PorsWasmExports {
  for (const name of REQUIRED_EXPORTS) {
    const value = raw[name];
    if (name === "memory") {
      if (!(value instanceof WebAssembly.Memory)) {
        throw new Error('O módulo WASM não exporta uma memória linear válida ("memory").');
      }
    } else if (typeof value !== "function") {
      throw new Error(`O módulo WASM não exporta a função obrigatória "${name}".`);
    }
  }
  return raw as unknown as PorsWasmExports;
}

function sanitizeBudgetValue(value: number | undefined): number {
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) {
    return 0;
  }
  return Math.floor(value);
}

/** Cria uma fachada tipada em torno das funções exportadas por `ferro_wasm.wasm`. */
export function createPorsWasmRuntime(rawExports: WebAssembly.Exports | PorsWasmExports): PorsWasmRuntime {
  const exportsObj = assertWasmExports(rawExports as WebAssembly.Exports);
  const encoder = new TextEncoder();
  const decoder = new TextDecoder("utf-8", { fatal: true });
  const openSessions = new Set<number>();

  function withUtf8<T>(text: string, fn: (ptr: number, len: number) => T): T {
    const encoded = encoder.encode(text);
    const len = encoded.length;
    if (len === 0) {
      return fn(0, 0);
    }
    const ptr = exportsObj.pors_alloc(len);
    if (ptr === 0) {
      throw new Error("Falha ao alocar memória linear no módulo WASM.");
    }
    try {
      new Uint8Array(exportsObj.memory.buffer, ptr, len).set(encoded);
      return fn(ptr, len);
    } finally {
      exportsObj.pors_dealloc(ptr, len);
    }
  }

  function readResultJson<T>(expectedLen: number): T {
    const ptr = exportsObj.pors_result_ptr();
    const actualLen = exportsObj.pors_result_len();
    const len = actualLen === expectedLen ? actualLen : expectedLen;
    const raw = decoder.decode(new Uint8Array(exportsObj.memory.buffer, ptr, len));
    return JSON.parse(raw) as T;
  }

  function closeUi(sessionId: number): boolean {
    const closed = exportsObj.pors_ui_close(sessionId) === 1;
    openSessions.delete(sessionId);
    return closed;
  }

  return {
    validate(source: string): PorsValidateResult {
      return withUtf8(source, (ptr, len) =>
        readResultJson<PorsValidateResult>(exportsObj.pors_validate(ptr, len)),
      );
    },

    format(source: string): PorsFormatResult {
      return withUtf8(source, (ptr, len) =>
        readResultJson<PorsFormatResult>(exportsObj.pors_format(ptr, len)),
      );
    },

    run(source: string, input = "", budget?: PorsRunBudget): PorsRunResult {
      const maxSteps = sanitizeBudgetValue(budget?.maxSteps);
      const maxStringBytes = sanitizeBudgetValue(budget?.maxStringBytes);
      const maxOutputBytes = sanitizeBudgetValue(budget?.maxOutputBytes);
      return withUtf8(source, (srcPtr, srcLen) =>
        withUtf8(input, (inPtr, inLen) =>
          readResultJson<PorsRunResult>(
            exportsObj.pors_run(
              srcPtr,
              srcLen,
              inPtr,
              inLen,
              maxSteps,
              maxStringBytes,
              maxOutputBytes,
            ),
          ),
        ),
      );
    },

    startUi(source: string, budget?: PorsUiBudget): PorsUiResult {
      const maxSteps = sanitizeBudgetValue(budget?.maxSteps);
      const maxNodes = sanitizeBudgetValue(budget?.maxNodes);
      const maxDepth = sanitizeBudgetValue(budget?.maxDepth);
      const result = withUtf8(source, (srcPtr, srcLen) =>
        readResultJson<PorsUiResult>(
          exportsObj.pors_ui_start(srcPtr, srcLen, maxSteps, maxNodes, maxDepth),
        ),
      );
      if (result.ok) {
        openSessions.add(result.session);
      }
      return result;
    },

    dispatchUi(sessionId: number, event: UiEvent): PorsUiResult {
      const value = event.kind === "entrada_alterada" ? event.value : "";
      return withUtf8(event.kind, (kindPtr, kindLen) =>
        withUtf8(event.id, (targetPtr, targetLen) =>
          withUtf8(value, (valuePtr, valueLen) =>
            readResultJson<PorsUiResult>(
              exportsObj.pors_ui_dispatch(
                sessionId,
                kindPtr,
                kindLen,
                targetPtr,
                targetLen,
                valuePtr,
                valueLen,
              ),
            ),
          ),
        ),
      );
    },

    snapshotUi(sessionId: number): PorsUiResult {
      return readResultJson<PorsUiResult>(exportsObj.pors_ui_snapshot(sessionId));
    },

    closeUi,

    closeAllUiSessions(): number {
      let closedCount = 0;
      for (const sessionId of [...openSessions]) {
        if (closeUi(sessionId)) {
          closedCount += 1;
        }
      }
      return closedCount;
    },

    get activeUiSessionIds(): readonly number[] {
      return [...openSessions];
    },
  };
}

function isResponseLike(value: unknown): value is Response {
  return (
    typeof Response !== "undefined" &&
    value instanceof Response
  );
}

/** Instancia `ferro_wasm.wasm` a partir de bytes, `WebAssembly.Module` ou `Response`. */
export async function loadPorsWasmRuntime(
  source: BufferSource | WebAssembly.Module | Response | PromiseLike<Response>,
): Promise<PorsWasmRuntime> {
  const resolved = await source;
  if (resolved instanceof WebAssembly.Module) {
    const instance = await WebAssembly.instantiate(resolved, {});
    return createPorsWasmRuntime(instance.exports);
  }
  if (isResponseLike(resolved)) {
    if (typeof WebAssembly.instantiateStreaming === "function") {
      try {
        const streamed = await WebAssembly.instantiateStreaming(resolved.clone(), {});
        return createPorsWasmRuntime(streamed.instance.exports);
      } catch {
        // Fallback para servidores sem MIME `application/wasm`.
      }
    }
    const bytes = await resolved.arrayBuffer();
    const instantiated = await WebAssembly.instantiate(bytes, {});
    return createPorsWasmRuntime(instantiated.instance.exports);
  }
  const instantiated = await WebAssembly.instantiate(resolved, {});
  return createPorsWasmRuntime(instantiated.instance.exports);
}

/**
 * Conecta um `PorsWasmRuntime` a um `mountUiPreview` seguro, gerenciando ciclo de vida
 * da sessão MVU, reinício, cancelamento e descarte de eventos obsoletos.
 */
export function createPorsUiSessionHost(
  options: PorsUiSessionHostOptions,
): PorsUiSessionHostController {
  const { runtime, mount, container, limits, onSnapshot, onError } = options;
  let destroyed = false;
  let active = false;
  let sessionId: number | null = null;
  let step = 0;
  let sessionGeneration = 0;
  let lastSource: string | null = null;
  let lastBudget: PorsUiBudget | undefined;

  const preview = mount(
    container,
    (event) => {
      dispatch(event);
    },
    limits,
  );

  function closeCurrentSession() {
    if (sessionId !== null) {
      runtime.closeUi(sessionId);
      sessionId = null;
    }
    active = false;
  }

  function applyResult(result: PorsUiResult, generation: number): PorsUiResult {
    if (destroyed || generation !== sessionGeneration) {
      if (result.ok) {
        runtime.closeUi(result.session);
      }
      return result;
    }
    if (!result.ok) {
      active = false;
      preview.setDisabled(true);
      onError?.(result);
      return result;
    }
    sessionId = result.session;
    try {
      preview.render(result.tree);
    } catch (error) {
      closeCurrentSession();
      preview.setDisabled(true);
      const failure: PorsUiFailure = {
        ok: false,
        code: "render_contract_error",
        error: error instanceof Error ? error.message : String(error),
      };
      onError?.(failure);
      return failure;
    }
    step = result.step;
    active = true;
    preview.setDisabled(false);
    onSnapshot?.(result);
    return result;
  }

  function start(source: string, budget?: PorsUiBudget): PorsUiResult {
    if (destroyed) {
      const failure: PorsUiFailure = {
        ok: false,
        code: "host_destroyed",
        error: "O host de preview visual já foi encerrado.",
      };
      onError?.(failure);
      return failure;
    }
    closeCurrentSession();
    sessionGeneration += 1;
    const generation = sessionGeneration;
    lastSource = source;
    lastBudget = budget;
    step = 0;
    const result = runtime.startUi(source, budget);
    return applyResult(result, generation);
  }

  function dispatch(event: UiEvent): PorsUiResult | null {
    if (destroyed || !active || sessionId === null) {
      return null;
    }
    const generation = sessionGeneration;
    const result = runtime.dispatchUi(sessionId, event);
    return applyResult(result, generation);
  }

  function snapshot(): PorsUiResult | null {
    if (destroyed || sessionId === null) {
      return null;
    }
    return runtime.snapshotUi(sessionId);
  }

  function restart(): PorsUiResult | null {
    if (destroyed || lastSource === null) {
      return null;
    }
    return start(lastSource, lastBudget);
  }

  function cancel() {
    if (destroyed) return;
    sessionGeneration += 1;
    closeCurrentSession();
    preview.setDisabled(true);
  }

  function destroy() {
    if (destroyed) return;
    destroyed = true;
    sessionGeneration += 1;
    closeCurrentSession();
    preview.destroy();
  }

  return {
    start,
    restart,
    dispatch,
    snapshot,
    cancel,
    destroy,
    get sessionId() {
      return sessionId;
    },
    get step() {
      return step;
    },
    get active() {
      return active;
    },
  };
}
