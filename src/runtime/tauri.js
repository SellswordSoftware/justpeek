// @ts-check

export const APP_EVENTS = {
  showPanel: "show-panel",
  showPanelPicker: "show-panel-picker",
  hidePanel: "hide-panel",
  openSettings: "open-settings",
  panelReady: "justpeek://panel-ready",
  themeChanged: "justpeek://theme-changed",
};

/**
 * @returns {any | null}
 */
function getTauriGlobal() {
  return typeof window !== "undefined" && "__TAURI__" in window ? window.__TAURI__ : null;
}

/**
 * @returns {{ windowApi: any | null, webviewWindowApi: any | null } }
 */
function getTauriWindowApis() {
  const tauri = getTauriGlobal();
  return {
    windowApi: tauri?.window ?? null,
    webviewWindowApi: tauri?.webviewWindow ?? null,
  };
}

/**
 * @returns {any | null}
 */
function getCurrentWindow() {
  const { windowApi, webviewWindowApi } = getTauriWindowApis();

  try {
    if (webviewWindowApi?.getCurrentWebviewWindow) {
      return webviewWindowApi.getCurrentWebviewWindow();
    }
  } catch {
    // Fall through.
  }

  try {
    if (windowApi?.getCurrentWindow) {
      return windowApi.getCurrentWindow();
    }
  } catch {
    return null;
  }

  return null;
}

/**
 * @typedef {"East" | "North" | "NorthEast" | "NorthWest" | "South" | "SouthEast" | "SouthWest" | "West"} ResizeDirection
 */

/**
 * @param {string} command
 * @param {Record<string, unknown>=} args
 * @returns {Promise<unknown>}
 */
export function invoke(command, args) {
  const tauri = getTauriGlobal();
  if (!tauri?.core?.invoke) {
    return Promise.reject(new Error("Tauri core.invoke is unavailable"));
  }

  return tauri.core.invoke(command, args);
}

/**
 * Invoke a JustPeek backend command.
 * Tauri exposes these commands with their Rust function names, which currently
 * use a `cmd_` prefix.
 *
 * @param {string} command
 * @param {Record<string, unknown>=} args
 * @returns {Promise<unknown>}
 */
export function invokeAppCommand(command, args) {
  return invoke(`cmd_${command}`, args);
}

/**
 * @typedef {object} AppConfig
 * @property {string} hotkey
 * @property {string} theme
 * @property {string | null | undefined} references_dir
 */

/**
 * @typedef {object} RuntimeInfo
 * @property {string} os
 * @property {string | null | undefined} session_type
 * @property {boolean} hotkey_editable
 */

/**
 * @returns {Promise<AppConfig>}
 */
export async function getConfig() {
  return /** @type {Promise<AppConfig>} */ (invokeAppCommand("get_config"));
}

/**
 * @returns {Promise<RuntimeInfo>}
 */
export async function getRuntimeInfo() {
  return /** @type {Promise<RuntimeInfo>} */ (invokeAppCommand("get_runtime_info"));
}

/**
 * @param {AppConfig} configData
 * @returns {Promise<void>}
 */
export async function setConfig(configData) {
  await invokeAppCommand("set_config", { configData });
}

/**
 * @returns {Promise<void>}
 */
export async function hidePanelWindow() {
  await invokeAppCommand("hide_panel");
}

/**
 * @returns {Promise<void>}
 */
export async function reloadShortcuts() {
  await invokeAppCommand("reload_shortcuts");
}

/**
 * @returns {Promise<void>}
 */
export async function openSettingsWindow() {
  await invokeAppCommand("open_settings_window");
}

/**
 * @returns {Promise<void>}
 */
export async function openShortcutsDir() {
  await invokeAppCommand("open_shortcuts_dir");
}

/**
 * @param {string} url
 * @returns {Promise<void>}
 */
export async function openExternalUrl(url) {
  await invokeAppCommand("open_external_url", { url });
}

/**
 * @returns {Promise<unknown>}
 */
export async function getPickerApps() {
  return invokeAppCommand("get_picker_apps");
}

/**
 * @param {string} pickerId
 * @returns {Promise<unknown>}
 */
export async function loadPickerApp(pickerId) {
  return invokeAppCommand("load_picker_app", { pickerId, picker_id: pickerId });
}

/**
 * @param {string} event
 * @param {(payload: { payload: unknown }) => void} handler
 * @returns {Promise<() => void>}
 */
export function listen(event, handler) {
  const tauri = getTauriGlobal();
  if (!tauri?.event?.listen) {
    return Promise.reject(new Error("Tauri event.listen is unavailable"));
  }

  return tauri.event.listen(event, handler);
}

/**
 * @param {string} event
 * @param {unknown=} payload
 * @returns {Promise<void>}
 */
export function emit(event, payload) {
  const tauri = getTauriGlobal();
  if (!tauri?.event?.emit) {
    return Promise.reject(new Error("Tauri event.emit is unavailable"));
  }

  return tauri.event.emit(event, payload);
}

/**
 * @param {string} theme
 * @returns {Promise<void>}
 */
export async function emitThemeChanged(theme) {
  await emit(APP_EVENTS.themeChanged, theme);
}

/**
 * @returns {boolean}
 */
export function isTauriRuntime() {
  const tauri = getTauriGlobal();
  return Boolean(tauri?.core?.invoke && tauri?.event?.listen);
}

/**
 * @returns {Promise<void>}
 */
export async function closeCurrentWindow() {
  const currentWindow = getCurrentWindow();
  if (!currentWindow?.close) {
    return;
  }

  await currentWindow.close();
}

/**
 * @param {ResizeDirection} direction
 * @returns {Promise<void>}
 */
export async function startResizeDragging(direction) {
  const currentWindow = getCurrentWindow();
  if (!currentWindow?.startResizeDragging) {
    throw new Error("Current window startResizeDragging is unavailable");
  }

  await currentWindow.startResizeDragging(direction);
}

/**
 * @returns {string | null}
 */
export function getCurrentWindowLabel() {
  const currentWindow = getCurrentWindow();
  if (!currentWindow) {
    return null;
  }

  try {
    if (typeof currentWindow.label === "function") {
      return currentWindow.label();
    }
    if (typeof currentWindow.label === "string") {
      return currentWindow.label;
    }
  } catch {
    return null;
  }

  return null;
}
