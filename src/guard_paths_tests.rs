//! Behavioural tests for the shipped `guard-paths.sh` hook.
//!
//! The hook is the only check on the edit tools' writes outside the repository (shell
//! commands are not guarded), so these tests run the real script under `bash` with a fake
//! `HOME` and a throwaway git repository as its working directory, and assert on its exit
//! code: `0` allows the write, `2` blocks it. They require `bash`, `git`, `jq`, `realpath`
//! and `python3` on `PATH`. The hook needs `python3` only when `realpath -m` is
//! unavailable; one test exercises that fallback. They fail rather than skip when a tool
//! is missing, so an absent tool never reads as a pass.

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
/// `home` becomes `$HOME` and `cwd` the hook's working directory: the payload carries no
/// `cwd`, so the hook uses it both to resolve the repository root and as the directory
/// `CLAUDE_PROJECT_DIR` must contain. Panics when the hook cannot be spawned or
/// is killed by a signal, since neither is a verdict.
pub(crate) fn verdict(home: &Path, cwd: &Path, target: &Path) -> i32 {
    verdict_with_env(home, cwd, target, &[])
}

/// Runs the hook like [`verdict`] with extra environment variables.
///
/// The variables the hook reads besides `HOME` (`CLAUDE_PROJECT_DIR`,
/// `CLAUDE_CODE_TMPDIR`, `TMPDIR`) are removed first, so a test run started from inside
/// an agent session sees only what `extra` sets. `TMPDIR` is set to `home` so an ambient
/// value cannot add a scratchpad root; the hook's fixed `/tmp` root still applies. Tests
/// assume the runner's own `TMPDIR` is not inside `/tmp/claude-<uid>`, since `tempdir()`
/// would then sit under the exempt shape.
pub(crate) fn verdict_with_env(
    home: &Path,
    cwd: &Path,
    target: &Path,
    extra: &[(&str, &Path)],
) -> i32 {
    let payload = format!(r#"{{"tool_input":{{"file_path":"{}"}}}}"#, target.display());
    let mut command = Command::new("bash");
    command
        .arg(hook())
        .current_dir(cwd)
        .env("HOME", home)
        .env_remove("CLAUDE_PROJECT_DIR")
        .env_remove("CLAUDE_CODE_TMPDIR")
        .env("TMPDIR", home);
    for (key, value) in extra {
        command.env(key, value);
    }
    let mut child = command
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
///
/// Panics when `git` is missing or `git init` fails.
pub(crate) fn init_repo(dir: &Path) {
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

#[test]
fn a_sibling_directory_sharing_the_repo_prefix_is_outside() {
    let home = tempdir().unwrap();
    let parent = tempdir().unwrap();
    let repo = parent.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    init_repo(&repo);

    assert_eq!(
        verdict(home.path(), &repo, &parent.path().join("repo-other/x.rs")),
        2
    );
}

/// Builds a `PATH` directory exposing only the tools the hook needs, plus a `realpath`
/// that rejects every call, so the hook's fallback and fail-closed paths can be driven.
///
/// `with_python` decides whether `python3` is exposed. Returns `<dir>/bin`, which the
/// caller passes as `PATH`; `dir` must be unique per call because the shim is written
/// without checking for an existing file. Panics when a required tool is not on the test
/// runner's `PATH`.
#[cfg(unix)]
fn restricted_path(dir: &Path, with_python: bool) -> PathBuf {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let mut tools = vec!["bash", "jq", "git", "id", "basename", "cat"];
    if with_python {
        tools.push("python3");
    }
    for tool in tools {
        let out = Command::new("bash")
            .args(["-c", &format!("command -v {tool}")])
            .output()
            .unwrap();
        let real = String::from_utf8(out.stdout).unwrap().trim().to_owned();
        assert!(!real.is_empty(), "{tool} must be on PATH");
        std::os::unix::fs::symlink(real, bin.join(tool)).unwrap();
    }
    let shim = bin.join("realpath");
    std::fs::write(
        &shim,
        "#!/bin/sh
exit 1
",
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

#[cfg(unix)]
#[test]
fn without_realpath_m_the_hook_falls_back_to_python_or_refuses() {
    let home = tempdir().unwrap();
    let repo = tempdir().unwrap();
    init_repo(repo.path());
    let tools = tempdir().unwrap();
    let inside = repo.path().join("src/x.rs");
    let escape = repo.path().join("src/../../escape.md");

    let with_python = restricted_path(&tools.path().join("py"), true);
    let env = [("PATH", with_python.as_path())];
    assert_eq!(verdict_with_env(home.path(), repo.path(), &inside, &env), 0);
    assert_eq!(verdict_with_env(home.path(), repo.path(), &escape, &env), 2);

    let bare = restricted_path(&tools.path().join("bare"), false);
    let env = [("PATH", bare.as_path())];
    assert_eq!(verdict_with_env(home.path(), repo.path(), &inside, &env), 2);
}

#[test]
fn key_name_patterns_match_file_names_not_folder_names() {
    let home = tempdir().unwrap();
    let parent = tempdir().unwrap();
    let repo = parent.path().join("id_rsa-parser");
    std::fs::create_dir_all(&repo).unwrap();
    init_repo(&repo);

    assert_eq!(verdict(home.path(), &repo, &repo.join("src/lib.rs")), 0);
    assert_eq!(verdict(home.path(), &repo, &repo.join("keys/id_rsa")), 2);
    assert_eq!(verdict(home.path(), &repo, &repo.join("id_ed25519.pub")), 2);
}
