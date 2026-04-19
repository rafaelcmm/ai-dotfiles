//! Integration-style tests for high-level install/update/debloat behavior.
//!
//! Tests focus on canonical path installs, metadata-driven lifecycle behavior,
//! non-destructive updates, and symlink handling during cleanup.

use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::symlink;

use tempfile::tempdir;

use crate::constants::{Platform, HOME_SCOPE};
use crate::external_skills::seed_test_external_skill_cache;
use crate::meta::{load_manifest, render_meta, PlatformManifest};
use crate::{run, Command};

fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn platform(root: &'static str) -> Platform {
    Platform {
        root,
        metadata_file: "_meta.md",
        allow_legacy_cleanup: true,
    }
}

#[test]
fn install_creates_meta_and_canonical_files() {
    let home = tempdir().expect("tempdir should be created");
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");
    let result = run(Command::Install, home.path()).expect("install should succeed");

    assert!(result.contains("Installed configuration version"));

    let claude_agent = home.path().join(".claude/agents/rust-specialist.md");
    assert!(claude_agent.exists());
    assert!(!home
        .path()
        .join(".claude/agents")
        .join(format!("ai-dotfiles-{}-rust-specialist.md", version()))
        .exists());

    let claude_skill = home.path().join(".claude/skills/clean-code/SKILL.md");
    assert!(claude_skill.exists());
    assert!(!home
        .path()
        .join(".copilot/skills/clean-code/SKILL.md")
        .exists());

    for script in ["format-file.sh", "guard-paths.sh", "scan-secrets.sh"] {
        assert!(home.path().join(".claude/.hooks").join(script).exists());
        assert!(home.path().join(".cursor/.hooks").join(script).exists());
        assert!(home.path().join(".copilot/hooks").join(script).exists());
    }

    let claude_settings =
        fs::read_to_string(home.path().join(".claude/settings.json")).expect("claude settings");
    assert!(claude_settings.contains("Edit|Write|MultiEdit"));
    assert!(claude_settings.contains("$HOME/.claude/.hooks/guard-paths.sh"));
    assert!(claude_settings.contains("$HOME/.claude/.hooks/scan-secrets.sh"));
    assert!(claude_settings.contains("\"timeout\": 5"));
    assert!(claude_settings.contains("$HOME/.claude/.hooks/format-file.sh"));
    assert!(claude_settings.contains("\"timeout\": 15"));

    let cursor_hooks =
        fs::read_to_string(home.path().join(".cursor/hooks.json")).expect("cursor hooks");
    assert!(cursor_hooks.contains("beforeShellExecution"));
    assert!(cursor_hooks.contains("afterFileEdit"));
    assert!(cursor_hooks.contains("$HOME/.cursor/.hooks/guard-paths.sh"));
    assert!(cursor_hooks.contains("$HOME/.cursor/.hooks/scan-secrets.sh"));
    assert!(cursor_hooks.contains("$HOME/.cursor/.hooks/format-file.sh"));

    let copilot_hooks =
        fs::read_to_string(home.path().join(".copilot/hooks/hooks.json")).expect("copilot hooks");
    assert!(copilot_hooks.contains("preToolUse"));
    assert!(copilot_hooks.contains("postToolUse"));
    assert!(copilot_hooks.contains("\"type\": \"command\""));
    assert!(copilot_hooks.contains("\"bash\": \"$HOME/.copilot/hooks/guard-paths.sh\""));
    assert!(copilot_hooks.contains("\"bash\": \"$HOME/.copilot/hooks/scan-secrets.sh\""));
    assert!(copilot_hooks.contains("\"bash\": \"$HOME/.copilot/hooks/format-file.sh\""));
    assert!(copilot_hooks.contains("\"timeoutSec\": 5"));
    assert!(copilot_hooks.contains("\"timeoutSec\": 15"));

    let vscode_mcp = fs::read_to_string(home.path().join(".config/Code/User/mcp.json"))
        .expect("vscode user mcp should be generated");
    assert!(vscode_mcp.contains("\"servers\""));
    assert!(vscode_mcp.contains("\"chrome-devtools\""));
    assert!(vscode_mcp.contains("\"next-devtools\""));

    for root in [".claude", ".copilot", ".cursor"] {
        assert!(home.path().join(root).join("AGENTS.md").exists());
        assert!(home.path().join(root).join("CLAUDE.md").exists());
    }

    let manifest = load_manifest(home.path(), platform(".claude"))
        .expect("manifest load should succeed")
        .expect("manifest should exist");
    assert_eq!(manifest.version, version());
    assert!(manifest.managed_files.contains("_meta.md"));
    assert!(manifest.managed_files.contains("AGENTS.md"));
    assert!(manifest.managed_files.contains("CLAUDE.md"));
    assert!(manifest.managed_files.contains("agents/rust-specialist.md"));
    assert!(manifest
        .managed_files
        .contains("skills/clean-code/SKILL.md"));
    assert!(manifest.managed_files.contains(".hooks/guard-paths.sh"));
    assert!(manifest.managed_files.contains(".hooks/scan-secrets.sh"));
    assert!(manifest.managed_files.contains(".hooks/format-file.sh"));
    assert!(manifest.managed_directories.contains("agents"));
    assert!(manifest.managed_directories.contains("skills"));
    assert!(manifest.managed_directories.contains("skills/clean-code"));

    let copilot_manifest = load_manifest(home.path(), platform(".copilot"))
        .expect("copilot manifest load should succeed")
        .expect("copilot manifest should exist");
    assert!(copilot_manifest.managed_files.contains("AGENTS.md"));
    assert!(copilot_manifest.managed_files.contains("CLAUDE.md"));
    assert!(copilot_manifest
        .managed_files
        .contains("hooks/guard-paths.sh"));
    assert!(copilot_manifest
        .managed_files
        .contains("hooks/scan-secrets.sh"));
    assert!(copilot_manifest
        .managed_files
        .contains("hooks/format-file.sh"));
    assert!(copilot_manifest.managed_files.contains("hooks/hooks.json"));

    let cursor_manifest = load_manifest(home.path(), platform(".cursor"))
        .expect("cursor manifest load should succeed")
        .expect("cursor manifest should exist");
    assert!(cursor_manifest.managed_files.contains("AGENTS.md"));
    assert!(cursor_manifest.managed_files.contains("CLAUDE.md"));
    assert!(cursor_manifest
        .managed_files
        .contains(".hooks/guard-paths.sh"));
    assert!(cursor_manifest
        .managed_files
        .contains(".hooks/scan-secrets.sh"));
    assert!(cursor_manifest
        .managed_files
        .contains(".hooks/format-file.sh"));
}

#[test]
fn second_install_requests_update() {
    let home = tempdir().expect("tempdir should be created");
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");
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
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");
    run(Command::Install, home.path()).expect("install should succeed");

    let message = run(Command::Update, home.path()).expect("update should succeed");
    assert_eq!(message, "Configuration is already up to date.".to_string());
}

#[test]
fn update_bootstraps_when_no_install_exists() {
    let home = tempdir().expect("tempdir should be created");
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");
    let message = run(Command::Update, home.path()).expect("update should succeed");

    assert!(message.contains("Updated configuration to version"));
    assert!(home.path().join(".claude/_meta.md").exists());
    assert!(home
        .path()
        .join(".claude/agents/rust-specialist.md")
        .exists());
}

#[test]
fn install_merges_vscode_mcp_servers_without_removing_existing_entries() {
    let home = tempdir().expect("tempdir should be created");
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");

    let vscode_user_dir = home.path().join(".config/Code/User");
    fs::create_dir_all(&vscode_user_dir).expect("vscode user dir should be created");
    fs::write(
        vscode_user_dir.join("mcp.json"),
        r#"{
    "servers": {
        "existing-server": {
            "command": "echo",
            "args": ["ok"]
        }
    }
}
"#,
    )
    .expect("existing vscode mcp should be created");

    run(Command::Install, home.path()).expect("install should succeed");

    let merged =
        fs::read_to_string(vscode_user_dir.join("mcp.json")).expect("vscode mcp should exist");
    assert!(merged.contains("\"existing-server\""));
    assert!(merged.contains("\"chrome-devtools\""));
    assert!(merged.contains("\"next-devtools\""));
}

#[test]
fn install_preserves_existing_vscode_server_on_name_collision() {
    let home = tempdir().expect("tempdir should be created");
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");

    let vscode_user_dir = home.path().join(".config/Code/User");
    fs::create_dir_all(&vscode_user_dir).expect("vscode user dir should be created");
    fs::write(
        vscode_user_dir.join("mcp.json"),
        r#"{
    "servers": {
        "chrome-devtools": {
            "command": "custom-chrome-mcp",
            "args": ["--custom"]
        }
    }
}
"#,
    )
    .expect("existing vscode mcp should be created");

    run(Command::Install, home.path()).expect("install should succeed");

    let merged =
        fs::read_to_string(vscode_user_dir.join("mcp.json")).expect("vscode mcp should exist");
    assert!(merged.contains("\"custom-chrome-mcp\""));
    assert!(merged.contains("\"next-devtools\""));
}

#[test]
fn update_refreshes_existing_canonical_install() {
    let home = tempdir().expect("tempdir should be created");
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");

    let managed_agent = home
        .path()
        .join(".claude/agents")
        .join("rust-specialist.md");
    fs::create_dir_all(managed_agent.parent().expect("managed parent should exist"))
        .expect("managed parent should be created");
    fs::write(&managed_agent, "stale").expect("managed file should be created");
    fs::write(
        home.path().join(".claude/_meta.md"),
        format!(
            "---\nversion: {}\n---\n\nThis file is automatically generated.\n",
            version()
        ),
    )
    .expect("metadata should be written");

    let message = run(Command::Update, home.path()).expect("update should succeed");

    assert!(message.contains("Updated configuration to version"));
    assert!(managed_agent.exists());
    assert_ne!(
        fs::read_to_string(&managed_agent).expect("managed file should be readable"),
        "stale"
    );

    let manifest = load_manifest(home.path(), platform(".claude"))
        .expect("manifest load should succeed")
        .expect("manifest should exist after update");
    assert!(manifest.managed_files.contains("agents/rust-specialist.md"));
    assert!(manifest.managed_directories.contains("agents"));
}

#[test]
fn update_removes_legacy_home_root_shared_files() {
    let home = tempdir().expect("tempdir should be created");
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");

    fs::write(home.path().join("AGENTS.md"), "legacy").expect("legacy AGENTS should be created");
    fs::write(home.path().join("CLAUDE.md"), "legacy").expect("legacy CLAUDE should be created");

    let home_manifest = PlatformManifest::new(
        version(),
        vec![
            PathBuf::from("AGENTS.md"),
            PathBuf::from("CLAUDE.md"),
            PathBuf::from(".ai-dotfiles-home-meta.md"),
        ],
        Vec::<PathBuf>::new(),
    )
    .expect("home manifest should be created");
    fs::write(
        home.path().join(".ai-dotfiles-home-meta.md"),
        render_meta(&home_manifest).expect("home metadata should render"),
    )
    .expect("home metadata should be written");

    let message = run(Command::Update, home.path()).expect("update should succeed");
    assert!(message.contains("Updated configuration to version"));

    assert!(!home.path().join("AGENTS.md").exists());
    assert!(!home.path().join("CLAUDE.md").exists());
    assert!(!home.path().join(".ai-dotfiles-home-meta.md").exists());

    for root in [".claude", ".copilot", ".cursor"] {
        assert!(home.path().join(root).join("AGENTS.md").exists());
        assert!(home.path().join(root).join("CLAUDE.md").exists());
    }

    assert!(load_manifest(home.path(), HOME_SCOPE)
        .expect("legacy home manifest should load")
        .is_none());
}

#[test]
fn debloat_removes_only_managed_content() {
    let home = tempdir().expect("tempdir should be created");
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");
    run(Command::Install, home.path()).expect("install should succeed");

    let user_file = home.path().join(".claude/agents/my-custom-agent.md");
    fs::create_dir_all(user_file.parent().expect("parent must exist"))
        .expect("parent dir should be created");
    fs::write(&user_file, "custom").expect("custom file should be created");

    run(Command::Debloat, home.path()).expect("debloat should succeed");

    let managed = home.path().join(".claude/agents/rust-specialist.md");
    assert!(!managed.exists());
    assert!(user_file.exists());
    assert!(!home.path().join(".claude/_meta.md").exists());
}

#[cfg(unix)]
#[test]
fn debloat_does_not_follow_symlinked_managed_dir() {
    let home = tempdir().expect("tempdir should be created");
    let outside = tempdir().expect("outside tempdir should be created");
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");

    let outside_file = outside.path().join("keep.txt");
    fs::write(&outside_file, "do not delete").expect("outside file should be created");

    let agents = home.path().join(".claude/agents");
    fs::create_dir_all(&agents).expect("agents dir should be created");
    let link_path = agents.join("ai-dotfiles-symlink");
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
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");

    let outside_file = outside.path().join("keep-update.txt");
    fs::write(&outside_file, "do not delete").expect("outside file should be created");

    let rules = home.path().join(".cursor/rules");
    fs::create_dir_all(&rules).expect("rules dir should be created");
    let link_path = rules.join("ai-dotfiles-symlink");
    symlink(outside.path(), &link_path).expect("symlink should be created");

    run(Command::Update, home.path()).expect("update should succeed");

    assert!(outside_file.exists());
    assert!(!link_path.exists());
}

#[cfg(unix)]
#[test]
fn install_refuses_symlinked_managed_destination_file() {
    let home = tempdir().expect("tempdir should be created");
    let outside = tempdir().expect("outside tempdir should be created");
    seed_test_external_skill_cache(home.path()).expect("test cache should be seeded");

    let destination = home.path().join(".claude/agents/rust-specialist.md");
    fs::create_dir_all(destination.parent().expect("managed parent should exist"))
        .expect("managed parent should be created");

    let external_target = outside.path().join("external-target.md");
    fs::write(&external_target, "outside").expect("outside file should be created");
    symlink(&external_target, &destination).expect("symlink should be created");

    let error = run(Command::Install, home.path()).expect_err("install should fail");
    assert!(
        error
            .to_string()
            .contains("refusing to write through symlink"),
        "unexpected error: {error}"
    );
    assert_eq!(
        fs::read_to_string(&external_target).expect("outside file should be readable"),
        "outside"
    );
}
