import { createPanel, pickerAppsToPanelData } from "./panel.js";
import { createSettings } from "./settings.js";
import {
  APP_EVENTS,
  emit,
  getCurrentWindowLabel,
  getConfig,
  hidePanelWindow,
  isTauriRuntime,
  listen,
} from "./runtime/tauri.js";

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

function showBrowserFallback() {
  unmountCurrentView();
  root.classList.remove("panel-host");

  const section = document.createElement("section");
  section.className = "peek-surface panel app-fallback";
  section.innerHTML = `
    <header class="peek-header">
      <div class="peek-header__meta">
        <p class="eyebrow">JustPeek</p>
        <h1 class="peek-title">Tauri runtime required</h1>
      </div>
    </header>
    <section class="app-fallback__body">
      <p class="app-fallback__text">
        This build is meant to run inside the desktop app.
      </p>
      <p class="app-fallback__text">
        Start it with <code>npm run dev</code> or open the packaged application.
      </p>
    </section>
  `;

  root.append(section);
  root.removeAttribute("data-loading");
}

/**
 * @param {string | undefined} theme
 * @returns {void}
 */
function applyTheme(theme) {
  document.documentElement.setAttribute("data-theme", theme === "light" ? "light" : "dark");
}

/**
 * @returns {Promise<void>}
 */
async function syncThemeFromConfig() {
  try {
    const config = await getConfig();
    applyTheme(config.theme);
  } catch {
    applyTheme("dark");
  }
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
 * @returns {Promise<void>}
 */
async function wireTauriPanelEvents() {
  const windowLabel = getCurrentWindowLabel();
  await syncThemeFromConfig();

  if (windowLabel === SETTINGS_WINDOW_LABEL) {
    showSettings();
  }

  await listen(APP_EVENTS.showPanel, (event) => {
    if (windowLabel !== PANEL_WINDOW_LABEL) {
      return;
    }
    void syncThemeFromConfig().finally(() => {
      showPanel(/** @type {import("./panel.js").PanelData} */ (event.payload));
    });
  });

  await listen(APP_EVENTS.showPanelPicker, (event) => {
    if (windowLabel !== PANEL_WINDOW_LABEL) {
      return;
    }
    const apps = /** @type {import("./panel.js").PickerApp[]} */ (event.payload);
    void syncThemeFromConfig().finally(() => {
      showPanel(pickerAppsToPanelData(apps), { initialMode: "picker", pickerApps: apps });
    });
  });

  await listen(APP_EVENTS.hidePanel, () => {
    if (windowLabel !== PANEL_WINDOW_LABEL) {
      return;
    }
    unmountCurrentView();
    root.classList.remove("panel-host");
    hidePanelWindow().catch(() => {});
  });

  await listen(APP_EVENTS.themeChanged, (event) => {
    applyTheme(/** @type {string | undefined} */ (event.payload));
  });

  await listen(APP_EVENTS.openSettings, () => {
    if (windowLabel !== SETTINGS_WINDOW_LABEL) {
      return;
    }
    void syncThemeFromConfig().finally(() => {
      showSettings();
    });
  });

  if (windowLabel === PANEL_WINDOW_LABEL) {
    await emit(APP_EVENTS.panelReady, windowLabel);
  }
}

if (isTauriRuntime()) {
  wireTauriPanelEvents().catch(() => {
    showBrowserFallback();
  });
} else {
  showBrowserFallback();
}
