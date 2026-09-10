#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod commands;
mod git;
mod settings;

use tauri::Manager;

use crate::app_state::{AppState, Workspace};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .map_err(|e| format!("Diretório de configuração indisponível: {e}"))?;

            app.manage(AppState::new(Workspace::load(config_dir)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_workspace_view,
            commands::select_root,
            commands::remove_root,
            commands::get_repo_status,
            commands::get_file_diff,
            commands::get_git_capabilities
        ])
        .run(tauri::generate_context!())
        .expect("Não foi possível iniciar o Git Notch");
}
