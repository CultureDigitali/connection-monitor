import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("README documents Windows installers and unsigned warning", async () => {
  const readme = await readFile("README.md", "utf8");

  assert.match(readme, /Windows 10\/11/);
  assert.match(readme, /\.exe/);
  assert.match(readme, /\.msi/);
  assert.match(readme, /SmartScreen/);
  assert.match(readme, /installer non firmati|unsigned installers/i);
  assert.match(readme, /logo=windows/);
});
