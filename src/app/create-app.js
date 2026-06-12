import { requireElement } from "../shared/runtime/naf.js";
import { mountAppLifecycle } from "./lifecycle.js";
import { mountPanelPage } from "../pages/panel/panel-page.js";

/**
 * @typedef {object} AppShell
 * @property {HTMLElement} root
 * @property {HTMLElement} peekSurface
 * @property {HTMLInputElement} searchInput
 * @property {HTMLElement} status
 * @property {HTMLElement} referenceGroups
 */

/**
 * @param {HTMLElement} root
 * @returns {AppShell}
 */
function collectShell(root) {
  return {
    root,
    peekSurface: requireElement(root, "#peek-surface", "peek-surface"),
    searchInput: /** @type {HTMLInputElement} */ (
      requireElement(root, "#peek-search-input", "peek-search-input")
    ),
    status: requireElement(root, "#peek-status", "peek-status"),
    referenceGroups: requireElement(
      root,
      "#peek-reference-groups",
      "peek-reference-groups",
    ),
  };
}

/**
 * @param {HTMLElement} root
 * @returns {AppShell}
 */
export function createApp(root) {
  const shell = collectShell(root);
  const panelPage = mountPanelPage(shell);

  mountAppLifecycle({
    featureCleanups: [panelPage],
  });

  root.removeAttribute("data-loading");
  return shell;
}
