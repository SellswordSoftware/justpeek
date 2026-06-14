// @ts-check

import { effect, listener, mount, requireRef, signal, template, text } from "./runtime/naf.js";
import {
  closeCurrentWindow,
  emitThemeChanged,
  getConfig,
  getRuntimeInfo,
  openShortcutsDir,
  reloadShortcuts,
  setConfig,
  startResizeDragging,
} from "./runtime/tauri.js";

/**
 * @returns {{ mount: (container: HTMLElement) => void, unmount: () => void }}
 */
export function createSettings() {
  const hotkey = signal("");
  const hotkeyEditable = signal(true);
  const hotkeyNotice = signal("");
  const theme = signal("dark");
  const referencesDir = signal("");
  const status = signal("Loading configuration...");

  Promise.all([getConfig(), getRuntimeInfo()])
    .then(([cfg, runtimeInfo]) => {
      hotkey(cfg.hotkey);
      hotkeyEditable(runtimeInfo.hotkey_editable);
      hotkeyNotice(
        runtimeInfo.hotkey_editable
          ? ""
          : "On Linux Wayland, the global shortcut is managed by the desktop environment.",
      );
      theme(cfg.theme);
      referencesDir(cfg.references_dir ?? "");
      document.documentElement.setAttribute("data-theme", cfg.theme);
      status("Hotkey changes apply after restarting the app.");
    })
    .catch(() => {
      status("Failed to load configuration.");
    });

  async function saveSettings() {
    status("Saving...");

    try {
      await setConfig({
        hotkey: hotkey().trim() || "CommandOrControl+Alt+Slash",
        theme: theme(),
        references_dir: referencesDir().trim() || null,
      });
      document.documentElement.setAttribute("data-theme", theme());
      await emitThemeChanged(theme());
      status("Saved. Hotkey changes apply after restarting the app.");
    } catch (error) {
      status(
        `Failed to save configuration: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  /** @type {import("./runtime/naf.js").TemplateOptions<HTMLElement>} */
  const settingsOptions = {
    root: ".peek-surface",
    onMount(_el, _parent, ctx) {
      const resizeHandles = requireRef(ctx.refs, "resizeHandles");
      const hotkeyField = /** @type {HTMLElement} */ (requireRef(ctx.refs, "hotkeyField"));
      const hotkeyNoticeField = /** @type {HTMLElement} */ (requireRef(ctx.refs, "hotkeyNoticeField"));
      const hotkeyNoticeNode = /** @type {HTMLElement} */ (requireRef(ctx.refs, "hotkeyNotice"));
      const hotkeyInput = /** @type {HTMLInputElement} */ (requireRef(ctx.refs, "hotkeyInput"));
      const themeSelect = /** @type {HTMLSelectElement} */ (requireRef(ctx.refs, "themeSelect"));
      const referencesDirInput = /** @type {HTMLInputElement} */ (
        requireRef(ctx.refs, "referencesDirInput")
      );
      const saveButton = /** @type {HTMLButtonElement} */ (requireRef(ctx.refs, "saveButton"));
      const closeButton = /** @type {HTMLButtonElement} */ (requireRef(ctx.refs, "closeButton"));
      const openDirButton = /** @type {HTMLButtonElement} */ (
        requireRef(ctx.refs, "openDirButton")
      );
      const reloadButton = /** @type {HTMLButtonElement} */ (requireRef(ctx.refs, "reloadButton"));
      const statusNode = /** @type {HTMLElement} */ (requireRef(ctx.refs, "statusNode"));

      ctx.cleanup.add(
        listener(hotkeyInput, "input", (event) => {
          hotkey((/** @type {HTMLInputElement} */ (event.currentTarget)).value);
        }),
        listener(themeSelect, "change", (event) => {
          theme((/** @type {HTMLSelectElement} */ (event.currentTarget)).value);
        }),
        listener(referencesDirInput, "input", (event) => {
          referencesDir((/** @type {HTMLInputElement} */ (event.currentTarget)).value);
        }),
        listener(saveButton, "click", () => {
          void saveSettings();
        }),
        listener(closeButton, "click", () => {
          void closeCurrentWindow();
        }),
        listener(resizeHandles, "mousedown", (event) => {
          if (!(event.target instanceof HTMLElement) || event.button !== 0) {
            return;
          }

          const handle = event.target.closest("[data-resize-direction]");
          if (!(handle instanceof HTMLElement)) {
            return;
          }

          const direction = handle.getAttribute("data-resize-direction");
          if (!direction) {
            return;
          }

          event.preventDefault();
          event.stopPropagation();
          startResizeDragging(
            /** @type {import("./runtime/tauri.js").ResizeDirection} */ (direction),
          ).catch(() => {});
        }),
        listener(openDirButton, "click", () => {
          void openShortcutsDir();
        }),
        listener(reloadButton, "click", () => {
          void reloadShortcuts()
            .then(() => {
              status("References reloaded.");
            })
            .catch(() => {
              status("Failed to reload references.");
            });
        }),
        effect(() => {
          hotkeyField.hidden = !hotkeyEditable();
          hotkeyNoticeField.hidden = hotkeyEditable();
          hotkeyNoticeNode.textContent = hotkeyNotice();
          hotkeyInput.value = hotkey();
          themeSelect.value = theme();
          referencesDirInput.value = referencesDir();
          statusNode.textContent = status();
        }),
      );
    },
  };

  const settings = template(settingsOptions) /*html*/`
    <section class="peek-surface panel settings-view">
      <div class="window-resize-handles" data-ref="resizeHandles" aria-hidden="true">
        <div class="window-resize-handle window-resize-handle--n" data-resize-direction="North"></div>
        <div class="window-resize-handle window-resize-handle--e" data-resize-direction="East"></div>
        <div class="window-resize-handle window-resize-handle--s" data-resize-direction="South"></div>
        <div class="window-resize-handle window-resize-handle--w" data-resize-direction="West"></div>
        <div class="window-resize-handle window-resize-handle--ne" data-resize-direction="NorthEast"></div>
        <div class="window-resize-handle window-resize-handle--nw" data-resize-direction="NorthWest"></div>
        <div class="window-resize-handle window-resize-handle--se" data-resize-direction="SouthEast"></div>
        <div class="window-resize-handle window-resize-handle--sw" data-resize-direction="SouthWest"></div>
      </div>
      <div class="peek-window-actions">
        <button
          type="button"
          class="peek-window-action peek-window-action--close"
          data-ref="closeButton"
          aria-label="Close settings"
          title="Close settings"
        >
          <span class="icon-mask peek-window-action__icon peek-window-action__icon--close" aria-hidden="true"></span>
        </button>
      </div>
      <header class="peek-header settings-view__header" data-tauri-drag-region>
        <div class="peek-header__meta" data-tauri-drag-region>
          <p class="eyebrow no-click">JustPeek</p>
          <h1 class="peek-title no-click">${text("Settings")}</h1>
        </div>
      </header>

      <section class="settings-form">
        <label class="settings-field" data-ref="hotkeyField">
          <span class="settings-field__label">Hotkey</span>
          <input
            class="input"
            data-ref="hotkeyInput"
            type="text"
            placeholder="CommandOrControl+Alt+Slash"
            autocomplete="off"
          />
          <span class="settings-field__hint">Use Tauri shortcut syntax.</span>
        </label>
        <div class="settings-field" data-ref="hotkeyNoticeField" hidden>
          <span class="settings-field__hint" data-ref="hotkeyNotice"></span>
        </div>

        <label class="settings-field">
          <span class="settings-field__label">Theme</span>
          <select class="input" data-ref="themeSelect">
            <option value="dark">Dark</option>
            <option value="light">Light</option>
          </select>
        </label>

        <label class="settings-field">
          <span class="settings-field__label">References Directory</span>
          <input
            class="input"
            data-ref="referencesDirInput"
            type="text"
            placeholder="Use the default JustPeek references directory"
            autocomplete="off"
          />
          <span class="settings-field__hint">
            Leave blank to use the default config directory.
          </span>
        </label>
      </section>

      <footer class="settings-actions">
        <p class="settings-status" data-ref="statusNode">${text("Loading configuration...")}</p>
        <div class="settings-actions__buttons">
          <button type="button" class="peek-action peek-action--ghost" data-ref="openDirButton">
            Open References
          </button>
          <button type="button" class="peek-action peek-action--ghost" data-ref="reloadButton">
            Reload
          </button>
          <button type="button" class="peek-action" data-ref="saveButton">
            Save
          </button>
        </div>
      </footer>
    </section>
  `;

  return {
    mount(container) {
      mount(settings, container);
    },
    unmount() {
      settings.unmount?.();
    },
  };
}
