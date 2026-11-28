// Painel Aprender: conceito explicado conforme o modo de aprendizagem, a
// seleção no editor (por mapeamento exato da tradução) ou a escolha manual,
// com tópico fixado por aba e atalho para o exercício de prática.

import type { EditorView } from "@codemirror/view";
import { $ } from "../dom";
import { createExerciseProgress } from "../exercise-progress";
import { conceptsForMode, resolveConceptForMode } from "../learning-content";
import { explainMappedConcept, learningContextAtSelection } from "../learning-context";
import type { LearningMode } from "../learning-mode";
import { learningPinForMode, setLearningPin, toggleLearningPin } from "../learning-pins";
import { LEARNING_TRAIL, learningTrailProgress } from "./trail";
import type { Tab } from "../state/tab";
import { mountOwnershipLab, type OwnershipLabHost } from "./ownership-view";

export interface LearningPanelHost {
  sourceView: EditorView;
  activeTab(): Tab | undefined;
  learningMode(): LearningMode;
  /** Abre exercício Ferro pelo identificador do catálogo .fer. */
  openFerroExercise(id: string): Promise<void>;
  /** Abre um exemplo .fer do catálogo pelo nome de arquivo (ex.: `01-ola-variaveis.fer`). */
  openFerroExample?(fileName: string): Promise<void>;
  /** Abre o Laboratório de Ownership em modo expandido no painel. */
  openOwnershipLab?(): void;
  ownership?: OwnershipLabHost;
}

export function mountLearningPanel(host: LearningPanelHost) {
  const ownership = mountOwnershipLab(host.ownership);
  const exerciseProgress = createExerciseProgress();
  let selectedLearningConceptId = "variables";
  let learningConceptManuallySelected = false;
  let learningPracticeBusy = false;

  async function openTrailExample(fileName: string) {
    const message = $("learning-trail-message");
    try {
      await host.openFerroExample?.(fileName);
      message.textContent = `Exemplo ${fileName} aberto em uma nova aba.`;
    } catch {
      message.textContent = `Não foi possível abrir o exemplo ${fileName}.`;
    }
  }

  async function openTrailExercise(exerciseId: string) {
    if (learningPracticeBusy) return;
    learningPracticeBusy = true;
    const message = $("learning-trail-message");
    message.textContent = "Abrindo exercício…";
    try {
      await host.openFerroExercise(exerciseId);
    } catch {
      message.textContent = "Não foi possível abrir o exercício. Tente novamente pela lista de exercícios.";
    } finally {
      learningPracticeBusy = false;
      renderLearningPanel();
    }
  }

  function renderLearningPanel() {
    const learningMode = host.learningMode();
    const concepts = conceptsForMode(learningMode);
    const tab = host.activeTab();
    const pinnedConceptId = learningPinForMode(tab?.learningPins, learningMode, concepts.map(({ id }) => id));
    const source = host.sourceView.state.doc.toString();
    const context = !pinnedConceptId && learningMode === "bridge" && !learningConceptManuallySelected && tab?.language === "ferro-rust"
      ? learningContextAtSelection({
        source,
        selectionFrom: host.sourceView.state.selection.main.from,
        selectionTo: host.sourceView.state.selection.main.to,
        artifact: tab?.rustPreview?.artifact,
        stale: Boolean(tab?.rustPreview?.stale || tab?.rustPreview?.pending),
      })
      : null;
    const concept = resolveConceptForMode(learningMode, pinnedConceptId ?? context?.conceptId ?? selectedLearningConceptId);

    const trail = learningTrailProgress(LEARNING_TRAIL, exerciseProgress.hasPassed);
    $("learning-trail-count").textContent = trail.status === "empty"
      ? "Sem exercícios"
      : `${trail.completedStages} de ${trail.totalStages} etapas concluídas`;
    $("learning-trail-message").textContent = trail.message;
    const trailStages = $<HTMLOListElement>("learning-trail-stages");
    const currentStageIndex = trail.stages.findIndex(({ completed }) => !completed);
    trailStages.replaceChildren(...trail.stages.map(({ stage, completed, completedExercises }, index) => {
      const item = document.createElement("li");
      item.className = completed ? "is-complete" : index === currentStageIndex ? "is-current" : "";
      if (index === currentStageIndex) item.setAttribute("aria-current", "step");
      const heading = document.createElement("div");
      heading.className = "learning-trail-stage-heading";
      const title = document.createElement("strong");
      title.textContent = stage.title;
      const status = document.createElement("span");
      status.textContent = completed
        ? "Concluída"
        : index === currentStageIndex || completedExercises > 0
          ? `${completedExercises}/${stage.exercises.length} exercícios`
          : "Não iniciada";
      heading.append(title, status);
      const description = document.createElement("p");
      description.textContent = stage.description;
      const examples = document.createElement("div");
      examples.className = "learning-trail-links";
      const examplesLabel = document.createElement("small");
      examplesLabel.textContent = "Exemplos:";
      examples.append(examplesLabel);
      for (const path of stage.examplePaths) {
        const link = document.createElement("button");
        link.type = "button";
        link.className = "learning-trail-link";
        link.textContent = path;
        link.title = `Abrir o exemplo ${path} em uma nova aba`;
        link.dataset.examplePath = path;
        link.onclick = () => void openTrailExample(path);
        examples.append(link);
      }
      const exercises = document.createElement("div");
      exercises.className = "learning-trail-links";
      const exercisesLabel = document.createElement("small");
      exercisesLabel.textContent = "Exercícios:";
      exercises.append(exercisesLabel);
      for (const exercise of stage.exercises) {
        const link = document.createElement("button");
        link.type = "button";
        link.className = "learning-trail-link";
        const done = exerciseProgress.hasPassed(`ferro:${exercise.id}`);
        link.textContent = done ? `✓ ${exercise.title}` : exercise.title;
        link.title = `Abrir o exercício “${exercise.title}”`;
        link.dataset.exerciseId = exercise.id;
        link.disabled = learningPracticeBusy;
        link.onclick = () => void openTrailExercise(exercise.id);
        exercises.append(link);
      }
      item.append(heading, description, examples, exercises);
      if (stage.id === "ownership" && host.openOwnershipLab) {
        const lab = document.createElement("button");
        lab.type = "button";
        lab.className = "learning-trail-link learning-trail-lab";
        lab.textContent = "Abrir Laboratório de Ownership";
        lab.onclick = () => host.openOwnershipLab?.();
        item.append(lab);
      }
      return item;
    }));
    const nextButton = $<HTMLButtonElement>("learning-trail-next");
    nextButton.disabled = !trail.suggestedExercise || learningPracticeBusy;
    nextButton.textContent = trail.suggestedExercise
      ? `Resolver: ${trail.suggestedExercise.title}`
      : trail.status === "completed" ? "Trilha concluída" : "Sem exercícios disponíveis";
    nextButton.dataset.exerciseId = trail.suggestedExercise ? String(trail.suggestedExercise.id) : "";

    const picker = $<HTMLSelectElement>("learning-concept");
    picker.replaceChildren(...concepts.map((item) => {
      const option = document.createElement("option");
      option.value = item.id;
      option.textContent = item.title;
      return option;
    }));
    picker.value = concept.id;
    const pinButton = $<HTMLButtonElement>("learning-pin");
    const topicPinned = pinnedConceptId === concept.id;
    pinButton.disabled = !tab;
    pinButton.dataset.conceptId = concept.id;
    pinButton.setAttribute("aria-pressed", String(topicPinned));
    pinButton.textContent = topicPinned ? "Desafixar tópico" : "Fixar tópico";
    const practiceButton = $<HTMLButtonElement>("learning-practice-open");
    practiceButton.hidden = concept.practiceFerroExerciseId === undefined;
    practiceButton.disabled = !tab || learningPracticeBusy;
    practiceButton.dataset.exerciseId = concept.practiceFerroExerciseId ?? "";
    const active = host.activeTab();
    $("learning-mode-description").textContent = learningMode === "bridge"
      ? "Compare os conceitos do pseudocódigo em português com exemplos Rust."
      : learningMode === "rust"
        ? active?.language === "rust"
          ? "Nível Rust: o arquivo .rs ativo é editado e executado com a toolchain Rust instalada."
          : "Nível Rust: abra ou crie um arquivo .rs para programar em Rust. O arquivo atual permanece intacto."
        : "Aprenda a linguagem Ferro, pratique com exercícios .fer e acompanhe cada exemplo no editor. Rust aparece como próximo passo, no seu ritmo.";
    $("learning-title").textContent = concept.title;
    $("learning-provenance").textContent = topicPinned
      ? "Tópico fixado — escolha manual nesta aba"
      : context
        ? "Relacionado à seleção por mapeamento exato da tradução atual"
        : "Conceito selecionado manualmente";
    $("learning-definition").textContent = context
      ? `${concept.definition} ${explainMappedConcept(context, "ferro-rust")}`
      : concept.definition;
    $("learning-what-changed").textContent = concept.whatChanged;
    $("learning-why-it-matters").textContent = concept.whyItMatters;
    $("learning-practice").textContent = concept.practice;
    const rustMode = learningMode === "rust";
    $("learning-ferro-example").hidden = rustMode;
    if (!rustMode) {
      $("learning-ferro-caption").textContent = context ? "Trecho selecionado na fonte" : concept.ferroExample.caption;
      $("learning-ferro-code").textContent = context ? context.sourceSnippet : concept.ferroExample.code;
    }

    const rustSection = $<HTMLElement>("learning-rust-example");
    const showRustExample = rustMode || (learningMode === "bridge" && (context !== null || concept.rustExample !== undefined));
    rustSection.hidden = !showRustExample;
    if (showRustExample) {
      $("learning-rust-heading").textContent = rustMode ? "Rust — exemplo" : "Rust — comparação didática";
      $("learning-rust-caption").textContent = context ? "Rust gerado para este trecho" : concept.rustExample!.caption;
      $("learning-rust-code").textContent = context ? context.rustSnippet : concept.rustExample!.code;
    }

    const ownershipLab = $<HTMLElement>("ownership-lab");
    const showOwnershipLab = learningMode !== "beginner" && concept.id === "ownership";
    ownershipLab.hidden = !showOwnershipLab;
    if (showOwnershipLab) ownership.render();
  }

  $<HTMLSelectElement>("learning-concept").onchange = (event) => {
    const learningMode = host.learningMode();
    selectedLearningConceptId = (event.currentTarget as HTMLSelectElement).value;
    const tab = host.activeTab();
    const availableConceptIds = conceptsForMode(learningMode).map(({ id }) => id);
    if (tab && learningPinForMode(tab.learningPins, learningMode, availableConceptIds)) {
      tab.learningPins = setLearningPin(tab.learningPins, learningMode, selectedLearningConceptId);
    }
    learningConceptManuallySelected = true;
    renderLearningPanel();
  };
  $<HTMLButtonElement>("learning-pin").onclick = (event) => {
    const tab = host.activeTab();
    if (!tab) return;
    const learningMode = host.learningMode();
    const button = event.currentTarget as HTMLButtonElement;
    tab.learningPins = toggleLearningPin(tab.learningPins, learningMode, button.dataset.conceptId ?? selectedLearningConceptId);
    learningConceptManuallySelected = false;
    renderLearningPanel();
  };
  $<HTMLButtonElement>("learning-practice-open").onclick = async (event) => {
    if (learningPracticeBusy) return;
    const button = event.currentTarget as HTMLButtonElement;
    const exerciseId = button.dataset.exerciseId;
    if (!exerciseId || !host.activeTab()) return;

    learningPracticeBusy = true;
    button.disabled = true;
    const status = $<HTMLElement>("learning-practice-status");
      status.textContent = "Abrindo exercício…";
    try {
      await host.openFerroExercise(exerciseId);
      status.textContent = "Exercício aberto no editor. A abertura não registra progresso.";
    } catch {
      status.textContent = "Não foi possível abrir o exercício. Tente novamente ou use a lista de exercícios.";
    } finally {
      learningPracticeBusy = false;
      renderLearningPanel();
    }
  };

  $<HTMLButtonElement>("learning-trail-next").onclick = async (event) => {
    if (learningPracticeBusy) return;
    const button = event.currentTarget as HTMLButtonElement;
    const exerciseId = button.dataset.exerciseId;
    if (!exerciseId) return;

    learningPracticeBusy = true;
    button.disabled = true;
    $("learning-trail-message").textContent = `Abrindo ${button.textContent}…`;
    try {
      await host.openFerroExercise(exerciseId);
    } catch {
      $("learning-trail-message").textContent = "Não foi possível abrir o próximo exercício. Tente novamente pela lista de exercícios.";
    } finally {
      learningPracticeBusy = false;
      renderLearningPanel();
    }
  };

  exerciseProgress.subscribe(renderLearningPanel);

  return {
    render: renderLearningPanel,
    /** Seleciona explicitamente um conceito (ex.: ao abrir o Laboratório de Ownership pela Ajuda). */
    selectConcept(id: string) {
      selectedLearningConceptId = id;
      learningConceptManuallySelected = true;
      renderLearningPanel();
    },
    /** A seleção mudou no editor: o contexto detectado volta a valer. */
    selectionChanged() {
      learningConceptManuallySelected = false;
    },
  };
}
