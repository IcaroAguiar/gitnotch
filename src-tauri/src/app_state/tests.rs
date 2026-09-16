use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use crate::settings::SETTINGS_SCHEMA_VERSION;

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
        let path =
            std::env::temp_dir().join(format!("gitnotch-workspace-{prefix}-{count}-{nanos}"));
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

#[test]
fn authorize_persists_canonical_root_and_increments_epoch() {
    let dir = TempDir::new("authorize");
    let root = dir.child("checkout");
    let mut workspace = Workspace::load(dir.path().join("config"));

    let view = workspace.authorize_root(&root).expect("autorização");

    assert_eq!(view.epoch, 2);
    assert_eq!(view.roots.len(), 1);
    assert_eq!(view.roots[0].id, "r1");
    assert_eq!(view.roots[0].display_name, "checkout");
    assert_eq!(
        view.roots[0].display_path,
        fs::canonicalize(&root).unwrap().to_string_lossy()
    );
    assert!(view.roots[0].available);

    let persisted: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("config").join("settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(persisted["roots"][0]["id"], "r1");
    assert_eq!(
        persisted["roots"][0]["path"].as_str().unwrap(),
        fs::canonicalize(&root).unwrap().to_string_lossy()
    );
}

#[test]
fn authorize_is_idempotent_for_the_same_canonical_path() {
    let dir = TempDir::new("dedupe");
    let root = dir.child("checkout");
    let mut workspace = Workspace::load(dir.path().join("config"));

    let first = workspace.authorize_root(&root).unwrap();
    assert_eq!(first.epoch, 2);

    let trailing = PathBuf::from(format!("{}/", root.to_string_lossy()));
    let second = workspace.authorize_root(&trailing).unwrap();

    assert_eq!(
        second.epoch, first.epoch,
        "duplicata não incrementa a época"
    );
    assert_eq!(second.roots.len(), 1);
    assert_eq!(second.roots[0].id, first.roots[0].id);
}

#[test]
#[cfg(unix)]
fn non_utf8_path_is_rejected_by_the_persistence_encoding_boundary() {
    use std::os::unix::ffi::OsStringExt;

    let dir = TempDir::new("non-utf8");
    let root = dir
        .path()
        .join(std::ffi::OsString::from_vec(b"checkout-\xff".to_vec()));

    let error = persisted_path(&root).expect_err("um caminho não UTF-8 não pode ser salvo em JSON");

    assert!(
        error.to_string().contains("UTF-8"),
        "erro inesperado: {error}"
    );
}

#[test]
#[cfg(target_os = "linux")]
fn non_utf8_directory_is_rejected_without_persisting_authorization_state() {
    use std::os::unix::ffi::OsStringExt;

    let dir = TempDir::new("non-utf8-authorize");
    let root = dir
        .path()
        .join(std::ffi::OsString::from_vec(b"checkout-\xff".to_vec()));
    fs::create_dir(&root).expect("a fixture Linux deve aceitar bytes não UTF-8");
    let config = dir.path().join("config");
    let mut workspace = Workspace::load(config.clone());

    let error = workspace
        .authorize_root(&root)
        .expect_err("uma raiz não UTF-8 não pode ser persistida como JSON");

    assert!(matches!(error, WorkspaceError::InvalidSelection(_)));
    assert!(
        error.to_string().contains("UTF-8"),
        "erro inesperado: {error}"
    );
    assert_eq!(workspace.epoch(), 1);
    assert!(workspace.view().roots.is_empty());
    assert!(
        !config.join("settings.json").exists(),
        "a autorização recusada não pode criar preferências"
    );
}

#[test]
#[cfg(target_os = "macos")]
fn canonicalizes_tmp_to_private_tmp_on_macos() {
    let dir = TempDir::new("macos-tmp");
    let mut workspace = Workspace::load(dir.path().join("config"));

    let view = workspace.authorize_root(Path::new("/tmp")).unwrap();

    assert_eq!(view.roots[0].display_path, "/private/tmp");
}

#[test]
fn remove_root_revokes_handle_and_never_revives_it() {
    let dir = TempDir::new("remove");
    let root = dir.child("checkout");
    let mut workspace = Workspace::load(dir.path().join("config"));

    let first = workspace.authorize_root(&root).unwrap();
    let removed_id = first.roots[0].id.clone();

    let after_remove = workspace.remove_root(&removed_id).unwrap();
    assert_eq!(after_remove.epoch, first.epoch + 1);
    assert!(after_remove.roots.is_empty());
    assert!(matches!(
        workspace.resolve_root(&removed_id),
        Err(WorkspaceError::UnknownRoot(_))
    ));

    let reauthorized = workspace.authorize_root(&root).unwrap();
    assert_eq!(reauthorized.epoch, after_remove.epoch + 1);
    assert_eq!(reauthorized.roots[0].id, "r2");
    assert_ne!(reauthorized.roots[0].id, removed_id);
}

#[test]
fn exhausted_root_ids_preserve_existing_roots_without_reuse() {
    let dir = TempDir::new("root-id-exhaustion");
    let existing = dir.child("existing");
    let selected = dir.child("selected");
    let config = dir.path().join("config");
    let store = SettingsStore::new(config.clone());
    let persisted_path = fs::canonicalize(&existing)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    store
        .save(&SettingsFile {
            schema_version: SETTINGS_SCHEMA_VERSION,
            next_root_id: u64::MAX,
            roots: vec![PersistedRoot {
                id: "r1".to_string(),
                path: persisted_path,
            }],
        })
        .unwrap();
    let before = fs::read(store.file_path()).unwrap();
    let mut workspace = Workspace::load(config);

    let error = workspace
        .authorize_root(&selected)
        .expect_err("não pode reutilizar nem avançar um identificador exaurido");

    assert!(
        error
            .to_string()
            .contains("identificadores de raiz foram esgotados"),
        "erro inesperado: {error}"
    );
    let view = workspace.view();
    assert_eq!(view.epoch, 1);
    assert_eq!(view.roots.len(), 1);
    assert_eq!(view.roots[0].id, "r1");
    assert_eq!(fs::read(store.file_path()).unwrap(), before);
}

#[test]
fn exhausted_epoch_rejects_authorization_without_persisting_or_reusing_state() {
    let dir = TempDir::new("authorize-epoch-exhaustion");
    let root = dir.child("checkout");
    let config = dir.path().join("config");
    let mut workspace = Workspace::load(config.clone());
    workspace.epoch = u64::MAX;

    let error = workspace
        .authorize_root(&root)
        .expect_err("uma época exaurida não pode voltar a zero");

    assert!(
        error
            .to_string()
            .contains("época de autorização foi esgotada"),
        "erro inesperado: {error}"
    );
    assert_eq!(workspace.epoch(), u64::MAX);
    assert!(workspace.view().roots.is_empty());
    assert!(
        !config.join("settings.json").exists(),
        "a falha não pode gravar uma raiz que não ficou autorizada"
    );
}

#[test]
fn exhausted_epoch_rejects_removal_without_revoking_the_handle() {
    let dir = TempDir::new("remove-epoch-exhaustion");
    let root = dir.child("checkout");
    let config = dir.path().join("config");
    let mut workspace = Workspace::load(config.clone());
    let authorized = workspace.authorize_root(&root).unwrap();
    let root_id = authorized.roots[0].id.clone();
    let before = fs::read(config.join("settings.json")).unwrap();
    workspace.epoch = u64::MAX;

    let error = workspace
        .remove_root(&root_id)
        .expect_err("uma época exaurida não pode revogar e voltar a zero");

    assert!(
        error
            .to_string()
            .contains("época de autorização foi esgotada"),
        "erro inesperado: {error}"
    );
    assert_eq!(workspace.epoch(), u64::MAX);
    assert_eq!(workspace.view().roots[0].id, root_id);
    assert_eq!(fs::read(config.join("settings.json")).unwrap(), before);
}

#[test]
fn unknown_handle_is_rejected() {
    let dir = TempDir::new("unknown");
    let mut workspace = Workspace::load(dir.path().join("config"));

    assert!(matches!(
        workspace.resolve_root("r99"),
        Err(WorkspaceError::UnknownRoot(_))
    ));
    assert!(matches!(
        workspace.remove_root("r99"),
        Err(WorkspaceError::UnknownRoot(_))
    ));
}

#[test]
fn stale_epoch_is_rejected() {
    let dir = TempDir::new("stale");
    let root = dir.child("checkout");
    let mut workspace = Workspace::load(dir.path().join("config"));

    workspace.authorize_root(&root).unwrap();
    workspace.ensure_epoch(2).unwrap();

    match workspace.ensure_epoch(1) {
        Err(WorkspaceError::RequestStale { expected, current }) => {
            assert_eq!(expected, 1);
            assert_eq!(current, 2);
        }
        other => panic!("Esperado RequestStale, obtido: {other:?}"),
    }
}

#[test]
fn missing_root_is_unavailable_and_rejected_on_read() {
    let dir = TempDir::new("unavailable");
    let root = dir.child("checkout");
    let mut workspace = Workspace::load(dir.path().join("config"));

    let authorized = workspace.authorize_root(&root).unwrap();
    let id = authorized.roots[0].id.clone();
    fs::remove_dir_all(&root).unwrap();

    let view = workspace.view();
    assert!(!view.roots[0].available);
    assert!(matches!(
        workspace.resolve_root(&id),
        Err(WorkspaceError::RootUnavailable(_))
    ));
}

#[test]
fn restore_reads_persisted_roots_from_disk() {
    let dir = TempDir::new("restore");
    let root = dir.child("checkout");
    let config = dir.path().join("config");

    let mut first = Workspace::load(config.clone());
    let authorized = first.authorize_root(&root).unwrap();
    drop(first);

    let mut second = Workspace::load(config);
    let restored = second.view();

    assert_eq!(restored.roots.len(), 1);
    assert_eq!(restored.roots[0].id, authorized.roots[0].id);
    assert_eq!(
        restored.roots[0].display_path,
        authorized.roots[0].display_path
    );
    assert!(restored.health.is_none());
}

#[test]
fn incompatible_schema_blocks_mutations() {
    let dir = TempDir::new("schema-block");
    let config = dir.path().join("config");
    let root = dir.child("checkout");
    fs::create_dir_all(&config).unwrap();
    fs::write(
        config.join("settings.json"),
        "{\"schemaVersion\":9,\"nextRootId\":1,\"roots\":[]}",
    )
    .unwrap();

    let mut workspace = Workspace::load(config.clone());

    assert!(matches!(
        workspace.authorize_root(&root),
        Err(WorkspaceError::SettingsReadOnly { found: 9 })
    ));
    assert!(matches!(
        workspace.remove_root("r1"),
        Err(WorkspaceError::SettingsReadOnly { found: 9 })
    ));

    let persisted = fs::read_to_string(config.join("settings.json")).unwrap();
    assert!(persisted.contains("\"schemaVersion\":9"));
}

#[test]
#[cfg(unix)]
fn replaced_root_is_rejected_on_read() {
    let dir = TempDir::new("replaced");
    let root = dir.child("checkout");
    let mut workspace = Workspace::load(dir.path().join("config"));

    let authorized = workspace.authorize_root(&root).unwrap();
    let id = authorized.roots[0].id.clone();

    fs::remove_dir_all(&root).unwrap();
    let other = dir.child("outro");
    std::os::unix::fs::symlink(&other, &root).unwrap();

    assert!(matches!(
        workspace.resolve_root(&id),
        Err(WorkspaceError::RootChanged(_))
    ));
}
