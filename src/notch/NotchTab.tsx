import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

export function NotchTab() {
  const [pending, setPending] = useState(false);

  function toggleDrawer() {
    if (pending) return;
    setPending(true);
    void invoke("toggle_drawer")
      .catch(() => undefined)
      .finally(() => setPending(false));
  }

  return (
    <button
      className="notch-tab"
      type="button"
      onClick={toggleDrawer}
      aria-label="Abrir a gaveta do Git Notch"
    >
      <span className="notch-tab-dot" aria-hidden="true" />
    </button>
  );
}
