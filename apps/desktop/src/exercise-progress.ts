export interface ExerciseEvaluation {
  total: number;
  passed: number;
  failures: string[];
}

export interface ExerciseAttempt {
  revision: string;
  status: "passed" | "failed";
  total: number;
  passed: number;
  gradedAt: string;
}

export interface ExerciseProgressEntry {
  latest: ExerciseAttempt;
  completedRevision?: string;
}

export interface ExerciseProgressState {
  version: 1;
  exercises: Record<string, ExerciseProgressEntry>;
}

export interface StorageLike {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export interface ExerciseProgressOutcome {
  recorded: boolean;
  completed: boolean;
  revision: string | null;
}

export interface CryptoLike {
  subtle: Pick<SubtleCrypto, "digest">;
}

const storageKey = "portugol.exercise-progress.v1";
const emptyState = (): ExerciseProgressState => ({ version: 1, exercises: {} });

async function digest(value: string, cryptoApi: CryptoLike | null | undefined): Promise<string | null> {
  if (!cryptoApi?.subtle?.digest) return null;
  try {
    const bytes = new TextEncoder().encode(value);
    const hash = await cryptoApi.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(hash), (byte) => byte.toString(16).padStart(2, "0")).join("");
  } catch {
    return null;
  }
}

export async function exerciseIdentity(
  id: number,
  contents?: string,
  cryptoApi: CryptoLike | null | undefined = globalThis.crypto,
): Promise<string | null> {
  if (!Number.isInteger(id) || id < 0) return null;
  if (contents === undefined) return `builtin:${id}`;
  const definitionHash = await digest(contents, cryptoApi);
  return definitionHash ? `imported:${id}:${definitionHash}` : null;
}

export async function sourceRevision(
  source: string,
  cryptoApi: CryptoLike | null | undefined = globalThis.crypto,
): Promise<string | null> {
  return digest(source, cryptoApi);
}

export function exerciseOptionLabel(name: string, completed: boolean): string {
  return completed ? `${name} — concluído` : name;
}

export function exerciseResultSummary(
  result: { total: number; passed: number; hints: string[] },
  progress: Pick<ExerciseProgressOutcome, "completed">,
): string {
  if (result.total === 0) {
    return result.hints.length ? "Análise estática concluída." : "A análise estática não encontrou observações.";
  }
  return progress.completed
    ? `${result.passed} de ${result.total} casos aprovados. Exercício concluído nesta revisão.`
    : `${result.passed} de ${result.total} casos aprovados. Ainda faltam casos para concluir.`;
}

export function exerciseRevisionStatus(progress: Pick<ExerciseProgressOutcome, "revision" | "recorded">): string {
  if (!progress.revision) return "Sem casos de teste avaliados; esta análise não gera conclusão de exercício.";
  return progress.recorded
    ? `Revisão avaliada: ${progress.revision.slice(0, 12)}. A conclusão se refere somente a esta revisão.`
    : `Revisão avaliada: ${progress.revision.slice(0, 12)}. Não foi possível salvar o progresso neste dispositivo.`;
}

function isAttempt(value: unknown): value is ExerciseAttempt {
  if (typeof value !== "object" || value === null) return false;
  const attempt = value as Partial<ExerciseAttempt>;
  return typeof attempt.revision === "string"
    && /^[a-f0-9]{64}$/.test(attempt.revision)
    && (attempt.status === "passed" || attempt.status === "failed")
    && Number.isInteger(attempt.total) && Number(attempt.total) > 0
    && Number.isInteger(attempt.passed) && Number(attempt.passed) >= 0 && Number(attempt.passed) <= Number(attempt.total)
    && typeof attempt.gradedAt === "string";
}

function decodeState(raw: string | null): ExerciseProgressState {
  if (!raw) return emptyState();
  try {
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null) return emptyState();
    const state = parsed as Partial<ExerciseProgressState>;
    if (state.version !== 1 || typeof state.exercises !== "object" || state.exercises === null) return emptyState();
    const exercises: Record<string, ExerciseProgressEntry> = {};
    for (const [identity, entry] of Object.entries(state.exercises)) {
      if (typeof entry !== "object" || entry === null) continue;
      const candidate = entry as Partial<ExerciseProgressEntry>;
      if (!isAttempt(candidate.latest)) continue;
      exercises[identity] = {
        latest: candidate.latest,
        ...(typeof candidate.completedRevision === "string" && /^[a-f0-9]{64}$/.test(candidate.completedRevision)
          ? { completedRevision: candidate.completedRevision }
          : {}),
      };
    }
    return { version: 1, exercises };
  } catch {
    return emptyState();
  }
}

export function createExerciseProgress(
  storage?: StorageLike | null,
  cryptoApi: CryptoLike | null | undefined = globalThis.crypto,
) {
  let localStorage: StorageLike | null | undefined = storage;
  if (arguments.length === 0) {
    try {
      localStorage = globalThis.localStorage;
    } catch {
      localStorage = null;
    }
  }

  function read(): ExerciseProgressState {
    try {
      return decodeState(localStorage?.getItem(storageKey) ?? null);
    } catch {
      return emptyState();
    }
  }

  function hasPassed(identity: string): boolean {
    return Boolean(read().exercises[identity]?.completedRevision);
  }

  async function record(
    identity: string | null,
    source: string,
    result: ExerciseEvaluation,
    isCurrent: () => boolean = () => true,
  ) {
    const completed = Number.isInteger(result.total) && result.total > 0
      && Number.isInteger(result.passed) && result.passed === result.total
      && Array.isArray(result.failures) && result.failures.length === 0;
    if (!identity || !Number.isInteger(result.total) || result.total <= 0
      || !Number.isInteger(result.passed) || result.passed < 0 || result.passed > result.total
      || !Array.isArray(result.failures)) {
      return { recorded: false, completed, revision: null };
    }
    const revision = await sourceRevision(source, cryptoApi);
    if (!revision || !isCurrent()) return { recorded: false, completed, revision };

    const state = read();
    const previous = state.exercises[identity];
    const attempt: ExerciseAttempt = {
      revision,
      status: completed ? "passed" : "failed",
      total: result.total,
      passed: result.passed,
      gradedAt: new Date().toISOString(),
    };
    state.exercises[identity] = {
      latest: attempt,
      ...(completed
        ? { completedRevision: revision }
        : previous?.completedRevision ? { completedRevision: previous.completedRevision } : {}),
    };
    try {
      if (!localStorage) return { recorded: false, completed, revision };
      localStorage.setItem(storageKey, JSON.stringify(state));
      return { recorded: true, completed, revision };
    } catch {
      return { recorded: false, completed, revision };
    }
  }

  return { read, hasPassed, record };
}

export function createValidatedExerciseGrader<T extends ExerciseEvaluation & { hints: string[] }>(
  check: (source: string) => Promise<T>,
  progress: Pick<ReturnType<typeof createExerciseProgress>, "record">,
  identity: string | null,
  isCurrent: () => boolean = () => true,
) {
  return async (source: string): Promise<{ evaluation: T; result: ExerciseProgressOutcome }> => {
    const evaluation = await check(source);
    if (!isCurrent()) return {
      evaluation,
      result: { recorded: false, completed: false, revision: null },
    };
    const result = await progress.record(identity, source, evaluation, isCurrent);
    return { evaluation, result };
  };
}

export interface ExerciseEvaluationDocument {
  exercise: { id: number };
  contents?: string;
}

export interface ExerciseEvaluationResult<T extends ExerciseEvaluation & { hints: string[] }> {
  evaluation: T;
  progress: ExerciseProgressOutcome;
  current: boolean;
}

export function createExerciseEvaluationRunner<T extends ExerciseEvaluation & { hints: string[] }>(options: {
  check: (id: number, source: string, contents?: string) => Promise<T>;
  identify: (id: number, contents?: string) => Promise<string | null>;
  progress: Pick<ReturnType<typeof createExerciseProgress>, "record">;
}) {
  let sequence = 0;
  const latestByExercise = new Map<string, number>();

  async function evaluate(
    document: ExerciseEvaluationDocument,
    source: string,
    isCurrent: () => boolean = () => true,
  ): Promise<ExerciseEvaluationResult<T>> {
    const request = ++sequence;
    const scope = `${document.exercise.id}:${document.contents ?? "builtin"}`;
    latestByExercise.set(scope, request);
    const identity = await options.identify(document.exercise.id, document.contents);
    const isRunCurrent = () => latestByExercise.get(scope) === request && isCurrent();
    const grader = createValidatedExerciseGrader(
      (gradedSource) => options.check(document.exercise.id, gradedSource, document.contents),
      options.progress,
      identity,
      isRunCurrent,
    );
    const { evaluation, result } = await grader(source);
    return { evaluation, progress: result, current: isCurrent() };
  }

  return { evaluate };
}
