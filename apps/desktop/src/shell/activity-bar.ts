export interface ActivityBarButton {
  dataset: DOMStringMap;
  onclick: GlobalEventHandlers["onclick"];
  setAttribute: (name: string, value: string) => void;
}

export interface ActivityBarDependencies {
  setInspectorView: (visible: boolean) => void;
  showPanel: (panel: "learning" | "debug") => void;
  activateHome: () => void;
  showUtilities: (visible: boolean) => void;
  openSettings: () => void;
  isLearningActive?: () => boolean;
}

/** Binds the activity rail to the current IDE views and keeps its pressed state in sync. */
export function mountActivityBar(
  buttons: Iterable<ActivityBarButton>,
  dependencies: ActivityBarDependencies,
) {
  const activities = [...buttons];

  function setPressed(activeActivity: string | null) {
    for (const activity of activities) {
      activity.setAttribute("aria-pressed", String(activeActivity !== null && activity.dataset.activity === activeActivity));
    }
  }

  for (const button of activities) {
    button.onclick = () => {
      if (button.dataset.activity === "learn" && dependencies.isLearningActive?.()) {
        setPressed(null);
        dependencies.showUtilities(false);
        return;
      }
      for (const activity of activities) {
        activity.setAttribute("aria-pressed", String(activity === button));
      }
      if (button.dataset.activity !== "learn") dependencies.setInspectorView(false);
      switch (button.dataset.activity) {
        case "learn":
          dependencies.showPanel("learning");
          break;
        case "examples":
          dependencies.activateHome();
          break;
        case "debug":
          dependencies.showUtilities(true);
          dependencies.showPanel("debug");
          break;
        case "settings":
          dependencies.openSettings();
          break;
      }
    };
  }

  return { setPressed };
}
