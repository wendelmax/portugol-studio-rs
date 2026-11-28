import type { LearningMode } from "../learning-mode";
import type { SourceLanguage } from "../document-language";

type DiagnosticSummary = { severity: string };
type DiagnosticAnalysis = { source: string; status: "complete" | "unavailable" };

export interface StatusBarTab {
  language: SourceLanguage;
  diagnostics: DiagnosticSummary[];
  runtimeCompileDiagnostics?: DiagnosticSummary[];
  diagnosticsAnalysis?: DiagnosticAnalysis;
}

export interface StatusBarState {
  language: string;
  mode: string;
  engines: string | null;
  indentation: string;
  encoding: string;
  diagnostics: {
    state: "pending" | "ready" | "unavailable";
    errors: number;
    warnings: number;
    label: string;
  };
}

const languages: Record<SourceLanguage, string> = {
  "ferro-rust": "Ferro",
  rust: "Rust",
  markup: "Marcação (pseudo-XAML)",
};

const modes: Record<LearningMode, string> = {
  beginner: "Iniciante",
  bridge: "Ponte para Rust",
  rust: "Rust",
};

function countDiagnostics(diagnostics: DiagnosticSummary[], severity: string) {
  return diagnostics.filter((diagnostic) => diagnostic.severity === severity).length;
}

export function statusBarState(
  tab: StatusBarTab | undefined,
  source: string,
  learningMode: LearningMode,
  rustcAvailable: boolean | null,
  tabSize: number,
): StatusBarState | null {
  if (!tab) return null;
  const diagnostics = [...tab.diagnostics, ...(tab.runtimeCompileDiagnostics ?? [])];
  const analysis = tab.diagnosticsAnalysis?.source === source
    ? tab.diagnosticsAnalysis.status
    : "pending";
  const errors = countDiagnostics(diagnostics, "error");
  const warnings = countDiagnostics(diagnostics, "warning");
  const countsLabel = `${errors} ${errors === 1 ? "erro" : "erros"} · ${warnings} ${warnings === 1 ? "aviso" : "avisos"}`;
  const label = analysis === "pending" ? "Analisando…"
    : analysis === "unavailable" ? errors + warnings > 0 ? `${countsLabel} · análise parcial` : "Análise indisponível"
      : errors === 0 && warnings === 0 ? "Sem erros"
        : countsLabel;

  return {
    language: languages[tab.language],
    mode: modes[learningMode],
    engines: tab.language === "ferro-rust" && rustcAvailable === true
      ? "VM e Rust nativo"
      : null,
    indentation: `${Math.max(1, Math.floor(tabSize))} espaços`,
    encoding: "UTF-8 ao salvar",
    diagnostics: { state: analysis === "complete" ? "ready" : analysis, errors, warnings, label },
  };
}
