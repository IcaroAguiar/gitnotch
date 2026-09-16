import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { transformWithOxc } from "vite";

const config = JSON.parse(
  await readFile(
    new URL("../src-tauri/tauri.conf.json", import.meta.url),
    "utf8",
  ),
);
const notchCapability = JSON.parse(
  await readFile(
    new URL("../src-tauri/capabilities/notch.json", import.meta.url),
    "utf8",
  ),
);
const workspaceEpochSource = await readFile(
  new URL("../src/settings/workspaceEpoch.ts", import.meta.url),
  "utf8",
);
const workspaceEpochModule = await import(
  `data:text/javascript;base64,${Buffer.from(
    (
      await transformWithOxc(workspaceEpochSource, "workspaceEpoch.ts", {
        lang: "ts",
        module: "esnext",
        target: "es2022",
      })
    ).code,
  ).toString("base64")}`
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

  assert.equal(notch.width, 28);
  assert.equal(notch.height, 112);
  assert.equal(notch.focus, false);
  assert.equal(notch.focusable, true);
  assert.equal(notch.acceptFirstMouse, true);
  assert.equal(notch.transparent, true);
  assert.equal(notch.alwaysOnTop, true);
  assert.equal(notch.skipTaskbar, true);
  assert.equal(notch.decorations, false);
  assert.equal(notch.shadow, false);
});

test("o único webview local recebe a allowlist explícita de IPC", () => {
  assert.deepEqual(config.app.security.capabilities, ["notch"]);
  assert.equal(config.app.macOSPrivateApi, true);
  assert.deepEqual(notchCapability.windows, ["notch"]);
  assert.deepEqual(notchCapability.permissions, [
    "core:event:allow-listen",
    "core:event:allow-unlisten",
    "allow-get-workspace-view",
    "allow-select-root",
    "allow-remove-root",
    "allow-get-repo-status",
    "allow-get-file-diff",
    "allow-get-git-capabilities",
    "allow-toggle-drawer",
    "allow-collapse-drawer",
    "allow-get-desktop-capabilities",
    "allow-set-drawer-interaction",
    "allow-refresh-desktop-appearance",
  ]);
});

test("a CSP permite somente o transporte IPC local necessário", () => {
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

test("a aceitação de WorkspaceView não deixa resposta de época antiga substituir a nova", () => {
  const acceptor = new workspaceEpochModule.WorkspaceEpochAcceptor();

  assert.equal(acceptor.accept(4), true);
  assert.equal(acceptor.accept(6), true);
  assert.equal(acceptor.accept(5), false);
  assert.equal(acceptor.current(), 6);
  assert.equal(acceptor.accept(6), true);
  assert.equal(acceptor.accept(7), true);
  assert.equal(acceptor.current(), 7);
});
