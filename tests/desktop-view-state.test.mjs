import assert from "node:assert/strict";
import test from "node:test";

import { shouldAcceptView } from "../src/desktop/viewState.js";

const previewOpening = {
  intent: "preview",
  phase: "opening",
  generation: 7,
};

test("uma visão de geração anterior não sobrescreve a intenção atual", () => {
  assert.equal(
    shouldAcceptView(
      { intent: "pinned", phase: "opening", generation: 8 },
      previewOpening,
    ),
    false,
  );
});

test("uma capacidade atrasada não reabre fase após a mesma geração assentar", () => {
  assert.equal(
    shouldAcceptView(
      { intent: "preview", phase: "resting", generation: 7 },
      previewOpening,
    ),
    false,
  );
  assert.equal(
    shouldAcceptView(previewOpening, {
      intent: "preview",
      phase: "resting",
      generation: 7,
    }),
    true,
  );
});

test("uma intenção diferente exige uma geração nova", () => {
  assert.equal(
    shouldAcceptView(
      { intent: "pinned", phase: "resting", generation: 7 },
      { intent: "preview", phase: "resting", generation: 7 },
    ),
    false,
  );
});
