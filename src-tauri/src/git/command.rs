use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::git::models::GitError;

pub const DEFAULT_MAX_OUTPUT_BYTES: usize = 20 * 1024 * 1024;
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

fn drain_stream<R: Read>(
    mut reader: R,
    max_bytes: usize,
    limit_flag: Arc<AtomicBool>,
) -> (Vec<u8>, bool) {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 8192];

    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if buffer.len() + n > max_bytes {
                    let remaining = max_bytes.saturating_sub(buffer.len());
                    buffer.extend_from_slice(&chunk[..remaining]);
                    limit_flag.store(true, Ordering::SeqCst);
                    return (buffer, true);
                }
                buffer.extend_from_slice(&chunk[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }

    (buffer, false)
}

pub fn run_git_command(
    git_path: &Path,
    worktree: Option<&Path>,
    subcmd_args: &[&OsStr],
    stdin_data: Option<&[u8]>,
    max_bytes: usize,
    timeout: Duration,
) -> Result<CommandOutput, GitError> {
    let mut cmd = Command::new(git_path);

    cmd.arg("--no-optional-locks");
    cmd.arg("--no-lazy-fetch");
    cmd.arg("--literal-pathspecs");
    cmd.arg("-c");
    cmd.arg("core.fsmonitor=false");
    cmd.arg("-c");
    cmd.arg("protocol.allow=never");

    if let Some(wt) = worktree {
        cmd.arg("-C");
        cmd.arg(wt);
    }

    for arg in subcmd_args {
        cmd.arg(arg);
    }

    for (key, _) in std::env::vars_os() {
        let key_str = key.to_string_lossy();
        if key_str.starts_with("GIT_") || key_str == "PAGER" {
            cmd.env_remove(key);
        }
    }

    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd.env("GIT_OPTIONAL_LOCKS", "0");
    cmd.env("GIT_NO_LAZY_FETCH", "1");

    if stdin_data.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| GitError::Io(format!("Falha ao iniciar processo git: {e}")))?;

    if let Some(data) = stdin_data
        && let Some(mut stdin) = child.stdin.take()
    {
        let data_owned = data.to_vec();
        thread::spawn(move || {
            let _ = stdin.write_all(&data_owned);
        });
    }

    let stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| GitError::Io("Falha ao abrir stdout do processo".to_string()))?;
    let stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| GitError::Io("Falha ao abrir stderr do processo".to_string()))?;

    let limit_flag = Arc::new(AtomicBool::new(false));
    let limit_flag_stdout = Arc::clone(&limit_flag);
    let limit_flag_stderr = Arc::clone(&limit_flag);

    let stdout_handle =
        thread::spawn(move || drain_stream(stdout_pipe, max_bytes, limit_flag_stdout));
    let stderr_handle =
        thread::spawn(move || drain_stream(stderr_pipe, max_bytes, limit_flag_stderr));

    let start_time = Instant::now();
    let status: ExitStatus;

    loop {
        if limit_flag.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();
            return Err(GitError::OutputLimitExceeded { max_bytes });
        }

        match child.try_wait() {
            Ok(Some(s)) => {
                status = s;
                break;
            }
            Ok(None) => {
                if start_time.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stdout_handle.join();
                    let _ = stderr_handle.join();
                    return Err(GitError::Timeout {
                        duration_ms: timeout.as_millis() as u64,
                    });
                }
                thread::sleep(Duration::from_millis(5));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_handle.join();
                let _ = stderr_handle.join();
                return Err(GitError::Io(format!("Erro aguardando processo: {e}")));
            }
        }
    }

    let (stdout, stdout_overflow) = stdout_handle
        .join()
        .map_err(|_| GitError::Io("Falha na thread de leitura do stdout".to_string()))?;
    let (stderr, stderr_overflow) = stderr_handle
        .join()
        .map_err(|_| GitError::Io("Falha na thread de leitura do stderr".to_string()))?;

    if stdout_overflow || stderr_overflow {
        return Err(GitError::OutputLimitExceeded { max_bytes });
    }

    Ok(CommandOutput {
        status,
        stdout,
        stderr,
    })
}

pub fn run_git_command_success(
    git_path: &Path,
    worktree: Option<&Path>,
    subcmd_args: &[&OsStr],
    stdin_data: Option<&[u8]>,
    max_bytes: usize,
    timeout: Duration,
) -> Result<Vec<u8>, GitError> {
    let output = run_git_command(
        git_path,
        worktree,
        subcmd_args,
        stdin_data,
        max_bytes,
        timeout,
    )?;

    if !output.status.success() {
        return Err(GitError::CommandFailed {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(output.stdout)
}
