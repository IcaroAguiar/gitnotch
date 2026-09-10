use std::collections::BTreeSet;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;
pub const SETTINGS_FILE_NAME: &str = "settings.json";
pub const SETTINGS_TEMP_FILE_NAME: &str = "settings.json.tmp";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsFile {
    pub schema_version: u32,
    pub next_root_id: u64,
    pub roots: Vec<PersistedRoot>,
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            next_root_id: 1,
            roots: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedRoot {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadHealth {
    Ok,
    RecoveredFromInvalidFile(String),
    IncompatibleSchema { found: u32 },
}

impl LoadHealth {
    pub fn warning(&self) -> Option<String> {
        match self {
            Self::Ok => None,
            Self::RecoveredFromInvalidFile(reason) => Some(format!(
                "As preferências não puderam ser lidas ({reason}). A próxima alteração sobrescreve o arquivo."
            )),
            Self::IncompatibleSchema { found } => Some(format!(
                "As preferências usam o schema {found}, mais novo que o suportado ({SETTINGS_SCHEMA_VERSION}). A gravação está bloqueada para não perder dados."
            )),
        }
    }
}

#[derive(Debug)]
pub struct SettingsError(String);

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SettingsError {}

pub struct SettingsStore {
    dir: PathBuf,
}

impl SettingsStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn file_path(&self) -> PathBuf {
        self.dir.join(SETTINGS_FILE_NAME)
    }

    pub fn load(&self) -> (SettingsFile, LoadHealth) {
        let raw = match fs::read_to_string(self.file_path()) {
            Ok(raw) => raw,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return (SettingsFile::default(), LoadHealth::Ok);
            }
            Err(e) => {
                return (
                    SettingsFile::default(),
                    LoadHealth::RecoveredFromInvalidFile(format!("falha ao ler o arquivo: {e}")),
                );
            }
        };

        let parsed: SettingsFile = match serde_json::from_str(&raw) {
            Ok(file) => file,
            Err(e) => {
                return (
                    SettingsFile::default(),
                    LoadHealth::RecoveredFromInvalidFile(format!("JSON inválido: {e}")),
                );
            }
        };

        if parsed.schema_version > SETTINGS_SCHEMA_VERSION {
            return (
                SettingsFile::default(),
                LoadHealth::IncompatibleSchema {
                    found: parsed.schema_version,
                },
            );
        }

        if parsed.schema_version != SETTINGS_SCHEMA_VERSION {
            return (
                SettingsFile::default(),
                LoadHealth::RecoveredFromInvalidFile(format!(
                    "schema {} não reconhecido",
                    parsed.schema_version
                )),
            );
        }

        match normalize(parsed) {
            Ok(file) => (file, LoadHealth::Ok),
            Err(reason) => (
                SettingsFile::default(),
                LoadHealth::RecoveredFromInvalidFile(reason),
            ),
        }
    }

    pub fn save(&self, file: &SettingsFile) -> Result<(), SettingsError> {
        fs::create_dir_all(&self.dir).map_err(|e| {
            SettingsError(format!(
                "Falha ao preparar o diretório de preferências: {e}"
            ))
        })?;

        let json = serde_json::to_vec_pretty(file)
            .map_err(|e| SettingsError(format!("Falha ao serializar preferências: {e}")))?;

        let tmp_path = self.dir.join(SETTINGS_TEMP_FILE_NAME);
        let write_result = (|| -> std::io::Result<()> {
            let mut tmp = fs::File::create(&tmp_path)?;
            tmp.write_all(&json)?;
            tmp.sync_all()?;
            Ok(())
        })();

        if let Err(e) = write_result {
            let _ = fs::remove_file(&tmp_path);
            return Err(SettingsError(format!("Falha ao gravar preferências: {e}")));
        }

        fs::rename(&tmp_path, self.file_path()).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            SettingsError(format!("Falha ao substituir preferências: {e}"))
        })
    }
}

pub fn parse_root_id(id: &str) -> Option<u64> {
    let number: u64 = id.strip_prefix('r')?.parse().ok()?;
    if format!("r{number}") == id {
        Some(number)
    } else {
        None
    }
}

fn normalize(mut file: SettingsFile) -> Result<SettingsFile, String> {
    let mut seen = BTreeSet::new();
    let mut max_id = 0u64;

    for root in &file.roots {
        if root.id.is_empty() || !seen.insert(root.id.clone()) {
            return Err(format!(
                "identificador de raiz repetido ou vazio: '{}'",
                root.id
            ));
        }
        if root.path.is_empty() {
            return Err(format!("caminho vazio para a raiz '{}'", root.id));
        }
        match parse_root_id(&root.id) {
            Some(number) => max_id = max_id.max(number),
            None => {
                return Err(format!("identificador de raiz inválido: '{}'", root.id));
            }
        }
    }

    file.next_root_id = file.next_root_id.max(max_id + 1).max(1);
    Ok(file)
}

#[cfg(test)]
mod tests;
