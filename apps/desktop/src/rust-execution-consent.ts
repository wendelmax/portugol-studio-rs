export const RUST_EXECUTION_CONSENT_KEY = "portugol-studio.rust-execution-consent.v1";
export const CARGO_EXECUTION_CONSENT_KEY = "portugol-studio.cargo-execution-consent.v1";

export function hasRustExecutionConsent(storage: Pick<Storage, "getItem">): boolean {
  return storage.getItem(RUST_EXECUTION_CONSENT_KEY) === "accepted";
}

export function rememberRustExecutionConsent(
  storage: Pick<Storage, "setItem">,
  accepted: boolean,
): void {
  if (accepted) storage.setItem(RUST_EXECUTION_CONSENT_KEY, "accepted");
}

export function hasCargoExecutionConsent(storage: Pick<Storage, "getItem">): boolean {
  return storage.getItem(CARGO_EXECUTION_CONSENT_KEY) === "accepted";
}

export function rememberCargoExecutionConsent(
  storage: Pick<Storage, "setItem">,
  accepted: boolean,
): void {
  if (accepted) storage.setItem(CARGO_EXECUTION_CONSENT_KEY, "accepted");
}
