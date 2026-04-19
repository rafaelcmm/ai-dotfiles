//! Embedded static tree access and canonical destination mapping logic.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir, DirEntry};

use crate::constants::Platform;

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

/// Provides read-only access to embedded static files.
pub(crate) fn static_dir() -> &'static Dir<'static> {
    &STATIC_DIR
}

/// Builds desired platform-relative destination files for one platform.
pub(crate) fn desired_files_for_platform(platform: Platform) -> Result<HashMap<PathBuf, Vec<u8>>> {
    let mut files = Vec::<(PathBuf, Vec<u8>)>::new();

    let platform_dir = static_dir()
        .get_dir(platform.root)
        .with_context(|| format!("missing static platform directory {}", platform.root))?;
    collect_embedded_files(platform_dir, Path::new(""), &mut files);

    if platform.normalized_name() == "claude" {
        let shared_dir = static_dir()
            .get_dir("__shared__")
            .context("missing static shared directory")?;
        collect_embedded_files(shared_dir, Path::new(""), &mut files);
    }

    let mut mapped = HashMap::<PathBuf, Vec<u8>>::new();
    for (relative, contents) in files {
        if mapped.insert(relative.clone(), contents).is_some() {
            anyhow::bail!("duplicate embedded destination {}", relative.display());
        }
    }

    Ok(mapped)
}

/// Builds desired files installed directly in HOME root.
pub(crate) fn desired_home_files() -> Result<HashMap<PathBuf, Vec<u8>>> {
    let mut output = HashMap::<PathBuf, Vec<u8>>::new();

    for file_name in ["AGENTS.md", "CLAUDE.md"] {
        let file = static_dir()
            .get_file(file_name)
            .with_context(|| format!("missing static home file {file_name}"))?;
        output.insert(PathBuf::from(file_name), file.contents().to_vec());
    }

    Ok(output)
}

fn collect_embedded_files(dir: &Dir<'_>, prefix: &Path, output: &mut Vec<(PathBuf, Vec<u8>)>) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(child_dir) => {
                let child_name = child_dir
                    .path()
                    .file_name()
                    .expect("embedded dir without name");
                let child_prefix = prefix.join(child_name);
                collect_embedded_files(child_dir, &child_prefix, output);
            }
            DirEntry::File(file) => {
                let file_name = file.path().file_name().expect("embedded file without name");
                let relative = prefix.join(file_name);
                output.push((relative, file.contents().to_vec()));
            }
        }
    }
}
