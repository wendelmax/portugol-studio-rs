export interface GeneratedRustDetachSnapshot<TTab, TDocument, TArtifact> {
  tab: TTab;
  document: TDocument;
  artifact: TArtifact;
}

export interface GeneratedRustDetachCurrent<TTab, TDocument, TArtifact> extends GeneratedRustDetachSnapshot<TTab, TDocument, TArtifact> {
  pending: boolean;
  stale: boolean;
}

export function generatedRustDetachIsCurrent<TTab, TDocument, TArtifact>(
  snapshot: GeneratedRustDetachSnapshot<TTab, TDocument, TArtifact>,
  current: GeneratedRustDetachCurrent<TTab, TDocument, TArtifact>,
): boolean {
  return snapshot.tab === current.tab
    && snapshot.document === current.document
    && snapshot.artifact === current.artifact
    && !current.pending
    && !current.stale;
}
