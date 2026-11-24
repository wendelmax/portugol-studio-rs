/** Elemento da página pelo id. Os ids usados pela IDE existem em `index.html`. */
export const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
