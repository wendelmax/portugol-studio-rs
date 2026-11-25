import test from "node:test";
import assert from "node:assert/strict";
import { cycleIdeTab, mountKeyboardShortcutDispatch } from "../src/commands/keyboard-dispatch.ts";

function fixture({ hasOpenDialog = false } = {}) {
  const events = [];
  let listener;
  const menus = [
    { open: false, focusSummary: () => events.push(["focus", 0]) },
    { open: true, focusSummary: () => events.push(["focus", 1]) },
  ];
  const shortcuts = [
    { matches: (event) => event.key === "F1", action: () => events.push(["action", "help"]) },
  ];
  mountKeyboardShortcutDispatch({
    target: { addEventListener: (type, callback) => { assert.equal(type, "keydown"); listener = callback; } },
    hasOpenDialog: () => hasOpenDialog,
    optionMenus: menus,
    shortcuts: () => shortcuts,
  });
  return {
    events,
    menus,
    press(key, { defaultPrevented = false, shiftKey = false } = {}) {
      let prevented = 0;
      listener({ key, shiftKey, defaultPrevented, preventDefault: () => { prevented += 1; } });
      return prevented;
    },
  };
}

test("keyboard dispatch ignores already handled events and open dialogs", () => {
  const alreadyHandled = fixture();
  assert.equal(alreadyHandled.press("F1", { defaultPrevented: true }), 0);
  assert.deepEqual(alreadyHandled.events, []);

  const dialogOpen = fixture({ hasOpenDialog: true });
  assert.equal(dialogOpen.press("F1"), 0);
  assert.deepEqual(dialogOpen.events, []);
});

test("Escape closes the first open option menu and focuses its summary", () => {
  const f = fixture();
  assert.equal(f.press("Escape"), 1);
  assert.deepEqual(f.events, [["focus", 1]]);
  assert.equal(f.menus[1].open, false);
  assert.equal(f.press("Escape", { shiftKey: true }), 0);
});

test("matching shortcuts prevent browser defaults and dispatch their action", () => {
  const f = fixture();
  assert.equal(f.press("F1"), 1);
  assert.deepEqual(f.events, [["action", "help"]]);
  assert.equal(f.press("F2"), 0);
  assert.deepEqual(f.events, [["action", "help"]]);
});

test("tab cycling preserves home/help/editor ordering and wraps both directions", () => {
  const tabs = [{ id: 10 }, { id: 20 }];
  const cycle = (activeId, step, helpOpen = true) => {
    let activated;
    cycleIdeTab(step, {
      homeId: 0,
      helpId: -1,
      helpOpen,
      tabs,
      activeId,
      activate: (id) => { activated = id; },
    });
    return activated;
  };

  assert.equal(cycle(20, 1), 0);
  assert.equal(cycle(0, -1), 20);
  assert.equal(cycle(0, 1), -1);
  assert.equal(cycle(0, 1, false), 10);
});
