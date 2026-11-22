import type { LearningMode } from "./learning-mode";

export type LearningPins = Partial<Record<LearningMode, string>>;

export function learningPinForMode(
  pins: LearningPins | undefined,
  mode: LearningMode,
  availableConceptIds: readonly string[],
): string | undefined {
  const conceptId = pins?.[mode];
  return conceptId && availableConceptIds.includes(conceptId) ? conceptId : undefined;
}

export function setLearningPin(pins: LearningPins | undefined, mode: LearningMode, conceptId: string): LearningPins {
  return { ...pins, [mode]: conceptId };
}

export function toggleLearningPin(pins: LearningPins | undefined, mode: LearningMode, conceptId: string): LearningPins {
  if (pins?.[mode] !== conceptId) return setLearningPin(pins, mode, conceptId);
  const next = { ...pins };
  delete next[mode];
  return next;
}
