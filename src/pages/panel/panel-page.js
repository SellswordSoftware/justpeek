import { createPanel } from "../../panel.js";

/** @type {import("../../panel.js").PanelGroup[]} */
const SAMPLE_GROUPS = [
  {
    group: "Editor",
    items: [
      { keys: "Ctrl+Shift+P", label: "Command palette", notes: "Fast path into app actions" },
      { keys: "Ctrl+P", label: "Quick open", notes: "Jump to a file by name" },
    ],
  },
  {
    group: "Git",
    items: [
      { label: "Annotated tag", value: 'git tag -a v1.2.3 -m "Release v1.2.3"' },
      { label: "Revert a commit", value: "git revert <commit>", notes: "Safe for shared history" },
    ],
  },
  {
    group: "People",
    items: [
      { label: "Jordan Lee", value: "Product lead", notes: "Owns roadmap sign-off" },
      { label: "Sam Patel", value: "SRE", notes: "Escalate incident follow-up questions" },
    ],
  },
];

/**
 * @param {import("../../app/create-app.js").AppShell} shell
 * @returns {{ cleanup: () => void }}
 */
export function mountPanelPage(shell) {
  shell.peekSurface.replaceChildren();
  shell.status.textContent = "Sample panel";
  shell.searchInput.closest(".peek-controls")?.setAttribute("hidden", "hidden");
  shell.referenceGroups.replaceChildren();

  const panel = createPanel({
    app_name: "Contextual quick reference",
    groups: SAMPLE_GROUPS,
  });
  panel.mount(shell.peekSurface);

  return {
    cleanup: () => {
      panel.unmount();
    },
  };
}
