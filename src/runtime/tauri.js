// @ts-check

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

/**
 * @param {"east"|"west"|"north"|"south"|"northeast"|"northwest"|"southeast"|"southwest"} direction
 * @returns {Promise<void>}
 */
export async function startResizeDragging(direction) {
  const currentWindow = getCurrentWindow();
  if (!currentWindow?.startResizeDragging) {
    return;
  }

  await currentWindow.startResizeDragging(direction);
}
