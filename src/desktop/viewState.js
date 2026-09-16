/**
 * @typedef {"closed" | "preview" | "pinned"} DrawerIntent
 * @typedef {"resting" | "opening" | "closing"} DrawerPhase
 * @typedef {{ intent: DrawerIntent, phase: DrawerPhase, generation: number }} DesktopView
 */

/**
 * Aceita somente uma visão que não faça a interface retornar no tempo.
 *
 * @param {DesktopView} current
 * @param {DesktopView} candidate
 */
export function shouldAcceptView(current, candidate) {
  if (candidate.generation !== current.generation) {
    return candidate.generation > current.generation;
  }

  if (candidate.intent !== current.intent) {
    return false;
  }

  return current.phase !== "resting" || candidate.phase === "resting";
}
