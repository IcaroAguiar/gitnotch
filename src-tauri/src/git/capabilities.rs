use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::git::command::run_git_command;
use crate::git::models::{GitCapabilities, GitError};

pub fn find_git_executable() -> Result<PathBuf, GitError> {
    let binary_name = if cfg!(windows) { "git.exe" } else { "git" };

    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(binary_name);
            if candidate.is_file() {
                // Verify executable permissions on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = candidate.metadata()
                        && metadata.permissions().mode() & 0o111 != 0
                    {
                        return Ok(candidate);
                    }
                }
                #[cfg(not(unix))]
                {
                    return Ok(candidate);
                }
            }
        }
    }

    Err(GitError::NotFound(
        "Executável 'git' não encontrado no PATH do sistema".to_string(),
    ))
}

pub fn parse_git_version(version_str: &str) -> Option<(u32, u32, u32)> {
    // Formats: "git version 2.47.3", "git version 2.39.5 (Apple Git-154)", etc.
    let marker = "git version ";
    let idx = version_str.find(marker)?;
    let rest = &version_str[idx + marker.len()..];
    let ver_part = rest.split_whitespace().next()?;

    let parts: Vec<&str> = ver_part.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = if parts.len() >= 3 {
        // Strip trailing non-digits if any, e.g. "3windows.1"
        let patch_digits: String = parts[2]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        patch_digits.parse::<u32>().unwrap_or(0)
    } else {
        0
    };

    Some((major, minor, patch))
}

pub fn probe_git_capabilities(git_path: &Path) -> Result<GitCapabilities, GitError> {
    let args = [OsStr::new("--version")];
    let output = run_git_command(
        git_path,
        None,
        &args,
        None,
        1024 * 1024,
        Duration::from_secs(5),
    )?;

    if !output.status.success() {
        return Err(GitError::CommandFailed {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let version_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version_tuple = parse_git_version(&version_output).ok_or_else(|| {
        GitError::VersionUnsupported(format!("Formato de versão inesperado: '{version_output}'"))
    })?;

    let (major, minor, _) = version_tuple;
    // Require Git >= 2.22 for porcelain v2 with all necessary flags (--no-lazy-fetch, --no-optional-locks, etc.)
    if major < 2 || (major == 2 && minor < 22) {
        return Err(GitError::VersionUnsupported(format!(
            "Git versão {major}.{minor} é inferior ao requisito mínimo (2.22)"
        )));
    }

    Ok(GitCapabilities {
        installed: true,
        version: version_output,
        supports_porcelain_v2: true,
        executable_path: git_path.to_string_lossy().to_string(),
    })
}
