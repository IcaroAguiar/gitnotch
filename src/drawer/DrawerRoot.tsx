import { invoke } from "@tauri-apps/api/core";

export function DrawerRoot() {
  function collapse() {
    void invoke("toggle_drawer").catch(() => undefined);
  }

  return (
    <section className="drawer" aria-label="Gaveta do Git Notch">
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
