use std::fs;

#[cfg(unix)]
use std::os::unix::fs::symlink;

use tempfile::tempdir;

use crate::{run, Command};

fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[test]
fn install_creates_meta_and_versioned_files() {
    let home = tempdir().expect("tempdir should be created");
    let result = run(Command::Install, home.path()).expect("install should succeed");

    assert!(result.contains("Installed configuration version"));

    let claude_agent = home
        .path()
        .join(".claude/agents")
        .join(format!("rafaelcmm-{}-rust-specialist.md", version()));
    assert!(claude_agent.exists());

    let copilot_skill = home
        .path()
        .join(".copilot/skills")
        .join(format!("rafaelcmm-{}-clean-code", version()))
        .join("SKILL.md");
    assert!(copilot_skill.exists());

    let meta = home.path().join(".cursor/_meta.md");
    let meta_contents = fs::read_to_string(meta).expect("meta should be readable");
    assert!(meta_contents.contains(&format!("version: {}", version())));
}

#[test]
fn second_install_requests_update() {
    let home = tempdir().expect("tempdir should be created");
    run(Command::Install, home.path()).expect("first install should succeed");

    let second =
        run(Command::Install, home.path()).expect("second install should succeed with no changes");
    assert_eq!(
        second,
        "Configuration already installed. Run `update` instead.".to_string()
    );
}

#[test]
fn update_reports_up_to_date_when_versions_match() {
    let home = tempdir().expect("tempdir should be created");
    run(Command::Install, home.path()).expect("install should succeed");

    let message = run(Command::Update, home.path()).expect("update should succeed");
    assert_eq!(message, "Configuration is already up to date.".to_string());
}

#[test]
fn update_bootstraps_when_no_install_exists() {
    let home = tempdir().expect("tempdir should be created");
    let message = run(Command::Update, home.path()).expect("update should succeed");

    assert!(message.contains("Updated configuration to version"));
    assert!(home.path().join(".claude/_meta.md").exists());
}

#[test]
fn debloat_removes_only_managed_content() {
    let home = tempdir().expect("tempdir should be created");
    run(Command::Install, home.path()).expect("install should succeed");

    let user_file = home.path().join(".claude/agents/my-custom-agent.md");
    fs::create_dir_all(user_file.parent().expect("parent must exist"))
        .expect("parent dir should be created");
    fs::write(&user_file, "custom").expect("custom file should be created");

    run(Command::Debloat, home.path()).expect("debloat should succeed");

    let managed = home
        .path()
        .join(".claude/agents")
        .join(format!("rafaelcmm-{}-rust-specialist.md", version()));
    assert!(!managed.exists());
    assert!(user_file.exists());
    assert!(!home.path().join(".claude/_meta.md").exists());
}

#[cfg(unix)]
#[test]
fn debloat_does_not_follow_symlinked_managed_dir() {
    let home = tempdir().expect("tempdir should be created");
    let outside = tempdir().expect("outside tempdir should be created");

    let outside_file = outside.path().join("keep.txt");
    fs::write(&outside_file, "do not delete").expect("outside file should be created");

    let agents = home.path().join(".claude/agents");
    fs::create_dir_all(&agents).expect("agents dir should be created");
    let link_path = agents.join("rafaelcmm-symlink");
    symlink(outside.path(), &link_path).expect("symlink should be created");

    run(Command::Debloat, home.path()).expect("debloat should succeed");

    assert!(outside_file.exists());
    assert!(!link_path.exists());
}

#[cfg(unix)]
#[test]
fn update_does_not_follow_symlinked_managed_dir() {
    let home = tempdir().expect("tempdir should be created");
    let outside = tempdir().expect("outside tempdir should be created");

    let outside_file = outside.path().join("keep-update.txt");
    fs::write(&outside_file, "do not delete").expect("outside file should be created");

    let rules = home.path().join(".cursor/rules");
    fs::create_dir_all(&rules).expect("rules dir should be created");
    let link_path = rules.join("rafaelcmm-symlink");
    symlink(outside.path(), &link_path).expect("symlink should be created");

    run(Command::Update, home.path()).expect("update should succeed");

    assert!(outside_file.exists());
    assert!(!link_path.exists());
}
