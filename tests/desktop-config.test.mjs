import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const config = JSON.parse(
  await readFile(
    new URL("../src-tauri/tauri.conf.json", import.meta.url),
    "utf8",
  ),
);

test("as duas janelas carregam somente o frontend empacotado", () => {
  assert.equal(config.build.frontendDist, "../dist");
  assert.equal(config.build.devUrl, undefined);
  assert.deepEqual(
    config.app.windows.map((window) => window.label),
    ["notch", "drawer"],
  );

  for (const window of config.app.windows) {
    assert.equal(window.url, undefined);
  }

  assert.notEqual(config.app.withGlobalTauri, true);
});

test("a aba não rouba foco e a gaveta começa recolhida", () => {
  const [notch, drawer] = config.app.windows;

  assert.equal(notch.focus, false);
  assert.equal(notch.focusable, false);
  assert.equal(notch.acceptFirstMouse, true);
  assert.equal(notch.transparent, true);
  assert.equal(notch.alwaysOnTop, true);
  assert.equal(notch.skipTaskbar, true);

  assert.equal(drawer.visible, false);
  assert.equal(drawer.transparent, true);
  assert.equal(drawer.alwaysOnTop, true);
  assert.equal(drawer.decorations, false);
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
