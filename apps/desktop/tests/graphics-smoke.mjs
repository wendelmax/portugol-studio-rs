// Teste de integração Linux. Execute com xvfb-run após gerar o executável
// com --features tauri/custom-protocol, para testar a CSP do app empacotado.
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { createServer } from "node:net";
import { setTimeout as delay } from "node:timers/promises";

const program = String.raw`programa {
    inclua biblioteca Graficos --> g
    inclua biblioteca Teclado --> t
    inclua biblioteca Mouse --> m
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
        escreva("TECLA=", tecla, "\n")
        inteiro botao = m.ler_botao()
        escreva("BOTAO=", botao, "\n")
        g.definir_cor(0x00ff00)
        g.desenhar_ponto(1, 1)
        inteiro imagem = g.renderizar_imagem(4, 4)
        escreva("PIXEL=", g.obter_cor_pixel(imagem, 1, 1), "\n")
    }
}`;

const portServer = createServer();
await new Promise((resolve) => portServer.listen(0, "127.0.0.1", resolve));
const inspector = "127.0.0.1:" + portServer.address().port;
await new Promise((resolve, reject) => portServer.close((error) => error ? reject(error) : resolve()));

const app = spawn(process.argv[2] ?? "target/debug/portugol-desktop", [], {
  env: { ...process.env, GDK_BACKEND: "x11", WEBKIT_INSPECTOR_HTTP_SERVER: inspector },
  stdio: ["ignore", "inherit", "inherit"],
});

function evaluate(target, expression) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket("ws://" + inspector + target);
    const timeout = setTimeout(() => {
      ws.close();
      reject(new Error("O inspetor WebKit não respondeu."));
    }, 3000);
    ws.onerror = () => {
      clearTimeout(timeout);
      reject(new Error("Falha no WebSocket do inspetor."));
    };
    ws.onmessage = ({ data }) => {
      const result = JSON.parse(data);
      if (result.method === "Target.targetCreated") {
        ws.send(JSON.stringify({
          id: 1,
          method: "Target.sendMessageToTarget",
          params: {
            targetId: result.params.targetInfo.targetId,
            message: JSON.stringify({
              id: 2,
              method: "Runtime.evaluate",
              params: { expression, returnByValue: true },
            }),
          },
        }));
      }
      if (result.method !== "Target.dispatchMessageFromTarget") return;
      const response = JSON.parse(result.params.message);
      if (response.id !== 2) return;
      clearTimeout(timeout);
      ws.close();
      if (response.error || response.result?.wasThrown) {
        reject(new Error(JSON.stringify(response)));
      } else {
        resolve(response.result?.result?.value);
      }
    };
  });
}

async function value(graphics, expression) {
  const html = await fetch("http://" + inspector, { signal: AbortSignal.timeout(3000) }).then((res) => res.text());
  for (const row of html.split("<tr>")) {
    const target = row.match(/\/socket\/\d+\/\d+\/WebPage/)?.[0];
    if (target && row.includes("/graphics.html") === graphics) return evaluate(target, expression);
  }
}

async function waitFor(label, read, accept = Boolean) {
  const deadline = Date.now() + 20_000;
  let latest;
  while (Date.now() < deadline) {
    try {
      latest = await read();
      if (accept(latest)) return latest;
    } catch (error) {
      latest = String(error);
    }
    await delay(200);
  }
  throw new Error(`${label}: ${latest ?? "sem resposta"}`);
}

const mouse = (...args) => execFileSync("xdotool", args, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
const mainWindow = () => mouse("search", "--onlyvisible", "--name", "^Portugol Studio$").trim().split("\n")[0];
const graphicsWindow = () => mouse("search", "--onlyvisible", "--name", "^Portugol Verificacao Grafica$").trim().split("\n")[0];
const consoleOutput = () => value(false, 'document.querySelector("#console-output")?.textContent');

try {
  await waitFor("A IDE não abriu", () => value(false, 'Boolean(document.querySelector("#tabs button"))'));
  const main = await waitFor("Janela principal não encontrada", mainWindow);
  mouse("windowfocus", "--sync", main, "key", "ctrl+n");
  await waitFor("A aba de código não abriu", () => value(false, 'Boolean(document.querySelector(".cm-content[contenteditable]"))'));
  execFileSync("xclip", ["-selection", "clipboard", "-i"], { input: program, stdio: ["pipe", "ignore", "ignore"] });
  mouse("key", "ctrl+a", "ctrl+v");
  await waitFor("Código não colado", () => value(false, 'document.querySelector(".cm-content")?.textContent.includes("PIXEL=")'));
  mouse("key", "shift+F6");
  const graphic = await waitFor("Janela gráfica não encontrada", graphicsWindow);
  const rgba = await waitFor("O primeiro quadro não foi desenhado", () => value(true, `JSON.stringify(
    Array.from(document.querySelector("canvas").getContext("2d").getImageData(50, 40, 1, 1).data))`),
  (data) => data === "[255,136,0,255]");
  assert.deepEqual(JSON.parse(rgba), [255, 136, 0, 255]);
  await delay(200);
  mouse("windowfocus", "--sync", graphic, "key", "l");
  await waitFor("A VM não recebeu a tecla", consoleOutput, (text) => text?.includes("TECLA=76"));
  mouse("mousemove", "--window", graphic, "50", "40", "click", "1");
  const output = await waitFor("A captura não terminou", consoleOutput, (text) => text?.includes("Programa finalizado."));
  assert.match(output, /BOTAO=0/);
  assert.match(output, /PIXEL=-16711936/);
  console.log("Janela gráfica verificada com renderização, teclado, mouse e captura RGBA.");
} finally {
  app.kill();
}
