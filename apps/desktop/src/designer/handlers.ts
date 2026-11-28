/**
 * Duplo clique num controle (ADR 0018, seção 3): liga o evento padrão a uma
 * função e a escreve no code-behind, como o Windows Forms. Aqui só o texto;
 * quem cria o arquivo, abre a aba e move o cursor é o `designer-shell`.
 */
import { handlerSuggestion } from "./properties-model.ts";

const INDENT = "    ";

/** Nome da função de um evento: o que já está ligado, ou o sugerido (`btnOk_clicar`). */
export function handlerName(current: string | undefined, controlName: string, event: string): string {
  const trimmed = current?.trim();
  return trimmed ? trimmed : handlerSuggestion(controlName, event);
}

/** A função de um evento, pronta para o aluno escrever a lógica no meio. */
export function handlerStub(handler: string, window: string): { text: string; cursor: number } {
  const head = `funcao ${handler}(tela: ${window}) -> ${window} {\n${INDENT}seja mutavel t: ${window} = tela;\n${INDENT}`;
  return { text: `${head}\n${INDENT}retorne t;\n}\n`, cursor: head.length };
}

/**
 * Conteúdo inicial de um code-behind novo. Se a janela declara `Dados`, o
 * registro já vem declarado, porque a tela depende dele.
 */
export function codeBehindTemplate(window: string, data: string | null): string {
  const header = `// Código da tela ${window}. Cada evento ligado na tela chama uma função\n// que recebe a janela e devolve a janela atualizada.\n`;
  const record = data ? `\nregistro ${data} {\n${INDENT}// Os campos que a tela mostra com {dados.campo}.\n}\n` : "";
  return header + record;
}

function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * Onde o cursor vai e o que inserir no fim do código. Se a função já existe,
 * nada é inserido e o cursor vai para o começo do corpo dela.
 */
export function placeHandler(code: string, handler: string, window: string): { insert: string | null; from: number; cursor: number } {
  const existing = new RegExp(`\\bfuncao\\s+${escapeRegExp(handler)}\\s*\\(`).exec(code);
  if (existing) {
    const brace = code.indexOf("{", existing.index);
    const cursor = brace < 0 ? existing.index : code.indexOf("\n", brace) + 1 || brace + 1;
    return { insert: null, from: code.length, cursor };
  }
  const stub = handlerStub(handler, window);
  const separator = code.length === 0 ? "" : code.endsWith("\n\n") ? "" : code.endsWith("\n") ? "\n" : "\n\n";
  const insert = separator + stub.text;
  return { insert, from: code.length, cursor: code.length + separator.length + stub.cursor };
}
