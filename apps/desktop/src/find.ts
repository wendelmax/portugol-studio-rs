import {
  SearchQuery, closeSearchPanel, findNext, findPrevious, getSearchQuery, replaceAll,
  replaceNext, search, searchKeymap, setSearchQuery,
} from "@codemirror/search";
import type { Command } from "@codemirror/view";

export const editorSearchKeys = searchKeymap.filter(({ key }) => key !== "F3");

export const findDialog = search({
  createPanel(view) {
    const template = document.querySelector<HTMLTemplateElement>("#find-template")!;
    const dom = document.createElement("div");
    dom.append(template.content.cloneNode(true));
    const dialog = dom.querySelector("dialog")!;
    const form = dom.querySelector("form")!;
    const input = dom.querySelector<HTMLInputElement>("[main-field]")!;
    const replacement = dom.querySelector<HTMLInputElement>("[name=replace]")!;
    const caseSensitive = dom.querySelector<HTMLInputElement>("[name=case]")!;
    const wholeWord = dom.querySelector<HTMLInputElement>("[name=whole]")!;
    const regexp = dom.querySelector<HTMLInputElement>("[name=regex]")!;
    const markAll = dom.querySelector<HTMLInputElement>("[name=mark]")!;
    const backwards = dom.querySelector<HTMLInputElement>("[name=direction][value=back]")!;
    const status = dom.querySelector("[role=status]")!;
    const replace = dom.querySelector<HTMLButtonElement>("[name=replace-one]")!;
    const all = dom.querySelector<HTMLButtonElement>("[name=replace-all]")!;
    let query = getSearchQuery(view.state);

    function fill() {
      input.value = query.search;
      replacement.value = query.replace;
      caseSensitive.checked = query.caseSensitive;
      wholeWord.checked = query.wholeWord;
      regexp.checked = query.regexp;
      replace.disabled = all.disabled = view.state.readOnly;
    }

    function commit() {
      const next = new SearchQuery({
        search: input.value,
        replace: replacement.value,
        caseSensitive: caseSensitive.checked,
        wholeWord: wholeWord.checked,
        regexp: regexp.checked,
      });
      if (!query.eq(next)) {
        query = next;
        view.dispatch({ effects: setSearchQuery.of(next) });
      }
      return next;
    }

    function run(command: Command) {
      const next = commit();
      if (!next.valid) {
        status.textContent = next.search ? "Expressão regular inválida." : "Digite o texto a procurar.";
        return;
      }
      status.textContent = command(view) ? "" : "Nenhuma ocorrência encontrada.";
    }

    form.oninput = () => {
      commit();
      status.textContent = "";
      view.dom.classList.toggle("find-mark-all", markAll.checked);
    };
    form.onsubmit = (event) => {
      event.preventDefault();
      run(backwards.checked ? findPrevious : findNext);
    };
    const replacePrevious: Command = () => {
      const { from, to } = view.state.selection.main;
      const match = query.getCursor(view.state, from, to).next();
      if (match.done || match.value.from !== from || match.value.to !== to) {
        return findPrevious(view);
      }
      if (!replaceNext(view)) return false;
      view.dispatch({ selection: { anchor: from }, userEvent: "select.search" });
      findPrevious(view);
      return true;
    };
    replace.onclick = () => run(backwards.checked ? replacePrevious : replaceNext);
    all.onclick = () => run(replaceAll);
    const close = () => { closeSearchPanel(view); };
    dom.querySelector<HTMLButtonElement>("[name=cancel]")!.onclick = close;
    dialog.oncancel = (event) => {
      event.preventDefault();
      close();
    };
    fill();

    return {
      dom,
      mount() {
        dialog.showModal();
        view.dom.classList.toggle("find-mark-all", markAll.checked);
      },
      update(update) {
        const next = getSearchQuery(update.state);
        if (!query.eq(next)) {
          query = next;
          fill();
        }
      },
      destroy() {
        dialog.close();
        view.dom.classList.remove("find-mark-all");
      },
    };
  },
});
