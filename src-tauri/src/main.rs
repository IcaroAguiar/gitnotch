#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod commands;
mod desktop;
mod git;
mod settings;

use std::sync::Mutex;
use tauri::Manager;

use crate::app_state::{AppState, Workspace};
use crate::desktop::DesktopState;

fn application_context<R: tauri::Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(DesktopState::new()))
        .setup(|app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .map_err(|e| format!("Diretório de configuração indisponível: {e}"))?;

            app.manage(AppState::new(Workspace::load(config_dir)));
            desktop::place_notch(app.handle())?;
            if let Err(error) = desktop::install_material(app.handle()) {
                eprintln!("material nativo indisponível: {error}");
            }
            desktop::spawn_hover_watcher(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == desktop::NOTCH_LABEL
                && matches!(event, tauri::WindowEvent::Focused(false))
            {
                desktop::blur_drawer(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_workspace_view,
            commands::select_root,
            commands::remove_root,
            commands::get_repo_status,
            commands::get_file_diff,
            commands::get_git_capabilities,
            desktop::toggle_drawer,
            desktop::collapse_drawer,
            desktop::get_desktop_capabilities,
            desktop::set_drawer_interaction,
            desktop::refresh_desktop_appearance
        ])
        .run(application_context())
        .expect("Não foi possível iniciar o Git Notch");
}
