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
  logClientEvent,
} from "./runtime/tauri.js";

const PANEL_WINDOW_LABEL = "panel";
const SETTINGS_WINDOW_LABEL = "main";

const rootNode = document.getElementById("app");

if (!(rootNode instanceof HTMLElement)) {
  throw new Error("App root #app was not found");
}

const root = rootNode;

function logStartup(message) {
  if (!isTauriRuntime()) {
    return;
  }

  void logClientEvent(message).catch(() => {});
}

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
  logStartup("showBrowserFallback");
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
    logStartup(`syncThemeFromConfig: theme=${config.theme}`);
    applyTheme(config.theme);
  } catch {
    logStartup("syncThemeFromConfig: failed, defaulting to dark");
    applyTheme("dark");
  }
}

/**
 * @param {import("./panel.js").PanelData} data
 * @param {import("./panel.js").CreatePanelOptions=} options
 * @returns {void}
 */
function showPanel(data, options) {
  logStartup(`showPanel: app=${data.app_name} groups=${data.groups.length}`);
  unmountCurrentView();
  root.classList.add("panel-host");
  mountedView = createPanel(data, options);
  mountedView.mount(root);
  root.removeAttribute("data-loading");
}

function showSettings() {
  logStartup("showSettings");
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
  logStartup(`wireTauriPanelEvents:start window=${windowLabel}`);
  await syncThemeFromConfig();

  if (windowLabel === SETTINGS_WINDOW_LABEL) {
    logStartup("wireTauriPanelEvents: bootstrap settings window");
    showSettings();
  }

  await listen(APP_EVENTS.showPanel, (event) => {
    if (windowLabel !== PANEL_WINDOW_LABEL) {
      return;
    }
    logStartup("event: show-panel");
    void syncThemeFromConfig().finally(() => {
      showPanel(/** @type {import("./panel.js").PanelData} */ (event.payload));
    });
  });

  await listen(APP_EVENTS.showPanelPicker, (event) => {
    if (windowLabel !== PANEL_WINDOW_LABEL) {
      return;
    }
    logStartup("event: show-panel-picker");
    const apps = /** @type {import("./panel.js").PickerApp[]} */ (event.payload);
    void syncThemeFromConfig().finally(() => {
      showPanel(pickerAppsToPanelData(apps), { initialMode: "picker", pickerApps: apps });
    });
  });

  await listen(APP_EVENTS.hidePanel, () => {
    if (windowLabel !== PANEL_WINDOW_LABEL) {
      return;
    }
    logStartup("event: hide-panel");
    unmountCurrentView();
    root.classList.remove("panel-host");
    hidePanelWindow().catch(() => {});
  });

  await listen(APP_EVENTS.themeChanged, (event) => {
    logStartup(`event: theme-changed payload=${String(event.payload)}`);
    applyTheme(/** @type {string | undefined} */ (event.payload));
  });

  await listen(APP_EVENTS.openSettings, () => {
    if (windowLabel !== SETTINGS_WINDOW_LABEL) {
      return;
    }
    logStartup("event: open-settings");
    void syncThemeFromConfig().finally(() => {
      showSettings();
    });
  });

  if (windowLabel === PANEL_WINDOW_LABEL) {
    logStartup("wireTauriPanelEvents: emitting panel-ready");
    await emit(APP_EVENTS.panelReady, windowLabel);
  }

  logStartup(`wireTauriPanelEvents:ready window=${windowLabel}`);
}

if (isTauriRuntime()) {
  logStartup("main.js detected Tauri runtime");
  window.addEventListener("error", (event) => {
    const detail = event.error instanceof Error ? event.error.stack ?? event.error.message : event.message;
    logStartup(`window.error: ${detail}`);
  });
  window.addEventListener("unhandledrejection", (event) => {
    const detail = event.reason instanceof Error ? event.reason.stack ?? event.reason.message : String(event.reason);
    logStartup(`window.unhandledrejection: ${detail}`);
  });
  wireTauriPanelEvents().catch(() => {
    logStartup("wireTauriPanelEvents: failed, falling back");
    showBrowserFallback();
  });
} else {
  console.info("[justpeek] browser fallback runtime");
  showBrowserFallback();
}
