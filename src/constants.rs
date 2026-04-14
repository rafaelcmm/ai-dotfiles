//! Constants and enums shared across installer modules.

/// Prefix used to identify files managed by this package.
pub(crate) const MANAGED_PREFIX: &str = "ai-dotfiles-";

/// Relative path of the external skill source manifest inside the embedded static tree.
pub(crate) const EXTERNAL_SKILLS_MANIFEST: &str = "external-skills.toml";

/// Cache location under HOME used for downloaded external skills.
pub(crate) const EXTERNAL_SKILLS_CACHE_DIR: &str = ".cache/ai-dotfiles/external-skills";

/// Managed roots where versioned content is installed.
pub(crate) const MANAGED_ROOTS: [&str; 4] = ["agents", "rules", "instructions", "skills"];

/// Supported CLI operations.
#[derive(Debug, Clone, Copy)]
pub enum Command {
    Install,
    Update,
    Debloat,
}

/// Supported platform roots under the user's HOME directory.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Platform {
    pub(crate) root: &'static str,
}

/// Platform set managed by this tool.
pub(crate) const PLATFORMS: [Platform; 3] = [
    Platform { root: ".claude" },
    Platform { root: ".copilot" },
    Platform { root: ".cursor" },
];
