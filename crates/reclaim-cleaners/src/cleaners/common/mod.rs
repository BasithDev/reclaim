//! Cleaners that exist on both Linux and Windows (developer tools, browsers, editors, downloads).

use crate::Cleaner;

mod browser_caches;
mod cargo_cache;
mod devscan;
mod docker;
mod go_build_cache;
mod gradle_cache;
mod node_modules;
mod npm_cache;
mod old_downloads;
mod pip_cache;
mod pnpm_store;
mod rust_targets;
mod vscode_caches;
mod yarn_cache;

/// Cross-platform cleaners in display order (availability is filtered by the registry).
pub fn all() -> Vec<Box<dyn Cleaner>> {
    vec![
        Box::new(docker::Docker),
        node_modules::cleaner(),
        rust_targets::cleaner(),
        npm_cache::cleaner(),
        Box::new(pnpm_store::PnpmStore),
        yarn_cache::cleaner(),
        pip_cache::cleaner(),
        cargo_cache::cleaner(),
        gradle_cache::cleaner(),
        Box::new(go_build_cache::GoBuildCache),
        browser_caches::cleaner(),
        vscode_caches::cleaner(),
        old_downloads::cleaner(),
    ]
}
