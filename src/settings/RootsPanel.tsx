import { useCallback, useEffect, useRef, useState } from "react";
import {
  getRepoStatus,
  getWorkspaceView,
  type RepoStatusSnapshot,
  removeRoot,
  selectRoot,
  type WorkspaceView,
} from "./workspaceBridge";

type StatusState =
  | { kind: "loading" }
  | { kind: "ready"; summary: string }
  | { kind: "error"; message: string };

function branchLabel(status: RepoStatusSnapshot): string {
  const { branch } = status;
  if (branch.isUnborn) return "sem commits";
  if (branch.isDetached) return "HEAD destacado";
  return branch.head;
}

function describeStatus(status: RepoStatusSnapshot): string {
  const counts: ReadonlyArray<readonly [number, string]> = [
    [status.staged.length, "staged"],
    [status.unstaged.length, "modificados"],
    [status.untracked.length, "novos"],
    [status.conflicts.length, "conflitos"],
  ];
  const parts = counts
    .filter(([count]) => count > 0)
    .map(([count, label]) => `${count} ${label}`);
  const changes = parts.length > 0 ? parts.join(" · ") : "sem alterações";
  return `${branchLabel(status)} · ${changes}`;
}

function friendlyError(message: string): string {
  if (message.includes("not a git repository")) {
    return "Não é um repositório Git.";
  }
  if (message.includes("obsoleta")) {
    return "Resposta obsoleta descartada após mudança nas raízes.";
  }
  if (message.includes("indisponível")) {
    return "Pasta indisponível no disco.";
  }
  return message;
}

function RootStatusLine({ status }: { status?: StatusState }) {
  if (!status || status.kind === "loading") {
    return <span className="status">Verificando a pasta…</span>;
  }
  if (status.kind === "error") {
    return <span className="status status-error">{status.message}</span>;
  }
  return <span className="status">{status.summary}</span>;
}

export function RootsPanel() {
  const [view, setView] = useState<WorkspaceView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [statuses, setStatuses] = useState<Record<string, StatusState>>({});
  const epochRef = useRef<number | null>(null);

  const loadView = useCallback(async () => {
    try {
      const next = await getWorkspaceView();
      epochRef.current = next.epoch;
      setView(next);
      setError(null);
    } catch (cause) {
      setError(String(cause));
    }
  }, []);

  useEffect(() => {
    void loadView();
  }, [loadView]);

  useEffect(() => {
    if (!view) return;
    const current = view;
    epochRef.current = current.epoch;
    let active = true;

    setStatuses(
      Object.fromEntries(
        current.roots.map((root) => [root.id, { kind: "loading" as const }]),
      ),
    );

    void (async () => {
      const entries = await Promise.all(
        current.roots.map(async (root): Promise<[string, StatusState]> => {
          if (!root.available) {
            return [
              root.id,
              { kind: "error", message: "Pasta indisponível no disco." },
            ];
          }
          try {
            const response = await getRepoStatus(root.id, current.epoch);
            if (response.workspaceEpoch !== current.epoch) {
              return [
                root.id,
                { kind: "error", message: "Resposta obsoleta descartada." },
              ];
            }
            return [
              root.id,
              { kind: "ready", summary: describeStatus(response.status) },
            ];
          } catch (cause) {
            return [
              root.id,
              { kind: "error", message: friendlyError(String(cause)) },
            ];
          }
        }),
      );
      if (!active || epochRef.current !== current.epoch) return;
      setStatuses(Object.fromEntries(entries));
    })();

    return () => {
      active = false;
    };
  }, [view]);

  const handleAdd = async () => {
    setBusy(true);
    try {
      const next = await selectRoot();
      if (next) {
        epochRef.current = next.epoch;
        setView(next);
      }
      setError(null);
    } catch (cause) {
      setError(String(cause));
    } finally {
      setBusy(false);
    }
  };

  const handleRemove = async (rootId: string) => {
    setBusy(true);
    try {
      const next = await removeRoot(rootId);
      epochRef.current = next.epoch;
      setView(next);
      setError(null);
    } catch (cause) {
      setError(String(cause));
    } finally {
      setBusy(false);
    }
  };

  return (
    <main>
      <header>
        <span className="wordmark">Git Notch</span>
        <h1>Pastas autorizadas</h1>
        <p>
          O aplicativo lê repositórios dentro destas pastas sem alterar nada
          neles.
        </p>
      </header>

      <div className="panel-actions">
        <button type="button" onClick={() => void handleAdd()} disabled={busy}>
          Adicionar pasta
        </button>
      </div>

      {view?.health ? (
        <p className="warning" role="status">
          {view.health}
        </p>
      ) : null}

      {error ? (
        <p className="error" role="alert">
          {error}
        </p>
      ) : null}

      {view && view.roots.length === 0 ? (
        <p className="empty">Nenhuma pasta autorizada ainda.</p>
      ) : null}

      <ul className="root-list">
        {view?.roots.map((root) => (
          <li key={root.id} className="root-item">
            <div className="root-heading">
              <span className="root-name">{root.displayName}</span>
              <button
                type="button"
                className="remove"
                onClick={() => void handleRemove(root.id)}
                disabled={busy}
              >
                Remover
              </button>
            </div>
            <span className="root-path" title={root.displayPath}>
              {root.displayPath}
            </span>
            <RootStatusLine status={statuses[root.id]} />
          </li>
        ))}
      </ul>

      <footer>Prévia de desenvolvimento · 0.1.0</footer>
    </main>
  );
}
