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
    verdict_with_env(home, cwd, target, &[])
}

/// Runs the hook like [`verdict`] with extra environment variables.
///
/// The variables the hook reads besides `HOME` (`CLAUDE_PROJECT_DIR`,
/// `CLAUDE_CODE_TMPDIR`, `TMPDIR`) are removed first, so a test run started from inside
/// an agent session sees only what `extra` sets. `TMPDIR` falls back to `home`, which
/// keeps the real `/tmp` scratchpad shape out of every test that does not opt in.
fn verdict_with_env(home: &Path, cwd: &Path, target: &Path, extra: &[(&str, &Path)]) -> i32 {
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

/// Numeric id of the user running the tests, as the hook reads it with `id -u`.
fn current_uid() -> String {
    let out = Command::new("id")
        .arg("-u")
        .output()
        .expect("id must be available");
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

#[test]
fn session_scratchpad_files_are_writable_from_inside_a_repo() {
    let home = tempdir().unwrap();
    let repo = tempdir().unwrap();
    init_repo(repo.path());
    let tmp = tempdir().unwrap();
    let session = tmp
        .path()
        .join(format!("claude-{}", current_uid()))
        .join("-work-repo/0b6f-session");

    for target in [
        session.join("scratchpad/notes.md"),
        session.join("scratchpad/covers/a.jpg"),
    ] {
        assert_eq!(
            verdict_with_env(home.path(), repo.path(), &target, &[("TMPDIR", tmp.path())]),
            0,
            "{} must be allowed",
            target.display()
        );
    }
}

#[test]
fn only_the_scratchpad_shape_of_the_current_user_is_exempt() {
    let home = tempdir().unwrap();
    let repo = tempdir().unwrap();
    init_repo(repo.path());
    let tmp = tempdir().unwrap();
    let mine = tmp.path().join(format!("claude-{}", current_uid()));

    let refused = [
        mine.join("-work-repo/0b6f-session/tool-results/x.txt"),
        mine.join("-work-repo/scratchpad/x.md"),
        mine.join("-work-repo/0b6f-session/scratchpad"),
        mine.join("-work-repo/0b6f-session/scratchpad/../../x.md"),
        tmp.path()
            .join("claude-999999/-work-repo/s/scratchpad/x.md"),
        tmp.path().join("other/-work-repo/s/scratchpad/x.md"),
    ];
    for target in refused {
        assert_eq!(
            verdict_with_env(home.path(), repo.path(), &target, &[("TMPDIR", tmp.path())]),
            2,
            "{} must be blocked",
            target.display()
        );
    }
}

#[test]
fn plan_markdown_is_writable_but_nothing_else_under_claude() {
    let home = tempdir().unwrap();
    let repo = tempdir().unwrap();
    init_repo(repo.path());
    let plans = home.path().join(".claude/plans");

    assert_eq!(
        verdict(home.path(), repo.path(), &plans.join("a-plan.md")),
        0
    );
    for target in [
        plans.join("nested/a.md"),
        plans.join("run.sh"),
        home.path().join(".claude/agents/x.md"),
        home.path().join(".claude/CLAUDE.md"),
    ] {
        assert_eq!(
            verdict(home.path(), repo.path(), &target),
            2,
            "{} must be blocked",
            target.display()
        );
    }
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

#[test]
fn writes_anywhere_under_the_session_project_dir_are_allowed() {
    let home = tempdir().unwrap();
    let workspace = tempdir().unwrap();
    let repo = workspace.path().join("site");
    std::fs::create_dir_all(&repo).unwrap();
    init_repo(&repo);
    let project = [("CLAUDE_PROJECT_DIR", workspace.path())];

    let sibling = workspace.path().join("dotfiles/static/agent.md");
    assert_eq!(verdict_with_env(home.path(), &repo, &sibling, &project), 0);
    // The blocklist still applies inside the project folder.
    let git_internal = workspace.path().join("dotfiles/.git/config");
    assert_eq!(
        verdict_with_env(home.path(), &repo, &git_internal, &project),
        2
    );
    // Outside the project folder the repo-root check is back in force.
    let outside = home.path().join(".zshrc");
    assert_eq!(verdict_with_env(home.path(), &repo, &outside, &project), 2);
}

#[test]
fn a_project_dir_at_or_above_home_exempts_nothing() {
    let home = tempdir().unwrap();
    let repo = home.path().join("work/site");
    std::fs::create_dir_all(&repo).unwrap();
    init_repo(&repo);
    let zshrc = home.path().join(".zshrc");

    for project in [home.path(), home.path().parent().unwrap(), Path::new("/")] {
        assert_eq!(
            verdict_with_env(
                home.path(),
                &repo,
                &zshrc,
                &[("CLAUDE_PROJECT_DIR", project)]
            ),
            2,
            "CLAUDE_PROJECT_DIR={} must not exempt $HOME",
            project.display()
        );
    }
}
