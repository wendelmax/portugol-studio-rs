import type { RustSourceCompileDiagnostic } from "../api";
import type { OwnershipScenario } from "../ownership-lab";

export interface OwnershipRustcItem {
  readonly severity: "error" | "warning";
  readonly code: string | null;
  readonly location: string | null;
  readonly message: string;
}

export interface OwnershipRustcReport {
  readonly provenance: "rustc-real";
  readonly provenanceLabel: string;
  readonly status: "compiled" | "compile-error" | "unavailable";
  readonly matchesExpected: boolean;
  readonly summary: string;
  readonly items: readonly OwnershipRustcItem[];
}

export const OWNERSHIP_RUSTC_PROVENANCE =
  "Verificação real via rustc — diagnóstico produzido pelo compilador Rust instalado.";

export function formatOwnershipRustcUnavailable(): OwnershipRustcReport {
  return {
    provenance: "rustc-real",
    provenanceLabel: OWNERSHIP_RUSTC_PROVENANCE,
    status: "unavailable",
    matchesExpected: false,
    summary: "O compilador rustc não foi detectado nesta máquina. Instale Rust (rustup.rs) para verificar cenários reais.",
    items: [],
  };
}

export function formatOwnershipRustcCheck(
  scenario: OwnershipScenario,
  diagnostics: readonly RustSourceCompileDiagnostic[],
): OwnershipRustcReport {
  const items: OwnershipRustcItem[] = diagnostics.map((diagnostic) => ({
    severity: diagnostic.severity === "error" ? "error" : "warning",
    code: diagnostic.code,
    location: diagnostic.start.line > 0
      ? `linha ${diagnostic.start.line}, coluna ${diagnostic.start.column}`
      : null,
    message: diagnostic.message,
  }));
  const errorCount = items.filter((item) => item.severity === "error").length;

  if (errorCount > 0) {
    const matchesExpected = scenario.expected === "compile-error";
    const plural = errorCount === 1 ? "erro" : "erros";
    return {
      provenance: "rustc-real",
      provenanceLabel: OWNERSHIP_RUSTC_PROVENANCE,
      status: "compile-error",
      matchesExpected,
      summary: matchesExpected
        ? `rustc rejeitou o cenário com ${errorCount} ${plural}, confirmando o erro de ownership esperado.`
        : `rustc encontrou ${errorCount} ${plural} inesperados ao verificar este cenário.`,
      items,
    };
  }

  const matchesExpected = scenario.expected === "compiles";
  return {
    provenance: "rustc-real",
    provenanceLabel: OWNERSHIP_RUSTC_PROVENANCE,
    status: "compiled",
    matchesExpected,
    summary: matchesExpected
      ? "rustc validou o cenário sem erros, confirmando que os empréstimos e escopos são válidos."
      : "rustc não reportou erros neste cenário (era esperado erro de compilação).",
    items,
  };
}
