const assets = import.meta.glob<string>("./ui-assets/*/*/*.png", {
  eager: true, query: "?url&no-inline", import: "default",
});

function setIcon(element: HTMLElement, file: string) {
  element.classList.add("ui-icon");
  element.setAttribute("aria-hidden", "true");
  element.style.setProperty("--icon-dark", `url("${assets[`./ui-assets/Dark/${file}.png`]}")`);
  element.style.setProperty("--icon-light", `url("${assets[`./ui-assets/Portugol/${file}.png`]}")`);
}

export function icon(file: string) {
  const element = document.createElement("span");
  setIcon(element, file);
  return element;
}

export function mountIcons() {
  for (const element of document.querySelectorAll<HTMLElement>("[data-icon]")) {
    const file = element.dataset.icon;
    if (file) setIcon(element, file);
  }
}
