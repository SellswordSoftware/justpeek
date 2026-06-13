// @ts-check

import { effect, listener, mount, requireRef, signal, template, text } from "./runtime/naf.js";
import {
  closeCurrentWindow,
  emitThemeChanged,
  getConfig,
  openShortcutsDir,
  reloadShortcuts,
  setConfig,
} from "./runtime/tauri.js";

/**
 * @returns {{ mount: (container: HTMLElement) => void, unmount: () => void }}
 */
export function createSettings() {
  const hotkey = signal("");
  const theme = signal("dark");
  const referencesDir = signal("");
  const status = signal("Loading configuration...");

  getConfig()
    .then((cfg) => {
      hotkey(cfg.hotkey);
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
    } catch {
      status("Failed to save configuration.");
    }
  }

  /** @type {import("./runtime/naf.js").TemplateOptions<HTMLElement>} */
  const settingsOptions = {
    root: ".peek-surface",
    onMount(_el, _parent, ctx) {
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
          hotkeyInput.value = hotkey();
          themeSelect.value = theme();
          referencesDirInput.value = referencesDir();
          statusNode.textContent = status();
        }),
      );
    },
  };

  const settings = template(settingsOptions)`
    <section class="peek-surface panel settings-view">
      <header class="peek-header settings-view__header">
        <div class="peek-header__meta">
          <p class="eyebrow">JustPeek</p>
          <h1 class="peek-title">${text("Settings")}</h1>
        </div>
        <button
          type="button"
          class="peek-close"
          data-ref="closeButton"
          aria-label="Close settings"
        >
          ×
        </button>
      </header>

      <section class="settings-form">
        <label class="settings-field">
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
