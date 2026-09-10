#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod desktop;
mod git;

use std::path::Path;
use std::sync::Mutex;
use tauri::{Manager, State};

use crate::desktop::DesktopState;
use crate::git::{
    DiffPatch, FileGroupKind, GitCapabilities, GitError, GitReader, RepoStatusSnapshot,
};

struct AppState {
    git_reader: Mutex<Option<GitReader>>,
}

impl AppState {
    fn with_git_reader<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&GitReader) -> Result<T, GitError>,
    {
        let mut guard = self
            .git_reader
            .lock()
            .map_err(|e| format!("Falha de sincronização interna: {e}"))?;

        if guard.is_none() {
            *guard = Some(GitReader::new().map_err(|e| e.to_string())?);
        }

        f(guard.as_ref().unwrap()).map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn get_git_capabilities(state: State<AppState>) -> Result<GitCapabilities, String> {
    state.with_git_reader(|reader| Ok(reader.capabilities().clone()))
}

#[tauri::command]
fn get_repo_status(
    checkout_path: String,
    state: State<AppState>,
) -> Result<RepoStatusSnapshot, String> {
    let path = Path::new(&checkout_path);
    if !path.is_dir() {
        return Err(
            GitError::InvalidPath(format!("Diretório não encontrado: '{checkout_path}'"))
                .to_string(),
        );
    }

    state.with_git_reader(|reader| reader.status(path))
}

#[tauri::command]
fn get_file_diff(
    checkout_path: String,
    rel_path: String,
    orig_path: Option<String>,
    group: FileGroupKind,
    state: State<AppState>,
) -> Result<DiffPatch, String> {
    let path = Path::new(&checkout_path);
    if !path.is_dir() {
        return Err(
            GitError::InvalidPath(format!("Diretório não encontrado: '{checkout_path}'"))
                .to_string(),
        );
    }

    state.with_git_reader(|reader| reader.diff(path, &rel_path, orig_path.as_deref(), group))
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            git_reader: Mutex::new(None),
        })
        .manage(Mutex::new(DesktopState::new()))
        .setup(|app| {
            desktop::place_notch(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == desktop::DRAWER_LABEL
                && matches!(event, tauri::WindowEvent::Focused(false))
            {
                desktop::collapse_drawer(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_git_capabilities,
            get_repo_status,
            get_file_diff,
            desktop::toggle_drawer,
            desktop::get_desktop_capabilities
        ])
        .run(tauri::generate_context!())
        .expect("Não foi possível iniciar o Git Notch");
}
