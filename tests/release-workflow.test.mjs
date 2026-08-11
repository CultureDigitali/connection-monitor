import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("release workflow does not pass unconfigured Apple signing secrets", async () => {
  const workflow = await readFile(".github/workflows/release.yml", "utf8");

  for (const variable of [
    "APPLE_CERTIFICATE",
    "APPLE_CERTIFICATE_PASSWORD",
    "APPLE_SIGNING_IDENTITY",
    "APPLE_ID",
    "APPLE_PASSWORD",
    "APPLE_TEAM_ID",
  ]) {
    assert.doesNotMatch(workflow, new RegExp(`^\\s+${variable}:`, "m"));
  }
});

test("release workflow uses current Node 24 GitHub actions", async () => {
  const workflow = await readFile(".github/workflows/release.yml", "utf8");

  assert.match(workflow, /uses: actions\/checkout@v7/);
  assert.match(workflow, /uses: actions\/setup-node@v7/);
  assert.match(workflow, /node-version: '24'/);
});

test("release workflow builds macOS and Windows updater artifacts", async () => {
  const workflow = await readFile(".github/workflows/release.yml", "utf8");

  assert.match(workflow, /windows-latest/);
  assert.match(workflow, /x86_64-pc-windows-msvc/);
  assert.match(workflow, /bundles: nsis,msi/);
  assert.match(workflow, /updaterJsonPreferNsis: true/);
  assert.match(workflow, /node --test tests\/\*\.test\.mjs/);
  assert.match(workflow, /cargo test/);
});

test("Windows CI builds and uploads both installers", async () => {
  const workflow = await readFile(".github/workflows/windows.yml", "utf8");

  assert.match(workflow, /windows-latest/);
  assert.match(workflow, /--bundles nsis,msi/);
  assert.match(workflow, /TAURI_SIGNING_PRIVATE_KEY/);
  assert.match(workflow, /uses: actions\/upload-artifact@v7/);
  assert.match(workflow, /bundle\/nsis/);
  assert.match(workflow, /bundle\/msi/);
});
