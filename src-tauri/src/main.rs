#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod git;

use std::path::Path;
use std::sync::Mutex;
use tauri::State;

use crate::git::{
    DiffPatch, FileGroupKind, GitCapabilities, GitError, GitReader, RepoStatusSnapshot,
};

struct AppState {
    git_reader: Mutex<Option<GitReader>>,
}

#[tauri::command]
fn get_git_capabilities(state: State<AppState>) -> Result<GitCapabilities, String> {
    let mut reader_guard = state
        .git_reader
        .lock()
        .map_err(|e| format!("Falha de sincronização interna: {e}"))?;

    if reader_guard.is_none() {
        let reader = GitReader::new().map_err(|e| e.to_string())?;
        *reader_guard = Some(reader);
    }

    Ok(reader_guard.as_ref().unwrap().capabilities().clone())
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

    let mut reader_guard = state
        .git_reader
        .lock()
        .map_err(|e| format!("Falha de sincronização interna: {e}"))?;

    if reader_guard.is_none() {
        let reader = GitReader::new().map_err(|e| e.to_string())?;
        *reader_guard = Some(reader);
    }

    reader_guard
        .as_ref()
        .unwrap()
        .status(path)
        .map_err(|e| e.to_string())
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

    let mut reader_guard = state
        .git_reader
        .lock()
        .map_err(|e| format!("Falha de sincronização interna: {e}"))?;

    if reader_guard.is_none() {
        let reader = GitReader::new().map_err(|e| e.to_string())?;
        *reader_guard = Some(reader);
    }

    reader_guard
        .as_ref()
        .unwrap()
        .diff(path, &rel_path, orig_path.as_deref(), group)
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            git_reader: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_git_capabilities,
            get_repo_status,
            get_file_diff
        ])
        .run(tauri::generate_context!())
        .expect("Não foi possível iniciar o Git Notch");
}
