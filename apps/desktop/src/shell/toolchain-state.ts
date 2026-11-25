export type ToolchainAvailability = boolean | null;

export interface ToolchainSnapshot {
  rustc: ToolchainAvailability;
  cargo: ToolchainAvailability;
  rustfmt: ToolchainAvailability;
}

/** Mantém o resultado independente da detecção de cada ferramenta Rust. */
export function createToolchainState() {
  let rustc: ToolchainAvailability = null;
  let cargo: ToolchainAvailability = null;
  let rustfmt: ToolchainAvailability = null;

  function snapshot(): ToolchainSnapshot {
    return { rustc, cargo, rustfmt };
  }

  return {
    snapshot,
    rustcAvailable: () => rustc,
    cargoAvailable: () => cargo,
    rustfmtAvailable: () => rustfmt,
    setRustcAvailable: (available: ToolchainAvailability) => { rustc = available; },
    setCargoAvailable: (available: ToolchainAvailability) => { cargo = available; },
    setRustfmtAvailable: (available: ToolchainAvailability) => { rustfmt = available; },
  };
}
