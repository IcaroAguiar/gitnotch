use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileGroupKind {
    Staged,
    Unstaged,
    Untracked,
    Conflicted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Untracked,
    Conflicted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_path: Option<String>,
    pub group: FileGroupKind,
    pub kind: ChangeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staged_status: Option<char>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unstaged_status: Option<char>,
    pub submodule: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oid: Option<String>,
    pub head: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ahead: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behind: Option<u32>,
    pub is_detached: bool,
    pub is_unborn: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoStatusSnapshot {
    pub branch: BranchInfo,
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<FileChange>,
    pub conflicts: Vec<FileChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FilterPreflightResult {
    Allowed,
    LimitedByExternalFilter { filter_name: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffPatch {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orig_path: Option<String>,
    pub group: FileGroupKind,
    pub patch: String,
    pub is_binary: bool,
    pub is_too_large: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCapabilities {
    pub installed: bool,
    pub version: String,
    pub supports_porcelain_v2: bool,
    pub executable_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitError {
    NotFound(String),
    VersionUnsupported(String),
    CommandFailed { code: Option<i32>, stderr: String },
    OutputLimitExceeded { max_bytes: usize },
    Timeout { duration_ms: u64 },
    InvalidPath(String),
    LimitedByExternalFilter { filter_name: String, reason: String },
    Io(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Git não encontrado: {msg}"),
            Self::VersionUnsupported(msg) => write!(f, "Versão do Git incompatível: {msg}"),
            Self::CommandFailed { code, stderr } => {
                write!(f, "Comando Git falhou (código {code:?}): {stderr}")
            }
            Self::OutputLimitExceeded { max_bytes } => {
                write!(f, "Limite de saída excedido ({max_bytes} bytes)")
            }
            Self::Timeout { duration_ms } => {
                write!(f, "Tempo limite excedido ({duration_ms} ms)")
            }
            Self::InvalidPath(msg) => write!(f, "Caminho inválido: {msg}"),
            Self::LimitedByExternalFilter {
                filter_name,
                reason,
            } => {
                write!(
                    f,
                    "Leitura limitada por filtro externo '{filter_name}': {reason}"
                )
            }
            Self::Io(msg) => write!(f, "Erro de E/S: {msg}"),
        }
    }
}

impl std::error::Error for GitError {}

impl serde::Serialize for GitError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
