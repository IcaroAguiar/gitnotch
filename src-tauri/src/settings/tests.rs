use std::fs;
use std::path::{Path, PathBuf};
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
        let path = std::env::temp_dir().join(format!("gitnotch-settings-{prefix}-{count}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn sample_file() -> SettingsFile {
    SettingsFile {
        schema_version: SETTINGS_SCHEMA_VERSION,
        next_root_id: 2,
        roots: vec![PersistedRoot {
            id: "r1".to_string(),
            path: "/tmp/checkout".to_string(),
        }],
    }
}

#[test]
fn missing_file_loads_default_without_warning() {
    let dir = TempDir::new("missing");
    let store = SettingsStore::new(dir.path().to_path_buf());

    let (file, health) = store.load();

    assert_eq!(file, SettingsFile::default());
    assert_eq!(health, LoadHealth::Ok);
    assert!(!store.file_path().exists());
}

#[test]
fn roundtrip_restores_persisted_preferences() {
    let dir = TempDir::new("roundtrip");
    let store = SettingsStore::new(dir.path().to_path_buf());
    let expected = sample_file();

    store.save(&expected).expect("gravação deve funcionar");
    let (loaded, health) = store.load();

    assert_eq!(loaded, expected);
    assert_eq!(health, LoadHealth::Ok);
}

#[test]
fn invalid_json_recovers_with_warning_and_does_not_overwrite() {
    let dir = TempDir::new("invalid");
    let store = SettingsStore::new(dir.path().to_path_buf());
    let corrupt = b"{ nao eh json";
    fs::write(store.file_path(), corrupt).unwrap();

    let (file, health) = store.load();

    assert_eq!(file, SettingsFile::default());
    match health {
        LoadHealth::RecoveredFromInvalidFile(_) => {}
        other => panic!("Esperado RecoveredFromInvalidFile, obtido: {other:?}"),
    }
    assert_eq!(
        fs::read(store.file_path()).unwrap(),
        corrupt,
        "a leitura não deve sobrescrever o arquivo inválido"
    );
}

#[test]
fn future_schema_blocks_writes() {
    let dir = TempDir::new("future-schema");
    let store = SettingsStore::new(dir.path().to_path_buf());
    let future = format!(
        "{{\"schemaVersion\": {}, \"nextRootId\": 1, \"roots\": []}}",
        SETTINGS_SCHEMA_VERSION + 1
    );
    fs::write(store.file_path(), future).unwrap();

    let (file, health) = store.load();

    assert_eq!(file, SettingsFile::default());
    assert!(matches!(
        health,
        LoadHealth::IncompatibleSchema { found } if found == SETTINGS_SCHEMA_VERSION + 1
    ));
    assert!(health.warning().is_some());
}

#[test]
fn duplicate_or_malformed_root_ids_recover() {
    let dir = TempDir::new("malformed-ids");
    let store = SettingsStore::new(dir.path().to_path_buf());

    for invalid in [
        "{\"schemaVersion\":1,\"nextRootId\":1,\"roots\":[{\"id\":\"r1\",\"path\":\"/a\"},{\"id\":\"r1\",\"path\":\"/b\"}]}",
        "{\"schemaVersion\":1,\"nextRootId\":1,\"roots\":[{\"id\":\"raiz\",\"path\":\"/a\"}]}",
        "{\"schemaVersion\":1,\"nextRootId\":1,\"roots\":[{\"id\":\"r01\",\"path\":\"/a\"}]}",
    ] {
        fs::write(store.file_path(), invalid).unwrap();
        let (file, health) = store.load();
        assert_eq!(file, SettingsFile::default(), "entrada: {invalid}");
        assert!(
            matches!(health, LoadHealth::RecoveredFromInvalidFile(_)),
            "entrada: {invalid}"
        );
    }
}

#[test]
fn next_root_id_is_raised_above_existing_ids() {
    let dir = TempDir::new("next-id");
    let store = SettingsStore::new(dir.path().to_path_buf());
    fs::write(
        store.file_path(),
        "{\"schemaVersion\":1,\"nextRootId\":1,\"roots\":[{\"id\":\"r7\",\"path\":\"/a\"}]}",
    )
    .unwrap();

    let (file, health) = store.load();

    assert_eq!(health, LoadHealth::Ok);
    assert_eq!(file.next_root_id, 8);
}

#[test]
fn atomic_write_replaces_content_without_leaving_temp_file() {
    let dir = TempDir::new("atomic");
    let store = SettingsStore::new(dir.path().to_path_buf());

    let mut first = sample_file();
    store.save(&first).unwrap();
    let mut second = sample_file();
    second.roots.push(PersistedRoot {
        id: "r2".to_string(),
        path: "/tmp/outro".to_string(),
    });
    second.next_root_id = 3;
    store.save(&second).unwrap();

    let (loaded, health) = store.load();
    assert_eq!(health, LoadHealth::Ok);
    assert_eq!(loaded, second);
    assert!(
        !dir.path().join(SETTINGS_TEMP_FILE_NAME).exists(),
        "o arquivo temporário não pode ficar órfão"
    );
    first.next_root_id = 0;
    assert_ne!(first, second);
}

#[test]
fn save_creates_missing_directory() {
    let dir = TempDir::new("create-dir");
    let nested = dir.path().join("a").join("b");
    let store = SettingsStore::new(nested.clone());

    store.save(&sample_file()).unwrap();

    assert!(nested.join(SETTINGS_FILE_NAME).is_file());
}
