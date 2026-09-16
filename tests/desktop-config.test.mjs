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
const mainCapability = JSON.parse(
  await readFile(
    new URL("../src-tauri/capabilities/main.json", import.meta.url),
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

test("a janela inicial carrega somente o frontend empacotado", () => {
  assert.equal(config.build.frontendDist, "../dist");
  assert.equal(config.build.devUrl, undefined);
  assert.equal(config.app.windows.length, 1);
  assert.equal(config.app.windows[0].url, undefined);
  assert.notEqual(config.app.withGlobalTauri, true);
});

test("o webview principal recebe somente os comandos locais declarados", () => {
  assert.deepEqual(config.app.security.capabilities, ["main"]);
  assert.deepEqual(mainCapability.windows, ["main"]);
  assert.deepEqual(mainCapability.permissions, [
    "allow-get-workspace-view",
    "allow-select-root",
    "allow-remove-root",
    "allow-get-repo-status",
    "allow-get-file-diff",
    "allow-get-git-capabilities",
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
