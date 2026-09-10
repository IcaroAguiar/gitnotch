import { invoke } from "@tauri-apps/api/core";

export type RootSummary = {
  id: string;
  displayName: string;
  displayPath: string;
  available: boolean;
};

export type WorkspaceView = {
  epoch: number;
  roots: RootSummary[];
  health: string | null;
};

export type FileGroupKind = "staged" | "unstaged" | "untracked" | "conflicted";

export type BranchInfo = {
  oid?: string;
  head: string;
  upstream?: string;
  ahead?: number;
  behind?: number;
  isDetached: boolean;
  isUnborn: boolean;
};

export type RepoStatusSnapshot = {
  branch: BranchInfo;
  staged: unknown[];
  unstaged: unknown[];
  untracked: unknown[];
  conflicts: unknown[];
};

export type StatusEnvelope = {
  workspaceEpoch: number;
  status: RepoStatusSnapshot;
};

export type DiffPatch = {
  path: string;
  origPath?: string;
  group: FileGroupKind;
  patch: string;
  isBinary: boolean;
  isTooLarge: boolean;
  fileSizeBytes?: number;
};

export type DiffEnvelope = {
  workspaceEpoch: number;
  patch: DiffPatch;
};

export type GitCapabilities = {
  installed: boolean;
  version: string;
  supportsPorcelainV2: boolean;
  executablePath: string;
};

export function getWorkspaceView(): Promise<WorkspaceView> {
  return invoke("get_workspace_view");
}

export function selectRoot(): Promise<WorkspaceView | null> {
  return invoke("select_root");
}

export function removeRoot(rootId: string): Promise<WorkspaceView> {
  return invoke("remove_root", { rootId });
}

export function getRepoStatus(
  rootId: string,
  expectedEpoch: number,
): Promise<StatusEnvelope> {
  return invoke("get_repo_status", { rootId, expectedEpoch });
}

export function getFileDiff(
  rootId: string,
  relPath: string,
  origPath: string | null,
  group: FileGroupKind,
  expectedEpoch: number,
): Promise<DiffEnvelope> {
  return invoke("get_file_diff", {
    rootId,
    relPath,
    origPath,
    group,
    expectedEpoch,
  });
}

export function getGitCapabilities(): Promise<GitCapabilities> {
  return invoke("get_git_capabilities");
}
