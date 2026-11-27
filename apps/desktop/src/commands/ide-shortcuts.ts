import { ctrlShortcut, type Shortcut } from "./shortcuts.ts";

export type IdeShortcutActions = Record<string, () => unknown>;

/** Monta o catálogo único usado pela paleta, diálogo e captura global de teclas. */
export function createIdeShortcuts(actions: IdeShortcutActions): Shortcut[] {
  const shortcuts: Shortcut[] = [
    { label: "Shift+F6 / Ctrl+Enter", command: "Executar", matches: (e) =>
      (e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F6")
      || ((e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.key === "Enter"), action: actions.runOrContinue },
    { label: "Shift+F5", command: "Depurar", matches: (e) => e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F5", action: actions.stepOrStart },
    { label: "Shift+F7", command: "Interromper", matches: (e) => e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F7", action: actions.stopProgram },
    { label: "F9", command: "Pausar", matches: (e) => !e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F9", action: actions.pauseDebug },
    { label: "F5", command: "Continuar", matches: (e) => !e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F5", action: actions.continueDebug },
    { label: "Shift+F9 / F10", command: "Próxima instrução",
      matches: (e) => (e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F9")
      || (!e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F10"), action: actions.stepOver },
    { label: "Alt+F11", command: "Entrar na função", matches: (e) => e.altKey && !e.shiftKey && !e.ctrlKey && !e.metaKey && e.key === "F11", action: actions.stepInto },
    { label: "Shift+F11", command: "Sair da função", matches: (e) => e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F11", action: actions.stepOut },
    { label: "F1", command: "Ajuda", matches: (e) => !e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F1", action: actions.help },
    { label: "Ctrl+R", command: "Renomear símbolo",
      matches: (e) => (e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.code === "KeyR", action: actions.renameSymbol },
    { label: "Ctrl+Alt+F", command: "Formatar código-fonte",
      matches: (e) => (e.ctrlKey || e.metaKey) && e.altKey && !e.shiftKey && e.code === "KeyF", action: actions.formatCode },
    { label: "via paleta de comandos", command: "Executar como Rust", matches: () => false, action: actions.runAsRust },
    { label: "via paleta de comandos", command: "Analisar projeto Cargo", matches: () => false, action: actions.checkCargoProject },
    { label: "via paleta de comandos", command: "Testar projeto Cargo", matches: () => false, action: actions.runCargoTests },
    { label: "F3", command: "Dicas de interface", matches: (e) => !e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F3", action: actions.openTips },
    { label: "F12", command: "Sobre (fora do editor); ir à declaração (no editor)", matches: (e) => !e.shiftKey && !e.altKey && !e.ctrlKey && !e.metaKey && e.key === "F12", action: actions.about },
    { label: "F11", command: "Atalhos de teclado", matches: (e) => !e.altKey && !e.shiftKey && !e.ctrlKey && !e.metaKey && e.key === "F11", action: actions.showShortcuts },
    { label: "Alt+Home", command: "Tela inicial", matches: (e) => e.altKey && e.key === "Home", action: actions.home },
    { label: "Alt+←", command: "Aba anterior", matches: (e) => e.altKey && e.key === "ArrowLeft", action: actions.previousTab },
    { label: "Alt+→", command: "Próxima aba", matches: (e) => e.altKey && e.key === "ArrowRight", action: actions.nextTab },
    { label: "Shift+Pause", command: "Ativar/desativar centralização", matches: (e) => e.shiftKey && e.key === "Pause", action: actions.toggleCenter },
    { label: "Shift+Esc", command: "Expandir/restaurar editor", matches: (e) => e.shiftKey && e.key === "Escape", action: actions.toggleExpanded },
    { label: "Ctrl+Shift+P", command: "Paleta de comandos", matches: (e) =>
      (e.ctrlKey || e.metaKey) && e.shiftKey && !e.altKey && e.code === "KeyP", action: actions.commandPalette },
    { label: "Ctrl+,", command: "Configurações", matches: (e) =>
      (e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.code === "Comma", action: actions.openSettings },
    { label: "via paleta de comandos", command: "Restaurar layout padrão", matches: () => false, action: actions.resetLayout },
    { label: "via paleta de comandos", command: "Abrir Laboratório de Ownership", matches: () => false, action: actions.openOwnershipLab },
    { label: "via paleta de comandos", command: "Alternar prévia Rust", matches: () => false, action: actions.toggleRustPreview },
    { label: "via paleta de comandos", command: "Alternar Preview Visual PORS", matches: () => false, action: actions.toggleUiPreview },
    { label: "via paleta de comandos", command: "Abrir pasta de projeto", matches: () => false, action: actions.openFolder },
    { label: "via paleta de comandos", command: "Novo projeto desktop", matches: () => false, action: actions.newDesktopProject },
    { label: "via paleta de comandos", command: "Novo arquivo Rust (.rs)", matches: () => false, action: actions.newRustFile },
    { label: "via paleta de comandos", command: "Abrir Exercícios", matches: () => false, action: actions.openExercises },
    { label: "via paleta de comandos", command: "Editor de temas", matches: () => false, action: actions.openThemeEditor },
    { label: "via paleta de comandos", command: "Abrir painel Aprender", matches: () => false, action: actions.openLearningPanel },
    { label: "via paleta de comandos", command: "Abrir Console", matches: () => false, action: actions.showConsolePanel },
    { label: "via paleta de comandos", command: "Abrir Mensagens", matches: () => false, action: actions.showMessagesPanel },
    { label: "via paleta de comandos", command: "Abrir Testes", matches: () => false, action: actions.showTestsPanel },
    { label: "via paleta de comandos", command: "Abrir Depuração", matches: () => false, action: actions.showDebugPanel },
    { label: "via paleta de comandos", command: "Limpar console", matches: () => false, action: actions.clearConsoleAction },
    { label: "via paleta de comandos", command: "Copiar saída do console", matches: () => false, action: actions.copyConsoleAction },
    { label: "via paleta de comandos", command: "Alternar quebra de linha", matches: () => false, action: actions.toggleWordWrap },
  ];
  const addFileShortcut = (key: string, shift: boolean, command: string, action: () => unknown) => {
    shortcuts.push(ctrlShortcut(key, shift, command, action));
  };

  addFileShortcut("N", false, "Novo arquivo", actions.newFile);
  addFileShortcut("O", false, "Abrir arquivo", actions.openFile);
  addFileShortcut("E", true, "Mostrar ou ocultar o explorador", actions.toggleExplorer);
  addFileShortcut("S", false, "Salvar", actions.save);
  addFileShortcut("S", true, "Salvar como", actions.saveAs);
  addFileShortcut("E", false, "Parâmetros de execução", actions.executionParameters);
  addFileShortcut("B", false, "Painel Utilitários", actions.utilities);
  addFileShortcut("L", false, "Buscar na estrutura", actions.searchOutline);
  addFileShortcut("Q", false, "Fechar aba", actions.closeTab);
  addFileShortcut("Q", true, "Fechar todas as abas", actions.closeAllTabs);
  shortcuts.push({ label: "Ctrl+F", command: "Localizar e substituir", matches: (e) =>
    (e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.code === "KeyF", action: actions.findReplace });

  const typeFilters = [
    ["I", "Filtrar tipo inteiro", actions.filterTypeInteger],
    ["C", "Filtrar tipo caracter", actions.filterTypeCharacter],
    ["R", "Filtrar tipo real", actions.filterTypeReal],
    ["A", "Filtrar tipo cadeia", actions.filterTypeString],
    ["L", "Filtrar tipo logico", actions.filterTypeLogical],
    ["V", "Filtrar tipo vazio", actions.filterTypeVoid],
    ["T", "Mostrar todos os tipos", actions.showAllTypes],
  ] as const;
  for (const [key, command, action] of typeFilters) {
    shortcuts.push({
      label: `Alt+Shift+${key}`,
      command,
      matches: (e) => e.altKey && e.shiftKey && e.code === `Key${key}`,
      action,
    });
  }

  const symbolFilters = [
    ["A", "variável", actions.filterVariable],
    ["V", "vetor", actions.filterArray],
    ["M", "matriz", actions.filterMatrix],
    ["F", "função", actions.filterFunction],
    ["T", "constante", actions.filterConstant],
  ] as const;
  for (const [key, label, action] of symbolFilters) {
    addFileShortcut(key, true, `Filtrar ${label}`, action);
  }
  return shortcuts;
}
