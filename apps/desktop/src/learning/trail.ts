export interface LearningTrailExercise {
  id: string;
  title: string;
}

export interface LearningTrailStage {
  id: string;
  title: string;
  description: string;
  examplePaths: string[];
  exercises: LearningTrailExercise[];
}

export interface LearningTrailStageProgress {
  stage: LearningTrailStage;
  completed: boolean;
  completedExercises: number;
  nextExercise: LearningTrailExercise | null;
}

export interface LearningTrailProgress {
  status: "empty" | "not-started" | "in-progress" | "completed";
  completedStages: number;
  totalStages: number;
  stages: LearningTrailStageProgress[];
  suggestedExercise: LearningTrailExercise | null;
  message: string;
}

/** Versão do roteiro didático; mantenha-a alinhada aos catálogos .fer do serviço. */
export const LEARNING_TRAIL_VERSION = 1;

/** Catálogo de etapas e vínculos para exemplos e exercícios .fer incluídos no projeto. */
export const LEARNING_TRAIL: LearningTrailStage[] = [
  {
    id: "logica",
    title: "Lógica e decisões",
    description: "Comece com entradas, variáveis e condições.",
    examplePaths: ["01-ola-variaveis.fer", "02-condicionais.fer"],
    exercises: [
      { id: "pors-12-saudacao-cadeia", title: "Saudação com cadeia" },
      { id: "pors-01-dobro", title: "Dobro de um número" },
      { id: "pors-11-par-ou-impar", title: "Par ou ímpar" },
      { id: "pors-02-maior-de-dois", title: "Maior de dois inteiros" },
    ],
  },
  {
    id: "tipos-funcoes",
    title: "Tipos e funções",
    description: "Organize o algoritmo em funções e trabalhe com tipos explícitos.",
    examplePaths: ["04-funcoes-recursao.fer", "07-entrada-tipada.fer"],
    exercises: [
      { id: "pors-03-soma-ate-n", title: "Soma de 1 até N" },
      { id: "pors-04-funcao-fatorial", title: "Fatorial com função" },
      { id: "pors-08-dias-do-mes", title: "Validar ano/mês com Tipos e Calendario" },
    ],
  },
  {
    id: "estruturas",
    title: "Estruturas e coleções",
    description: "Repita operações e processe sequências de valores.",
    examplePaths: ["03-lacos.fer", "05-vetores.fer", "10-vetores-dinamicos.fer"],
    exercises: [
      { id: "pors-06-filtrar-positivos", title: "Filtrar positivos em vetor dinâmico" },
      { id: "pors-07-normalizar-csv", title: "Normalizar lista CSV com Texto" },
    ],
  },
  {
    id: "opcao-resultado",
    title: "Erros com Opcao e Resultado",
    description: "Represente resultados ausentes ou inválidos sem esconder falhas.",
    examplePaths: ["09-opcao-resultado.fer", "11-matematica-texto-tipos.fer"],
    exercises: [
      { id: "pors-05-divisao-segura", title: "Divisão segura com Resultado" },
      { id: "pors-09-resumo-json", title: "Extrair e atualizar JSON" },
    ],
  },
  {
    id: "ownership",
    title: "Ownership e dados compostos",
    description: "Aplique tipos compostos (registro/enum) e regras de propriedade a coleções.",
    examplePaths: ["06-registros-enums.fer", "10-vetores-dinamicos.fer"],
    exercises: [
      { id: "pors-13-registro-ponto", title: "Área com registro Retangulo" },
      { id: "pors-14-enum-semaforo", title: "Semáforo com enum e escolha" },
      { id: "pors-10-somar-lista-json", title: "Somar lista de inteiros em JSON" },
    ],
  },
];

export function learningTrailProgress(
  stages: LearningTrailStage[],
  hasPassed: (identity: string) => boolean,
): LearningTrailProgress {
  const stageProgress = stages.map((stage) => {
    const exercises = stage.exercises.map((exercise) => ({
      exercise,
      completed: hasPassed(`pors:${exercise.id}`),
    }));
    const completedExercises = exercises.filter(({ completed }) => completed).length;
    return {
      stage,
      completed: exercises.length > 0 && completedExercises === exercises.length,
      completedExercises,
      nextExercise: exercises.find(({ completed }) => !completed)?.exercise ?? null,
    };
  });
  const completedStages = stageProgress.filter(({ completed }) => completed).length;
  const totalExercises = stages.reduce((count, stage) => count + stage.exercises.length, 0);
  const allCompleted = totalExercises > 0 && stageProgress.every(({ completed }) => completed);
  const suggestedExercise = stageProgress.find(({ nextExercise }) => nextExercise)?.nextExercise ?? null;
  const hasProgress = stageProgress.some(({ completedExercises }) => completedExercises > 0);
  const status = totalExercises === 0
    ? "empty"
    : allCompleted
      ? "completed"
      : hasProgress
        ? "in-progress"
        : "not-started";

  const message = status === "empty"
    ? "A trilha ainda não tem exercícios disponíveis."
    : status === "completed"
      ? "Trilha concluída! Você resolveu todos os exercícios."
      : status === "not-started"
        ? `Comece pela etapa “${stageProgress[0].stage.title}”: resolva ${suggestedExercise!.title}.`
        : `Resolver: ${suggestedExercise!.title}`;

  return {
    status,
    completedStages,
    totalStages: stages.length,
    stages: stageProgress,
    suggestedExercise,
    message,
  };
}
