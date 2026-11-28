export type HomeNewFileLanguage = "ferro-rust" | "rust";

export type HomeNewFileActions = Partial<Record<HomeNewFileLanguage, () => void>>;

export function mountHomeNewFileMenu(menu: HTMLDetailsElement, actions: HomeNewFileActions) {
  const summary = menu.querySelector("summary");

  for (const button of menu.querySelectorAll<HTMLButtonElement>("[data-home-new-language]")) {
    button.addEventListener("click", () => {
      const language = button.dataset.homeNewLanguage;
      if (language !== "ferro-rust" && language !== "rust") return;
      const action = actions[language];
      if (!action) return;
      menu.open = false;
      action();
    });
  }

  menu.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || !menu.open) return;
    event.preventDefault();
    menu.open = false;
    summary?.focus();
  });
}
