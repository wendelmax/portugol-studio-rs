import { invoke } from "@tauri-apps/api/core";
import { GraphicsRenderer } from "./graphics-renderer";
import "./graphics-window.css";

const canvas = document.querySelector<HTMLCanvasElement>("#screen")!;
const renderer = new GraphicsRenderer(canvas);
const errorBox = document.querySelector<HTMLElement>("#graphics-error")!;

const keys: Record<string, number> = {
  Enter: 10, NumpadEnter: 10, Backspace: 8, Tab: 9, Escape: 27, Space: 32,
  ShiftLeft: 16, ShiftRight: 16, ControlLeft: 17, ControlRight: 17,
  AltLeft: 18, AltRight: 65406, Pause: 19, CapsLock: 20,
  PageUp: 33, PageDown: 34, End: 35, Home: 36,
  ArrowLeft: 37, ArrowUp: 38, ArrowRight: 39, ArrowDown: 40,
  Comma: 44, Minus: 45, Period: 46, Slash: 47, Semicolon: 59, Equal: 61,
  BracketLeft: 91, Backslash: 92, BracketRight: 93, Backquote: 192, Quote: 222,
  NumpadMultiply: 106, NumpadAdd: 107, NumpadSubtract: 109,
  NumpadDecimal: 110, NumpadDivide: 111,
  Delete: 127, NumLock: 144, ScrollLock: 145,
  PrintScreen: 154, Insert: 155, Help: 156, MetaLeft: 524, MetaRight: 524,
  ContextMenu: 525,
};

function javaKey(event: KeyboardEvent): number | undefined {
  const code = event.code;
  if (/^Key[A-Z]$/.test(code)) return code.charCodeAt(3);
  if (/^Digit[0-9]$/.test(code)) return code.charCodeAt(5);
  if (/^Numpad[0-9]$/.test(code)) return 96 + Number(code.slice(6));
  if (/^F(?:[1-9]|1[0-2])$/.test(code)) return 111 + Number(code.slice(1));
  if (/^F(?:1[3-9]|2[0-4])$/.test(code)) return 61_427 + Number(code.slice(1));
  return keys[code];
}

type Input =
  | { type: "key_down" | "key_up"; code: number }
  | { type: "mouse_move"; x: number; y: number }
  | { type: "mouse_down" | "mouse_up"; button: number }
  | { type: "blur" };

const pending: Input[] = [];
let scheduled = false;
let transmitting = false;

function enqueue(event: Input) {
  if (event.type === "mouse_move" && pending[pending.length - 1]?.type === "mouse_move") {
    pending[pending.length - 1] = event;
  } else {
    pending.push(event);
  }
  if (!scheduled && !transmitting) {
    scheduled = true;
    setTimeout(() => void flushInput(), 0);
  }
}

async function flushInput() {
  scheduled = false;
  transmitting = true;
  try {
    while (pending.length > 0) {
      await invoke("graphics_input", { events: pending.splice(0) });
    }
  } catch {
    pending.length = 0;
  } finally {
    transmitting = false;
  }
}

const pressed = new Map<string, number>();
const counts = new Map<number, number>();
window.addEventListener("keydown", (event) => {
  const code = javaKey(event);
  if (code === undefined) return;
  event.preventDefault();
  if (pressed.has(event.code)) return;
  pressed.set(event.code, code);
  const count = counts.get(code) ?? 0;
  counts.set(code, count + 1);
  if (!count) enqueue({ type: "key_down", code });
});
window.addEventListener("keyup", (event) => {
  const code = pressed.get(event.code);
  if (code === undefined) return;
  event.preventDefault();
  pressed.delete(event.code);
  const count = (counts.get(code) ?? 1) - 1;
  if (count) counts.set(code, count);
  else {
    counts.delete(code);
    enqueue({ type: "key_up", code });
  }
});

const buttons = new Set<number>();
window.addEventListener("blur", () => {
  pressed.clear();
  counts.clear();
  buttons.clear();
  enqueue({ type: "blur" });
});

function position(event: PointerEvent) {
  const box = canvas.getBoundingClientRect();
  enqueue({
    type: "mouse_move",
    x: Math.floor((event.clientX - box.left) * canvas.width / box.width),
    y: Math.floor((event.clientY - box.top) * canvas.height / box.height),
  });
}

canvas.addEventListener("pointermove", position);
canvas.addEventListener("pointerdown", (event) => {
  const button = [0, 2, 1][event.button];
  if (button === undefined || buttons.has(button)) return;
  position(event);
  buttons.add(button);
  enqueue({ type: "mouse_down", button });
});
window.addEventListener("pointerup", (event) => {
  const button = [0, 2, 1][event.button];
  if (button === undefined || !buttons.delete(button)) return;
  position(event);
  enqueue({ type: "mouse_up", button });
});
window.addEventListener("pointercancel", () => {
  buttons.clear();
  enqueue({ type: "blur" });
});
window.addEventListener("contextmenu", (event) => event.preventDefault());

async function handleRequest(request: ArrayBuffer) {
  const data = new DataView(request);
  const kind = data.getUint8(0);
  const width = data.getInt32(1, true);
  const height = data.getInt32(5, true);
  const operations = new Uint8Array(request, 9);
  if (width <= 0 || height <= 0) throw new Error("As dimensões da janela precisam ser positivas.");
  if (kind === 1) {
    renderer.resize(width, height);
    await renderer.present(operations);
    // Dê à webview uma oportunidade de pintar antes de liberar a VM.
    if (document.visibilityState === "visible") {
      await new Promise<void>((resolve) => {
        const timeout = setTimeout(resolve, 50);
        requestAnimationFrame(() => {
          clearTimeout(timeout);
          setTimeout(resolve, 0);
        });
      });
    }
    return new Uint8Array([0]);
  }
  if (kind === 2) {
    const rgba = await renderer.snapshot(operations, width, height);
    const reply = new Uint8Array(1 + rgba.length);
    reply.set(rgba, 1);
    return reply;
  }
  throw new Error("Pedido de renderização desconhecido.");
}

async function receiveFrames() {
  for (;;) {
    let request: ArrayBuffer;
    try {
      request = await invoke<ArrayBuffer>("graphics_next");
    } catch (error) {
      errorBox.textContent = String(error);
      errorBox.hidden = false;
      return;
    }
    if (!request.byteLength) continue;
    let reply: Uint8Array<ArrayBuffer>;
    try {
      reply = await handleRequest(request);
    } catch (error) {
      const message = new TextEncoder().encode(String(error));
      reply = new Uint8Array(1 + message.length);
      reply[0] = 1;
      reply.set(message, 1);
    }
    try {
      await invoke("graphics_reply", reply);
    } catch {
      return;
    }
  }
}

void receiveFrames();
