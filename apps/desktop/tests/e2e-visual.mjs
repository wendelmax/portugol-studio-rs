// Verificação E2E e visual real da IDE compilada (apps/desktop/dist) em Firefox headless
// nas resoluções de referência 1536x960 e 1280x800 (DESIGN_AS_CODE.md §9).
//
// Uso:
//   npm run build && node tests/e2e-visual.mjs
//   node tests/e2e-visual.mjs --screenshots /tmp/ferro-e2e-shots
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import http from "node:http";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(__dirname, "..");
const distDir = path.join(desktopRoot, "dist");
const REPORT_TIMEOUT_MS = Number(process.env.E2E_REPORT_TIMEOUT_MS || 45_000);

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

function e2eHarnessScript(scenario, width, height, runId) {
  return `<script>
(function () {
  const RUN_ID = ${JSON.stringify(runId)};
  function postReport(payload) {
    return fetch("/__e2e_report", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ ...payload, runId: RUN_ID, scenario: ${JSON.stringify(scenario)}, width: ${width}, height: ${height} })
    });
  }
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
        case "ferro_examples":
        case "list_ferro_examples":
          return [
            { id: "ola_mundo", title: "Olá Mundo", category: "Básico", description: "Primeiro programa FERRO.", source: 'funcao inicio() {\\n    escreva("Olá Mundo!");\\n}\\n' },
            { id: "resultado", title: "Divisão segura com Resultado", category: "Tipos", description: "Tratamento de erro tipado.", source: 'funcao inicio() {\\n    escreva("Ok");\\n}\\n' }
          ];
        case "take_pending_files":
          return [];
        case "inspect_ferro_rs":
        case "inspect":
          return { diagnostics: [], symbols: [{ name: "inicio", kind: "function", data_type: "vazio", offset: 0, line: 1 }] };
        case "export_ferro_rs":
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
      await postReport({ viewport: { w: window.innerWidth, h: window.innerHeight }, checks });
      return;
    }

    // 2. Criar novo arquivo .fer e validar shell + TitleBar sem sobreposição
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
      const libButtons = [...document.querySelectorAll("#utilities-libraries button")];
      const libNames = libButtons.map((b) => b.textContent);
      check("bridge-split-visible-wide", isVisible(document.getElementById("rust-preview")) && isVisible(document.getElementById("rust-mapping-gutter")));
      check("editor-options-no-overlap-rust-header",
        !overlaps(rect(editorOptionsSummary), rect(previewHeader)),
        "options=" + JSON.stringify(rect(editorOptionsSummary)) + " rustHeader=" + JSON.stringify(rect(previewHeader)));
      check("inspector-ferro-libraries-enabled",
        libNames.includes("UI") && libNames.includes("Json") && !libNames.includes("Objetos") && libButtons.every((b) => !b.disabled),
        libNames.join(","));
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

    // 5. Abrir Aprender no Inspector e conferir novos conceitos FERRO + Ownership Lab
    document.querySelector('[data-activity="learn"]').click();
    await sleep(150);
    const learningPanel = document.getElementById("learning-panel");
    check("learning-panel-visible", isVisible(learningPanel));
    const conceptPicker = document.getElementById("learning-concept");
    const optionIds = [...conceptPicker.options].map((o) => o.value);
    check("learning-has-ferro-concepts",
      ["functions", "option-result", "collections", "records-enums"].every((id) => optionIds.includes(id)),
      optionIds.join(","));
    conceptPicker.value = "ownership";
    conceptPicker.dispatchEvent(new Event("change", { bubbles: true }));
    await sleep(100);
    check("ownership-lab-visible-with-provenance",
      isVisible(document.getElementById("ownership-lab")) &&
      (document.getElementById("ownership-provenance").textContent || "").includes("Simulação didática"));

    // 6. Abrir Ajuda e confirmar menus 100% FERRO
    if (${JSON.stringify(scenario)} === "editor-bridge-verify") {
      document.getElementById("btn-help").click();
      await sleep(350);
      const helpHeadings = [...document.querySelectorAll("#help-topics h2")].map((h) => h.textContent);
      check("help-ferro-menus-only",
        helpHeadings.includes("Primeiros passos com Ferro") &&
        helpHeadings.includes("Bibliotecas padrão") &&
        helpHeadings.includes("UI e multimídia") &&
        helpHeadings.includes("Referência da sintaxe (.fer)") &&
        !helpHeadings.includes("Bibliotecas clássicas"),
        helpHeadings.join(" | "));
      // Voltar para o editor no modo escuro IntelliJ Rust para eventual screenshot
      document.querySelectorAll("#tabs .file-tab button")[0]?.click();
      await sleep(150);
      themeSelect.value = "preset:intellij-rust-dark";
      themeSelect.dispatchEvent(new Event("change", { bubbles: true }));
      await sleep(100);
    }

    await postReport({ viewport: { w: window.innerWidth, h: window.innerHeight }, checks });
  }

  window.addEventListener("error", (ev) => {
    postReport({ error: "window.error: " + (ev.message || ev.error), checks: [] });
  });

  function startScenario() {
    runScenario().catch((err) => {
      postReport({ error: String(err), checks: [] });
    });
  }
  // O script é injetado no <head>; se o load já tiver ocorrido, não há evento
  // posterior e o relatório nunca seria enviado.
  if (document.readyState === "complete") startScenario();
  else window.addEventListener("load", startScenario);
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
      if (cmd === "ferro_examples" || cmd === "list_ferro_examples") {
        return [
          { id: "ola_mundo", title: "Olá Mundo", category: "Básico", description: "Primeiro programa FERRO.", source: 'funcao inicio() {\\n    escreva("Olá Mundo!");\\n}\\n' },
          { id: "resultado", title: "Divisão segura com Resultado", category: "Tipos", description: "Tratamento de erro tipado.", source: 'funcao inicio() {\\n    escreva("Ok");\\n}\\n' }
        ];
      }
      if (cmd === "take_pending_files") return [];
      if (cmd === "inspect_ferro_rs" || cmd === "inspect") return { diagnostics: [], symbols: [{ name: "inicio", kind: "function", data_type: "vazio", offset: 0, line: 1 }] };
      if (cmd === "export_ferro_rs" || cmd === "export_rust") {
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

// Cada execução de navegador tem um runId próprio: um relatório atrasado de
// uma execução anterior não pode mais satisfazer a espera da execução seguinte.
const pendingReports = new Map();

const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://127.0.0.1");
  if (req.method === "POST" && url.pathname === "/__e2e_report") {
    let body = "";
    req.on("data", (chunk) => { body += chunk; });
    req.on("end", () => {
      res.writeHead(204);
      res.end();
      let report;
      try { report = JSON.parse(body); } catch (error) { report = { error: String(error), checks: [] }; }
      const resolve = pendingReports.get(report.runId);
      if (resolve) resolve(report);
      else console.error(`Relatório E2E ignorado (sem espera ativa para runId=${report.runId ?? "desconhecido"})`);
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
    const runId = url.searchParams.get("rid") || "sem-rid";
    const html = readFileSync(filePath, "utf8");
    const injected = url.searchParams.has("shot")
      ? shotSetupScript(scenario)
      : e2eHarnessScript(scenario, width, height, runId);
    res.end(html.replace("<head>", `<head>${injected}`));
    return;
  }
  res.end(readFileSync(filePath));
});

function detectBrowser() {
  const candidates = process.env.E2E_BROWSER
    ? [process.env.E2E_BROWSER]
    : ["google-chrome", "chromium", "chromium-browser", "firefox"];
  for (const bin of candidates) {
    try {
      const resolved = execFileSync("which", [bin], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim();
      if (resolved) {
        return { bin: resolved, engine: bin.includes("firefox") ? "firefox" : "chromium" };
      }
    } catch {
      // Tentar próximo navegador instalado.
    }
  }
  throw new Error("Nenhum navegador headless encontrado (esperado google-chrome, chromium ou firefox).");
}

const browser = detectBrowser();

function browserArgs(profileDir, width, height, url, screenshotFile) {
  if (browser.engine === "chromium") {
    const flags = [
      "--headless=new",
      "--no-sandbox",
      "--disable-gpu",
      "--disable-dev-shm-usage",
      `--user-data-dir=${profileDir}`,
      `--window-size=${width},${height}`,
    ];
    if (screenshotFile) flags.push(`--screenshot=${screenshotFile}`);
    flags.push(url);
    return flags;
  }
  const flags = [
    "--headless",
    "--no-remote",
    "--profile", profileDir,
    `--window-size=${width},${height}`,
  ];
  if (screenshotFile) flags.push("--screenshot", screenshotFile);
  flags.push(url);
  return flags;
}

class BrowserLifecycleError extends Error {}

function tailFrom(stream) {
  let tail = "";
  stream.on("data", (chunk) => { tail = (tail + chunk.toString()).slice(-4000); });
  return () => tail;
}

async function runViewportAttempt({ scenario, width, height, screenshotName, runId }) {
  const profileDir = mkdtempSync(path.join(tmpdir(), "ferro-e2e-"));
  const address = server.address();
  const targetUrl = `http://127.0.0.1:${address.port}/?e2e=${encodeURIComponent(scenario)}&w=${width}&h=${height}&rid=${encodeURIComponent(runId)}`;

  let child;
  let settled = false;
  let timer;
  let heartbeat;
  let exitGrace;
  let rejectReport;
  let stderrTail = () => "";
  const finish = (callback) => {
    if (settled) return;
    settled = true;
    clearTimeout(timer);
    clearTimeout(exitGrace);
    clearInterval(heartbeat);
    pendingReports.delete(runId);
    callback();
  };
  const reportPromise = new Promise((resolve, reject) => {
    rejectReport = reject;
    timer = setTimeout(() => finish(() => reject(new Error(
      `Timeout aguardando relatório E2E (${scenario} ${width}x${height}) após ${REPORT_TIMEOUT_MS}ms.` +
      `\nNavegador pid=${child?.pid ?? "?"} código=${child?.exitCode ?? "?"} sinal=${child?.signalCode ?? "?"}` +
      `\nstderr: ${stderrTail() || "(vazio)"}`,
    ))), REPORT_TIMEOUT_MS);
    heartbeat = setInterval(() => {
      const tail = stderrTail();
      console.log(`Aguardando relatório E2E (${scenario} ${width}x${height}, ${runId})...${tail ? `\nstderr: ${tail}` : ""}`);
    }, 10_000);
    pendingReports.set(runId, (report) => finish(() => {
      if (report.scenario !== scenario || report.width !== width || report.height !== height) {
        reject(new Error(`Relatório E2E de outro cenário: ${JSON.stringify({ scenario: report.scenario, width: report.width, height: report.height })}`));
        return;
      }
      resolve(report);
    }));
  });

  try {
    child = spawn(browser.bin, browserArgs(profileDir, width, height, targetUrl, null), {
      env: { ...process.env, MOZ_HEADLESS: "1" },
      stdio: ["ignore", "pipe", "pipe"],
    });
    stderrTail = tailFrom(child.stderr);
    tailFrom(child.stdout);
    child.once("error", (error) => finish(() => rejectReport(new BrowserLifecycleError(`Falha ao iniciar navegador: ${error.message}`))));
    child.once("exit", (code, signal) => {
      // Um relatório já em trânsito pode chegar logo após o exit: dá esse
      // pequeno grace antes de considerar a falha como ciclo de vida.
      exitGrace = setTimeout(() => finish(() => rejectReport(new BrowserLifecycleError(
        `Navegador encerrado antes do relatório (código=${code}, sinal=${signal})\nstderr: ${stderrTail() || "(vazio)"}`,
      ))), 500);
    });

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
    if (child.exitCode === null && child.signalCode === null) child.kill("SIGTERM");
    await new Promise((resolve) => {
      if (child.exitCode !== null || child.signalCode !== null) return resolve();
      child.once("close", resolve);
    });

    if (screenshotsDir) {
      const shotProfile = mkdtempSync(path.join(tmpdir(), "ferro-shot-"));
      const shotFile = path.join(screenshotsDir, screenshotName);
      const shotUrl = `http://127.0.0.1:${address.port}/?shot=${encodeURIComponent(scenario)}&w=${width}&h=${height}`;
      try {
        await new Promise((resolve, reject) => {
          const shotProc = spawn(browser.bin, browserArgs(shotProfile, width, height, shotUrl, shotFile), {
            env: { ...process.env, MOZ_HEADLESS: "1" },
            stdio: ["ignore", "ignore", "ignore"],
          });
          shotProc.on("close", (code) => (code === 0 ? resolve() : reject(new Error(`Screenshot falhou (code ${code})`))));
        });
        assert.ok(existsSync(shotFile), `Screenshot esperado não gerado: ${shotFile}`);
      } finally {
        rmSync(shotProfile, { recursive: true, force: true });
      }
      console.log(`✔ E2E [${browser.engine}] ${scenario} @ ${width}x${height} (${report.checks.length} verificações OK) → ${shotFile}`);
    } else {
      console.log(`✔ E2E [${browser.engine}] ${scenario} @ ${width}x${height} (${report.checks.length} verificações OK)`);
    }
  } finally {
    if (child && child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
    finish(() => {});
    rmSync(profileDir, { recursive: true, force: true });
  }
}

async function runViewport(options) {
  for (let attempt = 1; attempt <= 2; attempt++) {
    const runId = `${options.scenario}-${options.width}x${options.height}-${attempt}`;
    try {
      await runViewportAttempt({ ...options, runId });
      return;
    } catch (error) {
      if (attempt === 2 || !(error instanceof BrowserLifecycleError)) throw error;
      console.log(`Navegador falhou antes do relatório (${options.scenario}); repetindo uma vez. ${error.message}`);
    }
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
