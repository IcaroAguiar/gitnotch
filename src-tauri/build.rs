fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_workspace_view",
            "select_root",
            "remove_root",
            "get_repo_status",
            "get_file_diff",
            "get_git_capabilities",
            "toggle_drawer",
            "collapse_drawer",
            "get_desktop_capabilities",
            "set_drawer_interaction",
            "refresh_desktop_appearance",
        ]),
    ))
    .expect("não foi possível gerar as permissões dos comandos do aplicativo");
}
