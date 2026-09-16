use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;

use crate::git::{GitError, GitReader};
use crate::settings::{LoadHealth, PersistedRoot, SettingsError, SettingsFile, SettingsStore};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootSummary {
    pub id: String,
    pub display_name: String,
    pub display_path: String,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceView {
    pub epoch: u64,
    pub roots: Vec<RootSummary>,
    pub health: Option<String>,
}

#[derive(Debug, Clone)]
struct RootRecord {
    id: String,
    canonical_path: PathBuf,
    display_name: String,
    available: bool,
}

#[derive(Debug)]
pub enum WorkspaceError {
    UnknownRoot(String),
    RequestStale { expected: u64, current: u64 },
    RootIdExhausted,
    EpochExhausted,
    RootUnavailable(String),
    RootChanged(String),
    InvalidSelection(String),
    SettingsReadOnly { found: u32 },
    Settings(SettingsError),
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRoot(id) => write!(f, "Raiz desconhecida ou removida: '{id}'"),
            Self::RequestStale { expected, current } => write!(
                f,
                "Requisição obsoleta: época esperada {expected}, época atual {current}"
            ),
            Self::RootIdExhausted => {
                f.write_str("Os identificadores de raiz foram esgotados; não é possível autorizar outra pasta")
            }
            Self::EpochExhausted => f.write_str(
                "A época de autorização foi esgotada; reinicie o aplicativo antes de alterar as raízes",
            ),
            Self::RootUnavailable(path) => write!(f, "Raiz indisponível: '{path}'"),
            Self::RootChanged(path) => write!(
                f,
                "A raiz '{path}' mudou no disco desde a autorização e não será lida"
            ),
            Self::InvalidSelection(msg) => write!(f, "Seleção inválida: {msg}"),
            Self::SettingsReadOnly { found } => write!(
                f,
                "As preferências usam o schema {found} e não podem ser alteradas por esta versão"
            ),
            Self::Settings(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for WorkspaceError {}

impl serde::Serialize for WorkspaceError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<SettingsError> for WorkspaceError {
    fn from(err: SettingsError) -> Self {
        Self::Settings(err)
    }
}

pub struct Workspace {
    store: SettingsStore,
    file: SettingsFile,
    roots: BTreeMap<String, RootRecord>,
    epoch: u64,
    health: LoadHealth,
}

impl Workspace {
    pub fn load(dir: PathBuf) -> Self {
        let store = SettingsStore::new(dir);
        let (file, health) = store.load();
        let roots = file
            .roots
            .iter()
            .map(|persisted| {
                let record = build_record(persisted);
                (record.id.clone(), record)
            })
            .collect();

        Self {
            store,
            file,
            roots,
            epoch: 1,
            health,
        }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn view(&mut self) -> WorkspaceView {
        for record in self.roots.values_mut() {
            record.available = root_available(&record.canonical_path);
        }

        let roots = self
            .file
            .roots
            .iter()
            .filter_map(|persisted| self.roots.get(&persisted.id))
            .map(|record| RootSummary {
                id: record.id.clone(),
                display_name: record.display_name.clone(),
                display_path: record.canonical_path.to_string_lossy().into_owned(),
                available: record.available,
            })
            .collect();

        WorkspaceView {
            epoch: self.epoch,
            roots,
            health: self.health.warning(),
        }
    }

    pub fn ensure_epoch(&self, expected: u64) -> Result<(), WorkspaceError> {
        if expected != self.epoch {
            return Err(WorkspaceError::RequestStale {
                expected,
                current: self.epoch,
            });
        }
        Ok(())
    }

    pub fn authorize_root(&mut self, selected: &Path) -> Result<WorkspaceView, WorkspaceError> {
        self.ensure_writable()?;

        let canonical = std::fs::canonicalize(selected).map_err(|e| {
            WorkspaceError::InvalidSelection(format!(
                "não foi possível resolver '{}': {e}",
                selected.display()
            ))
        })?;

        if !canonical.is_dir() {
            return Err(WorkspaceError::InvalidSelection(format!(
                "'{}' não é um diretório",
                canonical.display()
            )));
        }

        let persisted_path = persisted_path(&canonical)?;

        if self.roots.values().any(|r| r.canonical_path == canonical) {
            return Ok(self.view());
        }

        let next_root_id = self
            .file
            .next_root_id
            .checked_add(1)
            .ok_or(WorkspaceError::RootIdExhausted)?;
        let next_epoch = self
            .epoch
            .checked_add(1)
            .ok_or(WorkspaceError::EpochExhausted)?;
        let id = format!("r{}", self.file.next_root_id);
        let mut candidate = self.file.clone();
        candidate.next_root_id = next_root_id;
        candidate.roots.push(PersistedRoot {
            id: id.clone(),
            path: persisted_path,
        });

        self.store.save(&candidate)?;
        self.file = candidate;
        self.roots.insert(
            id.clone(),
            RootRecord {
                id,
                canonical_path: canonical.clone(),
                display_name: display_name(&canonical),
                available: true,
            },
        );
        self.epoch = next_epoch;

        Ok(self.view())
    }

    pub fn remove_root(&mut self, root_id: &str) -> Result<WorkspaceView, WorkspaceError> {
        self.ensure_writable()?;

        if !self.roots.contains_key(root_id) {
            return Err(WorkspaceError::UnknownRoot(root_id.to_string()));
        }

        let next_epoch = self
            .epoch
            .checked_add(1)
            .ok_or(WorkspaceError::EpochExhausted)?;

        let mut candidate = self.file.clone();
        candidate.roots.retain(|r| r.id != root_id);

        self.store.save(&candidate)?;
        self.file = candidate;
        self.roots.remove(root_id);
        self.epoch = next_epoch;

        Ok(self.view())
    }

    pub fn resolve_root(&self, root_id: &str) -> Result<PathBuf, WorkspaceError> {
        let record = self
            .roots
            .get(root_id)
            .ok_or_else(|| WorkspaceError::UnknownRoot(root_id.to_string()))?;

        let canonical = std::fs::canonicalize(&record.canonical_path).map_err(|_| {
            WorkspaceError::RootUnavailable(record.canonical_path.to_string_lossy().into_owned())
        })?;

        if canonical != record.canonical_path || !canonical.is_dir() {
            return Err(WorkspaceError::RootChanged(
                record.canonical_path.to_string_lossy().into_owned(),
            ));
        }

        Ok(canonical)
    }

    fn ensure_writable(&self) -> Result<(), WorkspaceError> {
        if let LoadHealth::IncompatibleSchema { found } = self.health {
            return Err(WorkspaceError::SettingsReadOnly { found });
        }
        Ok(())
    }
}

fn build_record(persisted: &PersistedRoot) -> RootRecord {
    let stored = PathBuf::from(&persisted.path);
    let canonical = std::fs::canonicalize(&stored).unwrap_or_else(|_| stored.clone());
    let available = root_available(&canonical);

    RootRecord {
        id: persisted.id.clone(),
        canonical_path: canonical.clone(),
        display_name: display_name(&canonical),
        available,
    }
}

fn root_available(path: &Path) -> bool {
    std::fs::canonicalize(path)
        .map(|canonical| canonical == path && canonical.is_dir())
        .unwrap_or(false)
}

fn persisted_path(path: &Path) -> Result<String, WorkspaceError> {
    path.to_str().map(str::to_owned).ok_or_else(|| {
        WorkspaceError::InvalidSelection(
            "o caminho da pasta contém bytes que não são UTF-8 e não pode ser salvo nas preferências"
                .to_string(),
        )
    })
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

pub struct AppState {
    git_reader: Mutex<Option<GitReader>>,
    workspace: Mutex<Workspace>,
}

impl AppState {
    pub fn new(workspace: Workspace) -> Self {
        Self {
            git_reader: Mutex::new(None),
            workspace: Mutex::new(workspace),
        }
    }

    pub fn with_git_reader<T, F>(&self, f: F) -> Result<T, String>
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

    pub fn with_workspace<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&mut Workspace) -> Result<T, WorkspaceError>,
    {
        let mut guard = self
            .workspace
            .lock()
            .map_err(|e| format!("Falha de sincronização interna: {e}"))?;

        f(&mut guard).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests;
