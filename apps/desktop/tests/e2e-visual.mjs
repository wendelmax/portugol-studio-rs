// Verificação E2E e visual real da IDE compilada (apps/desktop/dist) em Firefox headless
// nas resoluções de referência 1536x960 e 1280x800 (DESIGN_AS_CODE.md §9).
//
// Uso:
//   npm run build && node tests/e2e-visual.mjs
//   node tests/e2e-visual.mjs --screenshots /tmp/portugol-e2e-shots
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import http from "node:http";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(__dirname, "..");
const distDir = path.join(desktopRoot, "dist");

if (!existsSync(path.join(distDir, "index.html"))) {
  console.error("dist/index.html ausente. Execute `npm run build` em apps/desktop antes.");
  process.exit(1);
}

const args = process.argv.slice(2);
const shotIdx = args.indexOf("--screenshots");
const screenshotsDir = shotIdx >= 0 && args[shotIdx + 1] ? path.resolve(args[shotIdx + 1]) : null;
if (screenshotsDir) mkdirSync(screenshotsDir, { recursive: true });

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm",
};

function e2eHarnessScript(scenario, width, height) {
  return `<script>
(function () {
  const MOCK_RUST = 'fn main() {\\n    let mut total: i32 = 0_i32;\\n    println!("{}", total);\\n}\\n';
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
    transformCallback(cb) {
      const id = Math.floor(Math.random() * 1e9);
      window["_" + id] = cb;
      return id;
    },
    unregisterCallback() {},
    async invoke(cmd, payload) {
      switch (cmd) {
        case "pors_examples":
        case "list_pors_examples":
          return [
            { id: "ola_mundo", title: "Olá Mundo", category: "Básico", description: "Primeiro programa PORS.", source: 'funcao inicio() {\\n    escreva("Olá Mundo!");\\n}\\n' },
            { id: "resultado", title: "Divisão segura com Resultado", category: "Tipos", description: "Tratamento de erro tipado.", source: 'funcao inicio() {\\n    escreva("Ok");\\n}\\n' }
          ];
        case "take_pending_files":
          return [];
        case "inspect_portugol_rs":
        case "inspect":
          return { diagnostics: [], symbols: [{ name: "inicio", kind: "function", data_type: "vazio", offset: 0, line: 1 }] };
        case "export_portugol_rs":
        case "export_rust": {
          const src = (payload && payload.source) || "";
          const end = Math.max(1, src.length);
          return {
            code: MOCK_RUST,
            sourceMap: [{
              sourceStart: { offset: 0, line: 1, column: 1 },
              sourceEnd: { offset: end, line: 1, column: end + 1 },
              generatedStart: { offset: 14, line: 2, column: 5 },
              generatedEnd: { offset: 41, line: 2, column: 32 },
              quality: "exact"
            }],
            diagnostics: []
          };
        }
        case "rust_toolchain_available":
        case "cargo_toolchain_available":
        case "rustfmt_available":
          return true;
        case "restore_workspace":
          return null;
        case "plugin:event|listen":
          return 1;
        case "plugin:event|unlisten":
        case "plugin:window|minimize":
        case "plugin:window|toggle_maximize":
        case "plugin:window|close":
        case "plugin:window|start_dragging":
          return null;
        default:
          return null;
      }
    }
  };

  function sleep(ms) { return new Promise((r) => setTimeout(r, ms)); }
  function rect(el) {
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return { left: Math.round(r.left), top: Math.round(r.top), right: Math.round(r.right), bottom: Math.round(r.bottom), width: Math.round(r.width), height: Math.round(r.height) };
  }
  function isVisible(el) {
    if (!el) return false;
    const style = getComputedStyle(el);
    if (style.display === "none" || style.visibility === "hidden") return false;
    const r = el.getBoundingClientRect();
    return r.width > 0 && r.height > 0;
  }
  function overlaps(a, b) {
    return a.left < b.right - 1 && a.right > b.left + 1 && a.top < b.bottom - 1 && a.bottom > b.top + 1;
  }

  async function runScenario() {
    const checks = [];
    function check(name, pass, detail) {
      checks.push({ name, pass: Boolean(pass), detail: detail || "" });
    }

    await sleep(350);

    // 1. Verificar tela inicial (#home)
    const home = document.getElementById("home");
    check("home-visible", isVisible(home), "home rect=" + JSON.stringify(rect(home)));
    check("home-no-horizontal-overflow", document.documentElement.scrollWidth <= window.innerWidth + 1,
      "scrollWidth=" + document.documentElement.scrollWidth + " innerWidth=" + window.innerWidth);
    for (const id of ["home-new", "home-open", "home-open-folder", "home-help"]) {
      const el = document.getElementById(id);
      check("home-btn-" + id, isVisible(el), JSON.stringify(rect(el)));
    }

    if (${JSON.stringify(scenario)} === "home-dark") {
      document.documentElement.dataset.theme = "dark";
      await sleep(100);
      await fetch("/__e2e_report", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ scenario: ${JSON.stringify(scenario)}, width: ${width}, height: ${height}, viewport: { w: window.innerWidth, h: window.innerHeight }, checks })
      });
      return;
    }

    // 2. Criar novo arquivo .pors e validar shell + TitleBar sem sobreposição
    document.getElementById("home-new").click();
    await sleep(250);
    const code = document.getElementById("code");
    check("code-workspace-visible", isVisible(code));
    check("editor-no-horizontal-overflow", document.documentElement.scrollWidth <= window.innerWidth + 1,
      "scrollWidth=" + document.documentElement.scrollWidth + " innerWidth=" + window.innerWidth);

    const titlebarIds = [
      "titlebar-workspace", "titlebar-learning-mode",
      "btn-run", "btn-debug", "btn-stop", "btn-rust", "btn-ui-preview"
    ];
    const rects = [];
    for (const id of titlebarIds) {
      const el = document.getElementById(id);
      const r = rect(el);
      const inside = isVisible(el) && r.left >= 0 && r.right <= window.innerWidth + 1 && r.top >= 0 && r.bottom <= 70;
      check("titlebar-control-" + id, inside, JSON.stringify(r));
      rects.push({ id, r });
    }
    let overlapPair = "";
    for (let i = 0; i < rects.length; i++) {
      for (let j = i + 1; j < rects.length; j++) {
        if (overlaps(rects[i].r, rects[j].r)) {
          overlapPair = rects[i].id + " vs " + rects[j].id;
        }
      }
    }
    check("titlebar-no-overlaps", overlapPair === "", overlapPair);

    // 3. Alternar modo para "bridge" pelo seletor da TitleBar e validar sync + split/tabs
    const titleMode = document.getElementById("titlebar-learning-mode");
    titleMode.value = "bridge";
    titleMode.dispatchEvent(new Event("change", { bubbles: true }));
    await sleep(350);
    check("mode-synced-settings", document.getElementById("learning-mode").value === "bridge",
      "settings value=" + document.getElementById("learning-mode").value);
    if (window.innerWidth > 1280) {
      const previewHeader = document.querySelector("#rust-preview header");
      const editorOptionsSummary = document.querySelector("#editor-options > summary");
      check("bridge-split-visible-wide", isVisible(document.getElementById("rust-preview")) && isVisible(document.getElementById("rust-mapping-gutter")));
      check("editor-options-no-overlap-rust-header",
        !overlaps(rect(editorOptionsSummary), rect(previewHeader)),
        "options=" + JSON.stringify(rect(editorOptionsSummary)) + " rustHeader=" + JSON.stringify(rect(previewHeader)));
    } else {
      check("bridge-tabs-visible-compact", isVisible(document.getElementById("rust-comparison-tabs")));
    }
    check("bridge-no-horizontal-overflow", document.documentElement.scrollWidth <= window.innerWidth + 1,
      "scrollWidth=" + document.documentElement.scrollWidth + " innerWidth=" + window.innerWidth);

    // 4. Abrir Configurações dedicadas (#settings-dialog), trocar tema e restaurar layout
    document.querySelector('[data-activity="settings"]').click();
    await sleep(150);
    const settingsDialog = document.getElementById("settings-dialog");
    check("settings-dialog-open", settingsDialog.open && isVisible(settingsDialog));
    const themeSelect = document.getElementById("theme");
    for (const preset of ["preset:intellij-rust-dark", "preset:intellij-rust-light", "preset:vscode-dark-modern", "preset:vscode-light-modern", "system"]) {
      themeSelect.value = preset;
      themeSelect.dispatchEvent(new Event("change", { bubbles: true }));
    }
    check("theme-reset-to-system", themeSelect.value === "system");
    document.getElementById("settings-reset-layout").click();
    await sleep(80);
    check("layout-reset-status", (document.getElementById("status").textContent || "").includes("Layout"));
    settingsDialog.close();
    await sleep(100);

    // 5. Abrir Aprender no Inspector e conferir novos conceitos PORS + Ownership Lab
    document.querySelector('[data-activity="learn"]').click();
    await sleep(150);
    const learningPanel = document.getElementById("learning-panel");
    check("learning-panel-visible", isVisible(learningPanel));
    const conceptPicker = document.getElementById("learning-concept");
    const optionIds = [...conceptPicker.options].map((o) => o.value);
    check("learning-has-pors-concepts",
      ["functions", "option-result", "collections", "records-enums"].every((id) => optionIds.includes(id)),
      optionIds.join(","));
    conceptPicker.value = "ownership";
    conceptPicker.dispatchEvent(new Event("change", { bubbles: true }));
    await sleep(100);
    check("ownership-lab-visible-with-provenance",
      isVisible(document.getElementById("ownership-lab")) &&
      (document.getElementById("ownership-provenance").textContent || "").includes("Simulação didática"));

    // 6. Abrir Ajuda e confirmar menus 100% PORS
    if (${JSON.stringify(scenario)} === "editor-bridge-verify") {
      document.getElementById("btn-help").click();
      await sleep(350);
      const helpHeadings = [...document.querySelectorAll("#help-topics h2")].map((h) => h.textContent);
      check("help-pors-menus-only",
        helpHeadings.includes("Primeiros passos (PORS)") &&
        helpHeadings.includes("Bibliotecas padrão (PORS)") &&
        helpHeadings.includes("UI e Multimídia (PORS)") &&
        helpHeadings.includes("Referência da sintaxe (.pors)") &&
        !helpHeadings.includes("Bibliotecas clássicas"),
        helpHeadings.join(" | "));
      // Voltar para o editor no modo escuro IntelliJ Rust para eventual screenshot
      document.querySelectorAll("#tabs .file-tab button")[0]?.click();
      await sleep(150);
      themeSelect.value = "preset:intellij-rust-dark";
      themeSelect.dispatchEvent(new Event("change", { bubbles: true }));
      await sleep(100);
    }

    await fetch("/__e2e_report", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ scenario: ${JSON.stringify(scenario)}, width: ${width}, height: ${height}, viewport: { w: window.innerWidth, h: window.innerHeight }, checks })
    });
  }

  window.addEventListener("error", (ev) => {
    fetch("/__e2e_report", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ scenario: ${JSON.stringify(scenario)}, width: ${width}, height: ${height}, error: "window.error: " + (ev.message || ev.error), checks: [] })
    });
  });

  window.addEventListener("load", () => {
    runScenario().catch((err) => {
      fetch("/__e2e_report", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ scenario: ${JSON.stringify(scenario)}, width: ${width}, height: ${height}, error: String(err), checks: [] })
      });
    });
  });
})();
</script>`;
}

function shotSetupScript(scenario) {
  return `<script>
(function () {
  const MOCK_RUST = 'fn main() {\\n    let mut total: i32 = 0_i32;\\n    println!("{}", total);\\n}\\n';
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
    transformCallback(cb) { const id = Math.floor(Math.random() * 1e9); window["_" + id] = cb; return id; },
    unregisterCallback() {},
    async invoke(cmd, payload) {
      if (cmd === "pors_examples" || cmd === "list_pors_examples") {
        return [
          { id: "ola_mundo", title: "Olá Mundo", category: "Básico", description: "Primeiro programa PORS.", source: 'funcao inicio() {\\n    escreva("Olá Mundo!");\\n}\\n' },
          { id: "resultado", title: "Divisão segura com Resultado", category: "Tipos", description: "Tratamento de erro tipado.", source: 'funcao inicio() {\\n    escreva("Ok");\\n}\\n' }
        ];
      }
      if (cmd === "take_pending_files") return [];
      if (cmd === "inspect_portugol_rs" || cmd === "inspect") return { diagnostics: [], symbols: [{ name: "inicio", kind: "function", data_type: "vazio", offset: 0, line: 1 }] };
      if (cmd === "export_portugol_rs" || cmd === "export_rust") {
        const src = (payload && payload.source) || "";
        const end = Math.max(1, src.length);
        return { code: MOCK_RUST, sourceMap: [{ sourceStart: { offset: 0, line: 1, column: 1 }, sourceEnd: { offset: end, line: 1, column: end + 1 }, generatedStart: { offset: 14, line: 2, column: 5 }, generatedEnd: { offset: 41, line: 2, column: 32 }, quality: "exact" }], diagnostics: [] };
      }
      if (cmd === "rust_toolchain_available" || cmd === "cargo_toolchain_available" || cmd === "rustfmt_available") return true;
      if (cmd === "plugin:event|listen") return 1;
      return null;
    }
  };
  window.addEventListener("load", () => {
    const sc = ${JSON.stringify(scenario)};
    if (sc === "home-dark") {
      const t = document.getElementById("theme");
      if (t) { t.value = "preset:vscode-dark-modern"; t.dispatchEvent(new Event("change", { bubbles: true })); }
      return;
    }
    const t = document.getElementById("theme");
    if (t) { t.value = "preset:intellij-rust-dark"; t.dispatchEvent(new Event("change", { bubbles: true })); }
    document.getElementById("home-new")?.click();
    const mode = document.getElementById("titlebar-learning-mode");
    if (mode) { mode.value = "bridge"; mode.dispatchEvent(new Event("change", { bubbles: true })); }
  });
})();
</script>`;
}

let pendingReportResolve = null;

const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://127.0.0.1");
  if (req.method === "POST" && url.pathname === "/__e2e_report") {
    let body = "";
    req.on("data", (chunk) => { body += chunk; });
    req.on("end", () => {
      res.writeHead(204);
      res.end();
      if (pendingReportResolve) {
        const resolve = pendingReportResolve;
        pendingReportResolve = null;
        try { resolve(JSON.parse(body)); } catch (error) { resolve({ error: String(error), checks: [] }); }
      }
    });
    return;
  }

  const relPath = url.pathname === "/" ? "index.html" : url.pathname.replace(/^\/+/, "");
  const filePath = path.resolve(distDir, relPath);
  if (!filePath.startsWith(distDir) || !existsSync(filePath)) {
    res.writeHead(404);
    res.end("Not found");
    return;
  }
  const ext = path.extname(filePath);
  res.writeHead(200, { "content-type": MIME[ext] ?? "application/octet-stream" });
  if (relPath === "index.html" && (url.searchParams.has("e2e") || url.searchParams.has("shot"))) {
    const scenario = url.searchParams.get("e2e") || url.searchParams.get("shot") || "editor-bridge-verify";
    const width = Number(url.searchParams.get("w") || "1536");
    const height = Number(url.searchParams.get("h") || "960");
    const html = readFileSync(filePath, "utf8");
    const injected = url.searchParams.has("shot")
      ? shotSetupScript(scenario)
      : e2eHarnessScript(scenario, width, height);
    res.end(html.replace("<head>", `<head>${injected}`));
    return;
  }
  res.end(readFileSync(filePath));
});

async function runViewport({ scenario, width, height, screenshotName }) {
  const profileDir = mkdtempSync(path.join(tmpdir(), "portugol-ff-e2e-"));
  const address = server.address();
  const targetUrl = `http://127.0.0.1:${address.port}/?e2e=${encodeURIComponent(scenario)}&w=${width}&h=${height}`;

  const reportPromise = new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pendingReportResolve = null;
      reject(new Error(`Timeout aguardando relatório E2E (${scenario} ${width}x${height})`));
    }, 25_000);
    pendingReportResolve = (report) => {
      clearTimeout(timer);
      resolve(report);
    };
  });

  const child = spawn("firefox", [
    "--headless",
    "--no-remote",
    "--profile", profileDir,
    `--window-size=${width},${height}`,
    targetUrl,
  ], {
    env: { ...process.env, MOZ_HEADLESS: "1" },
    stdio: ["ignore", "pipe", "pipe"],
  });

  try {
    const report = await reportPromise;
    assert.equal(report.error, undefined, `Erro no cenário ${scenario}: ${report.error}`);
    assert.ok(Array.isArray(report.checks) && report.checks.length > 0, "Relatório E2E sem verificações");
    const failed = report.checks.filter((c) => !c.pass);
    if (failed.length > 0) {
      throw new Error(
        `Falhas E2E em ${scenario} (${width}x${height}):\n` +
        failed.map((c) => `  - ${c.name}: ${c.detail}`).join("\n"),
      );
    }
    child.kill("SIGTERM");
    await new Promise((resolve) => child.on("close", resolve));

    if (screenshotsDir) {
      const shotProfile = mkdtempSync(path.join(tmpdir(), "portugol-ff-shot-"));
      const shotFile = path.join(screenshotsDir, screenshotName);
      const shotUrl = `http://127.0.0.1:${address.port}/?shot=${encodeURIComponent(scenario)}&w=${width}&h=${height}`;
      try {
        await new Promise((resolve, reject) => {
          const shotProc = spawn("firefox", [
            "--headless",
            "--no-remote",
            "--profile", shotProfile,
            `--window-size=${width},${height}`,
            "--screenshot", shotFile,
            shotUrl,
          ], { env: { ...process.env, MOZ_HEADLESS: "1" }, stdio: ["ignore", "ignore", "ignore"] });
          shotProc.on("close", (code) => (code === 0 ? resolve() : reject(new Error(`Screenshot falhou (code ${code})`))));
        });
        assert.ok(existsSync(shotFile), `Screenshot esperado não gerado: ${shotFile}`);
      } finally {
        rmSync(shotProfile, { recursive: true, force: true });
      }
      console.log(`✔ E2E ${scenario} @ ${width}x${height} (${report.checks.length} verificações OK) → ${shotFile}`);
    } else {
      console.log(`✔ E2E ${scenario} @ ${width}x${height} (${report.checks.length} verificações OK)`);
    }
  } finally {
    child.kill("SIGKILL");
    rmSync(profileDir, { recursive: true, force: true });
  }
}

await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
try {
  await runViewport({ scenario: "editor-bridge-verify", width: 1536, height: 960, screenshotName: "ide-1536x960.png" });
  await runViewport({ scenario: "editor-bridge-verify", width: 1280, height: 800, screenshotName: "ide-1280x800.png" });
  await runViewport({ scenario: "home-dark", width: 1536, height: 960, screenshotName: "home-dark-1536x960.png" });
  console.log("Todos os cenários E2E e verificações visuais passaram.");
} finally {
  server.close();
}
