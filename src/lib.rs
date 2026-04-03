//! Library entrypoint for managing AI configuration files across supported platforms.

mod constants;
mod embedded;
mod external_skills;
mod fs_ops;
mod meta;
mod operations;

pub use constants::Command;
pub use operations::run;

#[cfg(test)]
mod tests;
