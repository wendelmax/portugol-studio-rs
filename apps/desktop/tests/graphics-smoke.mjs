// Execute com xvfb-run e com o binário compilado com tauri/custom-protocol.
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
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

const app = spawn(process.argv[2] ?? "target/debug/portugol-desktop", [], {
  env: { ...process.env, GDK_BACKEND: "x11" },
  stdio: ["ignore", "inherit", "inherit"],
});

function xdotool(...args) {
  return execFileSync("xdotool", args, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
}

function windowByTitle(title) {
  try {
    return xdotool("search", "--onlyvisible", "--name", title).trim().split("\n")[0];
  } catch {
    return "";
  }
}

function pixel(window) {
  const bytes = execFileSync("import", ["-window", window, "-crop", "1x1+50+40", "-depth", "8", "rgb:-"]);
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
  const main = await waitFor("A IDE não abriu", () => windowByTitle("^Portugol Studio$"));
  xdotool("windowfocus", "--sync", main);
  const copy = (text) => execFileSync("xclip", ["-selection", "clipboard", "-i"], {
    input: text, stdio: ["pipe", "ignore", "ignore"],
  });
  await waitFor("O editor não recebeu o programa", async () => {
    xdotool("key", "ctrl+n");
    await delay(150);
    copy(program);
    xdotool("key", "ctrl+a", "ctrl+v");
    await delay(250);
    copy("Verificação da área de transferência");
    xdotool("key", "ctrl+a", "ctrl+c");
    await delay(150);
    const copied = execFileSync("xclip", ["-selection", "clipboard", "-o"], { encoding: "utf8" });
    return copied.trimEnd() === program.trimEnd();
  });
  xdotool("key", "shift+F6");
  const graphics = await waitFor(
    "Janela gráfica não encontrada", () => windowByTitle("^Portugol Verificacao Grafica$"),
  );
  assert.equal(await waitFor("Primeiro quadro", () => pixel(graphics), (rgb) => rgb === "255,136,0"), "255,136,0");
  await delay(200);
  xdotool("windowfocus", "--sync", graphics, "key", "l");
  assert.equal(await waitFor("Evento de teclado", () => pixel(graphics), (rgb) => rgb === "255,0,255"), "255,0,255");
  await delay(200);
  xdotool("mousemove", "--window", graphics, "50", "40", "click", "1");
  assert.equal(await waitFor("Mouse e captura RGBA", () => pixel(graphics), (rgb) => rgb === "0,255,0"), "0,255,0");
  console.log("Janela gráfica verificada com renderização, teclado, mouse e captura RGBA.");
} finally {
  app.kill();
}
