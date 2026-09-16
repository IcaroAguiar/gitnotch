import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  type PointerEvent,
  type RefObject,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { shouldAcceptView } from "./viewState";

type DrawerIntent = "closed" | "preview" | "pinned";
type DrawerPhase = "resting" | "opening" | "closing";

type DesktopView = {
  intent: DrawerIntent;
  phase: DrawerPhase;
  generation: number;
};

type DesktopCapabilities = {
  drawer: DesktopView;
  material: string;
};

type DesktopAppearance = {
  material: string;
};

type InteractionGuards = {
  selection: boolean;
  pointerCapture: boolean;
};

const closedView: DesktopView = {
  intent: "closed",
  phase: "resting",
  generation: 1,
};

const noInteraction: InteractionGuards = {
  selection: false,
  pointerCapture: false,
};

function useInteractionGuards(
  open: boolean,
  contentRef: RefObject<HTMLElement | null>,
) {
  const guardsRef = useRef<InteractionGuards>(noInteraction);

  const updateGuards = useCallback((patch: Partial<InteractionGuards>) => {
    const next = { ...guardsRef.current, ...patch };
    if (
      next.selection === guardsRef.current.selection &&
      next.pointerCapture === guardsRef.current.pointerCapture
    ) {
      return;
    }

    guardsRef.current = next;
    void invoke("set_drawer_interaction", { interactionGuards: next }).catch(
      () => undefined,
    );
  }, []);

  useEffect(() => {
    if (!open) {
      updateGuards(noInteraction);
      return;
    }

    const syncSelection = () => {
      const selection = window.getSelection();
      const content = contentRef.current;
      const hasSelection = Boolean(
        content &&
          selection &&
          selection.type === "Range" &&
          selection.rangeCount > 0 &&
          selection.getRangeAt(0).intersectsNode(content),
      );
      updateGuards({ selection: hasSelection });
    };

    document.addEventListener("selectionchange", syncSelection);
    syncSelection();

    return () => {
      document.removeEventListener("selectionchange", syncSelection);
      updateGuards(noInteraction);
    };
  }, [contentRef, open, updateGuards]);

  useEffect(() => {
    if (!open) return;

    const clearPointerCapture = () => {
      updateGuards({ pointerCapture: false });
    };

    window.addEventListener("pointerup", clearPointerCapture, true);
    window.addEventListener("pointercancel", clearPointerCapture, true);
    window.addEventListener("blur", clearPointerCapture);

    return () => {
      window.removeEventListener("pointerup", clearPointerCapture, true);
      window.removeEventListener("pointercancel", clearPointerCapture, true);
      window.removeEventListener("blur", clearPointerCapture);
    };
  }, [open, updateGuards]);

  const capturePointer = useCallback(
    (event: PointerEvent<HTMLElement>) => {
      const target = event.target;
      if (
        target instanceof Element &&
        target.hasPointerCapture(event.pointerId)
      ) {
        updateGuards({ pointerCapture: true });
      }
    },
    [updateGuards],
  );

  const releasePointer = useCallback(() => {
    updateGuards({ pointerCapture: false });
  }, [updateGuards]);

  return {
    onGotPointerCapture: capturePointer,
    onLostPointerCapture: releasePointer,
  };
}

export function DesktopRoot() {
  const [view, setView] = useState<DesktopView>(closedView);
  const [material, setMaterial] = useState("solid");
  const contentRef = useRef<HTMLElement | null>(null);

  const acceptView = useCallback((candidate: DesktopView) => {
    setView((current) =>
      shouldAcceptView(current, candidate) ? candidate : current,
    );
  }, []);

  const collapseDrawer = useCallback(() => {
    void invoke<DesktopView>("collapse_drawer")
      .then(acceptView)
      .catch(() => undefined);
  }, [acceptView]);

  useEffect(() => {
    let active = true;
    const refreshAppearance = () => {
      void invoke<DesktopAppearance>("refresh_desktop_appearance")
        .then((appearance) => {
          if (active) setMaterial(appearance.material);
        })
        .catch(() => undefined);
    };
    const unlisten = listen<DesktopView>("gitnotch://drawer-state", (event) => {
      if (active) acceptView(event.payload);
    });
    const queries = [
      window.matchMedia("(prefers-reduced-motion: reduce)"),
      window.matchMedia("(prefers-reduced-transparency: reduce)"),
      window.matchMedia("(prefers-color-scheme: light)"),
    ];

    void invoke<DesktopCapabilities>("get_desktop_capabilities")
      .then((capabilities) => {
        if (!active) return;
        acceptView(capabilities.drawer);
        setMaterial(capabilities.material);
      })
      .catch(() => undefined);
    refreshAppearance();
    for (const query of queries) {
      query.addEventListener("change", refreshAppearance);
    }

    return () => {
      active = false;
      for (const query of queries) {
        query.removeEventListener("change", refreshAppearance);
      }
      void unlisten.then((off) => off()).catch(() => undefined);
    };
  }, [acceptView]);

  useEffect(() => {
    const root = document.documentElement;
    root.dataset.intent = view.intent;
    root.dataset.phase = view.phase;
    root.dataset.material = material;
  }, [material, view]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (
        event.defaultPrevented ||
        event.isComposing ||
        event.key !== "Escape" ||
        view.intent !== "pinned"
      ) {
        return;
      }

      event.preventDefault();
      collapseDrawer();
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [collapseDrawer, view.intent]);

  const open = view.intent !== "closed";
  const interactionHandlers = useInteractionGuards(open, contentRef);
  const ribbonLabel =
    view.intent === "pinned"
      ? "Recolher a gaveta do Git Notch"
      : view.intent === "preview"
        ? "Fixar a prévia da gaveta do Git Notch"
        : "Abrir e fixar a gaveta do Git Notch";

  const toggleDrawer = useCallback(() => {
    void invoke<DesktopView>("toggle_drawer")
      .then(acceptView)
      .catch(() => undefined);
  }, [acceptView]);

  return (
    <div className="surface">
      <button
        className="ribbon"
        type="button"
        onClick={toggleDrawer}
        aria-label={ribbonLabel}
        aria-pressed={view.intent === "pinned"}
      >
        <svg
          className="ribbon-glyph"
          viewBox="0 0 16 16"
          aria-hidden="true"
          focusable="false"
        >
          <path d="M5 3v10m0-5h5m0 0V5" />
          <circle cx="5" cy="3" r="1.25" />
          <circle cx="5" cy="13" r="1.25" />
          <circle cx="10" cy="5" r="1.25" />
        </svg>
      </button>
      <section
        ref={contentRef}
        className="drawer-content"
        aria-hidden={!open}
        aria-label="Gaveta do Git Notch"
        {...interactionHandlers}
      >
        <header className="drawer-header">
          <div>
            <span className="wordmark">Git Notch</span>
            <span className="drawer-kicker">Local</span>
          </div>
          <div className="drawer-actions">
            <button
              className="drawer-pin"
              type="button"
              aria-pressed={view.intent === "pinned"}
              onClick={toggleDrawer}
            >
              {view.intent === "pinned" ? "Fixada" : "Fixar"}
            </button>
            <button
              className="drawer-close"
              type="button"
              onClick={collapseDrawer}
            >
              Recolher
            </button>
          </div>
        </header>
        <main className="drawer-empty">
          <div className="empty-copy">
            <svg
              className="empty-icon"
              viewBox="0 0 32 32"
              aria-hidden="true"
              focusable="false"
            >
              <path d="M5.5 8.5h8l2.25 3H26.5v12H5.5z" />
              <path d="M10 15v5m0-2.5h7m0 0V15" />
              <circle cx="10" cy="15" r="1.35" />
              <circle cx="10" cy="20" r="1.35" />
              <circle cx="17" cy="15" r="1.35" />
            </svg>
            <h1>Nada para revisar</h1>
            <p>Os repositórios observados aparecerão aqui.</p>
          </div>
        </main>
        <footer className="drawer-footer">Somente leitura</footer>
      </section>
    </div>
  );
}
