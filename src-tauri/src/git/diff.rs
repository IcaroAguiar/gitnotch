use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::git::command::{DEFAULT_MAX_OUTPUT_BYTES, DEFAULT_TIMEOUT, run_git_command_success};
use crate::git::models::{DiffPatch, FileGroupKind, GitError};

const MAX_UNTRACKED_FILE_SIZE: u64 = 2 * 1024 * 1024;

fn is_buffer_binary(buf: &[u8]) -> bool {
    buf.contains(&0)
}

fn build_untracked_diff(worktree: &Path, rel_path: &str) -> Result<DiffPatch, GitError> {
    let full_path = worktree.join(rel_path);

    let canonical_worktree = worktree
        .canonicalize()
        .map_err(|e| GitError::Io(format!("Falha ao resolver diretório de trabalho: {e}")))?;
    let canonical_file = full_path
        .canonicalize()
        .map_err(|e| GitError::Io(format!("Falha ao resolver arquivo '{rel_path}': {e}")))?;

    if !canonical_file.starts_with(&canonical_worktree) {
        return Err(GitError::InvalidPath(format!(
            "Tentativa de acesso fora do diretório do repositório: '{rel_path}'"
        )));
    }

    let metadata = std::fs::metadata(&canonical_file)
        .map_err(|e| GitError::Io(format!("Falha ao ler metadados de '{rel_path}': {e}")))?;
    let file_size = metadata.len();

    if file_size > MAX_UNTRACKED_FILE_SIZE {
        return Ok(DiffPatch {
            path: rel_path.to_string(),
            orig_path: None,
            group: FileGroupKind::Untracked,
            patch: format!("Arquivo não rastreado excede o limite de exibição ({file_size} bytes)"),
            is_binary: false,
            is_too_large: true,
            file_size_bytes: Some(file_size),
        });
    }

    let mut file = File::open(&canonical_file)
        .map_err(|e| GitError::Io(format!("Falha ao abrir arquivo '{rel_path}': {e}")))?;

    let mut sample = vec![0u8; file_size.min(8192) as usize];
    file.read_exact(&mut sample)
        .map_err(|e| GitError::Io(format!("Falha ao ler amostra de '{rel_path}': {e}")))?;

    if is_buffer_binary(&sample) {
        return Ok(DiffPatch {
            path: rel_path.to_string(),
            orig_path: None,
            group: FileGroupKind::Untracked,
            patch: "Arquivo binário não rastreado".to_string(),
            is_binary: true,
            is_too_large: false,
            file_size_bytes: Some(file_size),
        });
    }

    let mut full_content = Vec::new();
    full_content.extend_from_slice(&sample);
    if file_size > sample.len() as u64 {
        file.read_to_end(&mut full_content)
            .map_err(|e| GitError::Io(format!("Falha ao ler conteúdo de '{rel_path}': {e}")))?;
    }

    let text = match String::from_utf8(full_content) {
        Ok(s) => s,
        Err(_) => {
            return Ok(DiffPatch {
                path: rel_path.to_string(),
                orig_path: None,
                group: FileGroupKind::Untracked,
                patch: "Arquivo com codificação não suportada (não UTF-8)".to_string(),
                is_binary: true,
                is_too_large: false,
                file_size_bytes: Some(file_size),
            });
        }
    };

    let line_count = text.lines().count();
    let mut patch = format!(
        "diff --git a/{rel_path} b/{rel_path}\nnew file mode 100644\n--- /dev/null\n+++ b/{rel_path}\n@@ -0,0 +1,{line_count} @@\n"
    );

    for line in text.lines() {
        patch.push('+');
        patch.push_str(line);
        patch.push('\n');
    }

    Ok(DiffPatch {
        path: rel_path.to_string(),
        orig_path: None,
        group: FileGroupKind::Untracked,
        patch,
        is_binary: false,
        is_too_large: false,
        file_size_bytes: Some(file_size),
    })
}

pub fn get_file_diff(
    git_path: &Path,
    worktree: &Path,
    rel_path: &str,
    orig_path: Option<&str>,
    group: FileGroupKind,
) -> Result<DiffPatch, GitError> {
    if group == FileGroupKind::Untracked {
        return build_untracked_diff(worktree, rel_path);
    }

    let mut args: Vec<&OsStr> = Vec::new();
    args.push(OsStr::new("diff"));

    if group == FileGroupKind::Staged {
        args.push(OsStr::new("--cached"));
    }

    args.push(OsStr::new("--no-ext-diff"));
    args.push(OsStr::new("--no-textconv"));
    args.push(OsStr::new("--no-color"));
    args.push(OsStr::new("--no-relative"));
    args.push(OsStr::new("--src-prefix=a/"));
    args.push(OsStr::new("--dst-prefix=b/"));
    args.push(OsStr::new("--unified=3"));
    args.push(OsStr::new("--diff-algorithm=myers"));
    args.push(OsStr::new("--find-renames=50%"));
    args.push(OsStr::new("--"));

    args.push(OsStr::new(rel_path));
    if let Some(op) = orig_path {
        args.push(OsStr::new(op));
    }

    let stdout_bytes = run_git_command_success(
        git_path,
        Some(worktree),
        &args,
        None,
        DEFAULT_MAX_OUTPUT_BYTES,
        DEFAULT_TIMEOUT,
    )?;

    let patch_str = String::from_utf8_lossy(&stdout_bytes).to_string();
    let is_binary = patch_str.contains("Binary files ") || patch_str.contains("GIT binary patch");

    Ok(DiffPatch {
        path: rel_path.to_string(),
        orig_path: orig_path.map(|s| s.to_string()),
        group,
        patch: patch_str,
        is_binary,
        is_too_large: false,
        file_size_bytes: None,
    })
}
