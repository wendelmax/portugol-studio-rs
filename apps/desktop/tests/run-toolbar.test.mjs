import test from "node:test";
import assert from "node:assert/strict";
import { mountRunToolbar } from "../src/run/toolbar.ts";

function fixture({ copyText } = {}) {
  const events = [];
  const ids = [
    "btn-run", "btn-debug", "btn-stop", "btn-debug-pause", "btn-debug-continue",
    "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out", "btn-clear", "btn-copy",
    "tab-console", "tab-tests", "tab-messages", "tab-debug",
  ];
  const buttons = new Map(ids.map((id) => [id, { onclick: null }]));
  const form = { hidden: false, handler: null, addEventListener(name, handler) { assert.equal(name, "submit"); this.handler = handler; } };
  const input = { value: "resposta" };
  let runId = 7;
  let language = "portugol";
  mountRunToolbar({
    button: (id) => buttons.get(id),
    form,
    input,
    runId: () => runId,
    activeLanguage: () => language,
    runOrContinue: () => events.push(["run"]),
    stepOrStart: () => events.push(["debug"]),
    stop: () => events.push(["stop"]),
    pause: (id) => events.push(["pause", id]),
    continueDebug: () => events.push(["continue"]),
    stepOver: () => events.push(["over"]),
    stepInto: () => events.push(["into"]),
    stepOut: () => events.push(["out"]),
    clearConsole: () => events.push(["clear"]),
    consoleText: () => "program output",
    copyText: copyText ?? (async (value) => events.push(["copy", value])),
    setStatus: (message, kind) => events.push(["status", message, kind]),
    showPanel: (name) => events.push(["panel", name]),
    appendConsole: (text, kind) => events.push(["append", text, kind]),
    sendInput: (id, value) => events.push(["input", id, value]),
  });
  return {
    buttons, events, form, input,
    setRunId: (value) => { runId = value; },
    setLanguage: (value) => { language = value; },
  };
}

test("run and debugger buttons dispatch their commands with the Portugol pause guard", () => {
  const f = fixture();

  f.buttons.get("btn-run").onclick();
  f.buttons.get("btn-debug").onclick();
  f.buttons.get("btn-stop").onclick();
  f.buttons.get("btn-debug-pause").onclick();
  f.buttons.get("btn-debug-continue").onclick();
  f.buttons.get("btn-debug-step-over").onclick();
  f.buttons.get("btn-debug-step-into").onclick();
  f.buttons.get("btn-debug-step-out").onclick();
  f.setLanguage("rust");
  f.buttons.get("btn-debug-pause").onclick();

  assert.deepEqual(f.events, [
    ["run"], ["debug"], ["stop"], ["pause", 7], ["continue"], ["over"], ["into"], ["out"],
  ]);
});

test("console actions copy output, report clipboard errors, and select their panels", async () => {
  const f = fixture();

  f.buttons.get("btn-clear").onclick();
  await f.buttons.get("btn-copy").onclick();
  for (const panel of ["console", "tests", "messages", "debug"]) f.buttons.get(`tab-${panel}`).onclick();

  assert.deepEqual(f.events, [
    ["clear"], ["copy", "program output"], ["status", "Saída copiada.", "ok"],
    ["panel", "console"], ["panel", "tests"], ["panel", "messages"], ["panel", "debug"],
  ]);

  const failed = fixture({ copyText: async () => { throw new Error("clipboard blocked"); } });
  await failed.buttons.get("btn-copy").onclick();
  assert.deepEqual(failed.events, [["status", "Error: clipboard blocked", "error"]]);
});

test("runtime input submission echoes and sends only for an active run", () => {
  const f = fixture();
  let prevented = 0;
  const submit = () => f.form.handler({ preventDefault: () => { prevented += 1; } });

  submit();
  assert.deepEqual(f.events, [["append", "resposta\n", "echo"], ["input", 7, "resposta"]]);
  assert.equal(f.form.hidden, true);

  f.events.length = 0;
  f.form.hidden = false;
  f.setRunId(null);
  submit();
  assert.deepEqual(f.events, []);
  assert.equal(f.form.hidden, false);
  assert.equal(prevented, 2);
});
