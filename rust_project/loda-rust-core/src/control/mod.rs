//! Loading of programs, manage dependencies between programs, caching of programs.
mod dependency_manager;

pub use dependency_manager::{DependencyManager,DependencyManagerFileSystemMode};
