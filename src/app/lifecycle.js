import { cleanupCollector, listener } from "../shared/runtime/naf.js";

/**
 * @param {object} options
 * @param {Array<{ cleanup: () => void }>} options.featureCleanups
 * @returns {{ cleanup: () => void }}
 */
export function mountAppLifecycle(options) {
  const cleanup = cleanupCollector(
    listener(window, "beforeunload", () => {
      cleanup.run();
    }),
    ...options.featureCleanups,
  );

  return {
    cleanup: () => cleanup.run(),
  };
}
