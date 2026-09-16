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
        hidden={open}
        aria-label="Abrir e fixar a gaveta do Git Notch"
      >
        <span className="ribbon-mark" aria-hidden="true" />
      </button>
      <section
        ref={contentRef}
        className="drawer-content"
        aria-hidden={!open}
        aria-label="Gaveta do Git Notch"
        {...interactionHandlers}
      >
        <div className="drawer-rail" aria-hidden="true">
          <svg viewBox="0 0 20 96" focusable="false">
            <title>Detalhe da gaveta</title>
            <path d="M10 0v38c0 7.6 4.2 11.4 10 11.4M10 96V58.6c0-7.6 4.2-11.4 10-11.4" />
            <circle cx="10" cy="48" r="2" />
          </svg>
        </div>
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
          <div className="empty-mark" aria-hidden="true" />
          <p className="empty-eyebrow">Git Notch</p>
          <h1>Nada para revisar</h1>
          <p>Os repositórios observados aparecerão aqui.</p>
        </main>
      </section>
    </div>
  );
}
