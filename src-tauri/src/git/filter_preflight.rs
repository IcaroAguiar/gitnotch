use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::time::Duration;

use crate::git::command::{DEFAULT_MAX_OUTPUT_BYTES, DEFAULT_TIMEOUT, run_git_command};
use crate::git::models::{FilterPreflightResult, GitError};

pub fn check_external_filters(
    git_path: &Path,
    worktree: &Path,
) -> Result<FilterPreflightResult, GitError> {
    let config_args = [
        OsStr::new("config"),
        OsStr::new("--get-regexp"),
        OsStr::new(r"^filter\..*\.(clean|process)$"),
    ];

    let output = run_git_command(
        git_path,
        Some(worktree),
        &config_args,
        None,
        DEFAULT_MAX_OUTPUT_BYTES,
        Duration::from_secs(5),
    )?;

    if !output.status.success() {
        if output.status.code() == Some(1) {
            return Ok(FilterPreflightResult::Allowed);
        }
        return Err(GitError::CommandFailed {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let config_text = String::from_utf8_lossy(&output.stdout);
    let mut configured_filters = HashSet::new();

    for line in config_text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("filter.")
            && let Some(dot_idx) = rest.find('.')
        {
            let filter_name = &rest[..dot_idx];
            let property = &rest[dot_idx + 1..];
            if property.starts_with("clean ") || property.starts_with("process ") {
                configured_filters.insert(filter_name.to_string());
            }
        }
    }

    if configured_filters.is_empty() {
        return Ok(FilterPreflightResult::Allowed);
    }

    let ls_files_args = [OsStr::new("ls-files"), OsStr::new("-z")];
    let ls_output = run_git_command(
        git_path,
        Some(worktree),
        &ls_files_args,
        None,
        DEFAULT_MAX_OUTPUT_BYTES,
        DEFAULT_TIMEOUT,
    )?;

    if !ls_output.status.success() {
        return Err(GitError::CommandFailed {
            code: ls_output.status.code(),
            stderr: String::from_utf8_lossy(&ls_output.stderr).to_string(),
        });
    }

    if ls_output.stdout.is_empty() {
        return Ok(FilterPreflightResult::Allowed);
    }

    let check_attr_args = [
        OsStr::new("check-attr"),
        OsStr::new("-z"),
        OsStr::new("filter"),
        OsStr::new("--stdin"),
    ];

    let attr_output = run_git_command(
        git_path,
        Some(worktree),
        &check_attr_args,
        Some(&ls_output.stdout),
        DEFAULT_MAX_OUTPUT_BYTES,
        DEFAULT_TIMEOUT,
    )?;

    if !attr_output.status.success() {
        return Err(GitError::CommandFailed {
            code: attr_output.status.code(),
            stderr: String::from_utf8_lossy(&attr_output.stderr).to_string(),
        });
    }

    let chunks: Vec<&[u8]> = attr_output
        .stdout
        .split(|&b| b == 0)
        .filter(|chunk| !chunk.is_empty())
        .collect();

    for triplet in chunks.chunks(3) {
        if triplet.len() == 3 {
            let path = String::from_utf8_lossy(triplet[0]);
            let value = String::from_utf8_lossy(triplet[2]);

            if configured_filters.contains(value.as_ref()) {
                return Ok(FilterPreflightResult::LimitedByExternalFilter {
                    filter_name: value.to_string(),
                    reason: format!(
                        "Filtro externo '{value}' configurado com clean/process aplica-se ao arquivo '{path}'"
                    ),
                });
            }
        }
    }

    Ok(FilterPreflightResult::Allowed)
}
