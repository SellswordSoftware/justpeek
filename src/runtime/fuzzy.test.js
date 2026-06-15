import test from "node:test";
import assert from "node:assert/strict";

import { fuzzyFilter, fuzzyMatch } from "./fuzzy.js";

test("fuzzyMatch matches common shortcut text", () => {
  assert.notEqual(fuzzyMatch("ctrl+p", "Ctrl+P Quick Open"), null);
});

test("fuzzyMatch is case-insensitive substring match", () => {
  assert.notEqual(fuzzyMatch("quick", "Ctrl+P Quick Open"), null);
});

test("fuzzyMatch returns null when query cannot be satisfied", () => {
  assert.equal(fuzzyMatch("zzz", "Ctrl+P"), null);
});

test("fuzzyMatch rejects non-substring noise", () => {
  assert.equal(fuzzyMatch("copy", "Synchronize repos and upgrade system"), null);
});

test("fuzzyMatch does not support abbreviations", () => {
  assert.equal(fuzzyMatch("cmdpal", "Command Palette"), null);
  assert.equal(fuzzyMatch("ghp", "GitHub Projects"), null);
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
    ["open", "preview"],
  );
});

test("fuzzyFilter drops weak cross-field noise", () => {
  const items = [
    { id: "bad", values: ["Synchronize repos and upgrade system"] },
    { id: "good", values: ["Copy selected item"] },
  ];

  const result = fuzzyFilter(items, "copy", (item) => item.values);

  assert.deepEqual(
    result.map((item) => item.id),
    ["good"],
  );
});
