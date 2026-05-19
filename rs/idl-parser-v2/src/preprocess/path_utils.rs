use std::path::{Component, Path};

/// `true` for paths that should be treated as absolute on either Unix or
/// Windows: `/etc/passwd` (root-only, not `is_absolute` on Windows),
/// `C:\...` (drive prefix), and UNC/verbatim prefixes.
pub(super) fn is_absolute_like(p: &Path) -> bool {
    p.is_absolute()
        || p.has_root()
        || p.components()
            .next()
            .is_some_and(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
}
