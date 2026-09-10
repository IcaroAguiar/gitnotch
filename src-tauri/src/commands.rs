use std::path::{Component, Path};

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::app_state::{AppState, WorkspaceView};
use crate::git::{DiffPatch, FileGroupKind, GitCapabilities, RepoStatusSnapshot};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEnvelope {
    pub workspace_epoch: u64,
    pub status: RepoStatusSnapshot,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffEnvelope {
    pub workspace_epoch: u64,
    pub patch: DiffPatch,
}

fn ensure_relative_path(value: &str) -> Result<(), String> {
    let path = Path::new(value);
    let rejected = value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)));

    if rejected {
        return Err(format!("Caminho relativo inválido: '{value}'"));
    }

    Ok(())
}

fn resolve_for_read(
    state: &State<AppState>,
    root_id: &str,
    expected_epoch: u64,
) -> Result<(std::path::PathBuf, u64), String> {
    state.with_workspace(|workspace| {
        workspace.ensure_epoch(expected_epoch)?;
        let path = workspace.resolve_root(root_id)?;
        Ok((path, workspace.epoch()))
    })
}

#[tauri::command]
pub fn get_workspace_view(state: State<AppState>) -> Result<WorkspaceView, String> {
    state.with_workspace(|workspace| Ok(workspace.view()))
}

#[tauri::command(async)]
pub fn select_root<R: tauri::Runtime>(
    app: AppHandle<R>,
    state: State<AppState>,
) -> Result<Option<WorkspaceView>, String> {
    let Some(selected) = app.dialog().file().blocking_pick_folder() else {
        return Ok(None);
    };

    let path = selected
        .into_path()
        .map_err(|e| format!("Seleção de pasta inválida: {e}"))?;

    state
        .with_workspace(|workspace| workspace.authorize_root(&path))
        .map(Some)
}

#[tauri::command]
pub fn remove_root(root_id: String, state: State<AppState>) -> Result<WorkspaceView, String> {
    state.with_workspace(|workspace| workspace.remove_root(&root_id))
}

#[tauri::command]
pub fn get_repo_status(
    root_id: String,
    expected_epoch: u64,
    state: State<AppState>,
) -> Result<StatusEnvelope, String> {
    let (path, epoch) = resolve_for_read(&state, &root_id, expected_epoch)?;
    let status = state.with_git_reader(|reader| reader.status(&path))?;

    Ok(StatusEnvelope {
        workspace_epoch: epoch,
        status,
    })
}

#[tauri::command]
pub fn get_file_diff(
    root_id: String,
    rel_path: String,
    orig_path: Option<String>,
    group: FileGroupKind,
    expected_epoch: u64,
    state: State<AppState>,
) -> Result<DiffEnvelope, String> {
    ensure_relative_path(&rel_path)?;
    if let Some(orig) = orig_path.as_deref() {
        ensure_relative_path(orig)?;
    }

    let (path, epoch) = resolve_for_read(&state, &root_id, expected_epoch)?;
    let patch = state
        .with_git_reader(|reader| reader.diff(&path, &rel_path, orig_path.as_deref(), group))?;

    Ok(DiffEnvelope {
        workspace_epoch: epoch,
        patch,
    })
}

#[tauri::command]
pub fn get_git_capabilities(state: State<AppState>) -> Result<GitCapabilities, String> {
    state.with_git_reader(|reader| Ok(reader.capabilities().clone()))
}

#[cfg(test)]
mod tests;
