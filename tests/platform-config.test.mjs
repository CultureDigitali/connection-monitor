import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const readJson = async (path) => JSON.parse(await readFile(path, "utf8"));

test("platform configs isolate macOS and Windows bundles", async () => {
  const common = await readJson("src-tauri/tauri.conf.json");
  const mac = await readJson("src-tauri/tauri.macos.conf.json");
  const windows = await readJson("src-tauri/tauri.windows.conf.json");

  assert.equal(common.version, "0.3.1");
  // Tauri validates dependency features against the merged config on every OS.
  // Keep this flag aligned with the common `tauri/macos-private-api` feature;
  // the API remains a no-op outside macOS.
  assert.equal(common.app.macOSPrivateApi, true);
  assert.equal(mac.app, undefined);
  assert.deepEqual(mac.bundle.targets, ["app", "dmg"]);
  assert.deepEqual(windows.bundle.targets, ["nsis", "msi"]);
});

test("package metadata and Windows dependencies target version 0.3.1", async () => {
  const packageJson = await readJson("package.json");
  const cargo = await readFile("src-tauri/Cargo.toml", "utf8");

  assert.equal(packageJson.version, "0.3.1");
  assert.match(cargo, /^version = "0\.3\.1"/m);
  assert.match(cargo, /cfg\(target_os = "windows"\)/);
  assert.match(cargo, /Win32_NetworkManagement_WiFi/);
});

test("Windows release binary uses the GUI subsystem", async () => {
  const main = await readFile("src-tauri/src/main.rs", "utf8");
  assert.match(
    main,
    /#!\[cfg_attr\(\s*all\(not\(debug_assertions\), target_os = "windows"\),\s*windows_subsystem = "windows"\s*\)\]/,
  );
});
