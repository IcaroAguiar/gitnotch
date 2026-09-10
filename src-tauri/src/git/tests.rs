use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::*;
use crate::git::command::run_git_command;

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TestRepo {
    path: PathBuf,
}

impl TestRepo {
    fn new(prefix: &str) -> Self {
        let count = FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir_name = format!("gitnotch-test-{prefix}-{count}-{timestamp}");
        let path = std::env::temp_dir().join(dir_name);
        fs::create_dir_all(&path).expect("Falha ao criar diretório temporário para fixture");

        let status = Command::new("git")
            .arg("init")
            .arg("-b")
            .arg("main")
            .arg(&path)
            .status()
            .expect("Falha ao executar git init");
        assert!(status.success());

        let _ = Command::new("git")
            .arg("-C")
            .arg(&path)
            .args(["config", "user.name", "Test User"])
            .status();
        let _ = Command::new("git")
            .arg("-C")
            .arg(&path)
            .args(["config", "user.email", "test@example.com"])
            .status();

        Self { path }
    }

    fn write_file(&self, rel_path: &str, content: &[u8]) {
        let full_path = self.path.join(rel_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(&full_path).unwrap();
        file.write_all(content).unwrap();
    }

    fn git(&self, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .args(args)
            .status()
            .expect("Falha ao executar comando git na fixture");
        assert!(status.success(), "Comando git {:?} falhou", args);
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn snapshot_directory_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut map = BTreeMap::new();
    collect_files_recursive(root, root, &mut map);
    map
}

fn collect_files_recursive(base: &Path, current: &Path, map: &mut BTreeMap<PathBuf, Vec<u8>>) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel) = path.strip_prefix(base)
                    && let Ok(bytes) = fs::read(&path)
                {
                    map.insert(rel.to_path_buf(), bytes);
                }
            } else if path.is_dir() {
                collect_files_recursive(base, &path, map);
            }
        }
    }
}

#[test]
fn test_git_capabilities_discovery() {
    let reader = GitReader::new().expect("Git deve ser encontrado no PATH do sistema");
    let caps = reader.capabilities();
    assert!(caps.installed);
    assert!(caps.supports_porcelain_v2);
    assert!(!caps.version.is_empty());
    assert!(!caps.executable_path.is_empty());
}

#[test]
fn test_output_limit_kills_process() {
    let reader = GitReader::new().unwrap();
    let repo = TestRepo::new("limit");

    let mut large_content = Vec::new();
    for i in 0..10_000 {
        large_content.extend_from_slice(
            format!("linha número {i} com bastante conteúdo para teste\n").as_bytes(),
        );
    }
    repo.write_file("big.txt", &large_content);

    let args = [
        std::ffi::OsStr::new("status"),
        std::ffi::OsStr::new("--porcelain=v2"),
    ];
    let result = run_git_command(
        &reader.git_executable,
        Some(&repo.path),
        &args,
        None,
        4,
        Duration::from_secs(5),
    );

    match result {
        Err(GitError::OutputLimitExceeded { max_bytes }) => {
            assert_eq!(max_bytes, 4);
        }
        other => panic!("Esperado OutputLimitExceeded, obtido: {:?}", other),
    }
}

#[test]
fn test_unborn_repository() {
    let reader = GitReader::new().unwrap();
    let repo = TestRepo::new("unborn");

    repo.write_file("staged.txt", b"conteudo staged");
    repo.git(&["add", "staged.txt"]);
    repo.write_file("untracked.txt", b"conteudo untracked");

    let status = reader
        .status(&repo.path)
        .expect("Status em unborn deve funcionar");
    assert!(
        status.branch.is_unborn,
        "Repositório deve ser identificado como unborn"
    );
    assert!(status.branch.oid.is_none());

    assert_eq!(status.staged.len(), 1);
    assert_eq!(status.staged[0].path, "staged.txt");
    assert_eq!(status.staged[0].kind, ChangeKind::Added);
    assert_eq!(status.staged[0].group, FileGroupKind::Staged);

    assert_eq!(status.untracked.len(), 1);
    assert_eq!(status.untracked[0].path, "untracked.txt");
    assert_eq!(status.untracked[0].group, FileGroupKind::Untracked);

    let staged_diff = reader
        .diff(&repo.path, "staged.txt", None, FileGroupKind::Staged)
        .expect("Diff staged em repo unborn deve ter sucesso");
    assert!(staged_diff.patch.contains("+conteudo staged"));

    let untracked_diff = reader
        .diff(&repo.path, "untracked.txt", None, FileGroupKind::Untracked)
        .expect("Diff untracked deve ter sucesso");
    assert!(untracked_diff.patch.contains("+conteudo untracked"));
}

#[test]
fn test_detached_head_repository() {
    let reader = GitReader::new().unwrap();
    let repo = TestRepo::new("detached");

    repo.write_file("c1.txt", b"commit 1");
    repo.git(&["add", "c1.txt"]);
    repo.git(&["commit", "-m", "first commit"]);

    repo.write_file("c2.txt", b"commit 2");
    repo.git(&["add", "c2.txt"]);
    repo.git(&["commit", "-m", "second commit"]);

    repo.git(&["checkout", "HEAD~1"]);

    let status = reader
        .status(&repo.path)
        .expect("Status em detached HEAD deve funcionar");
    assert!(status.branch.is_detached, "Deve identificar detached HEAD");
    assert_eq!(status.branch.head, "(detached)");
    assert!(status.branch.oid.is_some());
}

#[test]
fn test_same_file_staged_and_unstaged_groups() {
    let reader = GitReader::new().unwrap();
    let repo = TestRepo::new("both-groups");

    repo.write_file("file.txt", b"linha 1\n");
    repo.git(&["add", "file.txt"]);
    repo.git(&["commit", "-m", "initial"]);

    repo.write_file("file.txt", b"linha 1\nlinha 2\n");
    repo.git(&["add", "file.txt"]);

    repo.write_file("file.txt", b"linha 1\nlinha 2\nlinha 3\n");

    let status = reader.status(&repo.path).expect("Status deve ter sucesso");

    assert_eq!(status.staged.len(), 1);
    assert_eq!(status.staged[0].path, "file.txt");
    assert_eq!(status.staged[0].group, FileGroupKind::Staged);
    assert_eq!(status.staged[0].staged_status, Some('M'));

    assert_eq!(status.unstaged.len(), 1);
    assert_eq!(status.unstaged[0].path, "file.txt");
    assert_eq!(status.unstaged[0].group, FileGroupKind::Unstaged);
    assert_eq!(status.unstaged[0].unstaged_status, Some('M'));

    let staged_diff = reader
        .diff(&repo.path, "file.txt", None, FileGroupKind::Staged)
        .unwrap();
    assert!(staged_diff.patch.contains("+linha 2"));
    assert!(!staged_diff.patch.contains("+linha 3"));

    let unstaged_diff = reader
        .diff(&repo.path, "file.txt", None, FileGroupKind::Unstaged)
        .unwrap();
    assert!(unstaged_diff.patch.contains("+linha 3"));
}

#[test]
fn test_paths_with_spaces_and_special_pathspecs() {
    let reader = GitReader::new().unwrap();
    let repo = TestRepo::new("special-paths");

    repo.write_file("arquivo com espacos.txt", b"conteudo espacos");
    repo.write_file("arquivo:com:dois:pontos.txt", b"conteudo colons");
    repo.write_file("[colchetes].txt", b"conteudo colchetes");
    repo.write_file("arquivo_acentuado_café.txt", b"conteudo cafe");

    repo.git(&["add", "arquivo com espacos.txt", "[colchetes].txt"]);

    let status = reader
        .status(&repo.path)
        .expect("Status com caminhos especiais");

    let staged_paths: Vec<&str> = status.staged.iter().map(|f| f.path.as_str()).collect();
    assert!(staged_paths.contains(&"arquivo com espacos.txt"));
    assert!(staged_paths.contains(&"[colchetes].txt"));

    let untracked_paths: Vec<&str> = status.untracked.iter().map(|f| f.path.as_str()).collect();
    assert!(untracked_paths.contains(&"arquivo:com:dois:pontos.txt"));
    assert!(untracked_paths.contains(&"arquivo_acentuado_café.txt"));

    let bracket_diff = reader
        .diff(&repo.path, "[colchetes].txt", None, FileGroupKind::Staged)
        .expect("Diff no arquivo com colchetes deve tratar pathspec literalmente");
    assert!(bracket_diff.patch.contains("+conteudo colchetes"));
}

#[test]
fn test_rename_preserves_both_paths() {
    let reader = GitReader::new().unwrap();
    let repo = TestRepo::new("rename");

    repo.write_file("antigo.txt", b"conteudo para renomear\n");
    repo.git(&["add", "antigo.txt"]);
    repo.git(&["commit", "-m", "add antigo"]);

    repo.git(&["mv", "antigo.txt", "novo.txt"]);

    let status = reader.status(&repo.path).expect("Status rename");
    assert_eq!(status.staged.len(), 1);
    assert_eq!(status.staged[0].path, "novo.txt");
    assert_eq!(status.staged[0].orig_path.as_deref(), Some("antigo.txt"));
    assert_eq!(status.staged[0].kind, ChangeKind::Renamed);

    let rename_diff = reader
        .diff(
            &repo.path,
            "novo.txt",
            Some("antigo.txt"),
            FileGroupKind::Staged,
        )
        .expect("Diff do arquivo renomeado");
    assert!(rename_diff.patch.contains("rename from antigo.txt"));
    assert!(rename_diff.patch.contains("rename to novo.txt"));
}

#[test]
fn test_unmerged_conflicts() {
    let reader = GitReader::new().unwrap();
    let repo = TestRepo::new("conflict");

    repo.write_file("conflict.txt", b"base content\n");
    repo.git(&["add", "conflict.txt"]);
    repo.git(&["commit", "-m", "base commit"]);

    repo.git(&["checkout", "-b", "branch1"]);
    repo.write_file("conflict.txt", b"branch1 content\n");
    repo.git(&["commit", "-am", "branch1 commit"]);

    repo.git(&["checkout", "main"]);
    repo.git(&["checkout", "-b", "branch2"]);
    repo.write_file("conflict.txt", b"branch2 content\n");
    repo.git(&["commit", "-am", "branch2 commit"]);

    let _ = Command::new("git")
        .arg("-C")
        .arg(&repo.path)
        .args(["merge", "branch1"])
        .status();

    let status = reader.status(&repo.path).expect("Status com conflito");
    assert_eq!(status.conflicts.len(), 1);
    assert_eq!(status.conflicts[0].path, "conflict.txt");
    assert_eq!(status.conflicts[0].kind, ChangeKind::Conflicted);
    assert_eq!(status.conflicts[0].group, FileGroupKind::Conflicted);
}

#[test]
fn test_external_filter_synthetic_blocking() {
    let reader = GitReader::new().unwrap();
    let repo = TestRepo::new("filter-block");

    repo.write_file("documento.txt", b"texto original");
    repo.git(&["add", "documento.txt"]);
    repo.git(&["commit", "-m", "init"]);

    let marker_file = repo.path.join("filtro_executado_marcador.txt");
    assert!(!marker_file.exists());

    let clean_cmd = format!("touch \"{}\"", marker_file.to_string_lossy());
    repo.git(&["config", "filter.syntheticblock.clean", &clean_cmd]);

    repo.write_file(".gitattributes", b"*.txt filter=syntheticblock\n");
    repo.git(&["add", ".gitattributes"]);
    repo.git(&["commit", "-m", "add attributes"]);

    if marker_file.exists() {
        let _ = fs::remove_file(&marker_file);
    }
    assert!(
        !marker_file.exists(),
        "Marcador não deve existir antes da execução do Git Notch"
    );

    repo.write_file("documento.txt", b"texto alterado");

    let filter_result = reader
        .check_filters(&repo.path)
        .expect("Preflight deve executar");
    match filter_result {
        FilterPreflightResult::LimitedByExternalFilter { filter_name, .. } => {
            assert_eq!(filter_name, "syntheticblock");
        }
        FilterPreflightResult::Allowed => panic!("Deveria ter bloqueado o filtro externo"),
    }

    let status_result = reader.status(&repo.path);
    match status_result {
        Err(GitError::LimitedByExternalFilter { filter_name, .. }) => {
            assert_eq!(filter_name, "syntheticblock");
        }
        other => panic!("Esperado erro LimitedByExternalFilter, obtido: {:?}", other),
    }

    assert!(
        !marker_file.exists(),
        "O filtro externo sintético NUNCA deve ser executado pelo Git Notch!"
    );
}

#[test]
fn test_no_mutations_proof() {
    let reader = GitReader::new().unwrap();
    let repo = TestRepo::new("no-mutation");

    repo.write_file("tracked.txt", b"versao original 1\n");
    repo.write_file("removido.txt", b"para remover\n");
    repo.git(&["add", "tracked.txt", "removido.txt"]);
    repo.git(&["commit", "-m", "commit inicial"]);

    repo.write_file("novo_staged.txt", b"conteudo novo staged\n");
    repo.git(&["add", "novo_staged.txt"]);

    repo.write_file("tracked.txt", b"versao modificada no worktree\n");

    repo.write_file("untracked.txt", b"conteudo nao rastreado\n");

    let git_dir = repo.path.join(".git");
    let before_git = snapshot_directory_bytes(&git_dir);
    let before_worktree = snapshot_directory_bytes(&repo.path);

    for _ in 0..10 {
        let status = reader.status(&repo.path).expect("Status repetido");
        assert!(!status.staged.is_empty());
        assert!(!status.unstaged.is_empty());
        assert!(!status.untracked.is_empty());

        let _ = reader
            .diff(&repo.path, "tracked.txt", None, FileGroupKind::Unstaged)
            .unwrap();
        let _ = reader
            .diff(&repo.path, "novo_staged.txt", None, FileGroupKind::Staged)
            .unwrap();
        let _ = reader
            .diff(&repo.path, "untracked.txt", None, FileGroupKind::Untracked)
            .unwrap();
    }

    let after_git = snapshot_directory_bytes(&git_dir);
    let after_worktree = snapshot_directory_bytes(&repo.path);

    assert_eq!(
        before_git, after_git,
        "Metadados do Git (.git) não devem sofrer nenhuma mutação durante consultas de leitura!"
    );
    assert_eq!(
        before_worktree, after_worktree,
        "Arquivos do worktree não devem sofrer nenhuma mutação durante consultas de leitura!"
    );
}
