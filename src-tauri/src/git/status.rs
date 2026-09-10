use std::ffi::OsStr;
use std::path::Path;

use crate::git::command::{DEFAULT_MAX_OUTPUT_BYTES, DEFAULT_TIMEOUT, run_git_command_success};
use crate::git::models::{
    BranchInfo, ChangeKind, FileChange, FileGroupKind, GitError, RepoStatusSnapshot,
};

fn char_to_change_kind(c: char) -> ChangeKind {
    match c {
        'M' => ChangeKind::Modified,
        'A' => ChangeKind::Added,
        'D' => ChangeKind::Deleted,
        'R' => ChangeKind::Renamed,
        'C' => ChangeKind::Copied,
        'T' => ChangeKind::TypeChanged,
        'U' => ChangeKind::Conflicted,
        '?' => ChangeKind::Untracked,
        _ => ChangeKind::Modified,
    }
}

/// Splits a byte slice at space boundaries up to `max_splits` times.
/// Returns (tokens_before_remainder, remainder).
fn split_tokens(bytes: &[u8], count: usize) -> (Vec<&[u8]>, &[u8]) {
    let mut tokens = Vec::with_capacity(count);
    let mut current = bytes;

    for _ in 0..count {
        if let Some(space_pos) = current.iter().position(|&b| b == b' ') {
            tokens.push(&current[..space_pos]);
            current = &current[space_pos + 1..];
        } else {
            tokens.push(current);
            current = &[];
            break;
        }
    }

    (tokens, current)
}

pub fn parse_porcelain_v2_status(raw_bytes: &[u8]) -> Result<RepoStatusSnapshot, GitError> {
    let mut branch = BranchInfo {
        oid: None,
        head: String::new(),
        upstream: None,
        ahead: None,
        behind: None,
        is_detached: false,
        is_unborn: false,
    };

    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    let mut conflicts = Vec::new();

    let chunks: Vec<&[u8]> = raw_bytes.split(|&b| b == 0).collect();
    let mut i = 0;

    while i < chunks.len() {
        let chunk = chunks[i];
        i += 1;

        if chunk.is_empty() {
            continue;
        }

        // Branch headers: # branch.<key> <value>
        if chunk.starts_with(b"# ") {
            let line_str = String::from_utf8_lossy(&chunk[2..]);
            let mut parts = line_str.splitn(2, ' ');
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("").trim();

            match key {
                "branch.oid" => {
                    if val == "(initial)" {
                        branch.is_unborn = true;
                        branch.oid = None;
                    } else {
                        branch.oid = Some(val.to_string());
                    }
                }
                "branch.head" => {
                    if val == "(detached)" {
                        branch.is_detached = true;
                        branch.head = "(detached)".to_string();
                    } else {
                        branch.head = val.to_string();
                    }
                }
                "branch.upstream" => {
                    branch.upstream = Some(val.to_string());
                }
                "branch.ab" => {
                    // Format: +<ahead> -<behind>
                    for token in val.split_whitespace() {
                        if let Some(ahead_str) = token.strip_prefix('+') {
                            branch.ahead = ahead_str.parse().ok();
                        } else if let Some(behind_str) = token.strip_prefix('-') {
                            branch.behind = behind_str.parse().ok();
                        }
                    }
                }
                _ => {}
            }
            continue;
        }

        // Type 1: Ordinary changed entry: 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
        if chunk.starts_with(b"1 ") {
            // Split first 8 spaces to isolate <path>
            let (tokens, path_bytes) = split_tokens(chunk, 8);
            if tokens.len() == 8 {
                let xy_str = String::from_utf8_lossy(tokens[1]);
                let sub_str = String::from_utf8_lossy(tokens[2]);
                let is_submodule = sub_str.starts_with('S');

                let mut chars = xy_str.chars();
                let x = chars.next().unwrap_or('.');
                let y = chars.next().unwrap_or('.');
                let path = String::from_utf8_lossy(path_bytes).to_string();

                if x != '.' {
                    staged.push(FileChange {
                        path: path.clone(),
                        orig_path: None,
                        group: FileGroupKind::Staged,
                        kind: char_to_change_kind(x),
                        staged_status: Some(x),
                        unstaged_status: None,
                        submodule: is_submodule,
                    });
                }

                if y != '.' {
                    unstaged.push(FileChange {
                        path,
                        orig_path: None,
                        group: FileGroupKind::Unstaged,
                        kind: char_to_change_kind(y),
                        staged_status: None,
                        unstaged_status: Some(y),
                        submodule: is_submodule,
                    });
                }
            }
            continue;
        }

        // Type 2: Renamed or copied entry: 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path>\0<origPath>
        if chunk.starts_with(b"2 ") {
            let (tokens, path_bytes) = split_tokens(chunk, 9);
            let orig_path = if i < chunks.len() {
                let p = String::from_utf8_lossy(chunks[i]).to_string();
                i += 1;
                Some(p)
            } else {
                None
            };

            if tokens.len() == 9 {
                let xy_str = String::from_utf8_lossy(tokens[1]);
                let sub_str = String::from_utf8_lossy(tokens[2]);
                let is_submodule = sub_str.starts_with('S');
                let score_token = String::from_utf8_lossy(tokens[8]);

                let is_copy = score_token.starts_with('C');
                let base_kind = if is_copy {
                    ChangeKind::Copied
                } else {
                    ChangeKind::Renamed
                };

                let mut chars = xy_str.chars();
                let x = chars.next().unwrap_or('.');
                let y = chars.next().unwrap_or('.');
                let path = String::from_utf8_lossy(path_bytes).to_string();

                if x != '.' {
                    staged.push(FileChange {
                        path: path.clone(),
                        orig_path: orig_path.clone(),
                        group: FileGroupKind::Staged,
                        kind: base_kind,
                        staged_status: Some(x),
                        unstaged_status: None,
                        submodule: is_submodule,
                    });
                }

                if y != '.' {
                    unstaged.push(FileChange {
                        path,
                        orig_path,
                        group: FileGroupKind::Unstaged,
                        kind: base_kind,
                        staged_status: None,
                        unstaged_status: Some(y),
                        submodule: is_submodule,
                    });
                }
            }
            continue;
        }

        // Type u: Unmerged / conflicted entry: u <XY> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>
        if chunk.starts_with(b"u ") {
            let (tokens, path_bytes) = split_tokens(chunk, 10);
            if tokens.len() == 10 {
                let xy_str = String::from_utf8_lossy(tokens[1]);
                let sub_str = String::from_utf8_lossy(tokens[2]);
                let is_submodule = sub_str.starts_with('S');

                let mut chars = xy_str.chars();
                let x = chars.next();
                let y = chars.next();
                let path = String::from_utf8_lossy(path_bytes).to_string();

                conflicts.push(FileChange {
                    path,
                    orig_path: None,
                    group: FileGroupKind::Conflicted,
                    kind: ChangeKind::Conflicted,
                    staged_status: x,
                    unstaged_status: y,
                    submodule: is_submodule,
                });
            }
            continue;
        }

        // Type ?: Untracked entry: ? <path>
        if let Some(path_bytes) = chunk.strip_prefix(b"? ") {
            let path = String::from_utf8_lossy(path_bytes).to_string();
            untracked.push(FileChange {
                path,
                orig_path: None,
                group: FileGroupKind::Untracked,
                kind: ChangeKind::Untracked,
                staged_status: None,
                unstaged_status: None,
                submodule: false,
            });
            continue;
        }
    }

    Ok(RepoStatusSnapshot {
        branch,
        staged,
        unstaged,
        untracked,
        conflicts,
    })
}

pub fn get_repo_status(git_path: &Path, worktree: &Path) -> Result<RepoStatusSnapshot, GitError> {
    let args = [
        OsStr::new("status"),
        OsStr::new("--porcelain=v2"),
        OsStr::new("--branch"),
        OsStr::new("-z"),
        OsStr::new("--no-ahead-behind"),
        OsStr::new("--untracked-files=all"),
        OsStr::new("--find-renames=50%"),
    ];

    let stdout = run_git_command_success(
        git_path,
        Some(worktree),
        &args,
        None,
        DEFAULT_MAX_OUTPUT_BYTES,
        DEFAULT_TIMEOUT,
    )?;

    parse_porcelain_v2_status(&stdout)
}
