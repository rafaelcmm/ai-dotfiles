//! Embedded static tree access and destination mapping logic.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir, DirEntry};

use crate::constants::{Platform, MANAGED_PREFIX, MANAGED_ROOTS};
use crate::meta::render_meta;

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

/// Provides read-only access to embedded static files.
pub(crate) fn static_dir() -> &'static Dir<'static> {
    &STATIC_DIR
}

/// Builds desired destination files for one platform and the current package version.
pub(crate) fn desired_files_for_platform(
    platform: Platform,
    version: &str,
) -> Result<HashMap<PathBuf, Vec<u8>>> {
    let mut files = Vec::<(PathBuf, Vec<u8>)>::new();

    let platform_dir = static_dir()
        .get_dir(platform.root)
        .with_context(|| format!("missing static platform directory {}", platform.root))?;
    collect_embedded_files(platform_dir, Path::new(""), &mut files);

    let shared_dir = static_dir()
        .get_dir("__shared__")
        .context("missing static shared directory")?;
    collect_embedded_files(shared_dir, Path::new(""), &mut files);

    let mut mapped = HashMap::<PathBuf, Vec<u8>>::new();
    for (relative, contents) in files {
        let transformed = transform_managed_path(&relative, version);
        let destination = PathBuf::from(platform.root).join(transformed);
        mapped.insert(destination, contents);
    }

    mapped.insert(
        PathBuf::from(platform.root).join("_meta.md"),
        render_meta().into_bytes(),
    );

    Ok(mapped)
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

fn transform_managed_path(relative: &Path, version: &str) -> PathBuf {
    let components: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect();

    if components.len() < 2 {
        return relative.to_path_buf();
    }

    if MANAGED_ROOTS.contains(&components[0].as_str()) {
        let mut transformed = components;
        transformed[1] = format!("{MANAGED_PREFIX}{version}-{}", transformed[1]);

        let mut output = PathBuf::new();
        for component in transformed {
            output.push(component);
        }
        return output;
    }

    relative.to_path_buf()
}
