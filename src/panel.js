// @ts-check

import {
  computed,
  effect,
  listener,
  mount,
  requireRef,
  signal,
  template,
  text,
} from "./runtime/naf.js";
import { fuzzyFilter } from "./runtime/fuzzy.js";
import {
  closeCurrentWindow,
  getPickerApps,
  hidePanelWindow,
  loadPickerApp as loadPickerReference,
  openExternalUrl,
} from "./runtime/tauri.js";

/**
 * @typedef {object} PanelItem
 * @property {string[]} keys
 * @property {string} label
 * @property {string=} value
 * @property {string=} notes
 * @property {string=} url
 * @property {string[]=} search_terms
 */

/**
 * @typedef {object} PanelGroup
 * @property {string} group
 * @property {PanelItem[]} items
 */

/**
 * @typedef {object} PanelData
 * @property {string} app_name
 * @property {PanelGroup[]} groups
 */

/**
 * @typedef {object} PickerApp
 * @property {string} id
 * @property {string} name
 * @property {string[]} processes
 */

/**
 * @typedef {object} CreatePanelOptions
 * @property {"contextual"|"picker"} [initialMode]
 * @property {PickerApp[]} [pickerApps]
 */

/**
 * @param {string[]} keys
 * @returns {string}
 */
function renderKeys(keys) {
  return keys
    .map((variant) =>
      variant
        .split("+")
        .map((part) => `<kbd>${text(part.trim())}</kbd>`)
        .join('<span aria-hidden="true">+</span>'),
    )
    .join('<span class="peek-keys__or" aria-hidden="true">/</span>');
}

/**
 * @param {string} value
 * @returns {boolean}
 */
function isCommandLikeValue(value) {
  return /(^|[\s])(git|npm|pnpm|yarn|npx|cargo|docker|kubectl|ssh|curl)\b|--|[<>]/i.test(value);
}

/**
 * @param {PanelItem} item
 * @returns {string}
 */
function getItemKind(item) {
  if (item.keys.length > 0) {
    return "shortcut";
  }

  if (item.value && isCommandLikeValue(item.value)) {
    return "command";
  }

  return "detail";
}

/**
 * @param {PanelItem} item
 * @returns {string}
 */
function renderBody(item) {
  const value = item.value?.trim();
  const itemKind = getItemKind(item);
  const valueMarkup = value
    ? isCommandLikeValue(value)
      ? `<code class="peek-item__value peek-item__value--command">${text(value)}</code>`
      : `<p class="peek-item__value">${text(value)}</p>`
    : "";
  const notesMarkup = item.notes ? `<p class="peek-item__notes">${text(item.notes)}</p>` : "";
  const actionMarkup = item.url
    && itemKind === "command"
    ? `
        <button
          type="button"
          class="peek-item__action"
          data-item-url="${text(item.url)}"
          aria-label="Open link"
          title="Open link"
        >
          <span class="peek-item__action-label">Open</span>
          <span class="icon-mask peek-item__action-icon" aria-hidden="true"></span>
        </button>
      `
    : "";

  if (itemKind === "command") {
    return `${valueMarkup}${notesMarkup}${actionMarkup}`;
  }

  return `
    <div class="peek-item__meta">
      ${valueMarkup}
      ${notesMarkup}
      ${actionMarkup}
    </div>
  `;
}

/**
 * @param {PickerApp[]} apps
 * @returns {PanelData}
 */
export function pickerAppsToPanelData(apps) {
  return {
    app_name: "Available References",
    groups: [
      {
        group: "Reference Files",
        items: apps.map((app) => ({
          keys: [],
          label: app.name,
          value: app.processes.join(", "),
          notes: "Select a reference file to load it.",
        })),
      },
    ],
  };
}

/**
 * @param {PanelData} data
 * @param {CreatePanelOptions=} options
 * @returns {{ mount: (container: HTMLElement) => void, unmount: () => void }}
 */
export function createPanel(data, options = {}) {
  const appData = signal(data);
  const query = signal("");
  const mode = signal(options.initialMode ?? "contextual");
  const pickerApps = signal(options.pickerApps ?? []);
  const statusMessage = signal("");
  const collapsedGroups = signal(new Set());
  const highlightedIndex = signal(-1);

  const contextualGroups = computed(() => {
    const groups = appData().groups;
    const currentQuery = query().trim();
    return groups.map((group) => ({
      group: group.group,
      items: fuzzyFilter(group.items, currentQuery, (item) => [
        ...item.keys,
        item.keys.join(" "),
        item.label,
        item.value ?? "",
        item.notes ?? "",
        ...(item.search_terms ?? []),
      ]),
    }));
  });

  const filteredPickerApps = computed(() => {
    const currentQuery = query().trim();
    return fuzzyFilter(pickerApps(), currentQuery, (app) => [app.name, app.processes.join(" ")]);
  });

  const visibleContextualGroups = computed(() =>
    contextualGroups().filter((group) => group.items.length > 0),
  );

  const highlightedKey = computed(() => {
    if (mode() === "picker") {
      const visible = filteredPickerApps().map((app) => `picker::${app.id}`);
      const index = highlightedIndex();
      return index >= 0 && index < visible.length ? visible[index] : null;
    }

    const visible = visibleContextualGroups().flatMap((group) =>
      group.items.map((item) => `${group.group}::${item.label}`),
    );
    const index = highlightedIndex();
    return index >= 0 && index < visible.length ? visible[index] : null;
  });

  /**
   * @param {string} groupName
   * @returns {void}
   */
  function toggleGroup(groupName) {
    const next = new Set(collapsedGroups());
    if (next.has(groupName)) {
      next.delete(groupName);
    } else {
      next.add(groupName);
    }
    collapsedGroups(next);
  }

  /**
   * @returns {Promise<void>}
   */
  async function openPicker() {
    statusMessage("");
    const apps = /** @type {PickerApp[]} */ (await getPickerApps());
    pickerApps(apps);
    appData(pickerAppsToPanelData(apps));
    mode("picker");
    query("");
    highlightedIndex(apps.length > 0 ? 0 : -1);
  }

  /**
   * @param {string} pickerId
   * @returns {Promise<void>}
   */
  async function loadPickerApp(pickerId) {
    statusMessage("");
    const next = /** @type {PanelData} */ (await loadPickerReference(pickerId));
    appData(next);
    mode("contextual");
    query("");
    collapsedGroups(new Set());
    highlightedIndex(-1);
  }

  /**
   * @returns {Promise<void>}
   */
  async function closePanel() {
    try {
      await closeCurrentWindow();
    } catch {
      hidePanelWindow().catch(() => {});
    }
  }

  /**
   * @param {KeyboardEvent} event
   * @returns {void}
   */
  function handleKeydown(event) {
    if (mode() === "picker") {
      const apps = filteredPickerApps();

      if (apps.length === 0) {
        if (event.key === "Escape") {
          event.preventDefault();
          closePanel().catch(() => {});
        }
        return;
      }

      if (event.key === "ArrowDown") {
        event.preventDefault();
        highlightedIndex((highlightedIndex() + 1 + apps.length) % apps.length);
        return;
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        highlightedIndex((highlightedIndex() - 1 + apps.length) % apps.length);
        return;
      }

      if (event.key === "Enter") {
        event.preventDefault();
        const currentIndex = highlightedIndex();
        const app = currentIndex >= 0 ? apps[currentIndex] : apps[0];
        if (app) {
          loadPickerApp(app.id).catch(() => {});
        }
        return;
      }

      if (event.key === "Escape") {
        event.preventDefault();
        closePanel().catch(() => {});
      }
      return;
    }

    const flat = visibleContextualGroups().flatMap((group) => group.items);
    if (flat.length === 0) {
      if (event.key === "Escape") {
        event.preventDefault();
        openPicker().catch((error) => {
          statusMessage(`Picker failed: ${error instanceof Error ? error.message : String(error)}`);
        });
      }
      return;
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      highlightedIndex((highlightedIndex() + 1 + flat.length) % flat.length);
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      highlightedIndex((highlightedIndex() - 1 + flat.length) % flat.length);
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      openPicker().catch((error) => {
        statusMessage(`Picker failed: ${error instanceof Error ? error.message : String(error)}`);
      });
    }
  }

  /** @type {import("./runtime/naf.js").TemplateOptions<HTMLElement>} */
  const panelOptions = {
    root: ".peek-surface",
    onMount(_el, _parent, ctx) {
      const searchInput = /** @type {HTMLInputElement} */ (requireRef(ctx.refs, "searchInput"));
      const closeButton = /** @type {HTMLButtonElement} */ (requireRef(ctx.refs, "closeButton"));
      const title = /** @type {HTMLHeadingElement} */ (requireRef(ctx.refs, "title"));
      const groupsHost = requireRef(ctx.refs, "groupsHost");

      queueMicrotask(() => {
        searchInput.focus();
      });

      ctx.cleanup.add(
        listener(searchInput, "input", (event) => {
          const target = /** @type {HTMLInputElement} */ (event.currentTarget);
          query(target.value);
          highlightedIndex(-1);
        }),
        listener(closeButton, "click", () => {
          closePanel().catch(() => {});
        }),
        listener(groupsHost, "click", (event) => {
          const target = event.target instanceof HTMLElement ? event.target : null;
          const urlTarget = target?.closest("[data-item-url]");
          if (urlTarget instanceof HTMLElement) {
            const url = urlTarget.getAttribute("data-item-url");
            if (url) {
              openExternalUrl(url).catch((error) => {
                statusMessage(
                  `Link failed: ${error instanceof Error ? error.message : String(error)}`,
                );
              });
            }
            return;
          }

          const pickerTarget = target?.closest("[data-picker-id]");
          if (pickerTarget instanceof HTMLElement) {
            const pickerId = pickerTarget.getAttribute("data-picker-id");
            if (pickerId) {
              loadPickerApp(pickerId).catch(() => {});
            }
            return;
          }

          const groupTarget = target?.closest("[data-group-toggle]");
          if (!(groupTarget instanceof HTMLElement)) {
            return;
          }

          const groupName = groupTarget.getAttribute("data-group-toggle");
          if (!groupName) {
            return;
          }

          toggleGroup(groupName);
        }),
        listener(document, "keydown", handleKeydown),
        effect(() => {
          const currentMode = mode();
          const activeKey = highlightedKey();
          const collapsed = collapsedGroups();
          const currentStatus = statusMessage();

          title.textContent = appData().app_name;

          if (currentMode === "picker") {
            const apps = filteredPickerApps();
            groupsHost.innerHTML =
              apps.length === 0
                ? `
                    <div class="peek-empty">
                      No reference files match this filter.
                    </div>
                  `
                : `
                    <section class="peek-group">
                      <div class="peek-group__toggle peek-group__toggle--static">
                        <span>▾</span>
                        <span class="peek-group__title">Reference Files</span>
                      </div>
                      <div class="peek-item-list">
                        ${apps
                          .map((app) => {
                            const itemKey = `picker::${app.id}`;
                            const isHighlighted = itemKey === activeKey;
                            return `
                              <button
                                type="button"
                                class="peek-item peek-item--picker${isHighlighted ? " peek-item--highlighted" : ""}"
                                data-picker-id="${text(app.id)}"
                              >
                                <span class="peek-item__label">${text(app.name)}</span>
                                <p class="peek-item__notes">${text(app.processes.join(", "))}</p>
                              </button>
                            `;
                          })
                          .join("")}
                      </div>
                    </section>
                  `;
            return;
          }

          const groups = visibleContextualGroups();
          groupsHost.innerHTML =
            currentStatus
              ? `
                  <div class="peek-empty">
                    ${text(currentStatus)}
                  </div>
                `
              : groups.length === 0
              ? `
                  <div class="peek-empty">
                    No reference items match this filter. Press Escape for the reference picker.
                  </div>
                `
              : groups
                  .map((group) => {
                    const isCollapsed = collapsed.has(group.group);
                    const itemsMarkup = isCollapsed
                      ? ""
                      : group.items
                          .map((item) => {
                            const itemKey = `${group.group}::${item.label}`;
                            const isHighlighted = itemKey === activeKey;
                            const itemKind = getItemKind(item);
                            return `
                              <article class="peek-item peek-item--${itemKind}${isHighlighted ? " peek-item--highlighted" : ""}">
                                <div class="peek-item__topline${item.url ? " peek-item__topline--actionable" : ""}">
                                  <span class="peek-item__label">${text(item.label)}</span>
                                  ${itemKind !== "command" && item.url ? `<button
                                    type="button"
                                    class="peek-item__action"
                                    data-item-url="${text(item.url)}"
                                    aria-label="Open link"
                                    title="Open link"
                                  >
                                    <span class="peek-item__action-label">Open</span>
                                    <span class="icon-mask peek-item__action-icon" aria-hidden="true"></span>
                                  </button>` : ""}
                                </div>
                                ${item.keys.length > 0 ? `<div class="peek-item__keys"><span class="peek-keys">${renderKeys(item.keys)}</span></div>` : ""}
                                ${renderBody(item)}
                              </article>
                            `;
                          })
                          .join("");

                    return `
                      <section class="peek-group">
                        <button
                          type="button"
                          class="peek-group__toggle"
                          data-group-toggle="${text(group.group)}"
                        >
                          <span>${isCollapsed ? "▸" : "▾"}</span>
                          <span class="peek-group__title">${text(group.group)}</span>
                        </button>
                        <div class="peek-item-list">${itemsMarkup}</div>
                      </section>
                    `;
                  })
                  .join("");
        }),
      );
    },
  };

  const panel = template(panelOptions)/*html*/`
    <section class="peek-surface panel">
      <div class="window-resize-handles" aria-hidden="true">
        <div class="window-resize-handle window-resize-handle--n" data-tauri-drag-resize-region="Top"></div>
        <div class="window-resize-handle window-resize-handle--e" data-tauri-drag-resize-region="Right"></div>
        <div class="window-resize-handle window-resize-handle--s" data-tauri-drag-resize-region="Bottom"></div>
        <div class="window-resize-handle window-resize-handle--w" data-tauri-drag-resize-region="Left"></div>
        <div class="window-resize-handle window-resize-handle--ne" data-tauri-drag-resize-region="TopRight"></div>
        <div class="window-resize-handle window-resize-handle--nw" data-tauri-drag-resize-region="TopLeft"></div>
        <div class="window-resize-handle window-resize-handle--se" data-tauri-drag-resize-region="BottomRight"></div>
        <div class="window-resize-handle window-resize-handle--sw" data-tauri-drag-resize-region="BottomLeft"></div>
      </div>
      <header class="peek-header" data-tauri-drag-region>
        <div class="peek-header__meta" data-tauri-drag-region>
          <p class="eyebrow">JustPeek</p>
          <h1 class="peek-title" data-ref="title">${text(data.app_name)}</h1>
        </div>
        <button
          type="button"
          class="peek-close"
          data-ref="closeButton"
          aria-label="Close panel"
        >
          ×
        </button>
      </header>

      <section class="peek-controls">
        <label class="peek-search">
          <input
            class="input"
            type="search"
            data-ref="searchInput"
            placeholder="Filter references..."
            autocomplete="off"
          />
        </label>
      </section>

      <section
        class="peek-reference-groups"
        data-ref="groupsHost"
        aria-live="polite"
      ></section>
    </section>
  `;

  return {
    mount(container) {
      mount(panel, container);
    },
    unmount() {
      panel.unmount?.();
    },
  };
}
