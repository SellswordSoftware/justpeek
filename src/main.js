import { createApp } from "./app/create-app.js";
import { createPanel } from "./panel.js";
import { createSettings } from "./settings.js";
import { emit, getCurrentWindowLabel, invoke, isTauriRuntime, listen } from "./runtime/tauri.js";

const PANEL_WINDOW_LABEL = "panel";
const SETTINGS_WINDOW_LABEL = "main";

const rootNode = document.getElementById("app");

if (!(rootNode instanceof HTMLElement)) {
  throw new Error("App root #app was not found");
}

const root = rootNode;

/**
 * @typedef {object} MountedView
 * @property {(container: HTMLElement) => void} mount
 * @property {() => void} unmount
 */

/** @type {MountedView | null} */
let mountedView = null;

function unmountCurrentView() {
  mountedView?.unmount();
  mountedView = null;
  root.replaceChildren();
}

/**
 * @param {import("./panel.js").PanelData} data
 * @param {import("./panel.js").CreatePanelOptions=} options
 * @returns {void}
 */
function showPanel(data, options) {
  unmountCurrentView();
  root.classList.add("panel-host");
  mountedView = createPanel(data, options);
  mountedView.mount(root);
  root.removeAttribute("data-loading");
}

function showSettings() {
  unmountCurrentView();
  root.classList.remove("panel-host");
  mountedView = createSettings();
  mountedView.mount(root);
  root.removeAttribute("data-loading");
}

/**
 * @param {import("./panel.js").PickerApp[]} apps
 * @returns {import("./panel.js").PanelData}
 */
function pickerAppsToPanelData(apps) {
  return {
    app_name: "Available References",
    groups: [
      {
        group: "Reference Files",
        items: apps.map((app) => ({
          label: app.name,
          value: app.processes.join(", "),
          notes: "No contextual match was detected for the active window.",
        })),
      },
    ],
  };
}

/**
 * @returns {Promise<void>}
 */
async function wireTauriPanelEvents() {
  const windowLabel = getCurrentWindowLabel();

  await listen("show-panel", (event) => {
    if (windowLabel !== PANEL_WINDOW_LABEL) {
      return;
    }
    showPanel(/** @type {import("./panel.js").PanelData} */ (event.payload));
  });

  await listen("show-panel-picker", (event) => {
    if (windowLabel !== PANEL_WINDOW_LABEL) {
      return;
    }
    const apps = /** @type {import("./panel.js").PickerApp[]} */ (event.payload);
    showPanel(pickerAppsToPanelData(apps), { initialMode: "picker", pickerApps: apps });
  });

  await listen("hide-panel", () => {
    if (windowLabel !== PANEL_WINDOW_LABEL) {
      return;
    }
    unmountCurrentView();
    root.classList.remove("panel-host");
    invoke("hide_panel").catch(() => {});
  });

  await listen("open-settings", () => {
    if (windowLabel !== SETTINGS_WINDOW_LABEL) {
      return;
    }
    showSettings();
  });

  if (windowLabel === PANEL_WINDOW_LABEL) {
    await emit("justpeek://panel-ready", windowLabel);
  }
}

if (isTauriRuntime()) {
  wireTauriPanelEvents().catch(() => {
    root.classList.remove("panel-host");
    createApp(root);
  });
} else {
  root.classList.remove("panel-host");
  createApp(root);
}
