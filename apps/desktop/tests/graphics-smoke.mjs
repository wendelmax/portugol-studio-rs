// Execute com xvfb-run e com o binário compilado com tauri/custom-protocol.
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { setTimeout as delay } from "node:timers/promises";

const program = String.raw`programa {
    inclua biblioteca Graficos --> g
    inclua biblioteca Teclado --> t
    inclua biblioteca Mouse --> m
    inclua biblioteca Util --> u
    funcao inicio() {
        g.iniciar_modo_grafico(falso)
        g.definir_titulo_janela("Portugol Verificacao Grafica")
        g.definir_dimensoes_janela(320, 240)
        g.definir_cor(0x224466)
        g.limpar()
        g.definir_cor(0xff8800)
        g.desenhar_retangulo(24, 24, 80, 60, falso, verdadeiro)
        g.renderizar()

        inteiro tecla = t.ler_tecla()
        se (tecla == 76)
            g.definir_cor(0xff00ff)
        senao
            g.definir_cor(0xff0000)
        g.limpar()
        g.renderizar()

        inteiro botao = m.ler_botao()
        g.definir_cor(0x00ff00)
        g.desenhar_ponto(1, 1)
        inteiro imagem = g.renderizar_imagem(4, 4)
        se (botao == 0 e g.obter_cor_pixel(imagem, 1, 1) == -16711936)
            g.definir_cor(0x00ff00)
        senao
            g.definir_cor(0xff0000)
        g.limpar()
        g.renderizar()
        u.aguarde(8000)
    }
}`;

const smokeFile = join(tmpdir(), `portugol-graphics-smoke-${process.pid}.por`);
writeFileSync(smokeFile, program, "utf8");

const app = spawn(process.argv[2] ?? "target/debug/portugol-desktop", [smokeFile], {
  env: {
    ...process.env,
    GDK_BACKEND: "x11",
    WEBKIT_DISABLE_COMPOSITING_MODE: "1",
  },
  stdio: ["ignore", "inherit", "inherit"],
});

function xdotool(...args) {
  return execFileSync("xdotool", args, { encoding: "utf8", timeout: 3000, stdio: ["ignore", "pipe", "ignore"] });
}

function windowByTitle(title) {
  try {
    return xdotool("search", "--onlyvisible", "--name", title).trim().split("\n")[0];
  } catch {
    return "";
  }
}

function mainWindow() {
  const named = windowByTitle("^Portugol Studio");
  if (named) return named;
  // O título exposto pelo GTK pode variar com o gerenciador de janelas.
  // O Xvfb é exclusivo do teste e a IDE é sua única janela grande.
  try {
    const [screenWidth, screenHeight] = xdotool("getdisplaygeometry").trim().split(/\s+/).map(Number);
    return xdotool("search", "--onlyvisible", "--name", ".*").trim().split("\n").find((id) => {
      const size = xdotool("getwindowgeometry", "--shell", id);
      const width = Number(size.match(/^WIDTH=(\d+)/m)?.[1]);
      const height = Number(size.match(/^HEIGHT=(\d+)/m)?.[1]);
      return width >= 640 && height >= 480 && (width < screenWidth || height < screenHeight);
    });
  } catch {
    return "";
  }
}

function pixel(window) {
  const bytes = execFileSync("import", ["-window", window, "-crop", "1x1+50+40", "-depth", "8", "rgb:-"], {
    timeout: 3000,
  });
  return [...bytes.slice(0, 3)].join(",");
}

async function waitFor(label, read, accept = Boolean) {
  const deadline = Date.now() + 20_000;
  let latest;
  while (Date.now() < deadline) {
    latest = await read();
    if (accept(latest)) return latest;
    if (app.exitCode !== null) throw new Error(`A IDE encerrou com código ${app.exitCode}.`);
    await delay(200);
  }
  throw new Error(`${label}: ${latest || "sem resposta"}`);
}

try {
  const main = await waitFor("A IDE não abriu", mainWindow);
  console.log("Janela principal identificada:", main);
  xdotool("windowfocus", main);
  const selection = ["-selection", "clipboard", "-target", "UTF8_STRING"];
  const copy = (text) => execFileSync("xclip", [...selection, "-i"], {
    input: text, timeout: 3000, stdio: ["pipe", "ignore", "ignore"],
  });
  await waitFor("O editor não carregou o programa .por", async () => {
    copy("verificacao-do-teste");
    xdotool("key", "ctrl+a", "ctrl+c");
    await delay(150);
    try {
      const copied = execFileSync("xclip", [...selection, "-o"], {
        encoding: "utf8", timeout: 3000, stdio: ["ignore", "pipe", "ignore"],
      });
      return copied.trimEnd() === program.trimEnd();
    } catch {
      return false;
    }
  });
  console.log("Programa .por carregado e conferido no editor.");
  xdotool("key", "shift+F6");
  const graphics = await waitFor(
    "Janela gráfica não encontrada", () => windowByTitle("^Portugol Verificacao Grafica$"),
  );
  assert.equal(await waitFor("Primeiro quadro", () => pixel(graphics), (rgb) => rgb === "255,136,0"), "255,136,0");
  await delay(200);
  xdotool("windowfocus", graphics, "key", "l");
  assert.equal(await waitFor("Evento de teclado", () => pixel(graphics), (rgb) => rgb === "255,0,255"), "255,0,255");
  await delay(200);
  xdotool("mousemove", "--window", graphics, "50", "40", "click", "1");
  assert.equal(await waitFor("Mouse e captura RGBA", () => pixel(graphics), (rgb) => rgb === "0,255,0"), "0,255,0");
  console.log("Janela gráfica verificada com renderização, teclado, mouse e captura RGBA.");
} catch (error) {
  try {
    console.error(execFileSync("xwininfo", ["-root", "-tree"], { encoding: "utf8", timeout: 3000 }));
  } catch (displayError) {
    console.error(displayError.message);
  }
  throw error;
} finally {
  app.kill();
  rmSync(smokeFile, { force: true });
}
