use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

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

    let persisted = fs::read_to_string(dir.path().join("config").join("settings.json")).unwrap();
    assert!(persisted.contains("\"id\": \"r1\""));
    assert!(
        persisted.contains(
            &fs::canonicalize(&root)
                .unwrap()
                .to_string_lossy()
                .to_string()
        )
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
