import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createRecentFilesView } from "../src/files/recent-view.ts";

class FakeElement {
  children = [];
  hidden = false;
  title = "";
  textContent = "";
  onclick = null;
  append(...children) { this.children.push(...children); }
  replaceChildren(...children) { this.children = children; }
}

function fixture({ openFile } = {}) {
  const events = [];
  const recent = new FakeElement();
  const list = new FakeElement();
  const view = createRecentFilesView({
    recent,
    list,
    document: { createElement: () => new FakeElement() },
    icon: (name) => ({ name }),
    openFile: openFile ?? (async (path) => events.push(["open", path])),
    setStatus: (...args) => events.push(["status", ...args]),
  });
  return { events, recent, list, view };
}

test("recent files view hides an empty list and renders paths with basename labels", () => {
  const f = fixture();
  f.view.render([]);
  assert.equal(f.recent.hidden, true);
  assert.deepEqual(f.list.children, []);

  f.view.render(["/aula/primeiro.fer", "C:\\curso\\segundo.rs"]);
  assert.equal(f.recent.hidden, false);
  assert.deepEqual(f.list.children.map(({ children, title }) => [children[1].textContent, title]), [
    ["primeiro.fer", "/aula/primeiro.fer"],
    ["segundo.rs", "C:\\curso\\segundo.rs"],
  ]);
  assert.deepEqual(f.list.children[0].children[0], { name: "grande/file" });
});

test("recent files view opens the chosen path and reports open errors", async () => {
  const f = fixture({ openFile: async (path) => {
    if (path === "/falha.fer") throw new Error("arquivo ausente");
    f.events.push(["open", path]);
  } });
  f.view.render(["/ok.fer", "/falha.fer"]);

  f.list.children[0].onclick();
  f.list.children[1].onclick();
  await new Promise((resolve) => setImmediate(resolve));

  assert.deepEqual(f.events, [["open", "/ok.fer"], ["status", "Error: arquivo ausente", "error"]]);
});

test("main delegates recent-file rendering to its view controller", async () => {
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  const shell = await readFile(new URL("../src/files/workspace-files-shell.ts", import.meta.url), "utf8");
  assert.match(main, /createWorkspaceFilesShell\(/);
  assert.match(shell, /createRecentFilesView\(/);
});
