//! Behavioural tests for the shipped `guard-paths.sh` hook.
//!
//! The hook is the only thing between an agent and writes outside the repository, so
//! these tests run the real script under `bash` with a fake `HOME` and a throwaway git
//! repository as its working directory, and assert on its exit code: `0` allows the
//! write, `2` blocks it. They require `bash`, `git`, `jq` and `realpath` on `PATH`,
//! the same tools the hook itself requires at runtime, and fail rather than skip
//! when one is missing so an absent tool never reads as a pass.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::tempdir;

/// Absolute path to the hook as embedded into every install.
fn hook() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("static/__shared__/scripts/guard-paths.sh")
}

/// Runs the hook for one target path and returns its exit code.
///
/// `home` becomes `$HOME` and `cwd` the hook's working directory, which is what the
/// hook resolves the repository root from. Panics when the hook cannot be spawned or
/// is killed by a signal, since neither is a verdict.
fn verdict(home: &Path, cwd: &Path, target: &Path) -> i32 {
    let payload = format!(r#"{{"tool_input":{{"file_path":"{}"}}}}"#, target.display());
    let mut child = Command::new("bash")
        .arg(hook())
        .current_dir(cwd)
        .env("HOME", home)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("bash must be available to run the hook");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(payload.as_bytes())
        .expect("payload written to the hook");
    child
        .wait()
        .expect("hook ran")
        .code()
        .expect("hook exited normally")
}

/// Creates a git repository at `dir` so the hook's repo-root check is active.
fn init_repo(dir: &Path) {
    let status = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .status()
        .expect("git must be available");
    assert!(status.success(), "git init failed");
}

#[test]
fn claude_memory_markdown_is_writable_from_inside_a_repo() {
    let home = tempdir().unwrap();
    let repo = tempdir().unwrap();
    init_repo(repo.path());
    let memory = home.path().join(".claude/projects/-work-repo/memory");

    assert_eq!(
        verdict(home.path(), repo.path(), &memory.join("MEMORY.md")),
        0
    );
    assert_eq!(
        verdict(home.path(), repo.path(), &memory.join("a-fact.md")),
        0
    );
}

#[test]
fn only_the_flat_markdown_memory_shape_is_exempt() {
    let home = tempdir().unwrap();
    let repo = tempdir().unwrap();
    init_repo(repo.path());
    let projects = home.path().join(".claude/projects");
    let memory = projects.join("-work-repo/memory");

    // Each of these sits outside the repository and outside the exempt shape, so the
    // repo-root check must still refuse it.
    let refused = [
        memory.join("nested/x.md"),
        memory.join("script.sh"),
        memory.join("../../../settings.json"),
        projects.join("a/memory/../../../.zshrc.md"),
        projects.join("memory/x.md"),
        home.path().join(".claude/settings.json"),
        home.path().join(".zshrc"),
    ];
    for target in refused {
        assert_eq!(
            verdict(home.path(), repo.path(), &target),
            2,
            "{} must be blocked",
            target.display()
        );
    }
}

#[cfg(unix)]
#[test]
fn a_symlinked_memory_directory_cannot_redirect_the_exemption() {
    let home = tempdir().unwrap();
    let repo = tempdir().unwrap();
    init_repo(repo.path());
    let elsewhere = tempdir().unwrap();
    let project = home.path().join(".claude/projects/-work-repo");
    std::fs::create_dir_all(&project).unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), project.join("memory")).unwrap();

    assert_eq!(
        verdict(home.path(), repo.path(), &project.join("memory/x.md")),
        2
    );
}

#[test]
fn writes_inside_the_repository_stay_allowed() {
    let home = tempdir().unwrap();
    let repo = tempdir().unwrap();
    init_repo(repo.path());

    assert_eq!(
        verdict(home.path(), repo.path(), &repo.path().join("src/x.rs")),
        0
    );
}
