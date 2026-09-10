pub mod capabilities;
pub mod command;
pub mod diff;
pub mod filter_preflight;
pub mod models;
pub mod status;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

pub use capabilities::{find_git_executable, probe_git_capabilities};
pub use diff::get_file_diff;
pub use filter_preflight::check_external_filters;
pub use models::*;
pub use status::get_repo_status;

#[derive(Debug, Clone)]
pub struct GitReader {
    git_executable: PathBuf,
    capabilities: GitCapabilities,
}

impl GitReader {
    pub fn new() -> Result<Self, GitError> {
        let exe = find_git_executable()?;
        Self::from_executable(exe)
    }

    pub fn from_executable(exe: PathBuf) -> Result<Self, GitError> {
        let caps = probe_git_capabilities(&exe)?;
        Ok(Self {
            git_executable: exe,
            capabilities: caps,
        })
    }

    pub fn capabilities(&self) -> &GitCapabilities {
        &self.capabilities
    }

    pub fn check_filters(&self, worktree: &Path) -> Result<FilterPreflightResult, GitError> {
        check_external_filters(&self.git_executable, worktree)
    }

    pub fn status(&self, worktree: &Path) -> Result<RepoStatusSnapshot, GitError> {
        let filter_status = self.check_filters(worktree)?;
        if let FilterPreflightResult::LimitedByExternalFilter {
            filter_name,
            reason,
        } = filter_status
        {
            return Err(GitError::LimitedByExternalFilter {
                filter_name,
                reason,
            });
        }

        get_repo_status(&self.git_executable, worktree)
    }

    pub fn diff(
        &self,
        worktree: &Path,
        rel_path: &str,
        orig_path: Option<&str>,
        group: FileGroupKind,
    ) -> Result<DiffPatch, GitError> {
        if group != FileGroupKind::Untracked {
            let filter_status = self.check_filters(worktree)?;
            if let FilterPreflightResult::LimitedByExternalFilter {
                filter_name,
                reason,
            } = filter_status
            {
                return Err(GitError::LimitedByExternalFilter {
                    filter_name,
                    reason,
                });
            }
        }

        get_file_diff(&self.git_executable, worktree, rel_path, orig_path, group)
    }
}
