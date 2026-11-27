import tips from "./assets/tips/index.json";

const startupKey = "portugol.tips.showOnStartup";
const SVG_NS = "http://www.w3.org/2000/svg";

type TipItem = {
  id: string;
  badge: string;
  title: string;
  shortcut: string;
  visual: string;
  description: string;
};

function el(tag: string, attrs: Record<string, string>, text?: string): SVGElement {
  const node = document.createElementNS(SVG_NS, tag);
  for (const [key, value] of Object.entries(attrs)) {
    node.setAttribute(key, value);
  }
  if (text !== undefined) node.textContent = text;
  return node;
}

function renderTipVisual(kind: string): SVGSVGElement {
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("viewBox", "0 0 560 210");
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-hidden", "true");
  svg.classList.add("tip-illustration");

  // Moldura limpa de janela proprietária
  svg.append(
    el("rect", { x: "12", y: "10", width: "536", height: "190", rx: "12", class: "tip-frame" }),
    el("rect", { x: "12", y: "10", width: "536", height: "28", rx: "12", class: "tip-bar" }),
    el("rect", { x: "12", y: "26", width: "536", height: "12", class: "tip-bar" }),
    el("circle", { cx: "32", cy: "24", r: "4", class: "tip-dot-accent" }),
    el("circle", { cx: "46", cy: "24", r: "4", class: "tip-dot" }),
    el("circle", { cx: "60", cy: "24", r: "4", class: "tip-dot" }),
  );

  if (kind === "rust-bridge") {
    svg.append(
      el("rect", { x: "28", y: "52", width: "244", height: "132", rx: "8", class: "tip-card" }),
      el("rect", { x: "288", y: "52", width: "244", height: "132", rx: "8", class: "tip-card" }),
      el("text", { x: "42", y: "74", class: "tip-label" }, "programa.fer"),
      el("text", { x: "302", y: "74", class: "tip-label-accent" }, "Rust gerado (.rs)"),
      el("text", { x: "42", y: "102", class: "tip-code" }, "seja nota = 10;"),
      el("text", { x: "42", y: "124", class: "tip-code" }, "escreva(\"Nota: \", nota);"),
      el("text", { x: "302", y: "102", class: "tip-code" }, "let nota: i32 = 10;"),
      el("text", { x: "302", y: "124", class: "tip-code" }, "print!(\"Nota: {}\", nota);"),
      el("path", { d: "M274 114 L286 114", class: "tip-stroke-accent" }),
      el("rect", { x: "42", y: "148", width: "92", height: "22", rx: "5", class: "tip-pill" }),
      el("text", { x: "54", y: "163", class: "tip-pill-text" }, "Source map"),
    );
  } else if (kind === "debugger") {
    svg.append(
      el("rect", { x: "28", y: "52", width: "312", height: "132", rx: "8", class: "tip-card" }),
      el("rect", { x: "352", y: "52", width: "180", height: "132", rx: "8", class: "tip-card" }),
      el("rect", { x: "34", y: "98", width: "300", height: "22", rx: "4", class: "tip-highlight" }),
      el("circle", { cx: "46", cy: "109", r: "5", class: "tip-dot-error" }),
      el("text", { x: "62", y: "84", class: "tip-code" }, "seja mutavel soma = 0;"),
      el("text", { x: "62", y: "113", class: "tip-code" }, "soma += i;  // ponto de parada"),
      el("text", { x: "62", y: "142", class: "tip-code" }, "escreva(soma);"),
      el("text", { x: "366", y: "74", class: "tip-label-accent" }, "Inspector"),
      el("text", { x: "366", y: "102", class: "tip-code" }, "soma : inteiro = 15"),
      el("text", { x: "366", y: "124", class: "tip-code" }, "i    : inteiro = 5"),
      el("text", { x: "366", y: "152", class: "tip-label" }, "Pilha: inicio()"),
    );
  } else if (kind === "learning") {
    svg.append(
      el("rect", { x: "28", y: "52", width: "240", height: "132", rx: "8", class: "tip-card" }),
      el("rect", { x: "282", y: "52", width: "250", height: "132", rx: "8", class: "tip-card" }),
      el("text", { x: "42", y: "74", class: "tip-label-accent" }, "Trilha de aprendizagem"),
      el("circle", { cx: "48", cy: "100", r: "6", class: "tip-dot-ok" }),
      el("text", { x: "64", y: "104", class: "tip-code" }, "1. Fundamentos (.fer)"),
      el("circle", { cx: "48", cy: "128", r: "6", class: "tip-dot-ok" }),
      el("text", { x: "64", y: "132", class: "tip-code" }, "2. Funções e vetores"),
      el("circle", { cx: "48", cy: "156", r: "6", class: "tip-dot-accent" }),
      el("text", { x: "64", y: "160", class: "tip-code" }, "3. Opcao e Resultado"),
      el("text", { x: "296", y: "74", class: "tip-label" }, "Painel Testes"),
      el("rect", { x: "296", y: "92", width: "220", height: "28", rx: "6", class: "tip-highlight" }),
      el("text", { x: "308", y: "110", class: "tip-code" }, "3/3 casos aprovados"),
      el("text", { x: "296", y: "146", class: "tip-label" }, "Progresso validado por execução"),
    );
  } else if (kind === "ui-preview") {
    svg.append(
      el("rect", { x: "28", y: "52", width: "260", height: "132", rx: "8", class: "tip-card" }),
      el("rect", { x: "302", y: "52", width: "230", height: "132", rx: "8", class: "tip-card" }),
      el("text", { x: "42", y: "74", class: "tip-label" }, "visualizar(estado)"),
      el("text", { x: "42", y: "102", class: "tip-code" }, "UI.coluna(["),
      el("text", { x: "56", y: "122", class: "tip-code" }, "UI.texto(\"Contagem: 3\"),"),
      el("text", { x: "56", y: "142", class: "tip-code" }, "UI.botao(\"inc\", \"+1\")"),
      el("text", { x: "42", y: "162", class: "tip-code" }, "])"),
      el("text", { x: "318", y: "74", class: "tip-label-accent" }, "Preview Visual PORS"),
      el("text", { x: "354", y: "114", class: "tip-title-sm" }, "Contagem: 3"),
      el("rect", { x: "354", y: "130", width: "124", height: "30", rx: "6", class: "tip-pill" }),
      el("text", { x: "386", y: "149", class: "tip-pill-text" }, "+1 Somar"),
    );
  } else if (kind === "workspace") {
    svg.append(
      el("rect", { x: "28", y: "52", width: "180", height: "132", rx: "8", class: "tip-card" }),
      el("rect", { x: "222", y: "52", width: "310", height: "132", rx: "8", class: "tip-card" }),
      el("text", { x: "42", y: "74", class: "tip-label-accent" }, "Explorador"),
      el("text", { x: "42", y: "100", class: "tip-code" }, "meu-projeto/"),
      el("text", { x: "54", y: "122", class: "tip-code" }, "main.fer"),
      el("text", { x: "54", y: "144", class: "tip-code" }, "Cargo.toml"),
      el("text", { x: "54", y: "166", class: "tip-code" }, "src/main.rs"),
      el("text", { x: "238", y: "74", class: "tip-label" }, "Cargo & Rust nativo"),
      el("text", { x: "238", y: "106", class: "tip-code" }, "cargo build · cargo check · cargo test"),
      el("rect", { x: "238", y: "126", width: "140", height: "24", rx: "5", class: "tip-pill" }),
      el("text", { x: "252", y: "142", class: "tip-pill-text" }, "Exportar projeto"),
    );
  } else if (kind === "palette") {
    svg.append(
      el("rect", { x: "90", y: "48", width: "380", height: "138", rx: "10", class: "tip-card" }),
      el("rect", { x: "106", y: "62", width: "348", height: "28", rx: "6", class: "tip-highlight" }),
      el("text", { x: "118", y: "80", class: "tip-code" }, "> Formatar código-fonte"),
      el("text", { x: "118", y: "112", class: "tip-code" }, "Executar programa                   Shift+F6"),
      el("text", { x: "118", y: "136", class: "tip-code" }, "Renomear símbolo                      Ctrl+R"),
      el("text", { x: "118", y: "160", class: "tip-code" }, "Configurações                         Ctrl+,"),
    );
  } else if (kind === "themes") {
    svg.append(
      el("rect", { x: "28", y: "52", width: "156", height: "132", rx: "8", class: "tip-card" }),
      el("rect", { x: "202", y: "52", width: "156", height: "132", rx: "8", class: "tip-card" }),
      el("rect", { x: "376", y: "52", width: "156", height: "132", rx: "8", class: "tip-card" }),
      el("text", { x: "42", y: "76", class: "tip-label-accent" }, "IntelliJ Rust"),
      el("rect", { x: "42", y: "92", width: "128", height: "14", rx: "4", class: "tip-pill" }),
      el("rect", { x: "42", y: "114", width: "92", height: "10", rx: "3", class: "tip-highlight" }),
      el("text", { x: "216", y: "76", class: "tip-label" }, "GitHub / Dracula"),
      el("rect", { x: "216", y: "92", width: "128", height: "14", rx: "4", class: "tip-highlight" }),
      el("rect", { x: "216", y: "114", width: "100", height: "10", rx: "3", class: "tip-highlight" }),
      el("text", { x: "390", y: "76", class: "tip-label" }, "Alto contraste"),
      el("rect", { x: "390", y: "92", width: "128", height: "14", rx: "4", class: "tip-highlight" }),
      el("text", { x: "390", y: "146", class: "tip-code" }, "WCAG AA"),
    );
  } else {
    // Default: "pors"
    svg.append(
      el("rect", { x: "28", y: "52", width: "504", height: "132", rx: "8", class: "tip-card" }),
      el("text", { x: "44", y: "76", class: "tip-label-accent" }, "ola.fer — Portugol-RS"),
      el("text", { x: "44", y: "104", class: "tip-code" }, "funcao inicio() {"),
      el("text", { x: "60", y: "126", class: "tip-code" }, "seja nome = \"Portugol-RS\";"),
      el("text", { x: "60", y: "148", class: "tip-code" }, "escreva(\"Olá, \", nome, \"!\");"),
      el("text", { x: "44", y: "170", class: "tip-code" }, "}"),
      el("rect", { x: "386", y: "146", width: "130", height: "26", rx: "6", class: "tip-pill" }),
      el("text", { x: "400", y: "163", class: "tip-pill-text" }, "VM nativa pronta"),
    );
  }

  return svg;
}

export function mountTips(toggleTheme: () => void) {
  const dialog = document.querySelector<HTMLDialogElement>("#tips")!;
  const title = document.querySelector<HTMLElement>("#tips-title")!;
  const description = document.querySelector<HTMLElement>("#tips-description")!;
  const visual = document.querySelector<HTMLElement>("#tips-visual")!;
  const badge = document.querySelector<HTMLElement>("#tips-badge")!;
  const shortcut = document.querySelector<HTMLElement>("#tips-shortcut")!;
  const startup = document.querySelector<HTMLInputElement>("#tips-startup")!;
  const items = tips as TipItem[];
  let index = 0;

  function showTip(offset: number) {
    index = (index + offset + items.length) % items.length;
    const tip = items[index];
    title.textContent = `${index + 1}/${items.length} — ${tip.title}`;
    badge.textContent = tip.badge;
    shortcut.textContent = tip.shortcut;
    description.textContent = tip.description;
    visual.replaceChildren(renderTipVisual(tip.visual));
  }

  function open() {
    showTip(0);
    dialog.showModal();
  }

  document.querySelector<HTMLButtonElement>("#tips-previous")!.onclick = () => showTip(-1);
  document.querySelector<HTMLButtonElement>("#tips-next")!.onclick = () => showTip(1);
  document.querySelector<HTMLButtonElement>("#tips-theme")!.onclick = toggleTheme;
  dialog.addEventListener("keydown", (event) => {
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      event.preventDefault();
      showTip(event.key === "ArrowLeft" ? -1 : 1);
    }
  });
  startup.onchange = () => {
    try {
      localStorage.setItem(startupKey, String(startup.checked));
    } catch {
      // Em armazenamento indisponível, a opção vale nesta sessão.
    }
  };
  try {
    startup.checked = localStorage.getItem(startupKey) === "true";
  } catch {
    // Sem preferência gravada, a janela abre quando solicitada.
  }
  if (startup.checked) open();
  return { open };
}
