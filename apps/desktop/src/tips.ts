import tips from "./assets/tips/index.json";

const images = import.meta.glob<string>("./assets/tips/*.{png,gif}", {
  eager: true,
  query: "?url",
  import: "default",
});
const startupKey = "portugol.tips.showOnStartup";

export function mountTips(toggleTheme: () => void) {
  const dialog = document.querySelector<HTMLDialogElement>("#tips")!;
  const title = document.querySelector<HTMLElement>("#tips-title")!;
  const description = document.querySelector<HTMLElement>("#tips-description")!;
  const image = document.querySelector<HTMLImageElement>("#tips-image")!;
  const startup = document.querySelector<HTMLInputElement>("#tips-startup")!;
  let index = 0;

  function showTip(offset: number) {
    index = (index + offset + tips.length) % tips.length;
    const tip = tips[index];
    title.textContent = `${index + 1}/${tips.length} – ${tip.title}`;
    description.textContent = tip.description;
    image.src = images[`./assets/tips/${tip.image}`];
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
