import { writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const srcDir = resolve(root, "src");
const entryPath = resolve(srcDir, "main.entry.js");

writeFileSync(entryPath, 'import "./main.js";\n');
console.log("Activated dev entry: src/main.entry.js -> src/main.js");
