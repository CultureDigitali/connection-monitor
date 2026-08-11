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
