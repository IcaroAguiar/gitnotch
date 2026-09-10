import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const config = JSON.parse(
  await readFile(
    new URL("../src-tauri/tauri.conf.json", import.meta.url),
    "utf8",
  ),
);

test("a janela inicial carrega somente o frontend empacotado", () => {
  assert.equal(config.build.frontendDist, "../dist");
  assert.equal(config.build.devUrl, undefined);
  assert.equal(config.app.windows.length, 1);
  assert.equal(config.app.windows[0].url, undefined);
  assert.notEqual(config.app.withGlobalTauri, true);
});

test("a prévia não concede capacidades nativas nem conexão de rede ao frontend", () => {
  assert.deepEqual(config.app.security.capabilities, []);
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
