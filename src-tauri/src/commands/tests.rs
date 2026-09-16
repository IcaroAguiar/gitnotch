use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tauri::{Manager, WebviewWindow, WebviewWindowBuilder};

use super::*;
use crate::app_state::Workspace;

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let count = FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("gitnotch-ipc-{prefix}-{count}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn child(&self, name: &str) -> PathBuf {
        let child = self.path.join(name);
        fs::create_dir_all(&child).unwrap();
        child
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn init_repo(path: &Path) {
    let run = |args: &[&str]| {
        let status = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .status()
            .expect("falha ao executar git na fixture");
        assert!(status.success(), "git {:?} falhou", args);
    };

    fs::create_dir_all(path).unwrap();
    run(&["init", "-b", "main"]);
    run(&["config", "user.name", "Test User"]);
    run(&["config", "user.email", "test@example.com"]);
    fs::write(path.join("rastreado.txt"), b"versao inicial\n").unwrap();
    run(&["add", "rastreado.txt"]);
    run(&["commit", "-m", "commit inicial"]);
    fs::write(path.join("rastreado.txt"), b"versao modificada\n").unwrap();
    fs::write(path.join("novo.txt"), b"conteudo novo\n").unwrap();
}

fn snapshot_directory_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut map = BTreeMap::new();
    collect_files_recursive(root, root, &mut map);
    map
}

fn collect_files_recursive(base: &Path, current: &Path, map: &mut BTreeMap<PathBuf, Vec<u8>>) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel) = path.strip_prefix(base)
                    && let Ok(bytes) = fs::read(&path)
                {
                    map.insert(rel.to_path_buf(), bytes);
                }
            } else if path.is_dir() {
                collect_files_recursive(base, &path, map);
            }
        }
    }
}

fn invoke(
    webview: &WebviewWindow<MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    let url = if cfg!(any(windows, target_os = "android")) {
        "http://tauri.localhost"
    } else {
        "tauri://localhost"
    };

    let response = get_ipc_response(
        webview,
        InvokeRequest {
            cmd: cmd.to_string(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: url.parse().unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    );

    match response {
        Ok(body) => Ok(body.deserialize::<serde_json::Value>().unwrap()),
        Err(err) => Err(err),
    }
}

struct TestApp {
    app: tauri::App<MockRuntime>,
    webview: WebviewWindow<MockRuntime>,
    _dir: TempDir,
    repo: PathBuf,
}

impl TestApp {
    fn new() -> Self {
        let dir = TempDir::new("app");
        let repo = dir.child("repo");
        init_repo(&repo);

        let workspace = Workspace::load(dir.path().join("config"));
        let mut context = crate::application_context();
        context.config_mut().app.windows.clear();
        let app = mock_builder()
            .invoke_handler(tauri::generate_handler![
                get_workspace_view,
                select_root,
                remove_root,
                get_repo_status,
                get_file_diff,
                get_git_capabilities
            ])
            .build(context)
            .expect("app de teste");
        app.manage(AppState::new(workspace));
        let webview = WebviewWindowBuilder::new(&app, "notch", Default::default())
            .build()
            .expect("webview de teste");

        Self {
            app,
            webview,
            _dir: dir,
            repo,
        }
    }

    fn authorize_repo(&self) -> (String, u64) {
        let view = self
            .app
            .state::<AppState>()
            .with_workspace(|workspace| workspace.authorize_root(&self.repo))
            .expect("autorização da fixture");
        (view.roots[0].id.clone(), view.epoch)
    }
}

#[test]
fn custom_commands_are_limited_to_the_notch_webview_and_declared_allowlist() {
    let test = TestApp::new();

    invoke(&test.webview, "get_workspace_view", serde_json::json!({}))
        .expect("o webview notch deve receber o comando declarado");

    let other_webview = WebviewWindowBuilder::new(&test.app, "untrusted", Default::default())
        .build()
        .expect("webview de teste sem capability");
    let wrong_window = invoke(&other_webview, "get_workspace_view", serde_json::json!({}))
        .expect_err("um webview fora da capability não pode invocar comandos do aplicativo");
    assert!(
        wrong_window
            .as_str()
            .unwrap_or_default()
            .contains("not allowed on window"),
        "erro inesperado: {wrong_window}"
    );

    let undeclared = invoke(&test.webview, "undeclared_command", serde_json::json!({}))
        .expect_err("um comando fora da allowlist deve ser recusado antes do handler");
    assert!(
        undeclared
            .as_str()
            .unwrap_or_default()
            .contains("not allowed"),
        "erro inesperado: {undeclared}"
    );
}

#[test]
fn arbitrary_path_is_rejected_by_ipc() {
    let test = TestApp::new();
    let (root_id, epoch) = test.authorize_repo();

    let raw_path = test.repo.to_string_lossy().into_owned();
    let error = invoke(
        &test.webview,
        "get_repo_status",
        serde_json::json!({ "rootId": raw_path, "expectedEpoch": epoch }),
    )
    .expect_err("caminho absoluto não pode ser aceito como identificador");

    assert!(
        error
            .as_str()
            .unwrap_or_default()
            .contains("Raiz desconhecida"),
        "erro inesperado: {error}"
    );

    let response = invoke(
        &test.webview,
        "get_repo_status",
        serde_json::json!({ "rootId": root_id, "expectedEpoch": epoch }),
    )
    .expect("identificador opaco deve funcionar");

    assert_eq!(response["workspaceEpoch"], epoch);
    assert_eq!(response["status"]["branch"]["head"], "main");
    assert_eq!(response["status"]["unstaged"][0]["path"], "rastreado.txt");
}

#[test]
fn removed_handle_and_stale_epoch_are_rejected() {
    let test = TestApp::new();
    let (root_id, epoch) = test.authorize_repo();

    let removed = invoke(
        &test.webview,
        "remove_root",
        serde_json::json!({ "rootId": root_id }),
    )
    .expect("remoção deve funcionar");
    let new_epoch = removed["epoch"].as_u64().unwrap();
    assert_eq!(new_epoch, epoch + 1);
    assert!(removed["roots"].as_array().unwrap().is_empty());

    let unknown = invoke(
        &test.webview,
        "get_repo_status",
        serde_json::json!({ "rootId": root_id, "expectedEpoch": new_epoch }),
    )
    .expect_err("handle removido não pode ser reutilizado");
    assert!(
        unknown
            .as_str()
            .unwrap_or_default()
            .contains("desconhecida"),
        "erro inesperado: {unknown}"
    );

    let stale = invoke(
        &test.webview,
        "get_repo_status",
        serde_json::json!({ "rootId": root_id, "expectedEpoch": epoch }),
    )
    .expect_err("época antiga não pode ser aceita");
    assert!(
        stale.as_str().unwrap_or_default().contains("obsoleta"),
        "erro inesperado: {stale}"
    );
}

#[test]
fn stale_epoch_is_rejected_before_any_read() {
    let test = TestApp::new();
    let (root_id, epoch) = test.authorize_repo();

    let error = invoke(
        &test.webview,
        "get_repo_status",
        serde_json::json!({ "rootId": root_id, "expectedEpoch": epoch + 1 }),
    )
    .expect_err("época divergente deve falhar");

    assert!(
        error.as_str().unwrap_or_default().contains("obsoleta"),
        "erro inesperado: {error}"
    );
}

#[test]
fn relative_path_guard_blocks_traversal_and_absolute_paths() {
    let test = TestApp::new();
    let (root_id, epoch) = test.authorize_repo();

    for rel_path in ["../fora.txt", "/etc/passwd", ""] {
        let error = invoke(
            &test.webview,
            "get_file_diff",
            serde_json::json!({
                "rootId": root_id,
                "relPath": rel_path,
                "origPath": null,
                "group": "unstaged",
                "expectedEpoch": epoch,
            }),
        )
        .expect_err("caminho fora do repositório deve falhar");

        assert!(
            error
                .as_str()
                .unwrap_or_default()
                .contains("Caminho relativo inválido"),
            "entrada '{rel_path}' produziu erro inesperado: {error}"
        );
    }

    let response = invoke(
        &test.webview,
        "get_file_diff",
        serde_json::json!({
            "rootId": root_id,
            "relPath": "rastreado.txt",
            "origPath": null,
            "group": "unstaged",
            "expectedEpoch": epoch,
        }),
    )
    .expect("diff de caminho válido deve funcionar");

    assert_eq!(response["workspaceEpoch"], epoch);
    assert!(
        response["patch"]["patch"]
            .as_str()
            .unwrap_or_default()
            .contains("+versao modificada")
    );
}

#[test]
fn unavailable_root_is_rejected_via_ipc() {
    let test = TestApp::new();
    let (root_id, epoch) = test.authorize_repo();

    fs::remove_dir_all(&test.repo).unwrap();

    let error = invoke(
        &test.webview,
        "get_repo_status",
        serde_json::json!({ "rootId": root_id, "expectedEpoch": epoch }),
    )
    .expect_err("raiz removida do disco deve ser rejeitada");

    assert!(
        error.as_str().unwrap_or_default().contains("indisponível"),
        "erro inesperado: {error}"
    );
}

#[test]
fn workspace_view_exposes_epoch_and_health_without_paths_from_webview() {
    let test = TestApp::new();
    let (root_id, epoch) = test.authorize_repo();

    let response = invoke(&test.webview, "get_workspace_view", serde_json::json!({}))
        .expect("view deve funcionar");

    assert_eq!(response["epoch"], epoch);
    assert_eq!(response["health"], serde_json::Value::Null);
    assert_eq!(response["roots"][0]["id"], root_id);
    assert_eq!(response["roots"][0]["available"], true);
    assert!(response["roots"][0]["displayPath"].is_string());
}

#[test]
fn read_commands_do_not_mutate_the_repository() {
    let test = TestApp::new();
    let (root_id, epoch) = test.authorize_repo();

    let git_dir = test.repo.join(".git");
    let before_git = snapshot_directory_bytes(&git_dir);
    let before_worktree = snapshot_directory_bytes(&test.repo);

    for _ in 0..5 {
        invoke(
            &test.webview,
            "get_repo_status",
            serde_json::json!({ "rootId": root_id, "expectedEpoch": epoch }),
        )
        .expect("status");

        invoke(
            &test.webview,
            "get_file_diff",
            serde_json::json!({
                "rootId": root_id,
                "relPath": "rastreado.txt",
                "origPath": null,
                "group": "unstaged",
                "expectedEpoch": epoch,
            }),
        )
        .expect("diff rastreado");

        invoke(
            &test.webview,
            "get_file_diff",
            serde_json::json!({
                "rootId": root_id,
                "relPath": "novo.txt",
                "origPath": null,
                "group": "untracked",
                "expectedEpoch": epoch,
            }),
        )
        .expect("diff não rastreado");
    }

    let after_git = snapshot_directory_bytes(&git_dir);
    let after_worktree = snapshot_directory_bytes(&test.repo);

    assert_eq!(
        before_git, after_git,
        "os metadados do Git não podem mudar durante leituras via IPC"
    );
    assert_eq!(
        before_worktree, after_worktree,
        "os arquivos do worktree não podem mudar durante leituras via IPC"
    );
}
