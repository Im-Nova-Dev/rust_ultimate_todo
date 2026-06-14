//! Internal logging for non-fatal filesystem and persistence errors.

pub(crate) fn fs_error(context: &str, err: impl std::fmt::Display) {
    eprintln!("rust_tui: {context}: {err}");
}
