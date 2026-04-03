//! Metadata rendering helpers for `_meta.md` generation and version discovery.

use std::fs;
use std::path::Path;

use crate::constants::Platform;
use crate::embedded::static_dir;

/// Generates `_meta.md` from `static/_meta_template.md` and package metadata.
pub(crate) fn render_meta() -> String {
    let template_file = static_dir()
        .get_file("_meta_template.md")
        .expect("static/_meta_template.md not found");
    let template = String::from_utf8_lossy(template_file.contents()).to_string();

    let version = env!("CARGO_PKG_VERSION");
    let repository = env!("CARGO_PKG_REPOSITORY");
    let authors = env!("CARGO_PKG_AUTHORS");
    let (author_name, author_email) = parse_author(authors);

    template
        .replace("{{package.version}}", version)
        .replace("{{package.author.name}}", author_name)
        .replace("{{package.author.email}}", author_email)
        .replace("{{package.repository.url}}", repository)
}

/// Reads installed version from `<platform>/_meta.md`, when present.
pub(crate) fn installed_version(home: &Path, platform: &Platform) -> Option<String> {
    let meta = home.join(platform.root).join("_meta.md");
    let content = fs::read_to_string(meta).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("version:") {
            let version = value.trim();
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }

    None
}

fn parse_author(authors: &str) -> (&str, &str) {
    let first = authors.split(':').next().unwrap_or(authors).trim();
    if let Some(start) = first.find('<') {
        if let Some(end) = first.find('>') {
            let name = first[..start].trim();
            let email = first[start + 1..end].trim();
            return (name, email);
        }
    }

    (first, "")
}
