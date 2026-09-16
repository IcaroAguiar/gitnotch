import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const config = JSON.parse(
  await readFile(
    new URL("../src-tauri/tauri.conf.json", import.meta.url),
    "utf8",
  ),
);

test("a janela única carrega somente o frontend empacotado", () => {
  assert.equal(config.build.frontendDist, "../dist");
  assert.equal(config.build.devUrl, undefined);
  assert.deepEqual(
    config.app.windows.map((window) => window.label),
    ["notch"],
  );

  for (const window of config.app.windows) {
    assert.equal(window.url, undefined);
  }

  assert.notEqual(config.app.withGlobalTauri, true);
});

test("a fita nasce recolhida e não recebe foco ao aparecer", () => {
  const [notch] = config.app.windows;

  assert.equal(notch.width, 16);
  assert.equal(notch.height, 96);
  assert.equal(notch.focus, false);
  assert.equal(notch.focusable, true);
  assert.equal(notch.acceptFirstMouse, true);
  assert.equal(notch.transparent, true);
  assert.equal(notch.alwaysOnTop, true);
  assert.equal(notch.skipTaskbar, true);
  assert.equal(notch.decorations, false);
  assert.equal(notch.shadow, false);
});

test("a janela recebe apenas o canal de eventos do estado", async () => {
  const capability = JSON.parse(
    await readFile(
      new URL("../src-tauri/capabilities/notch.json", import.meta.url),
      "utf8",
    ),
  );

  assert.deepEqual(capability.windows, ["notch"]);
  assert.deepEqual(capability.permissions, [
    "core:event:allow-listen",
    "core:event:allow-unlisten",
  ]);
});

test("a prévia não concede capacidades nativas e limita conexões ao IPC", () => {
  assert.deepEqual(config.app.security.capabilities, []);
  assert.equal(config.app.macOSPrivateApi, true);

  const directives = new Map(
    config.app.security.csp.split(";").map((directive) => {
      const [name, ...sources] = directive.trim().split(/\s+/);
      return [name, sources];
    }),
  );

  assert.deepEqual(directives.get("default-src"), ["'self'"]);
  assert.deepEqual(directives.get("script-src"), ["'self'"]);
  assert.deepEqual(directives.get("connect-src"), [
    "'self'",
    "ipc:",
    "http://ipc.localhost",
  ]);
  assert.deepEqual(directives.get("frame-src"), ["'none'"]);
  assert.deepEqual(directives.get("object-src"), ["'none'"]);
});
