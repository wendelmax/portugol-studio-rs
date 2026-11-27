import { describeExtensions, fileKind, isProgramKind, isRetiredFile } from "../file-formats.ts";
import type { ExerciseDocument } from "../exercises";

export interface IncomingFileDependencies {
  readExercise: (path: string) => Promise<ExerciseDocument>;
  openExercise: (exercise: ExerciseDocument) => void;
  openFile: (path: string) => Promise<void>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
}

const OPENABLE = describeExtensions(["source", "rust", "lesson"], "ou");

/** Abre arquivos recebidos pelo sistema ou por arrastar e soltar. */
export function createIncomingFileOpener(dependencies: IncomingFileDependencies) {
  return async function openIncomingFiles(paths: string[], warnUnsupported = false) {
    let supported = false;
    let sawLegacy = false;
    for (const path of paths) {
      try {
        const kind = fileKind(path);
        if (kind === "lesson") {
          supported = true;
          dependencies.openExercise(await dependencies.readExercise(path));
        } else if (isProgramKind(kind)) {
          supported = true;
          await dependencies.openFile(path);
        } else if (isRetiredFile(path)) {
          sawLegacy = true;
        }
      } catch (error) {
        dependencies.setStatus(typeof error === "string" ? error : "Não foi possível abrir o arquivo.", "error");
      }
    }
    if (warnUnsupported && !supported && paths.length) {
      dependencies.setStatus(
        sawLegacy
          ? `Arquivos legados .por e .pex não são suportados no Portugol Studio RS. Use o Portugol Studio original para abri-los, ou escolha um arquivo ${OPENABLE}.`
          : `Escolha um arquivo ${OPENABLE}.`,
        "error",
      );
    }
  };
}
