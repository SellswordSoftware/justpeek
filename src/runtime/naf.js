// @ts-check

export {
  cleanupCollector,
  computed,
  effect,
  listener,
  mount,
  raw,
  requireElement,
  requireRef,
  setText,
  signal,
  template,
  text,
  untrack,
  when,
} from "../vendor/naf/dist/naf.min.js";

const FOCUSABLE_SELECTOR = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
].join(",");

/**
 * @param {HTMLElement} container
 * @returns {HTMLElement[]}
 */
export function getFocusableElements(container) {
  return Array.from(container.querySelectorAll(FOCUSABLE_SELECTOR)).filter(
    isFocusableElement,
  );
}

/**
 * @param {Element} element
 * @returns {element is HTMLElement}
 */
function isFocusableElement(element) {
  return (
    element instanceof HTMLElement &&
    !element.hasAttribute("disabled") &&
    element.offsetParent !== null
  );
}

/**
 * Trap tab focus inside a container.
 *
 * @param {KeyboardEvent} event
 * @param {HTMLElement | null | undefined} container
 * @returns {boolean}
 */
export function trapFocusInContainer(event, container) {
  if (event.key !== "Tab" || !container) {
    return false;
  }

  const focusable = getFocusableElements(container);
  if (focusable.length === 0) {
    event.preventDefault();
    container.focus();
    return true;
  }

  const activeElement =
    document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const currentIndex = activeElement ? focusable.indexOf(activeElement) : -1;

  if (event.shiftKey) {
    if (currentIndex <= 0) {
      event.preventDefault();
      focusable[focusable.length - 1].focus();
      return true;
    }
    return false;
  }

  if (currentIndex === -1 || currentIndex >= focusable.length - 1) {
    event.preventDefault();
    focusable[0].focus();
    return true;
  }

  return false;
}
