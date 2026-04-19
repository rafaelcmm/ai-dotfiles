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
    /// Root directory under HOME where managed files are installed.
    pub(crate) root: &'static str,
    /// Metadata filename tracking managed files for this target.
    pub(crate) metadata_file: &'static str,
    /// Enables migration cleanup for legacy prefixed entries in this target.
    pub(crate) allow_legacy_cleanup: bool,
}

impl Platform {
    /// Returns true when this target represents the HOME root itself.
    pub(crate) fn is_home_root(self) -> bool {
        self.root.is_empty()
    }

    /// Normalized provider name used in filters and logs.
    ///
    /// HOME root scope returns an empty string and should be handled explicitly.
    pub(crate) fn normalized_name(self) -> &'static str {
        self.root.trim_start_matches('.')
    }
}

/// Platform set managed by this tool.
pub(crate) const PLATFORMS: [Platform; 3] = [
    Platform {
        root: ".claude",
        metadata_file: "_meta.md",
        allow_legacy_cleanup: true,
    },
    Platform {
        root: ".copilot",
        metadata_file: "_meta.md",
        allow_legacy_cleanup: true,
    },
    Platform {
        root: ".cursor",
        metadata_file: "_meta.md",
        allow_legacy_cleanup: true,
    },
];

/// User HOME root managed scope for shared cross-tool files.
pub(crate) const HOME_SCOPE: Platform = Platform {
    root: "",
    metadata_file: ".ai-dotfiles-home-meta.md",
    allow_legacy_cleanup: false,
};
