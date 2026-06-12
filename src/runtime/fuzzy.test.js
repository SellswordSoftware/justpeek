import test from "node:test";
import assert from "node:assert/strict";

import { fuzzyFilter, fuzzyMatch } from "./fuzzy.js";

test("fuzzyMatch matches common shortcut text", () => {
  assert.notEqual(fuzzyMatch("ctrl+p", "Ctrl+P Quick Open"), null);
});

test("fuzzyMatch matches non-contiguous abbreviations", () => {
  assert.notEqual(fuzzyMatch("cmdpal", "Command Palette"), null);
});

test("fuzzyMatch returns null when query cannot be satisfied", () => {
  assert.equal(fuzzyMatch("zzz", "Ctrl+P"), null);
});

test("fuzzyFilter returns matching items sorted by score", () => {
  const items = [
    { id: "palette", values: ["Command Palette"] },
    { id: "open", values: ["Quick Open"] },
    { id: "preview", values: ["Open Preview"] },
  ];

  const result = fuzzyFilter(items, "op", (item) => item.values);

  assert.deepEqual(
    result.map((item) => item.id),
    ["preview", "open", "palette"],
  );
});
