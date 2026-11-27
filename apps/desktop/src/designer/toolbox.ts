/**
 * Caixa de ferramentas do designer (ADR 0018): os controles, contêineres e
 * componentes do esquema `.design`, arrastáveis para o canvas. Clicar ou
 * apertar Enter num item insere o controle no contêiner selecionado, para
 * quem usa teclado.
 */
import type { DesignElementInfo, DesignSchema } from "../api.ts";
import { TOOL_MIME } from "./interactions.ts";

const GROUPS: { category: DesignElementInfo["category"]; title: string }[] = [
  { category: "control", title: "Controles" },
  { category: "container", title: "Contêineres" },
  { category: "component", title: "Componentes" },
];

/** Grupos da caixa na ordem do esquema; a `Janela` não entra, porque é a raiz. */
export function toolboxGroups(schema: DesignSchema): { title: string; tools: DesignElementInfo[] }[] {
  return GROUPS.map(({ category, title }) => ({
    title,
    tools: schema.elements.filter((element) => element.category === category),
  })).filter((group) => group.tools.length > 0);
}

export function renderToolbox(host: HTMLElement, schema: DesignSchema, activate: (tag: string) => void): void {
  const heading = document.createElement("h2");
  heading.className = "design-toolbox-title";
  heading.textContent = "Caixa de ferramentas";
  const groups = toolboxGroups(schema).map((group) => {
    const section = document.createElement("section");
    section.className = "design-toolbox-group";
    const title = document.createElement("h3");
    title.textContent = group.title;
    const list = document.createElement("div");
    list.className = "design-toolbox-items";
    list.setAttribute("role", "list");
    for (const tool of group.tools) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "design-tool";
      button.draggable = true;
      button.dataset.tag = tool.name;
      button.setAttribute("role", "listitem");
      button.textContent = tool.label;
      button.title = `${tool.description}. Arraste para a tela ou aperte Enter para inserir.`;
      button.addEventListener("dragstart", (event) => {
        event.dataTransfer?.setData(TOOL_MIME, tool.name);
        event.dataTransfer?.setData("text/plain", `<${tool.name} />`);
        if (event.dataTransfer) event.dataTransfer.effectAllowed = "copy";
      });
      button.addEventListener("click", () => activate(tool.name));
      list.append(button);
    }
    section.append(title, list);
    return section;
  });
  host.replaceChildren(heading, ...groups);
}
