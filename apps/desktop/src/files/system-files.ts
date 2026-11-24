import type { ExerciseDocument } from "../exercises";

export interface IncomingFileDependencies {
  readExercise: (path: string) => Promise<ExerciseDocument>;
  openExercise: (exercise: ExerciseDocument) => void;
  openFile: (path: string) => Promise<void>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
}

/** Abre arquivos recebidos pelo sistema ou por arrastar e soltar. */
export function createIncomingFileOpener(dependencies: IncomingFileDependencies) {
  return async function openIncomingFiles(paths: string[], warnUnsupported = false) {
    let supported = false;
    for (const path of paths) {
      try {
        if (/\.pex$/i.test(path)) {
          supported = true;
          dependencies.openExercise(await dependencies.readExercise(path));
        } else if (/\.(por|pors|rs)$/i.test(path)) {
          supported = true;
          await dependencies.openFile(path);
        }
      } catch (error) {
        dependencies.setStatus(typeof error === "string" ? error : "Não foi possível abrir o arquivo.", "error");
      }
    }
    if (warnUnsupported && !supported && paths.length) {
      dependencies.setStatus("Escolha um arquivo .por, .pors, .rs ou .pex.", "error");
    }
  };
}
