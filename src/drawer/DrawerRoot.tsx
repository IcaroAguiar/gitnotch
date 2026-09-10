import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

type DrawerView = {
  drawer: "open" | "closed";
  generation: number;
};

type DesktopCapabilities = {
  drawer: DrawerView;
};

export function DrawerRoot() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    let active = true;

    const unlisten = listen<DrawerView>("gitnotch://drawer-state", (event) => {
      setOpen(event.payload.drawer === "open");
    });

    invoke<DesktopCapabilities>("get_desktop_capabilities")
      .then((capabilities) => {
        if (active) setOpen(capabilities.drawer.drawer === "open");
      })
      .catch(() => undefined);

    return () => {
      active = false;
      void unlisten.then((off) => off()).catch(() => undefined);
    };
  }, []);

  function collapse() {
    void invoke("toggle_drawer").catch(() => undefined);
  }

  return (
    <section
      className={open ? "drawer drawer--open" : "drawer"}
      aria-label="Gaveta do Git Notch"
      aria-hidden={!open}
    >
      <header className="drawer-header">
        <span className="wordmark">Git Notch</span>
        <button className="drawer-close" type="button" onClick={collapse}>
          Recolher
        </button>
      </header>
      <div className="drawer-body">
        <p>
          A gaveta está pronta para receber a navegação de repositórios nas
          próximas etapas.
        </p>
      </div>
    </section>
  );
}
