import test from "node:test";
import assert from "node:assert/strict";

import {
  APP_EVENTS,
  closeCurrentWindow,
  emitThemeChanged,
  getConfig,
  getCurrentWindowLabel,
  getPickerApps,
  hidePanelWindow,
  invokeAppCommand,
  isTauriRuntime,
  listen,
  loadPickerApp,
  openExternalUrl,
  openShortcutsDir,
  reloadShortcuts,
  setConfig,
} from "./tauri.js";

/**
 * @param {{
 *   invokeImpl?: (command: string, args?: Record<string, unknown>) => unknown | Promise<unknown>,
 *   emitImpl?: (event: string, payload?: unknown) => unknown | Promise<unknown>,
 *   listenImpl?: (event: string, handler: (payload: { payload: unknown }) => void) => unknown | Promise<unknown>,
 *   currentWindow?: Record<string, unknown> | null,
 * }} [options]
 * @returns {() => void}
 */
function installTauriStub(options = {}) {
  const originalWindow = globalThis.window;
  const currentWindow = options.currentWindow ?? null;

  globalThis.window = /** @type {typeof window} */ (/** @type {unknown} */ ({
    __TAURI__: {
      core: {
        invoke: options.invokeImpl ?? (async () => undefined),
      },
      event: {
        emit: options.emitImpl ?? (async () => undefined),
        listen: options.listenImpl ?? (async () => () => {}),
      },
      webviewWindow: currentWindow
        ? {
            getCurrentWebviewWindow() {
              return currentWindow;
            },
          }
        : undefined,
      window: currentWindow
        ? {
            getCurrentWindow() {
              return currentWindow;
            },
          }
        : undefined,
    },
  }));

  return () => {
    if (originalWindow === undefined) {
      // @ts-expect-error restoring test global
      delete globalThis.window;
      return;
    }

    globalThis.window = originalWindow;
  };
}

test("invokeAppCommand prefixes app commands with cmd_", async () => {
  /** @type {{ command?: string, args?: Record<string, unknown> }} */
  const call = {};
  const restore = installTauriStub({
    invokeImpl: async (command, args) => {
      call.command = command;
      call.args = args;
      return "ok";
    },
  });

  try {
    const result = await invokeAppCommand("get_config", { sample: true });

    assert.equal(result, "ok");
    assert.equal(call.command, "cmd_get_config");
    assert.deepEqual(call.args, { sample: true });
  } finally {
    restore();
  }
});

test("config helpers use the expected backend commands", async () => {
  /** @type {Array<{ command: string, args?: Record<string, unknown> }>} */
  const calls = [];
  const config = {
    hotkey: "CommandOrControl+Alt+Slash",
    theme: "light",
    preferred_shortcut_os: "auto",
    shortcut_display_mode: "current",
    references_dir: "/tmp/references",
  };

  const restore = installTauriStub({
    invokeImpl: async (command, args) => {
      calls.push({ command, args });
      if (command === "cmd_get_config") {
        return config;
      }
      return undefined;
    },
  });

  try {
    assert.deepEqual(await getConfig(), config);
    await setConfig(config);
  await reloadShortcuts();
  await openShortcutsDir();
  await openExternalUrl("https://example.com");
  await hidePanelWindow();
  await getPickerApps();
  await loadPickerApp("picker-1");

    assert.deepEqual(calls, [
      { command: "cmd_get_config", args: undefined },
    { command: "cmd_set_config", args: { configData: config } },
    { command: "cmd_reload_shortcuts", args: undefined },
    { command: "cmd_open_shortcuts_dir", args: undefined },
    { command: "cmd_open_external_url", args: { url: "https://example.com" } },
    { command: "cmd_hide_panel", args: undefined },
    { command: "cmd_get_picker_apps", args: undefined },
      { command: "cmd_load_picker_app", args: { pickerId: "picker-1", picker_id: "picker-1" } },
    ]);
  } finally {
    restore();
  }
});

test("emitThemeChanged emits the shared theme event", async () => {
  /** @type {{ event?: string, payload?: unknown }} */
  const call = {};
  const restore = installTauriStub({
    emitImpl: async (event, payload) => {
      call.event = event;
      call.payload = payload;
    },
  });

  try {
    await emitThemeChanged("light");

    assert.equal(call.event, APP_EVENTS.themeChanged);
    assert.equal(call.payload, "light");
  } finally {
    restore();
  }
});

test("listen delegates to the Tauri event API", async () => {
  /** @type {{ event?: string, handler?: (payload: { payload: unknown }) => void }} */
  const call = {};
  const unlisten = () => {};
  const restore = installTauriStub({
    listenImpl: async (event, handler) => {
      call.event = event;
      call.handler = handler;
      return unlisten;
    },
  });

  try {
    const handler = () => {};
    const result = await listen(APP_EVENTS.showPanel, handler);

    assert.equal(call.event, APP_EVENTS.showPanel);
    assert.equal(call.handler, handler);
    assert.equal(result, unlisten);
  } finally {
    restore();
  }
});

test("window helpers use the current tauri window when available", async () => {
  let closed = false;
  const currentWindow = {
    label: "panel",
    async close() {
      closed = true;
    },
  };

  const restore = installTauriStub({ currentWindow });

  try {
    assert.equal(isTauriRuntime(), true);
    assert.equal(getCurrentWindowLabel(), "panel");

    await closeCurrentWindow();

    assert.equal(closed, true);
  } finally {
    restore();
  }
});

test("runtime helpers fail safely when Tauri is unavailable", async () => {
  // @ts-expect-error removing test global
  delete globalThis.window;

  assert.equal(isTauriRuntime(), false);
  assert.equal(getCurrentWindowLabel(), null);
  await closeCurrentWindow();
  await assert.rejects(() => getConfig(), /Tauri core\.invoke is unavailable/);
});
