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
  getConfig,
  getPickerApps,
  getRuntimeInfo,
  hidePanelWindow,
  loadPickerApp as loadPickerReference,
  openSettingsWindow,
  openExternalUrl,
  startResizeDragging,
} from "./runtime/tauri.js";

/**
 * @typedef {object} KeysByOs
 * @property {string[]=} macos
 * @property {string[]=} windows
 * @property {string[]=} linux
 */

/**
 * @typedef {object} PanelItem
 * @property {string[]} keys
 * @property {KeysByOs=} keys_by_os
 * @property {string} label
 * @property {string=} value
 * @property {string=} command
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
 * @property {string} group
 * @property {string[]} processes
 * @property {string} source_path
 * @property {string=} error
 */

/**
 * @typedef {object} CreatePanelOptions
 * @property {"contextual"|"picker"} [initialMode]
 * @property {PickerApp[]} [pickerApps]
 */

/**
 * @typedef {{ type: "group", group: string } | { type: "item", group: string, item: PanelItem }} ContextualEntry
 * @typedef {{ type: "group", group: string } | { type: "app", group: string, app: PickerApp }} PickerEntry
 */

/**
 * @typedef {object} DisplayKeySet
 * @property {string | null} label
 * @property {string[]} keys
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
 * @param {DisplayKeySet[]} keySets
 * @returns {string}
 */
function renderDisplayKeySets(keySets) {
  return keySets
    .map(
      (keySet) => `
        <span class="peek-keyset">
          ${keySet.label ? `<span class="peek-keyset__label">${text(keySet.label)}</span>` : ""}
          <span class="peek-keys">${renderKeys(keySet.keys)}</span>
        </span>
      `,
    )
    .join("");
}

/**
 * @param {PanelItem} item
 * @param {"auto" | "macos" | "windows" | "linux"} preferredOs
 * @param {"macos" | "windows" | "linux"} currentRuntimeOs
 * @param {"current" | "all"} displayMode
 * @returns {DisplayKeySet[]}
 */
function resolveDisplayKeys(item, preferredOs, currentRuntimeOs, displayMode) {
  const effectiveOs =
    preferredOs === "macos" || preferredOs === "windows" || preferredOs === "linux"
      ? preferredOs
      : currentRuntimeOs;

  if (displayMode === "all") {
    /** @type {DisplayKeySet[]} */
    const keySets = [];
    /** @type {Array<["macos" | "windows" | "linux", string]>} */
    const ordered = [
      ["macos", "macOS"],
      ["windows", "Windows"],
      ["linux", "Linux"],
    ];

    for (const [osKey, label] of ordered) {
      const keys = item.keys_by_os?.[osKey];
      if (keys && keys.length > 0) {
        keySets.push({ label, keys });
      }
    }

    if (item.keys.length > 0) {
      keySets.push({ label: "Default", keys: item.keys });
    }

    return keySets;
  }

  const osKeys = item.keys_by_os?.[effectiveOs];
  return [{ label: null, keys: osKeys && osKeys.length > 0 ? osKeys : item.keys }];
}

/**
 * @param {PanelItem} item
 * @param {string[]=} displayKeys
 * @returns {string}
 */
function getItemKind(item, displayKeys = item.keys) {
  if (displayKeys.length > 0) {
    return "shortcut";
  }

  if (item.command?.trim()) {
    return "command";
  }

  return "detail";
}

/**
 * @param {PanelItem} item
 * @param {string[]=} displayKeys
 * @returns {string}
 */
function renderBody(item, displayKeys = item.keys) {
  const value = item.value?.trim();
  const command = item.command?.trim();
  const itemKind = getItemKind(item, displayKeys);
  const valueMarkup = value ? `<p class="peek-item__value">${text(value)}</p>` : "";
  const commandMarkup = command
    ? `<code class="peek-item__value peek-item__value--command">${text(command)}</code>`
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
    return `${valueMarkup}${commandMarkup}${notesMarkup}${actionMarkup}`;
  }

  return `
    <div class="peek-item__meta">
      ${valueMarkup}
      ${commandMarkup}
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
  /** @type {Map<string, PickerApp[]>} */
  const appsByGroup = new Map();
  for (const app of apps) {
    const group = app.group || "Ungrouped";
    const groupApps = appsByGroup.get(group) ?? [];
    groupApps.push(app);
    appsByGroup.set(group, groupApps);
  }

  return {
    app_name: "Available References",
    groups: Array.from(appsByGroup.entries())
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([group, groupApps]) => ({
        group,
        items: groupApps
          .slice()
          .sort((left, right) => left.name.localeCompare(right.name))
          .map((app) => ({
          keys: [],
          label: app.name,
          value: app.processes.length > 0 ? `Processes: ${app.processes.join(", ")}` : app.source_path,
          notes: app.error ? `Error: ${app.error}` : "Select a reference file to load it.",
          })),
      })),
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
  const preferredShortcutOs = signal("auto");
  const shortcutDisplayMode = signal("current");
  const runtimeOs = signal("linux");
  const statusMessage = signal("");
  const collapsedGroups = signal(new Set());
  const highlightedIndex = signal(-1);
  const copiedItemKey = signal(/** @type {string | null} */ (null));
  /** @type {HTMLInputElement | null} */
  let searchInputElement = null;
  /** @type {number | null} */
  let copiedItemTimer = null;

  const contextualGroups = computed(() => {
    const groups = appData().groups;
    const currentQuery = query().trim();
    return groups.map((group) => ({
      group: group.group,
      items: fuzzyFilter(group.items, currentQuery, (item) => [
        ...item.keys,
        ...Object.values(item.keys_by_os ?? {}).flat(),
        item.keys.join(" "),
        item.label,
        item.value ?? "",
        item.command ?? "",
        item.notes ?? "",
        ...(item.search_terms ?? []),
      ]),
    }));
  });

  const filteredPickerApps = computed(() => {
    const currentQuery = query().trim();
    return fuzzyFilter(pickerApps(), currentQuery, (app) => [
      app.group,
      app.name,
      app.processes.join(" "),
      app.source_path,
      app.error ?? "",
    ]);
  });

  const visiblePickerGroups = computed(() => {
    /** @type {Map<string, PickerApp[]>} */
    const appsByGroup = new Map();
    for (const app of filteredPickerApps()) {
      const groupApps = appsByGroup.get(app.group) ?? [];
      groupApps.push(app);
      appsByGroup.set(app.group, groupApps);
    }

    return Array.from(appsByGroup.entries())
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([group, apps]) => ({
        group,
        apps: apps.slice().sort((left, right) => left.name.localeCompare(right.name)),
      }));
  });

  const navigablePickerEntries = computed(
    /** @returns {PickerEntry[]} */ () => {
      /** @type {PickerEntry[]} */
      const entries = [];
      for (const group of visiblePickerGroups()) {
        entries.push({ type: "group", group: group.group });
        if (collapsedGroups().has(group.group)) {
          continue;
        }

        for (const app of group.apps) {
          entries.push({
            type: "app",
            group: group.group,
            app,
          });
        }
      }
      return entries;
    },
  );

  const visibleContextualGroups = computed(() =>
    contextualGroups().filter((group) => group.items.length > 0),
  );

  const navigableContextualEntries = computed(
    /** @returns {ContextualEntry[]} */ () => {
      /** @type {ContextualEntry[]} */
      const entries = [];
      for (const group of visibleContextualGroups()) {
        if (collapsedGroups().has(group.group)) {
          entries.push({ type: "group", group: group.group });
          continue;
        }

        for (const item of group.items) {
          entries.push({
            type: "item",
            group: group.group,
            item,
          });
        }
      }
      return entries;
    },
  );

  const highlightedKey = computed(() => {
    if (mode() === "picker") {
      const visible = navigablePickerEntries().map((entry) =>
        entry.type === "group" ? `${entry.group}::group` : `picker::${entry.app.id}`,
      );
      const index = highlightedIndex();
      return index >= 0 && index < visible.length ? visible[index] : null;
    }

    const visible = navigableContextualEntries().map((entry) =>
      entry.type === "group" ? `${entry.group}::group` : `${entry.group}::${entry.item.label}`,
    );
    const index = highlightedIndex();
    return index >= 0 && index < visible.length ? visible[index] : null;
  });

  /**
   * @returns {boolean}
   */
  function isEditingTextField() {
    const active = document.activeElement;
    return active instanceof HTMLInputElement
      || active instanceof HTMLTextAreaElement
      || Boolean(active instanceof HTMLElement && active.isContentEditable);
  }

  /**
   * @returns {boolean}
   */
  function isSearchInputFocused() {
    return document.activeElement === searchInputElement;
  }

  /**
   * @returns {void}
   */
  function focusSearchInput() {
    if (!searchInputElement) {
      return;
    }

    searchInputElement.focus();
    const end = searchInputElement.value.length;
    searchInputElement.setSelectionRange(end, end);
  }

  /**
   * @param {string} value
   * @returns {void}
   */
  function copyValueToClipboard(value) {
    if (!navigator.clipboard?.writeText) {
      return;
    }

    navigator.clipboard.writeText(value).catch(() => {});
  }

  /**
   * @param {string} itemKey
   * @returns {void}
   */
  function flashCopiedItem(itemKey) {
    copiedItemKey(itemKey);
    if (copiedItemTimer !== null) {
      clearTimeout(copiedItemTimer);
    }
    copiedItemTimer = window.setTimeout(() => {
      if (copiedItemKey() === itemKey) {
        copiedItemKey(null);
      }
      copiedItemTimer = null;
    }, 300);
  }

  /**
   * @returns {PickerApp | null}
   */
  function getActivePickerApp() {
    const entries = navigablePickerEntries();
    const currentIndex = highlightedIndex();
    if (currentIndex >= 0 && currentIndex < entries.length) {
      const entry = entries[currentIndex];
      if (entry?.type === "app") {
        return entry.app;
      }
    }

    return entries.find((entry) => entry.type === "app")?.app ?? null;
  }

  /**
   * @returns {string | null}
   */
  function getActiveContextualGroupName() {
    const entries = navigableContextualEntries();
    const currentIndex = highlightedIndex();
    if (currentIndex >= 0 && currentIndex < entries.length) {
      return entries[currentIndex]?.group ?? null;
    }

    return visibleContextualGroups()[0]?.group ?? null;
  }

  /**
   * @returns {string | null}
   */
  function getActivePickerGroupName() {
    const entries = navigablePickerEntries();
    const currentIndex = highlightedIndex();
    if (currentIndex >= 0 && currentIndex < entries.length) {
      return entries[currentIndex]?.group ?? null;
    }

    return visiblePickerGroups()[0]?.group ?? null;
  }

  /**
   * @returns {void}
   */
  function syncHighlightedIndex() {
    if (mode() === "picker") {
      const apps = navigablePickerEntries();
      highlightedIndex(
        apps.length === 0 ? -1 : Math.min(Math.max(highlightedIndex(), 0), apps.length - 1),
      );
      return;
    }

    const items = navigableContextualEntries();
    highlightedIndex(
      items.length === 0 ? -1 : Math.min(Math.max(highlightedIndex(), 0), items.length - 1),
    );
  }

  /**
   * @param {number} delta
   * @returns {void}
   */
  function moveContextualSelection(delta) {
    const entries = navigableContextualEntries();
    if (entries.length === 0) {
      highlightedIndex(-1);
      return;
    }

    const nextIndex = (highlightedIndex() + delta + entries.length) % entries.length;
    highlightedIndex(nextIndex);
  }

  /**
   * @returns {ContextualEntry | null}
   */
  function getActiveContextualEntry() {
    const entries = navigableContextualEntries();
    const currentIndex = highlightedIndex();
    return currentIndex >= 0 && currentIndex < entries.length ? entries[currentIndex] : null;
  }

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
    queueMicrotask(() => {
      syncHighlightedIndex();
    });
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
    if (
      event.key === "/"
      && !event.altKey
      && !event.ctrlKey
      && !event.metaKey
      && !isEditingTextField()
    ) {
      event.preventDefault();
      focusSearchInput();
      return;
    }

    if (
      isSearchInputFocused()
      && (event.key === "ArrowDown" || event.key === "ArrowUp")
      && !event.altKey
      && !event.ctrlKey
      && !event.metaKey
    ) {
      event.preventDefault();
      searchInputElement?.blur();
    }

    if (mode() === "picker") {
      const entries = navigablePickerEntries();

      if (entries.length === 0) {
        if (event.key === "Escape") {
          event.preventDefault();
          closePanel().catch(() => {});
        }
        return;
      }

      if (event.key === "ArrowDown") {
        event.preventDefault();
        highlightedIndex((highlightedIndex() + 1 + entries.length) % entries.length);
        return;
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        highlightedIndex((highlightedIndex() - 1 + entries.length) % entries.length);
        return;
      }

      if (event.key === "Enter" || (event.key === "ArrowRight" && !isEditingTextField())) {
        event.preventDefault();
        const app = getActivePickerApp();
        if (app) {
          loadPickerApp(app.id).catch(() => {});
        }
        return;
      }

      if (event.key === " " && !isEditingTextField()) {
        event.preventDefault();
        const groupName = getActivePickerGroupName();
        if (groupName) {
          toggleGroup(groupName);
        }
        return;
      }

      if (event.key === "Escape") {
        event.preventDefault();
        closePanel().catch(() => {});
      }
      return;
    }

    const items = navigableContextualEntries();
    if (items.length === 0) {
      if (event.key === "Escape" || (event.key === "ArrowLeft" && !isEditingTextField())) {
        event.preventDefault();
        openPicker().catch((error) => {
          statusMessage(`Picker failed: ${error instanceof Error ? error.message : String(error)}`);
        });
      }
      return;
    }

    if (event.key === "ArrowDown") {
      event.preventDefault();
      moveContextualSelection(1);
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      moveContextualSelection(-1);
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      const entry = getActiveContextualEntry();
      if (entry?.type === "item") {
        const copyTarget = entry.item.command ?? entry.item.value;
        if (!copyTarget) {
          return;
        }

        flashCopiedItem(`${entry.group}::${entry.item.label}`);
        copyValueToClipboard(copyTarget);
      }
      return;
    }

    if (
      event.key.toLowerCase() === "o"
      && !event.altKey
      && !event.ctrlKey
      && !event.metaKey
      && !isEditingTextField()
    ) {
      const entry = getActiveContextualEntry();
      if (entry?.type === "item" && entry.item.url) {
        event.preventDefault();
        openExternalUrl(entry.item.url).catch((error) => {
          statusMessage(`Link failed: ${error instanceof Error ? error.message : String(error)}`);
        });
        return;
      }
    }

    if (event.key === " " && !isEditingTextField()) {
      event.preventDefault();
      const groupName = getActiveContextualGroupName();
      if (groupName) {
        toggleGroup(groupName);
      }
      return;
    }

    if (event.key === "ArrowLeft" && !isEditingTextField()) {
      event.preventDefault();
      openPicker().catch((error) => {
        statusMessage(`Picker failed: ${error instanceof Error ? error.message : String(error)}`);
      });
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
      const resizeHandles = requireRef(ctx.refs, "resizeHandles");
      const searchInput = /** @type {HTMLInputElement} */ (requireRef(ctx.refs, "searchInput"));
      searchInputElement = searchInput;
      const closeButton = /** @type {HTMLButtonElement} */ (requireRef(ctx.refs, "closeButton"));
      const settingsButton = /** @type {HTMLButtonElement} */ (requireRef(ctx.refs, "settingsButton"));
      const title = /** @type {HTMLHeadingElement} */ (requireRef(ctx.refs, "title"));
      const groupsHost = requireRef(ctx.refs, "groupsHost");

      queueMicrotask(() => {
        searchInput.focus();
      });

      void Promise.all([getConfig(), getRuntimeInfo()])
        .then(([config, info]) => {
          preferredShortcutOs(config.preferred_shortcut_os ?? "auto");
          shortcutDisplayMode(config.shortcut_display_mode === "all" ? "all" : "current");
          runtimeOs(
            info.os === "macos" || info.os === "windows" || info.os === "linux" ? info.os : "linux",
          );
        })
        .catch(() => {});

      ctx.cleanup.add(
        listener(searchInput, "input", (event) => {
          const target = /** @type {HTMLInputElement} */ (event.currentTarget);
          query(target.value);
          highlightedIndex(-1);
        }),
        listener(closeButton, "click", () => {
          closePanel().catch(() => {});
        }),
        listener(settingsButton, "click", () => {
          openSettingsWindow().catch((error) => {
            statusMessage(
              `Settings failed: ${error instanceof Error ? error.message : String(error)}`,
            );
          });
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
          ).catch((error) => {
            statusMessage(
              `Resize failed: ${error instanceof Error ? error.message : String(error)}`,
            );
          });
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
          const copiedKey = copiedItemKey();
          const collapsed = collapsedGroups();
          const currentStatus = statusMessage();

          title.textContent = appData().app_name;

          if (currentMode === "picker") {
            const groups = visiblePickerGroups();
            groupsHost.innerHTML =
              groups.length === 0
                ? `
                    <div class="peek-empty">
                      No reference files match this filter.
                    </div>
                  `
                : `
                    ${groups
                      .map((group) => {
                        const isCollapsed = collapsed.has(group.group);
                        const itemsMarkup = isCollapsed
                          ? ""
                          : group.apps
                          .map((app) => {
                            const itemKey = `picker::${app.id}`;
                            const isHighlighted = itemKey === activeKey;
                            const processText = app.processes.join(", ");
                            const detailText = app.error
                              ? `Error: ${app.error}`
                              : processText
                                ? `Processes: ${processText}`
                                : app.source_path;
                            return `
                              <button
                                type="button"
                                class="peek-item peek-item--picker${app.error ? " peek-item--invalid" : ""}${isHighlighted ? " peek-item--highlighted" : ""}"
                                data-picker-id="${text(app.id)}"
                                ${isHighlighted ? 'data-active-entry="true"' : ""}
                              >
                                <div class="peek-item__topline">
                                  <span class="peek-item__label">${text(app.name)}</span>
                                  ${app.error ? '<span class="peek-item__badge">Error</span>' : ""}
                                </div>
                                <p class="peek-item__notes">${text(detailText)}</p>
                                ${app.error ? `<p class="peek-item__value">${text(app.source_path)}</p>` : ""}
                              </button>
                            `;
                          })
                          .join("");

                        return `
                          <section class="peek-group">
                            <button
                              type="button"
                              class="peek-group__toggle${activeKey === `${group.group}::group` ? " peek-group__toggle--highlighted" : ""}"
                              data-group-toggle="${text(group.group)}"
                              ${activeKey === `${group.group}::group` ? 'data-active-entry="true"' : ""}
                            >
                              <span>${isCollapsed ? "▸" : "▾"}</span>
                              <span class="peek-group__title">${text(group.group)}</span>
                            </button>
                            <div class="peek-item-list">${itemsMarkup}</div>
                          </section>
                        `;
                      })
                      .join("")}
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
                            const isCopied = itemKey === copiedKey;
                            const displayKeySets = resolveDisplayKeys(
                              item,
                              /** @type {"auto" | "macos" | "windows" | "linux"} */ (preferredShortcutOs()),
                              /** @type {"macos" | "windows" | "linux"} */ (runtimeOs()),
                              /** @type {"current" | "all"} */ (shortcutDisplayMode()),
                            );
                            const primaryDisplayKeys = displayKeySets[0]?.keys ?? [];
                            const itemKind = getItemKind(item, primaryDisplayKeys);
                            return `
                              <article
                                class="peek-item peek-item--${itemKind}${isHighlighted ? " peek-item--highlighted" : ""}${isCopied ? " peek-item--copied" : ""}"
                                ${isHighlighted ? 'data-active-entry="true"' : ""}
                              >
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
                                ${displayKeySets.length > 0 ? `<div class="peek-item__keys${displayKeySets.length > 1 ? " peek-item__keys--stacked" : ""}">${renderDisplayKeySets(displayKeySets)}</div>` : ""}
                                ${renderBody(item, primaryDisplayKeys)}
                              </article>
                            `;
                          })
                  .join("");

                    return `
                      <section class="peek-group">
                        <button
                          type="button"
                          class="peek-group__toggle${activeKey === `${group.group}::group` ? " peek-group__toggle--highlighted" : ""}"
                          data-group-toggle="${text(group.group)}"
                          ${activeKey === `${group.group}::group` ? 'data-active-entry="true"' : ""}
                        >
                          <span>${isCollapsed ? "▸" : "▾"}</span>
                          <span class="peek-group__title">${text(group.group)}</span>
                        </button>
                        <div class="peek-item-list">${itemsMarkup}</div>
                      </section>
                    `;
                  })
                  .join("");

          queueMicrotask(() => {
            const activeEntry = groupsHost.querySelector("[data-active-entry='true']");
            if (activeEntry instanceof HTMLElement) {
              activeEntry.scrollIntoView({
                block: "nearest",
                inline: "nearest",
              });
            }
          });
        }),
      );
      ctx.cleanup.add(() => {
        if (searchInputElement === searchInput) {
          searchInputElement = null;
        }
        if (copiedItemTimer !== null) {
          clearTimeout(copiedItemTimer);
          copiedItemTimer = null;
        }
      });
    },
  };

  const panel = template(panelOptions)/*html*/`
    <section class="peek-surface">
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
          class="peek-window-action peek-window-action--settings"
          data-ref="settingsButton"
          aria-label="Open settings"
          title="Open settings"
        >
          <span class="icon-mask peek-window-action__icon peek-window-action__icon--settings" aria-hidden="true"></span>
        </button>
        <button
          type="button"
          class="peek-window-action peek-window-action--close"
          data-ref="closeButton"
          aria-label="Close panel"
          title="Close panel"
        >
          <span class="icon-mask peek-window-action__icon peek-window-action__icon--close" aria-hidden="true"></span>
        </button>
      </div>
      <header class="peek-header" data-tauri-drag-region>
        <div class="peek-header__meta" data-tauri-drag-region>
          <p class="eyebrow no-click">JustPeek</p>
          <h1 class="peek-title no-click" data-ref="title">${text(data.app_name)}</h1>
        </div>
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
