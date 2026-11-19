//! Behavioural tests for the session exemptions of the shipped `guard-paths.sh` hook:
//! the session scratchpad, plan files and `CLAUDE_PROJECT_DIR`.
//!
//! They drive the real script through the helpers in `guard_paths_tests`, with a fake
//! `HOME`, a throwaway repository as working directory and, where a test opts in, a
//! private `TMPDIR` or `CLAUDE_PROJECT_DIR`. Exit code `0` allows the write, `2` blocks it.

use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

use crate::guard_paths_tests::{init_repo, verdict, verdict_with_env};

/// Numeric id of the user running the tests, as the hook reads it with `id -u`.
///
/// Panics when `id` is unavailable.
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
    std::fs::create_dir_all(&session).unwrap();

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
    std::fs::create_dir_all(&mine).unwrap();

    let refused = [
        mine.join("-work-repo/0b6f-session/tool-results/x.txt"),
        mine.join("-work-repo/scratchpad/x.md"),
        mine.join("-work-repo/0b6f-session/scratchpad"),
        mine.join("-work-repo/0b6f-session/scratchpad/../../x.md"),
        // Right shape, so only the blocklist refuses these two.
        mine.join("-work-repo/0b6f-session/scratchpad/.git/config"),
        mine.join("-work-repo/0b6f-session/scratchpad/deploy.pem"),
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
fn writes_anywhere_under_the_session_project_dir_are_allowed() {
    let home = tempdir().unwrap();
    let workspace = home.path().join("Workspace");
    let repo = workspace.join("site");
    std::fs::create_dir_all(&repo).unwrap();
    init_repo(&repo);
    let project = [("CLAUDE_PROJECT_DIR", workspace.as_path())];

    let sibling = workspace.join("dotfiles/static/agent.md");
    assert_eq!(verdict_with_env(home.path(), &repo, &sibling, &project), 0);
    // The blocklist still applies inside the project folder.
    let git_internal = workspace.join("dotfiles/.git/config");
    assert_eq!(
        verdict_with_env(home.path(), &repo, &git_internal, &project),
        2
    );
    // Outside the project folder the repo-root check is back in force.
    let outside = home.path().join(".zshrc");
    assert_eq!(verdict_with_env(home.path(), &repo, &outside, &project), 2);
}

#[test]
fn a_project_dir_that_is_hidden_outside_home_or_foreign_exempts_nothing() {
    let home = tempdir().unwrap();
    let elsewhere = tempdir().unwrap();
    let cases = [
        // A hidden folder below $HOME, e.g. a session started inside ~/.ssh.
        (
            home.path().join(".ssh"),
            home.path().join(".ssh/authorized_keys"),
        ),
        (
            home.path().join(".claude"),
            home.path().join(".claude/settings.json"),
        ),
        // A folder outside $HOME.
        (
            elsewhere.path().join("etc"),
            elsewhere.path().join("etc/cron.d/x"),
        ),
    ];
    for (project, target) in cases {
        // The agent works in a repository inside the project folder; the target sits in
        // the project folder but outside that repository.
        let repo = project.join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        init_repo(&repo);
        assert_eq!(
            verdict_with_env(
                home.path(),
                &repo,
                &target,
                &[("CLAUDE_PROJECT_DIR", &project)]
            ),
            2,
            "CLAUDE_PROJECT_DIR={} must not exempt {}",
            project.display(),
            target.display()
        );
    }

    // A project folder that does not hold the hook's working directory is ignored.
    let workspace = home.path().join("Workspace");
    let repo = home.path().join("other/site");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&repo).unwrap();
    init_repo(&repo);
    assert_eq!(
        verdict_with_env(
            home.path(),
            &repo,
            &workspace.join("x.md"),
            &[("CLAUDE_PROJECT_DIR", workspace.as_path())]
        ),
        2
    );
}

#[cfg(unix)]
#[test]
fn a_symlinked_or_missing_scratch_root_exempts_nothing() {
    let home = tempdir().unwrap();
    let repo = tempdir().unwrap();
    init_repo(repo.path());
    let tmp = tempdir().unwrap();
    let target_dir = tempdir().unwrap();
    let uid_dir = tmp.path().join(format!("claude-{}", current_uid()));
    let target = uid_dir.join("-p/s/scratchpad/x.md");

    // Missing: no claude-<uid> folder at all.
    assert_eq!(
        verdict_with_env(home.path(), repo.path(), &target, &[("TMPDIR", tmp.path())]),
        2
    );
    // A symlink someone else could have planted.
    std::os::unix::fs::symlink(target_dir.path(), &uid_dir).unwrap();
    assert_eq!(
        verdict_with_env(home.path(), repo.path(), &target, &[("TMPDIR", tmp.path())]),
        2
    );
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

#[cfg(unix)]
#[test]
fn a_scratch_root_writable_by_others_exempts_nothing() {
    use std::os::unix::fs::PermissionsExt;
    let home = tempdir().unwrap();
    let repo = tempdir().unwrap();
    init_repo(repo.path());
    let tmp = tempdir().unwrap();
    let uid_dir = tmp.path().join(format!("claude-{}", current_uid()));
    std::fs::create_dir_all(&uid_dir).unwrap();
    let target = uid_dir.join("-p/s/scratchpad/x.md");
    let env = [("TMPDIR", tmp.path())];

    for mode in [0o777, 0o770] {
        std::fs::set_permissions(&uid_dir, std::fs::Permissions::from_mode(mode)).unwrap();
        assert_eq!(
            verdict_with_env(home.path(), repo.path(), &target, &env),
            2,
            "mode {mode:o} must not be exempt"
        );
    }
    std::fs::set_permissions(&uid_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(verdict_with_env(home.path(), repo.path(), &target, &env), 0);
}
